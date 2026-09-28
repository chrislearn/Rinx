//! Rinx-owned transport. Never publishes into AppCard's connection registry.
use octos_app_transport::{ConnectionState, OutboundCommand, TransportConfig, TransportEvent};
use serde_json::Value;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use tokio::{
    runtime::{Handle, Runtime},
    sync::{broadcast, mpsc, oneshot},
};

pub(super) struct Notification {
    pub session: String,
    pub data: Value,
}
pub(super) struct Connection {
    runtime: Mutex<Option<(Runtime, oneshot::Receiver<()>)>>,
    handle: Handle,
    active: Arc<AtomicBool>,
    commands: mpsc::Sender<OutboundCommand>,
    events: broadcast::Sender<Arc<Notification>>,
    state: Arc<Mutex<ConnectionState>>,
    pub profile: String,
}

impl Connection {
    pub fn start(config: TransportConfig) -> Result<Arc<Self>, String> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;
        let profile = config.profile_id.0.clone();
        let (commands, mut incoming) = {
            let _enter = runtime.enter();
            if config.stdio.is_some() {
                octos_app_transport::stdio::spawn(config)
            } else {
                octos_app_transport::ws::spawn(config)
            }
        };
        let active = Arc::new(AtomicBool::new(true));
        let state = Arc::new(Mutex::new(ConnectionState::Dialing));
        let (events, _) = broadcast::channel(256);
        let (finished_tx, finished) = oneshot::channel();
        let live = active.clone();
        let status = state.clone();
        let fanout = events.clone();
        runtime.spawn(async move {
            while let Some(event) = incoming.recv().await {
                match event {
                    TransportEvent::DurableNotification { payload, .. }
                    | TransportEvent::EphemeralNotification { payload } => {
                        let session = payload.session_id().0.clone();
                        if let Ok(data) = serde_json::to_value(payload) {
                            let _ = fanout.send(Arc::new(Notification { session, data }));
                        }
                    }
                    TransportEvent::ConnectionState(next) => {
                        if next == ConnectionState::Failed {
                            live.store(false, Ordering::Release);
                        }
                        *status.lock().unwrap() = next;
                    }
                    _ => {}
                }
                makepad_widgets::SignalToUI::set_ui_signal();
            }
            live.store(false, Ordering::Release);
            let _ = finished_tx.send(());
        });
        Ok(Arc::new(Self {
            handle: runtime.handle().clone(),
            runtime: Mutex::new(Some((runtime, finished))),
            active,
            commands,
            events,
            state,
            profile,
        }))
    }
    pub fn handle(&self) -> &Handle {
        &self.handle
    }
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }
    pub fn status(&self) -> String {
        format!("{:?}", self.state.lock().unwrap())
    }
    pub fn subscribe(&self) -> broadcast::Receiver<Arc<Notification>> {
        self.events.subscribe()
    }
    pub async fn request(&self, method: &'static str, params: Value) -> Result<Value, String> {
        if !self.is_active() {
            return Err("Octos connection is closed; reconnect in Rinx".into());
        }
        let (reply, result) = oneshot::channel();
        self.commands
            .try_send(OutboundCommand::Rpc {
                method,
                params,
                reply,
            })
            .map_err(|_| "Octos request queue is unavailable")?;
        let value = tokio::time::timeout(std::time::Duration::from_secs(30), result)
            .await
            .map_err(|_| "Octos request timed out")?
            .map_err(|_| "Octos connection closed")?
            .map_err(|e| format!("Octos protocol error: {}", e.message))?;
        if !self.is_active() {
            return Err("Octos connection changed before its reply arrived".into());
        }
        Ok(value)
    }
    /// Disconnect closes only this transport. For stdio the transport owns and
    /// kills its child; for remote mode this does not request server shutdown.
    pub fn shutdown(&self) -> Option<std::thread::JoinHandle<()>> {
        self.active.store(false, Ordering::Release);
        let (runtime, finished) = self.runtime.lock().unwrap().take()?;
        let commands = self.commands.clone();
        // Do not block Makepad or drop a Tokio runtime on one of its workers.
        Some(std::thread::spawn(move || {
            runtime.block_on(async move {
                let _ = tokio::time::timeout(std::time::Duration::from_secs(3), async move {
                    let _ = commands.send(OutboundCommand::Disconnect).await;
                    let _ = finished.await;
                    // The upstream stdio task calls start_kill before dropping
                    // Child. Keep the reactor alive for Tokio's orphan reaper
                    // and cancellation observers after that task exits.
                    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                })
                .await;
            });
            runtime.shutdown_timeout(std::time::Duration::from_secs(1));
        }))
    }
}
impl Drop for Connection {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use octos_app_transport::{Capabilities, ProfileId, SecretString, StdioSpawn};
    struct Fixture(std::path::PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn fixture() -> Fixture {
        let path = std::env::temp_dir().join(format!("rinx-oup-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("peer.py"), r#"import json, os, sys
for line in sys.stdin:
    request = json.loads(line)
    print(json.dumps({'jsonrpc':'2.0','id':request['id'],'result':{'pid':os.getpid(),'params':request.get('params')}}), flush=True)
"#).unwrap();
        Fixture(path)
    }
    fn connection(fixture: &Fixture) -> Arc<Connection> {
        Connection::start(TransportConfig {
            base_url: url::Url::parse("http://127.0.0.1").unwrap(),
            bearer: SecretString::new(""),
            profile_id: ProfileId::new("main"),
            cursor: None,
            cursor_file: None,
            requested_capabilities: Capabilities::requested(),
            workspace_cwd: None,
            stdio: Some(StdioSpawn {
                program: "/usr/bin/python3".into(),
                args: vec![
                    "-u".into(),
                    fixture.0.join("peer.py").to_string_lossy().into_owned(),
                ],
                env: vec![],
                cwd: Some(fixture.0.clone()),
            }),
        })
        .unwrap()
    }
    fn pid(connection: &Connection) -> i32 {
        connection
            .handle()
            .block_on(connection.request(
                "session/hydrate",
                serde_json::json!({"session_id":"main:api:test"}),
            ))
            .unwrap()["pid"]
            .as_i64()
            .unwrap() as i32
    }
    fn stopped(pid: i32) -> bool {
        // These PIDs come only from the disposable child fixture above.
        unsafe { libc::kill(pid, 0) != 0 }
    }
    #[test]
    fn owned_transport_shutdown_does_not_retire_another_connection() {
        let fixture = fixture();
        let first = connection(&fixture);
        let second = connection(&fixture);
        let first_pid = pid(&first);
        let second_pid = pid(&second);
        assert_ne!(first_pid, second_pid);
        let _ = first.shutdown();
        assert!(!first.is_active());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !stopped(first_pid) && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        assert!(stopped(first_pid), "owned child must terminate");
        assert_eq!(
            pid(&second),
            second_pid,
            "another owned transport must still work"
        );
        let _ = second.shutdown();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !stopped(second_pid) && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        assert!(stopped(second_pid));
    }
}

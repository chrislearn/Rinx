//! Explicit standalone ownership or a borrowed, host-injected Octos service.
#[cfg(feature = "standalone")]
use super::{KernelProvider, transport::Connection};
#[cfg(feature = "standalone")]
use octos_app_transport::{Capabilities, ProfileId, SecretString, StdioSpawn, TransportConfig};
use rinx_miniapp_core::{Lease, OctosHost, OctosProvider, OCTOS_SERVICES};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Clone)]
enum Source {
    Unconfigured,
    Hosted(Option<Arc<dyn OctosHost>>),
    #[cfg(feature = "standalone")]
    Standalone {
        connection: Arc<Connection>,
        workspace: Workspace,
    },
}
#[cfg(feature = "standalone")]
#[derive(Clone)]
enum Workspace {
    Local,
    Remote(String),
}
thread_local! { static SOURCE: RefCell<Source> = const { RefCell::new(Source::Unconfigured) }; }

#[cfg(feature = "standalone")]
thread_local! { static CLEANUPS: RefCell<Vec<std::thread::JoinHandle<()>>> = const { RefCell::new(Vec::new()) }; }

fn finish_cleanups() {
    #[cfg(feature = "standalone")]
    for cleanup in CLEANUPS.with(|pending| std::mem::take(&mut *pending.borrow_mut())) {
        let _ = cleanup.join();
    }
}

fn replace(next: Source) {
    super::invalidate_sessions();
    let previous = SOURCE.with(|s| s.replace(next));
    #[cfg(feature = "standalone")]
    if let Source::Standalone { connection, .. } = &previous {
        if let Some(cleanup) = connection.shutdown() {
            CLEANUPS.with(|pending| {
                let mut pending = pending.borrow_mut();
                pending.retain(|thread| !thread.is_finished());
                pending.push(cleanup);
            });
        }
    }
    drop(previous);
}

/// Called only by the native module entry. No provider means unavailable, never
/// permission to discover AppCard or start a fallback kernel.
pub fn enter_hosted(provider: Option<Arc<dyn OctosHost>>) {
    replace(Source::Hosted(provider));
}
pub fn shutdown() {
    replace(Source::Unconfigured);
    finish_cleanups();
}
pub fn is_hosted() -> bool {
    SOURCE.with(|s| matches!(*s.borrow(), Source::Hosted(_)))
}
fn require_standalone() -> Result<(), String> {
    if is_hosted() {
        Err("OctoSense owns this app's AI provider; use system AI settings".into())
    } else {
        Ok(())
    }
}
pub fn status() -> String {
    SOURCE.with(|s| match &*s.borrow() {
        Source::Unconfigured => "AI is not configured".into(),
        Source::Hosted(None) => "OctoSense has not supplied an AI service".into(),
        Source::Hosted(Some(_)) => "AI service supplied by OctoSense".into(),
        #[cfg(feature = "standalone")]
        Source::Standalone {
            connection,
            workspace,
        } => format!(
            "{} Octos: {}",
            if matches!(workspace, Workspace::Local) {
                "Local"
            } else {
                "Remote"
            },
            connection.status()
        ),
    })
}
pub fn disconnect() -> Result<(), String> {
    require_standalone()?;
    shutdown();
    Ok(())
}

#[cfg(feature = "standalone")]
fn profile(value: &str) -> Result<ProfileId, String> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > 128
        || !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
    {
        return Err("Use a profile ID containing letters, digits, hyphens or underscores".into());
    }
    Ok(ProfileId::new(value))
}
#[cfg(feature = "standalone")]
fn config(profile_id: ProfileId) -> TransportConfig {
    TransportConfig {
        base_url: url::Url::parse("http://127.0.0.1").unwrap(),
        bearer: SecretString::new(""),
        profile_id,
        cursor: None,
        cursor_file: None,
        requested_capabilities: Capabilities::requested(),
        workspace_cwd: None,
        stdio: None,
    }
}
#[cfg(feature = "standalone")]
pub fn configure_remote(
    endpoint: &str,
    profile_id: &str,
    token: &str,
    workspace_root: &str,
) -> Result<(), String> {
    require_standalone()?;
    let url = url::Url::parse(endpoint.trim()).map_err(|_| "Enter a complete Octos server URL")?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("Use an HTTP or HTTPS URL without credentials, query or fragment".into());
    }
    let root = remote_root(workspace_root)?;
    let mut cfg = config(profile(profile_id)?);
    cfg.base_url = url;
    cfg.bearer = SecretString::new(token);
    let connection = Connection::start(cfg)?;
    replace(Source::Standalone {
        connection,
        workspace: Workspace::Remote(root),
    });
    Ok(())
}
#[cfg(feature = "standalone")]
pub fn configure_local(
    executable: &Path,
    config_file: &Path,
    profile_id: &str,
) -> Result<(), String> {
    require_standalone()?;
    let profile_id = profile(profile_id)?;
    let executable = absolute_file(executable, "kernel executable")?;
    let config_file = absolute_file(config_file, "kernel configuration")?;
    let root = crate::app_data_dir().join("octos");
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let mut cfg = config(profile_id);
    cfg.stdio = Some(StdioSpawn {
        program: executable,
        args: vec![
            "serve".into(),
            "--stdio".into(),
            "--data-dir".into(),
            root.to_string_lossy().into_owned(),
            "--config".into(),
            config_file.to_string_lossy().into_owned(),
        ],
        env: Vec::new(),
        cwd: Some(root),
    });
    // A previous local kernel may still hold this data root's serve lock.
    // Finish its bounded teardown before launching the replacement process.
    replace(Source::Unconfigured);
    finish_cleanups();
    let connection = Connection::start(cfg)?;
    replace(Source::Standalone {
        connection,
        workspace: Workspace::Local,
    });
    Ok(())
}
#[cfg(feature = "standalone")]
fn absolute_file(path: &Path, label: &str) -> Result<PathBuf, String> {
    if !path.is_absolute() || !path.is_file() {
        return Err(format!("Select an existing absolute path for the {label}"));
    }
    path.canonicalize().map_err(|e| e.to_string())
}
#[cfg(any(feature = "standalone", test))]
fn remote_root(value: &str) -> Result<String, String> {
    let value = value.trim().trim_end_matches('/');
    if !value.starts_with('/')
        || value.is_empty()
        || value.contains('\\')
        || value.chars().any(char::is_control)
        || value.split('/').any(|part| part == "." || part == "..")
    {
        return Err(
            "Enter an absolute server-side workspace directory (without . or .. components)".into(),
        );
    }
    Ok(value.into())
}
#[cfg(any(feature = "standalone", test))]
fn encoded(value: &str) -> String {
    value
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(not(feature = "standalone"))]
pub fn configure_remote(_: &str, _: &str, _: &str, _: &str) -> Result<(), String> {
    Err("This build uses an OctoSense-supplied AI service".into())
}
#[cfg(not(feature = "standalone"))]
pub fn configure_local(_: &Path, _: &Path, _: &str) -> Result<(), String> {
    Err("This build uses an OctoSense-supplied AI service".into())
}

pub(super) fn open(
    lease: &Lease,
    local_workspace: &Path,
) -> Result<Option<Arc<dyn OctosProvider>>, String> {
    if !OCTOS_SERVICES
        .iter()
        .any(|service| lease.services().contains(*service))
    {
        return Ok(None);
    }
    lease.check(&lease.identity().account)?;
    let source = SOURCE.with(|s| s.borrow().clone());
    match source {
        Source::Hosted(Some(host)) => host.open(lease, local_workspace).map(Some),
        Source::Hosted(None) | Source::Unconfigured => Ok(None),
        #[cfg(feature = "standalone")]
        Source::Standalone {
            connection,
            workspace,
        } => {
            let path = match workspace {
                Workspace::Local => local_workspace
                    .canonicalize()
                    .map_err(|e| e.to_string())?
                    .to_string_lossy()
                    .into_owned(),
                Workspace::Remote(root) => format!(
                    "{root}/{}/{}",
                    encoded(&lease.identity().account),
                    encoded(&lease.identity().app)
                ),
            };
            Ok(Some(
                KernelProvider::from_connection(connection.clone(), path) as Arc<dyn OctosProvider>,
            ))
        }
    }
}
/// Local session sandbox roots live under the owned kernel's data directory.
/// Remote/hosted local UI storage never masquerades as a kernel workspace.
pub(super) fn data_root() -> PathBuf {
    SOURCE.with(|s| match &*s.borrow() {
        #[cfg(feature = "standalone")]
        Source::Standalone {
            workspace: Workspace::Local,
            ..
        } => crate::app_data_dir().join("octos"),
        _ => crate::app_data_dir().to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rinx_miniapp_core::{InstanceId, ServiceEvent};
    use std::sync::{Mutex, mpsc::SyncSender};
    struct Host(Mutex<Vec<InstanceId>>);
    struct Provider;
    impl OctosProvider for Provider {
        fn request(
            &self,
            _: Lease,
            _: &str,
            _: serde_json::Value,
            _: SyncSender<ServiceEvent>,
        ) -> Result<(), String> {
            Ok(())
        }
        fn decide(
            &self,
            _: Lease,
            _: &str,
            _: bool,
            _: SyncSender<ServiceEvent>,
        ) -> Result<(), String> {
            Ok(())
        }
        fn close(&self, _: &InstanceId) {}
    }
    impl OctosHost for Host {
        fn open(&self, lease: &Lease, _: &Path) -> Result<Arc<dyn OctosProvider>, String> {
            self.0.lock().unwrap().push(lease.identity().clone());
            Ok(Arc::new(Provider))
        }
    }
    fn lease(account: &str, service: &str) -> Lease {
        Lease::new(
            InstanceId {
                app: "test".into(),
                account: account.into(),
                room: None,
                generation: 1,
            },
            [service.into()].into(),
            Default::default(),
            std::time::Instant::now() + std::time::Duration::from_secs(60),
        )
    }
    #[test]
    fn hosted_requests_preserve_identity_and_require_a_live_octos_grant() {
        let host = Arc::new(Host(Mutex::new(Vec::new())));
        enter_hosted(Some(host.clone()));
        assert!(
            open(
                &lease("@a:example", "matrix.account_info"),
                Path::new("/local")
            )
            .unwrap()
            .is_none()
        );
        let expired = lease("@a:example", "octos.turn.start");
        expired.revoke();
        assert!(open(&expired, Path::new("/local")).is_err());
        for account in ["@a:example", "@b:example"] {
            assert!(
                open(&lease(account, "octos.turn.start"), Path::new("/local"))
                    .unwrap()
                    .is_some()
            );
        }
        let seen = host.0.lock().unwrap();
        assert_eq!(seen.len(), 2);
        assert_ne!(seen[0].account, seen[1].account);
        drop(seen);
        shutdown();
        // The host remains owned by the shell after Rinx releases it.
        assert_eq!(Arc::strong_count(&host), 1);
        assert!(
            open(
                &lease("@a:example", "octos.turn.start"),
                Path::new("/local")
            )
            .unwrap()
            .is_none()
        );
    }
    #[test]
    fn remote_paths_cannot_escape_the_explicit_server_root() {
        for invalid in ["", "/", "relative", "/a/../b", "/a/./b", "/a\\b", "/a\nb"] {
            assert!(remote_root(invalid).is_err(), "{invalid:?}");
        }
        assert_eq!(remote_root("/srv/rinx/").unwrap(), "/srv/rinx");
        assert!(!encoded("../../app").contains('/'));
        assert_ne!(encoded("@a:server"), encoded("@a:other"));
    }
    #[test]
    fn hosted_mode_rejects_all_owned_connection_configuration() {
        enter_hosted(None);
        assert!(configure_remote("https://example.org", "main", "", "/apps").is_err());
        assert!(configure_local(Path::new("/missing"), Path::new("/missing"), "main").is_err());
        assert!(disconnect().is_err());
        assert!(is_hosted());
        shutdown();
    }
}

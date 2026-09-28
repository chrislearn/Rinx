//! Non-secret standalone settings. Access tokens are intentionally session-only.
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum Settings {
    Local {
        executable: PathBuf,
        config_file: PathBuf,
        profile: String,
    },
    Remote {
        endpoint: String,
        profile: String,
        workspace_root: String,
    },
}
impl Settings {
    pub fn load() -> Result<Option<Self>, String> {
        let path = crate::app_data_dir().join("octos-deployment.json");
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.to_string()),
        };
        if bytes.len() > 16 * 1024 {
            return Err("Octos settings file is too large".into());
        }
        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|_| "Invalid Octos deployment settings".into())
    }
    pub fn save(&self) -> Result<(), String> {
        self.save_in(crate::app_data_dir())
    }
    fn save_in(&self, root: &Path) -> Result<(), String> {
        std::fs::create_dir_all(root).map_err(|e| e.to_string())?;
        let temporary = root.join(format!(".octos-deployment-{}", uuid::Uuid::new_v4()));
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let result = (|| {
            let mut file = options.open(&temporary).map_err(|e| e.to_string())?;
            file.write_all(&serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            file.sync_all().map_err(|e| e.to_string())?;
            std::fs::rename(&temporary, root.join("octos-deployment.json"))
                .map_err(|e| e.to_string())
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(temporary);
        }
        result
    }
}

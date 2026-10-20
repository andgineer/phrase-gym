use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::adapter::CliAdapter;
use crate::transfer::client;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    version: u8,
    pub server: String,
    pub adapter: CliAdapter,
}

impl Settings {
    pub fn load(path: &Path) -> Result<Option<Self>, String> {
        if !path.exists() {
            return Ok(None);
        }
        let settings: Self =
            serde_json::from_slice(&crate::platform::files::read_selected(path, 4096)?)
                .map_err(|_| "Saved connection settings are invalid; configure the Server again")?;
        if settings.version != 1 {
            return Err("Saved settings need a compatible Coach version".into());
        }
        client::server_origin(&settings.server)?;
        Ok(Some(settings))
    }

    pub fn save(path: &Path, server: &str) -> Result<(), String> {
        let adapter = Self::load(path)?.map_or(CliAdapter::Codex, |settings| settings.adapter);
        Self::save_selected(path, server, adapter)
    }

    pub fn save_selected(path: &Path, server: &str, adapter: CliAdapter) -> Result<(), String> {
        crate::platform::files::save(
            path,
            &Self {
                version: 1,
                adapter,
                server: client::server_origin(server)?,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_are_atomic_nonsecret_and_reject_unknown_versions_and_remote_origins() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("settings.json");
        assert_eq!(Settings::load(&path).unwrap(), None);
        Settings::save(&path, "http://127.0.0.1:8181/").unwrap();
        assert_eq!(
            Settings::load(&path).unwrap().unwrap().server,
            "http://127.0.0.1:8181"
        );
        Settings::save_selected(&path, "http://127.0.0.1:8181", CliAdapter::Claude).unwrap();
        Settings::save(&path, "http://127.0.0.1:8182").unwrap();
        assert_eq!(
            Settings::load(&path).unwrap().unwrap().adapter,
            CliAdapter::Claude
        );
        assert!(Settings::save(&path, "https://example.com").is_err());
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
        std::fs::write(&path, r#"{"version":2,"server":"http://127.0.0.1:8181"}"#).unwrap();
        assert!(Settings::load(&path).is_err());
    }
}

use std::path::Path;
use std::sync::Mutex;

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

// Connection and tool choice are saved from different threads; each read-modify-write must not
// overwrite the other's newer field.
static WRITES: Mutex<()> = Mutex::new(());

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
        let _writing = WRITES
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let adapter = Self::load(path)?.map_or(CliAdapter::Codex, |settings| settings.adapter);
        Self::write(path, server, adapter)
    }

    pub fn select_adapter(path: &Path, adapter: CliAdapter) -> Result<(), String> {
        let _writing = WRITES
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let server = Self::load(path)?.map_or_else(
            phrase_gym_contracts::connection::default_server_origin,
            |settings| settings.server,
        );
        Self::write(path, &server, adapter)
    }

    pub fn save_selected(path: &Path, server: &str, adapter: CliAdapter) -> Result<(), String> {
        let _writing = WRITES
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::write(path, server, adapter)
    }

    fn write(path: &Path, server: &str, adapter: CliAdapter) -> Result<(), String> {
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

    #[test]
    fn choosing_a_tool_keeps_the_saved_server_and_concurrent_saves_lose_neither_field() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("settings.json");
        Settings::select_adapter(&path, CliAdapter::Claude).unwrap();
        let fresh = Settings::load(&path).unwrap().unwrap();
        assert_eq!(
            fresh.server,
            phrase_gym_contracts::connection::default_server_origin()
        );
        assert_eq!(fresh.adapter, CliAdapter::Claude);
        Settings::save(&path, "http://127.0.0.1:18487").unwrap();
        std::thread::scope(|scope| {
            scope.spawn(|| {
                for _ in 0..50 {
                    Settings::save(&path, "http://127.0.0.1:18487").unwrap();
                }
            });
            scope.spawn(|| {
                for _ in 0..50 {
                    Settings::select_adapter(&path, CliAdapter::Codex).unwrap();
                }
            });
        });
        let settings = Settings::load(&path).unwrap().unwrap();
        assert_eq!(settings.server, "http://127.0.0.1:18487");
        assert_eq!(settings.adapter, CliAdapter::Codex);
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            r#"{"version":1,"server":"http://127.0.0.1:18487","adapter":"codex"}"#
        );
    }
}

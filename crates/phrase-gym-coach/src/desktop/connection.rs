use crate::platform::paths::CoachPaths;
use crate::platform::settings::Settings;
use crate::transfer::client::ServerClient;

pub fn connect(server: &str, paths: &CoachPaths) -> Result<ServerClient, String> {
    let connected = ServerClient::new(server)?;
    connected.get::<serde_json::Value>("/api/v1/settings")?;
    Settings::save(&paths.config.join("settings.json"), &connected.server)?;
    Ok(connected)
}

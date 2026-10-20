use super::settings::Settings;
use crate::platform::credentials::CredentialStore;
use crate::platform::paths::CoachPaths;
use crate::transfer::client::{self, ServerClient};

pub fn connect(
    server: &str,
    submit: &str,
    worker: &str,
    paths: &CoachPaths,
) -> Result<ServerClient, String> {
    let server = client::loopback(server)?;
    let store = CredentialStore::native()
        .map_err(|_| "Cannot access the OS credential store; unlock it and retry")?;
    let credential = |role: &str, supplied: &str| -> Result<String, String> {
        let account = format!("{role}:{server}");
        if !supplied.is_empty() {
            store
                .write(&account, supplied)
                .map_err(|_| "Cannot save Server access in the OS credential store")?;
        }
        store
            .read(&account)
            .map_err(|_| "Cannot read Server access from the OS credential store")?
            .ok_or_else(|| {
                format!("Configure the existing {role} development credential in connection setup")
            })
    };
    let connected = ServerClient::new(
        &server,
        credential("material-submit", submit)?,
        credential("worker", worker)?,
    )?;
    let probe =
        connected.get::<serde_json::Value>(&format!("/api/v1/inbox/lessons/{}", uuid::Uuid::nil()));
    if let Err(problem) = probe
        && !problem.starts_with("Server 404")
    {
        return Err(problem);
    }
    Settings::save(&paths.config.join("settings.json"), &server)?;
    Ok(connected)
}

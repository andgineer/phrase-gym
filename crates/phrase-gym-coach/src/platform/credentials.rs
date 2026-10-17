use std::sync::Arc;

use keyring_core::{CredentialStore as Backend, Entry, Error};

use super::paths::BUNDLE_IDENTIFIER;

pub const SERVER_ACCOUNT: &str = "server";

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CredentialError {
    #[error("the credential store denied access: {0}")]
    Denied(String),
    #[error("the credential store failed: {0}")]
    Failed(String),
}

// Secrets live only in the OS store. A failure is returned to the caller for display; there is
// deliberately no fallback that writes the secret anywhere else.
#[derive(Clone)]
pub struct CredentialStore {
    backend: Arc<Backend>,
}

impl CredentialStore {
    pub fn native() -> Result<Self, CredentialError> {
        native_backend().map(Self::with_backend)
    }

    pub fn with_backend(backend: Arc<Backend>) -> Self {
        Self { backend }
    }

    pub fn vendor(&self) -> String {
        self.backend.vendor()
    }

    pub fn read(&self, account: &str) -> Result<Option<String>, CredentialError> {
        match self.entry(account)?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(Error::NoEntry) => Ok(None),
            Err(error) => Err(classify(error)),
        }
    }

    pub fn write(&self, account: &str, secret: &str) -> Result<(), CredentialError> {
        self.entry(account)?.set_password(secret).map_err(classify)
    }

    pub fn delete(&self, account: &str) -> Result<(), CredentialError> {
        match self.entry(account)?.delete_credential() {
            Ok(()) | Err(Error::NoEntry) => Ok(()),
            Err(error) => Err(classify(error)),
        }
    }

    fn entry(&self, account: &str) -> Result<Entry, CredentialError> {
        self.backend
            .build(BUNDLE_IDENTIFIER, account, None)
            .map_err(classify)
    }
}

pub fn classify(error: Error) -> CredentialError {
    match error {
        Error::NoStorageAccess(cause) => CredentialError::Denied(cause.to_string()),
        other => CredentialError::Failed(other.to_string()),
    }
}

#[cfg(target_os = "macos")]
fn native_backend() -> Result<Arc<Backend>, CredentialError> {
    let store: Arc<Backend> =
        apple_native_keyring_store::keychain::Store::new().map_err(classify)?;
    Ok(store)
}

#[cfg(windows)]
fn native_backend() -> Result<Arc<Backend>, CredentialError> {
    let store: Arc<Backend> = windows_native_keyring_store::Store::new().map_err(classify)?;
    Ok(store)
}

#[cfg(not(any(target_os = "macos", windows)))]
fn native_backend() -> Result<Arc<Backend>, CredentialError> {
    Err(CredentialError::Failed(
        "no supported credential store on this platform".to_owned(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock() -> CredentialStore {
        CredentialStore::with_backend(keyring_core::mock::Store::new().unwrap())
    }

    #[test]
    fn missing_secrets_read_as_absent_and_deleting_them_is_harmless() {
        let store = mock();
        assert_eq!(store.read("server"), Ok(None));
        assert_eq!(store.delete("server"), Ok(()));
    }

    #[test]
    fn a_written_secret_reads_back_until_deleted() {
        let store = mock();
        store.write("server", "token-Ђ").unwrap();
        assert_eq!(store.read("server"), Ok(Some("token-Ђ".to_owned())));
        store.delete("server").unwrap();
        assert_eq!(store.read("server"), Ok(None));
    }

    #[test]
    fn access_denial_is_distinguished_from_other_failures() {
        let denied = classify(Error::NoStorageAccess("keychain is locked".into()));
        assert_eq!(
            denied,
            CredentialError::Denied("keychain is locked".to_owned())
        );
        assert!(denied.to_string().contains("denied access"));
        assert!(matches!(
            classify(Error::PlatformFailure("user canceled".into())),
            CredentialError::Failed(message) if message.contains("user canceled")
        ));
    }
}

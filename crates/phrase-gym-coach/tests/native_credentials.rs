#![cfg(any(target_os = "macos", windows))]

use phrase_gym_coach::platform::credentials::CredentialStore;

// Removes the synthetic entry even when an assertion fails midway.
struct Cleanup<'a> {
    store: &'a CredentialStore,
    account: &'a str,
}

impl Drop for Cleanup<'_> {
    fn drop(&mut self) {
        let _ = self.store.delete(self.account);
    }
}

#[test]
fn the_os_credential_store_round_trips_a_synthetic_secret() {
    let store = CredentialStore::native().unwrap();
    let account = format!("coach-test-{}", std::process::id());
    let _cleanup = Cleanup {
        store: &store,
        account: &account,
    };
    assert_eq!(store.read(&account), Ok(None));
    store.write(&account, "synthetic-secret-Ђ").unwrap();
    assert_eq!(
        store.read(&account),
        Ok(Some("synthetic-secret-Ђ".to_owned()))
    );
    store.delete(&account).unwrap();
    assert_eq!(store.read(&account), Ok(None));
}

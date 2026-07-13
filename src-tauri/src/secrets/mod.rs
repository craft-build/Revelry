use anyhow::Result;
use keyring::v1::Entry;

/// Manages API keys in the OS-native credential store.
///
/// API keys are stored exclusively in the OS keychain (macOS Keychain,
/// GNOME Keyring/Secret Service, Windows Credential Manager) and NEVER in
/// the database. This is a core security invariant (PROV-04).
///
/// All keyring operations are synchronous and MUST run via
/// `tokio::task::spawn_blocking()` to avoid blocking the Tauri async runtime.
pub struct KeyringStore;

impl KeyringStore {
    /// Service name used as the keyring namespace.
    /// All Revelry API keys are stored under this service.
    const SERVICE_NAME: &'static str = "revelry";

    /// Saves an API key to the OS keychain for the given provider ID.
    ///
    /// If an entry already exists for this provider, it will be overwritten.
    pub fn save_key(provider_id: &str, api_key: &str) -> Result<()> {
        let entry = Entry::new(Self::SERVICE_NAME, provider_id)?;
        entry.set_password(api_key)?;
        Ok(())
    }

    /// Loads an API key from the OS keychain for the given provider ID.
    ///
    /// Returns an error if no key exists for this provider or if the
    /// keychain is unavailable (e.g., headless Linux without a credential manager).
    pub fn load_key(provider_id: &str) -> Result<String> {
        let entry = Entry::new(Self::SERVICE_NAME, provider_id)?;
        let password = entry.get_password()?;
        Ok(password)
    }

    /// Deletes the API key from the OS keychain for the given provider ID.
    ///
    /// Returns an error if no entry exists. Deleting a non-existent entry
    /// returns a `NoEntry` error from the keyring crate.
    pub fn delete_key(provider_id: &str) -> Result<()> {
        let entry = Entry::new(Self::SERVICE_NAME, provider_id)?;
        entry.delete_credential()?;
        Ok(())
    }

    /// Check if the OS keyring is available by attempting
    /// a round-trip: create → write → read → delete a test entry.
    ///
    /// Per D-10 and UI-SPEC Decision 7: Check once on app startup.
    /// Cache the result in frontend state. Show a persistent banner
    /// in settings if unavailable.
    pub fn is_available() -> bool {
        let test_user = "__keyring_test__";
        if let Ok(entry) = Entry::new(Self::SERVICE_NAME, test_user) {
            if entry.set_password("availability-check").is_ok() {
                // Verify we can read it back
                if entry.get_password().is_ok() {
                    // Clean up the test entry
                    let _ = entry.delete_credential();
                    return true;
                }
                let _ = entry.delete_credential();
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    /// Initialize keyring store before any tests run.
    /// keyring 4.1+ initializes lazily; this is now a no-op kept for call-site
    /// stability.
    fn ensure_keyring_init() {
        // No-op: keyring 4.1+ does not require explicit store initialization.
    }

    /// Helper to create a unique test provider ID that won't conflict
    /// with real keys or other test runs.
    fn test_provider_id() -> String {
        format!("test-provider-{}", Uuid::new_v4())
    }

    #[test]
    fn test_save_and_load() {
        ensure_keyring_init();
        let provider_id = test_provider_id();
        let api_key = "sk-test-key-12345";

        // Save the key
        KeyringStore::save_key(&provider_id, api_key).expect("save_key should succeed");

        // Load it back
        let loaded =
            KeyringStore::load_key(&provider_id).expect("load_key should succeed after saving");
        assert_eq!(loaded, api_key, "loaded key should match saved key");

        // Clean up
        let _ = KeyringStore::delete_key(&provider_id);
    }

    #[test]
    fn test_delete_removes_key() {
        ensure_keyring_init();
        let provider_id = test_provider_id();
        let api_key = "sk-test-key-to-delete";

        // Save the key
        KeyringStore::save_key(&provider_id, api_key).expect("save_key should succeed");

        // Delete it
        KeyringStore::delete_key(&provider_id).expect("delete_key should succeed for existing key");

        // Verify load fails
        let result = KeyringStore::load_key(&provider_id);
        assert!(result.is_err(), "load_key should fail after deletion");
    }

    #[test]
    fn test_load_nonexistent() {
        ensure_keyring_init();
        let provider_id = format!("nonexistent-{}", Uuid::new_v4());

        let result = KeyringStore::load_key(&provider_id);
        assert!(
            result.is_err(),
            "load_key should fail for non-existent entry"
        );
    }
}

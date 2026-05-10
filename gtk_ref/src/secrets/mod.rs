use anyhow::Result;
use keyring::Entry;

/// Manages API keys in the OS-native credential store.
///
/// API keys are stored exclusively in the OS keychain (macOS Keychain,
/// GNOME Keyring, Windows Credential Manager) and NEVER in the database.
/// This is a core security invariant (PROV-05, D-16).
///
/// All keyring operations are synchronous and MUST run on a background thread
/// via `spawn_oneshot_command()` to avoid blocking the GTK main loop.
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    /// Helper to create a unique test provider ID that won't conflict
    /// with real keys or other test runs.
    fn test_provider_id() -> String {
        format!("test-provider-{}", Uuid::new_v4())
    }

    #[test]
    fn test_save_and_load() {
        let provider_id = test_provider_id();
        let api_key = "sk-test-key-12345";

        // Save the key
        KeyringStore::save_key(&provider_id, api_key)
            .expect("save_key should succeed");

        // Load it back
        let loaded = KeyringStore::load_key(&provider_id)
            .expect("load_key should succeed after saving");
        assert_eq!(loaded, api_key, "loaded key should match saved key");

        // Clean up
        let _ = KeyringStore::delete_key(&provider_id);
    }

    #[test]
    fn test_delete_removes_key() {
        let provider_id = test_provider_id();
        let api_key = "sk-test-key-to-delete";

        // Save the key
        KeyringStore::save_key(&provider_id, api_key)
            .expect("save_key should succeed");

        // Delete it
        KeyringStore::delete_key(&provider_id)
            .expect("delete_key should succeed for existing key");

        // Verify load fails
        let result = KeyringStore::load_key(&provider_id);
        assert!(result.is_err(), "load_key should fail after deletion");
    }

    #[test]
    fn test_load_nonexistent() {
        let provider_id = format!("nonexistent-{}", Uuid::new_v4());

        let result = KeyringStore::load_key(&provider_id);
        assert!(result.is_err(), "load_key should fail for non-existent entry");
    }
}

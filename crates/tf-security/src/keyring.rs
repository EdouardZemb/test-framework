//! Secure secret storage using OS keyring.
//!
//! This module provides a safe interface to the operating system's credential store:
//! - Linux: gnome-keyring, kwallet (via Secret Service D-Bus API)
//! - macOS: Keychain Access
//! - Windows: Credential Manager
//!
//! # Security Guarantees
//!
//! - Secrets are stored in the OS keyring, not in files or environment variables
//! - Secret values are NEVER logged (Debug impl is safe)
//! - All errors include actionable hints without exposing secret values
//!
//! # Example
//!
//! ```no_run
//! use tf_security::{SecretStore, SecretError};
//!
//! fn main() -> Result<(), SecretError> {
//!     let store = SecretStore::new("my-app");
//!
//!     // Store a secret
//!     store.store_secret("api-token", "secret-value")?;
//!
//!     // Retrieve it later
//!     let token = store.get_secret("api-token")?;
//!
//!     // Check if a secret exists (simple version)
//!     if store.has_secret("optional-key") {
//!         // ...
//!     }
//!
//!     // Check with error handling (for distinguishing keyring errors)
//!     match store.try_has_secret("optional-key")? {
//!         true => println!("Secret exists"),
//!         false => println!("Secret not found"),
//!     }
//!
//!     // Delete when no longer needed
//!     store.delete_secret("api-token")?;
//!     Ok(())
//! }
//! ```

use keyring::Entry;

use crate::error::SecretError;

/// Secure storage for secrets using the OS keyring.
///
/// Each `SecretStore` is scoped to a service name, which acts as a namespace
/// for secrets. Typically, use your application name as the service name.
///
/// # Thread Safety
///
/// `SecretStore` is `Send + Sync` and can be safely shared across threads.
/// The underlying OS keyring handles concurrent access.
///
/// # Security
///
/// The `Debug` implementation for this struct intentionally does NOT expose
/// any secret values or internal state that could leak sensitive information.
pub struct SecretStore {
    service_name: String,
}

// Custom Debug implementation that doesn't expose secrets
impl std::fmt::Debug for SecretStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretStore")
            .field("service_name", &self.service_name)
            .finish()
    }
}

impl SecretStore {
    /// Create a new secret store for the given service.
    ///
    /// # Arguments
    ///
    /// * `service_name` - A unique identifier for your application/service.
    ///   This acts as a namespace to avoid conflicts with other applications.
    ///   Should be non-empty; empty service names may cause unexpected behavior
    ///   depending on the OS keyring implementation.
    ///
    /// # Example
    ///
    /// ```
    /// use tf_security::SecretStore;
    ///
    /// let store = SecretStore::new("test-framework");
    /// assert_eq!(store.service_name(), "test-framework");
    /// ```
    ///
    /// # Note
    ///
    /// Empty service names are allowed but not recommended. The behavior with
    /// empty service names depends on the underlying OS keyring implementation.
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
        }
    }

    /// Store a secret in the OS keyring.
    ///
    /// If a secret with the same key already exists, it will be overwritten.
    ///
    /// # Arguments
    ///
    /// * `key` - The identifier for the secret (e.g., "jira-token", "db-password")
    /// * `value` - The secret value to store
    ///
    /// # Errors
    ///
    /// Returns `SecretError` if:
    /// - The keyring is unavailable (`KeyringUnavailable`)
    /// - Permission is denied (`AccessDenied`)
    /// - Storage fails for other reasons (`StoreFailed`)
    ///
    /// # Security
    ///
    /// The `value` is passed directly to the OS keyring and is NEVER logged.
    pub fn store_secret(&self, key: &str, value: &str) -> Result<(), SecretError> {
        let entry = Entry::new(&self.service_name, key)
            .map_err(|e| SecretError::from_keyring_error(e, key))?;

        entry
            .set_password(value)
            .map_err(|e| SecretError::from_keyring_error(e, key))
    }

    /// Retrieve a secret from the OS keyring.
    ///
    /// # Arguments
    ///
    /// * `key` - The identifier for the secret
    ///
    /// # Errors
    ///
    /// Returns `SecretError` if:
    /// - The secret is not found (`SecretNotFound`)
    /// - The keyring is unavailable (`KeyringUnavailable`)
    /// - Permission is denied (`AccessDenied`)
    pub fn get_secret(&self, key: &str) -> Result<String, SecretError> {
        let entry = Entry::new(&self.service_name, key)
            .map_err(|e| SecretError::from_keyring_error(e, key))?;

        entry
            .get_password()
            .map_err(|e| SecretError::from_keyring_error(e, key))
    }

    /// Delete a secret from the OS keyring.
    ///
    /// # Arguments
    ///
    /// * `key` - The identifier for the secret to delete
    ///
    /// # Errors
    ///
    /// Returns `SecretError` if:
    /// - The secret is not found (`SecretNotFound`)
    /// - The keyring is unavailable (`KeyringUnavailable`)
    /// - Permission is denied (`AccessDenied`)
    ///
    /// # Note
    ///
    /// If you don't care whether the secret existed, use `has_secret` first
    /// or ignore the `SecretNotFound` error.
    pub fn delete_secret(&self, key: &str) -> Result<(), SecretError> {
        let entry = Entry::new(&self.service_name, key)
            .map_err(|e| SecretError::from_keyring_error(e, key))?;

        entry
            .delete_credential()
            .map_err(|e| SecretError::from_keyring_error(e, key))
    }

    /// Check if a secret exists in the keyring.
    ///
    /// This is a convenience method that attempts to retrieve the secret
    /// and returns `true` if successful, `false` otherwise.
    ///
    /// # Arguments
    ///
    /// * `key` - The identifier for the secret
    ///
    /// # Note
    ///
    /// This method will return `false` for any error (not found, unavailable, denied).
    /// Use [`try_has_secret`](Self::try_has_secret) if you need to distinguish between
    /// "not found" and other errors.
    pub fn has_secret(&self, key: &str) -> bool {
        self.get_secret(key).is_ok()
    }

    /// Check if a secret exists, with full error reporting.
    ///
    /// Unlike [`has_secret`](Self::has_secret), this method returns a `Result`
    /// that distinguishes between:
    /// - `Ok(true)` - the secret exists
    /// - `Ok(false)` - the secret does not exist (keyring accessible, key not found)
    /// - `Err(...)` - keyring unavailable, access denied, or other errors
    ///
    /// # Arguments
    ///
    /// * `key` - The identifier for the secret
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tf_security::SecretStore;
    ///
    /// let store = SecretStore::new("my-app");
    /// match store.try_has_secret("api-token") {
    ///     Ok(true) => println!("Secret exists"),
    ///     Ok(false) => println!("Secret not found"),
    ///     Err(e) => eprintln!("Keyring error: {}", e),
    /// }
    /// ```
    pub fn try_has_secret(&self, key: &str) -> Result<bool, SecretError> {
        match self.get_secret(key) {
            Ok(_) => Ok(true),
            Err(SecretError::SecretNotFound { .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Get the service name this store is scoped to.
    ///
    /// This is useful for debugging and logging (safe to log).
    pub fn service_name(&self) -> &str {
        &self.service_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // TEST CONSTANTS
    // ============================================================

    /// Service name for tests - unique to avoid conflicts
    const TEST_SERVICE: &str = "tf-security-test";

    /// Generate a unique key for each test to avoid conflicts in parallel runs
    fn unique_key(base: &str) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{}-{}", base, timestamp)
    }

    // ============================================================
    // AC #1: Keyring disponible → secret stocké et récupérable
    // ============================================================

    /// Test: Store a secret successfully
    ///
    /// Given: un keyring disponible
    /// When: j'enregistre un secret
    /// Then: store_secret retourne Ok
    #[test]
    #[ignore = "Requires OS keyring - run manually or in CI with keyring available"]
    fn test_store_secret_success() {
        let store = SecretStore::new(TEST_SERVICE);
        let key = unique_key("store-test");

        // When: store a secret
        let result = store.store_secret(&key, "test-value");

        // Then: operation succeeds
        assert!(result.is_ok(), "store_secret should succeed: {:?}", result);

        // Cleanup
        let _ = store.delete_secret(&key);
    }

    /// Test: Retrieve a stored secret
    ///
    /// Given: un secret stocké dans le keyring
    /// When: je le récupère
    /// Then: la valeur est identique
    #[test]
    #[ignore = "Requires OS keyring - run manually or in CI with keyring available"]
    fn test_get_secret_success() {
        let store = SecretStore::new(TEST_SERVICE);
        let key = unique_key("get-test");
        let value = "my-secret-value-12345";

        // Given: a stored secret
        store.store_secret(&key, value).expect("Setup failed");

        // When: retrieve it
        let result = store.get_secret(&key);

        // Then: value matches
        assert!(result.is_ok(), "get_secret should succeed: {:?}", result);
        assert_eq!(
            result.unwrap(),
            value,
            "Retrieved value should match stored value"
        );

        // Cleanup
        let _ = store.delete_secret(&key);
    }

    /// Test: Store then get roundtrip
    ///
    /// Given: un keyring disponible
    /// When: je stocke puis récupère un secret
    /// Then: la valeur récupérée est identique
    #[test]
    #[ignore = "Requires OS keyring - run manually or in CI with keyring available"]
    fn test_store_get_roundtrip() {
        let store = SecretStore::new(TEST_SERVICE);
        let key = unique_key("roundtrip-test");
        let original_value = "roundtrip-secret-!@#$%^&*()";

        // When: store
        let store_result = store.store_secret(&key, original_value);
        assert!(store_result.is_ok(), "Store should succeed");

        // When: retrieve
        let get_result = store.get_secret(&key);
        assert!(get_result.is_ok(), "Get should succeed");

        // Then: values match
        assert_eq!(
            get_result.unwrap(),
            original_value,
            "Roundtrip should preserve value exactly"
        );

        // Cleanup
        let _ = store.delete_secret(&key);
    }

    /// Test: Delete a secret successfully
    ///
    /// Given: un secret existant
    /// When: je le supprime
    /// Then: delete_secret retourne Ok
    #[test]
    #[ignore = "Requires OS keyring - run manually or in CI with keyring available"]
    fn test_delete_secret_success() {
        let store = SecretStore::new(TEST_SERVICE);
        let key = unique_key("delete-test");

        // Given: a stored secret
        store.store_secret(&key, "to-delete").expect("Setup failed");

        // When: delete it
        let result = store.delete_secret(&key);

        // Then: operation succeeds
        assert!(result.is_ok(), "delete_secret should succeed: {:?}", result);

        // Verify it's actually gone
        assert!(
            !store.has_secret(&key),
            "Secret should no longer exist after deletion"
        );
    }

    /// Test: has_secret returns true for existing secret
    ///
    /// Given: un secret existant
    /// When: j'appelle has_secret
    /// Then: retourne true
    #[test]
    #[ignore = "Requires OS keyring - run manually or in CI with keyring available"]
    fn test_has_secret_true() {
        let store = SecretStore::new(TEST_SERVICE);
        let key = unique_key("has-test");

        // Given: a stored secret
        store.store_secret(&key, "exists").expect("Setup failed");

        // When/Then: has_secret returns true
        assert!(
            store.has_secret(&key),
            "has_secret should return true for existing secret"
        );

        // Cleanup
        let _ = store.delete_secret(&key);
    }

    /// Test: has_secret returns false for non-existent secret
    ///
    /// Given: une clé qui n'existe pas
    /// When: j'appelle has_secret
    /// Then: retourne false
    ///
    /// Note: This test passes without keyring because has_secret returns false
    /// for ANY error (including keyring unavailable). For proper "not found"
    /// semantics with keyring, use try_has_secret.
    #[test]
    #[ignore = "Requires OS keyring - run manually or in CI with keyring available"]
    fn test_has_secret_false() {
        let store = SecretStore::new(TEST_SERVICE);
        let key = unique_key("nonexistent");

        // When/Then: has_secret returns false
        assert!(
            !store.has_secret(&key),
            "has_secret should return false for non-existent secret"
        );
    }

    /// Test: try_has_secret returns Ok(false) for non-existent secret
    ///
    /// Given: une clé qui n'existe pas et un keyring fonctionnel
    /// When: j'appelle try_has_secret
    /// Then: retourne Ok(false)
    #[test]
    #[ignore = "Requires OS keyring - run manually or in CI with keyring available"]
    fn test_try_has_secret_not_found() {
        let store = SecretStore::new(TEST_SERVICE);
        let key = unique_key("try-nonexistent");

        // When: try_has_secret for non-existent key
        let result = store.try_has_secret(&key);

        // Then: returns Ok(false), not an error
        assert!(result.is_ok(), "try_has_secret should return Ok, not Err");
        assert!(
            !result.unwrap(),
            "try_has_secret should return Ok(false) for missing key"
        );
    }

    /// Test: try_has_secret returns Ok(true) for existing secret
    ///
    /// Given: un secret existant
    /// When: j'appelle try_has_secret
    /// Then: retourne Ok(true)
    #[test]
    #[ignore = "Requires OS keyring - run manually or in CI with keyring available"]
    fn test_try_has_secret_found() {
        let store = SecretStore::new(TEST_SERVICE);
        let key = unique_key("try-exists");

        // Given: a stored secret
        store.store_secret(&key, "exists").expect("Setup failed");

        // When: try_has_secret for existing key
        let result = store.try_has_secret(&key);

        // Then: returns Ok(true)
        assert!(result.is_ok(), "try_has_secret should return Ok");
        assert!(
            result.unwrap(),
            "try_has_secret should return Ok(true) for existing key"
        );

        // Cleanup
        let _ = store.delete_secret(&key);
    }

    /// Test: Overwriting an existing secret
    ///
    /// Given: un secret existant
    /// When: je stocke une nouvelle valeur avec la même clé
    /// Then: la nouvelle valeur remplace l'ancienne
    #[test]
    #[ignore = "Requires OS keyring - run manually or in CI with keyring available"]
    fn test_overwrite_existing_secret() {
        let store = SecretStore::new(TEST_SERVICE);
        let key = unique_key("overwrite-test");

        // Given: an existing secret
        store
            .store_secret(&key, "original-value")
            .expect("Setup failed");

        // When: overwrite with new value
        let new_value = "new-value-replaced";
        let result = store.store_secret(&key, new_value);
        assert!(result.is_ok(), "Overwrite should succeed");

        // Then: new value is returned
        let retrieved = store.get_secret(&key).expect("Get should succeed");
        assert_eq!(
            retrieved, new_value,
            "Should return new value after overwrite"
        );

        // Cleanup
        let _ = store.delete_secret(&key);
    }

    // ============================================================
    // AC #2: Keyring indisponible → message explicite avec hint
    // ============================================================

    /// Test: Getting a non-existent secret returns SecretNotFound with hint
    ///
    /// Given: une clé qui n'existe pas
    /// When: j'essaie de la récupérer
    /// Then: erreur SecretNotFound avec hint actionnable
    #[test]
    #[ignore = "Requires OS keyring - run manually or in CI with keyring available"]
    fn test_secret_not_found_error() {
        let store = SecretStore::new(TEST_SERVICE);
        let key = unique_key("not-found-test");

        // When: try to get non-existent secret
        let result = store.get_secret(&key);

        // Then: SecretNotFound error with hint
        assert!(result.is_err(), "Should fail for non-existent secret");
        let err = result.unwrap_err();
        match err {
            SecretError::SecretNotFound { key: k, hint } => {
                assert_eq!(k, key, "Error should contain the key name");
                assert!(
                    hint.contains("tf secret set"),
                    "Hint should contain CLI command: {}",
                    hint
                );
            }
            _ => panic!("Expected SecretNotFound, got {:?}", err),
        }
    }

    /// Test: Deleting a non-existent secret returns SecretNotFound
    ///
    /// Given: une clé qui n'existe pas
    /// When: j'essaie de la supprimer
    /// Then: erreur SecretNotFound
    #[test]
    #[ignore = "Requires OS keyring - run manually or in CI with keyring available"]
    fn test_delete_nonexistent_secret() {
        let store = SecretStore::new(TEST_SERVICE);
        let key = unique_key("delete-nonexistent");

        // When: try to delete non-existent secret
        let result = store.delete_secret(&key);

        // Then: SecretNotFound error
        assert!(result.is_err(), "Should fail for non-existent secret");
        match result.unwrap_err() {
            SecretError::SecretNotFound { .. } => (), // Expected
            other => panic!("Expected SecretNotFound, got {:?}", other),
        }
    }

    // ============================================================
    // AC #3: Logs sans donnée sensible
    // ============================================================

    /// Test: Debug implementation doesn't expose secrets
    ///
    /// Given: un SecretStore
    /// When: on utilise Debug
    /// Then: aucune valeur secrète n'est exposée
    #[test]
    fn test_debug_impl_no_secrets() {
        let store = SecretStore::new("my-service");

        // When: format with Debug
        let debug_str = format!("{:?}", store);

        // Then: only contains service name, no internal state
        assert!(
            debug_str.contains("SecretStore"),
            "Should contain struct name"
        );
        assert!(
            debug_str.contains("my-service"),
            "Should contain service name (safe to log)"
        );
        // Should NOT contain any secret values (there are none in the struct anyway)
        // This test documents the safe Debug behavior
    }

    /// Test: Service name is accessible and safe to log
    ///
    /// Given: un SecretStore
    /// When: j'accède au service_name
    /// Then: c'est le nom configuré (safe to log)
    #[test]
    fn test_service_name_accessible() {
        let store = SecretStore::new("test-app");

        // When/Then: service name is accessible
        assert_eq!(store.service_name(), "test-app");
    }

    // ============================================================
    // EDGE CASES
    // ============================================================

    /// Test: Empty service name handling
    ///
    /// Given: un SecretStore avec service_name vide
    /// When: on crée le store
    /// Then: le store est créé (behavior depends on OS keyring)
    #[test]
    fn test_empty_service_name() {
        // Empty service names are allowed at construction time
        let store = SecretStore::new("");

        // Should create successfully
        assert_eq!(
            store.service_name(),
            "",
            "Empty service name should be preserved"
        );

        // Debug output should still work
        let debug_str = format!("{:?}", store);
        assert!(
            debug_str.contains("SecretStore"),
            "Debug should contain struct name"
        );
    }

    /// Test: Empty key handling
    #[test]
    #[ignore = "Requires OS keyring - run manually or in CI with keyring available"]
    fn test_empty_key() {
        let store = SecretStore::new(TEST_SERVICE);

        // Empty keys might be rejected by the OS keyring
        // This test documents the behavior
        let result = store.store_secret("", "value");
        // Either succeeds or fails with a meaningful error
        if let Err(e) = result {
            // Error should still have a hint
            let msg = e.to_string();
            assert!(!msg.is_empty(), "Error message should not be empty");
        }
    }

    /// Test: Special characters in key and value
    #[test]
    #[ignore = "Requires OS keyring - run manually or in CI with keyring available"]
    fn test_special_characters() {
        let store = SecretStore::new(TEST_SERVICE);
        let key = unique_key("special-key-émoji-🔐");
        let value = "secret-with-émoji-🔑-and-special-chars-!@#$%^&*()";

        // When: store and retrieve with special characters
        store
            .store_secret(&key, value)
            .expect("Store should succeed");
        let retrieved = store.get_secret(&key).expect("Get should succeed");

        // Then: values match exactly
        assert_eq!(retrieved, value, "Special characters should be preserved");

        // Cleanup
        let _ = store.delete_secret(&key);
    }

    /// Test: Long value handling
    #[test]
    #[ignore = "Requires OS keyring - run manually or in CI with keyring available"]
    fn test_long_value() {
        let store = SecretStore::new(TEST_SERVICE);
        let key = unique_key("long-value-test");
        let value = "x".repeat(10_000); // 10KB secret

        // When: store and retrieve long value
        store
            .store_secret(&key, &value)
            .expect("Store should succeed");
        let retrieved = store.get_secret(&key).expect("Get should succeed");

        // Then: values match
        assert_eq!(retrieved, value, "Long values should be preserved");

        // Cleanup
        let _ = store.delete_secret(&key);
    }

    // ============================================================
    // CONSTRUCTOR TESTS (no keyring required)
    // ============================================================

    /// Test: new() creates a SecretStore with the given service name
    ///
    /// Given: un nom de service quelconque
    /// When: on crée un SecretStore
    /// Then: service_name() retourne la valeur donnée
    #[test]
    fn test_new_creates_store_with_correct_service_name() {
        let store = SecretStore::new("my-test-service");
        assert_eq!(store.service_name(), "my-test-service");
    }

    /// Test: new() with different service names produces distinct stores
    ///
    /// Given: deux noms de service différents
    /// When: on crée deux SecretStores
    /// Then: chacun retourne son propre service_name
    #[test]
    fn test_new_distinct_service_names() {
        let store_a = SecretStore::new("service-a");
        let store_b = SecretStore::new("service-b");

        assert_eq!(store_a.service_name(), "service-a");
        assert_eq!(store_b.service_name(), "service-b");
        assert_ne!(
            store_a.service_name(),
            store_b.service_name(),
            "Different service names should produce distinct stores"
        );
    }

    /// Test: new() with long service name
    ///
    /// Given: un nom de service très long
    /// When: on crée un SecretStore
    /// Then: le nom est préservé intégralement
    #[test]
    fn test_new_with_long_service_name() {
        let long_name = "a".repeat(1000);
        let store = SecretStore::new(&long_name);

        assert_eq!(
            store.service_name(),
            long_name,
            "Long service name should be preserved exactly"
        );
        assert_eq!(store.service_name().len(), 1000);
    }

    /// Test: new() with unicode service name
    ///
    /// Given: un nom de service contenant des caractères Unicode
    /// When: on crée un SecretStore
    /// Then: le nom Unicode est préservé
    #[test]
    fn test_new_with_unicode_service_name() {
        let store = SecretStore::new("service-émoji-🔐-日本語");
        assert_eq!(store.service_name(), "service-émoji-🔐-日本語");
    }

    /// Test: new() with whitespace service name
    ///
    /// Given: un nom de service contenant des espaces
    /// When: on crée un SecretStore
    /// Then: les espaces sont préservés
    #[test]
    fn test_new_with_whitespace_service_name() {
        let store = SecretStore::new("  my service  ");
        assert_eq!(
            store.service_name(),
            "  my service  ",
            "Whitespace should be preserved as-is"
        );
    }

    // ============================================================
    // DEBUG IMPL TESTS (no keyring required)
    // ============================================================

    /// Test: Debug output uses debug_struct format with field name
    ///
    /// Given: un SecretStore
    /// When: on utilise le format Debug
    /// Then: le format est structuré avec le nom du champ
    #[test]
    fn test_debug_format_contains_field_name() {
        let store = SecretStore::new("debug-test-svc");
        let debug_str = format!("{:?}", store);

        assert!(
            debug_str.contains("service_name"),
            "Debug output should contain the field name 'service_name': got '{}'",
            debug_str
        );
        assert!(
            debug_str.contains("debug-test-svc"),
            "Debug output should contain the service name value: got '{}'",
            debug_str
        );
    }

    /// Test: Debug alternate format (pretty-print)
    ///
    /// Given: un SecretStore
    /// When: on utilise le format Debug alternatif {:#?}
    /// Then: le format est structuré et lisible
    #[test]
    fn test_debug_alternate_format() {
        let store = SecretStore::new("alt-debug-svc");
        let debug_str = format!("{:#?}", store);

        assert!(
            debug_str.contains("SecretStore"),
            "Alternate Debug should contain struct name: got '{}'",
            debug_str
        );
        assert!(
            debug_str.contains("service_name"),
            "Alternate Debug should contain field name: got '{}'",
            debug_str
        );
        assert!(
            debug_str.contains("alt-debug-svc"),
            "Alternate Debug should contain service name: got '{}'",
            debug_str
        );
    }

    /// Test: Debug output with empty service name
    ///
    /// Given: un SecretStore avec service_name vide
    /// When: on utilise Debug
    /// Then: le format est valide avec une chaîne vide
    #[test]
    fn test_debug_with_empty_service_name() {
        let store = SecretStore::new("");
        let debug_str = format!("{:?}", store);

        assert!(
            debug_str.contains("SecretStore"),
            "Debug should contain struct name even with empty service"
        );
        // The empty string should appear as \"\" in debug output
        assert!(
            debug_str.contains("service_name: \"\""),
            "Debug should show empty string for service_name: got '{}'",
            debug_str
        );
    }

    // ============================================================
    // API SIGNATURE / TYPE TESTS (no keyring required)
    // ============================================================

    /// Test: has_secret returns bool (not Result)
    ///
    /// Given: un SecretStore
    /// When: on appelle has_secret
    /// Then: le type de retour est bool (compilation check)
    #[test]
    #[ignore = "Requires OS keyring - run manually or in CI with keyring available"]
    fn test_has_secret_returns_bool() {
        let store = SecretStore::new(TEST_SERVICE);
        let key = unique_key("api-check-bool");

        // This is a compile-time type check: has_secret returns bool
        let result: bool = store.has_secret(&key);
        // Without keyring, has_secret returns false for any error
        assert!(!result, "Non-existent key should return false");
    }

    /// Test: try_has_secret returns Result<bool, SecretError>
    ///
    /// Given: un SecretStore
    /// When: on appelle try_has_secret
    /// Then: le type de retour est Result<bool, SecretError>
    #[test]
    #[ignore = "Requires OS keyring - run manually or in CI with keyring available"]
    fn test_try_has_secret_returns_result_bool() {
        let store = SecretStore::new(TEST_SERVICE);
        let key = unique_key("api-check-result");

        // This is a compile-time type check: try_has_secret returns Result<bool, SecretError>
        let result: Result<bool, SecretError> = store.try_has_secret(&key);
        // With keyring available, non-existent key returns Ok(false)
        assert!(
            result.is_ok(),
            "try_has_secret should return Ok for non-existent key"
        );
        assert!(!result.unwrap(), "Non-existent key should return Ok(false)");
    }

    /// Test: SecretStore is Send + Sync
    ///
    /// Given: le trait SecretStore
    /// When: on vérifie les traits auto-implémentés
    /// Then: Send et Sync sont implémentés (sécurité thread)
    #[test]
    fn test_secret_store_is_send_and_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<SecretStore>();
        assert_sync::<SecretStore>();
    }
}

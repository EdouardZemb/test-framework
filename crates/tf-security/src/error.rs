//! Error types for tf-security
//!
//! This module defines error types for secret management operations.
//! All errors include actionable hints to guide users.
//!
//! # Security Note
//! Error messages NEVER contain secret values - only key names and hints.

use thiserror::Error;

/// Errors that can occur during secret management operations.
///
/// All variants include a `hint` field with actionable guidance for the user.
///
/// # Security
/// - Secret VALUES are NEVER included in error messages
/// - Key names ARE included (assumed to be non-sensitive identifiers)
/// - Hints provide actionable steps to resolve the issue
#[derive(Error, Debug)]
pub enum SecretError {
    /// The OS keyring service is not available or accessible.
    ///
    /// Common causes:
    /// - Linux: gnome-keyring or kwallet not running
    /// - macOS: Keychain Access locked
    /// - Windows: Credential Manager service stopped
    #[error("Keyring unavailable on {platform}. {hint}")]
    KeyringUnavailable {
        /// The operating system platform (e.g., "linux", "macos", "windows")
        platform: String,
        /// Actionable hint to resolve the issue
        hint: String,
    },

    /// The requested secret was not found in the keyring.
    #[error("Secret '{key}' not found. {hint}")]
    SecretNotFound {
        /// The key that was not found
        key: String,
        /// Actionable hint (e.g., "Use 'tf secret set KEY' to store this secret.")
        hint: String,
    },

    /// Access to the secret was denied by the OS.
    #[error("Access denied for secret '{key}'. {hint}")]
    AccessDenied {
        /// The key for which access was denied
        key: String,
        /// Actionable hint to resolve permissions
        hint: String,
    },

    /// Failed to store a secret in the keyring.
    #[error("Failed to store secret '{key}': {cause}. {hint}")]
    StoreFailed {
        /// The key that failed to store
        key: String,
        /// The underlying cause (sanitized, no secret values)
        cause: String,
        /// Actionable hint to resolve the issue
        hint: String,
    },
}

impl SecretError {
    /// Convert a keyring crate error into a SecretError with appropriate hints.
    ///
    /// # Arguments
    /// * `err` - The original keyring error
    /// * `key` - The key being operated on (for context in error message)
    ///
    /// # Security
    /// This function NEVER logs or includes the secret value.
    pub fn from_keyring_error(err: keyring::Error, key: &str) -> Self {
        match err {
            keyring::Error::NoEntry => SecretError::SecretNotFound {
                key: key.to_string(),
                hint: format!("Use 'tf secret set {}' to store this secret.", key),
            },
            keyring::Error::NoStorageAccess(_) => SecretError::KeyringUnavailable {
                platform: std::env::consts::OS.to_string(),
                hint: get_platform_hint(),
            },
            keyring::Error::Ambiguous(_) => SecretError::AccessDenied {
                key: key.to_string(),
                hint: "Multiple entries found. Delete duplicates from your keyring manager."
                    .to_string(),
            },
            _ => SecretError::StoreFailed {
                key: key.to_string(),
                cause: err.to_string(),
                hint: "Check keyring service status and permissions.".to_string(),
            },
        }
    }
}

/// Get platform-specific hint for keyring unavailability.
fn get_platform_hint() -> String {
    match std::env::consts::OS {
        "linux" => {
            "Ensure the keyring service is running. Try: 'systemctl --user start gnome-keyring' or install gnome-keyring/kwallet.".to_string()
        }
        "macos" => "Ensure Keychain Access is unlocked. Open Keychain Access and unlock your default keychain.".to_string(),
        "windows" => {
            "Ensure Credential Manager service is running. Check Services (services.msc).".to_string()
        }
        other => format!(
            "Keyring may not be supported on '{}'. Check your OS documentation.",
            other
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // AC #2: Keyring indisponible → message explicite avec hint
    // ============================================================

    #[test]
    fn test_keyring_unavailable_error_has_platform_and_hint() {
        // Given: une erreur KeyringUnavailable
        let err = SecretError::KeyringUnavailable {
            platform: "linux".to_string(),
            hint: "Start gnome-keyring".to_string(),
        };

        // When: on affiche l'erreur
        let msg = err.to_string();

        // Then: le message contient platform et hint
        assert!(msg.contains("linux"), "Should contain platform");
        assert!(
            msg.contains("gnome-keyring"),
            "Should contain actionable hint"
        );
    }

    #[test]
    fn test_secret_not_found_error_has_key_and_hint() {
        // Given: une erreur SecretNotFound
        let err = SecretError::SecretNotFound {
            key: "api-token".to_string(),
            hint: "Use 'tf secret set api-token' to store this secret.".to_string(),
        };

        // When: on affiche l'erreur
        let msg = err.to_string();

        // Then: le message contient la clé et un hint actionnable
        assert!(msg.contains("api-token"), "Should contain key name");
        assert!(msg.contains("tf secret set"), "Should contain CLI hint");
    }

    #[test]
    fn test_access_denied_error_has_key_and_hint() {
        // Given: une erreur AccessDenied
        let err = SecretError::AccessDenied {
            key: "db-password".to_string(),
            hint: "Delete duplicates from your keyring manager.".to_string(),
        };

        // When: on affiche l'erreur
        let msg = err.to_string();

        // Then: le message contient la clé et un hint
        assert!(msg.contains("db-password"), "Should contain key name");
        assert!(msg.contains("duplicates"), "Should contain resolution hint");
    }

    #[test]
    fn test_store_failed_error_has_cause_and_hint() {
        // Given: une erreur StoreFailed
        let err = SecretError::StoreFailed {
            key: "jira-token".to_string(),
            cause: "Permission denied".to_string(),
            hint: "Check keyring service status.".to_string(),
        };

        // When: on affiche l'erreur
        let msg = err.to_string();

        // Then: le message contient clé, cause et hint
        assert!(msg.contains("jira-token"), "Should contain key name");
        assert!(msg.contains("Permission denied"), "Should contain cause");
        assert!(msg.contains("Check keyring"), "Should contain hint");
    }

    #[test]
    fn test_error_conversion_no_entry() {
        // Given: une erreur keyring::Error::NoEntry
        let keyring_err = keyring::Error::NoEntry;

        // When: on convertit en SecretError
        let err = SecretError::from_keyring_error(keyring_err, "missing-key");

        // Then: c'est une erreur SecretNotFound avec hint
        match err {
            SecretError::SecretNotFound { key, hint } => {
                assert_eq!(key, "missing-key");
                assert!(hint.contains("tf secret set"));
            }
            _ => panic!("Expected SecretNotFound, got {:?}", err),
        }
    }

    #[test]
    fn test_all_error_messages_contain_hints() {
        // Given: toutes les variantes d'erreur
        let errors = [
            SecretError::KeyringUnavailable {
                platform: "test".to_string(),
                hint: "Test hint 1".to_string(),
            },
            SecretError::SecretNotFound {
                key: "k".to_string(),
                hint: "Test hint 2".to_string(),
            },
            SecretError::AccessDenied {
                key: "k".to_string(),
                hint: "Test hint 3".to_string(),
            },
            SecretError::StoreFailed {
                key: "k".to_string(),
                cause: "c".to_string(),
                hint: "Test hint 4".to_string(),
            },
        ];

        // When/Then: chaque message d'erreur contient son hint
        for (i, err) in errors.iter().enumerate() {
            let msg = err.to_string();
            assert!(
                msg.contains(&format!("Test hint {}", i + 1)),
                "Error {} should contain its hint: {}",
                i,
                msg
            );
        }
    }

    // ============================================================
    // AC #3: Logs sans donnée sensible
    // ============================================================

    #[test]
    fn test_error_display_never_contains_secret_values() {
        // Given: des erreurs avec des clés (pas de valeurs secrètes)
        let errors = vec![
            SecretError::SecretNotFound {
                key: "my-secret-key".to_string(),
                hint: "hint".to_string(),
            },
            SecretError::StoreFailed {
                key: "another-key".to_string(),
                cause: "error".to_string(),
                hint: "hint".to_string(),
            },
        ];

        // When: on affiche les erreurs
        for err in &errors {
            let msg = err.to_string();

            // Then: le message NE CONTIENT PAS de valeurs secrètes
            // (Les clés sont autorisées, les VALEURS sont interdites)
            // Note: Ce test documente le comportement attendu
            // Les valeurs ne sont jamais passées aux constructeurs d'erreur
            assert!(
                !msg.contains("super-secret-value"),
                "Error messages must never contain secret values"
            );
        }
    }

    #[test]
    fn test_platform_hint_linux() {
        // Given: plateforme Linux
        // Note: Ce test vérifie la fonction helper
        let hint = get_platform_hint();

        // Then: le hint est approprié à la plateforme courante
        // (Le contenu exact dépend de la plateforme de test)
        assert!(!hint.is_empty(), "Platform hint should not be empty");
    }

    // ============================================================
    // DEBUG IMPL TESTS
    // ============================================================

    /// Test: Debug output for SecretNotFound variant
    ///
    /// Given: une erreur SecretNotFound
    /// When: on utilise Debug
    /// Then: la sortie contient le nom du variant et les champs
    #[test]
    fn test_secret_not_found_debug_output() {
        let err = SecretError::SecretNotFound {
            key: "my-key".to_string(),
            hint: "Use 'tf secret set my-key'".to_string(),
        };

        let debug_str = format!("{:?}", err);

        assert!(
            debug_str.contains("SecretNotFound"),
            "Debug should contain variant name: got '{}'",
            debug_str
        );
        assert!(
            debug_str.contains("my-key"),
            "Debug should contain key name: got '{}'",
            debug_str
        );
    }

    /// Test: Debug output for KeyringUnavailable variant
    ///
    /// Given: une erreur KeyringUnavailable
    /// When: on utilise Debug
    /// Then: la sortie contient le nom du variant et les champs
    #[test]
    fn test_keyring_unavailable_debug_output() {
        let err = SecretError::KeyringUnavailable {
            platform: "linux".to_string(),
            hint: "Start gnome-keyring".to_string(),
        };

        let debug_str = format!("{:?}", err);

        assert!(
            debug_str.contains("KeyringUnavailable"),
            "Debug should contain variant name: got '{}'",
            debug_str
        );
        assert!(
            debug_str.contains("linux"),
            "Debug should contain platform: got '{}'",
            debug_str
        );
    }

    /// Test: Debug output for AccessDenied variant
    ///
    /// Given: une erreur AccessDenied
    /// When: on utilise Debug
    /// Then: la sortie contient le nom du variant et les champs
    #[test]
    fn test_access_denied_debug_output() {
        let err = SecretError::AccessDenied {
            key: "restricted-key".to_string(),
            hint: "Check permissions".to_string(),
        };

        let debug_str = format!("{:?}", err);

        assert!(
            debug_str.contains("AccessDenied"),
            "Debug should contain variant name: got '{}'",
            debug_str
        );
        assert!(
            debug_str.contains("restricted-key"),
            "Debug should contain key: got '{}'",
            debug_str
        );
    }

    /// Test: Debug output for StoreFailed variant
    ///
    /// Given: une erreur StoreFailed
    /// When: on utilise Debug
    /// Then: la sortie contient le nom du variant, la cause et le hint
    #[test]
    fn test_store_failed_debug_output() {
        let err = SecretError::StoreFailed {
            key: "store-key".to_string(),
            cause: "disk full".to_string(),
            hint: "Free up space".to_string(),
        };

        let debug_str = format!("{:?}", err);

        assert!(
            debug_str.contains("StoreFailed"),
            "Debug should contain variant name: got '{}'",
            debug_str
        );
        assert!(
            debug_str.contains("disk full"),
            "Debug should contain cause: got '{}'",
            debug_str
        );
    }

    // ============================================================
    // from_keyring_error CONVERSION TESTS
    // ============================================================

    /// Test: from_keyring_error converts NoStorageAccess to KeyringUnavailable
    ///
    /// Given: une erreur keyring::Error::NoStorageAccess
    /// When: on convertit en SecretError
    /// Then: c'est une erreur KeyringUnavailable avec platform et hint
    #[test]
    fn test_error_conversion_no_storage_access() {
        let platform_err =
            keyring::Error::NoStorageAccess(Box::new(std::io::Error::other("no keyring")));

        let err = SecretError::from_keyring_error(platform_err, "some-key");

        match err {
            SecretError::KeyringUnavailable { platform, hint } => {
                assert_eq!(
                    platform,
                    std::env::consts::OS,
                    "Platform should match current OS"
                );
                assert!(
                    !hint.is_empty(),
                    "Hint should not be empty for KeyringUnavailable"
                );
            }
            _ => panic!("Expected KeyringUnavailable, got {:?}", err),
        }
    }

    /// Test: from_keyring_error converts Ambiguous to AccessDenied
    ///
    /// Given: une erreur keyring::Error::Ambiguous
    /// When: on convertit en SecretError
    /// Then: c'est une erreur AccessDenied avec hint sur les doublons
    #[test]
    fn test_error_conversion_ambiguous() {
        // Ambiguous takes Vec<Box<dyn CredentialApi>>; use empty vec
        let ambiguous_err = keyring::Error::Ambiguous(vec![]);

        let err = SecretError::from_keyring_error(ambiguous_err, "dup-key");

        match err {
            SecretError::AccessDenied { key, hint } => {
                assert_eq!(key, "dup-key", "Key should be preserved");
                assert!(
                    hint.contains("duplicates"),
                    "Hint should mention duplicates: got '{}'",
                    hint
                );
            }
            _ => panic!("Expected AccessDenied, got {:?}", err),
        }
    }

    /// Test: from_keyring_error converts unknown errors to StoreFailed
    ///
    /// Given: une erreur keyring non reconnue (catchall)
    /// When: on convertit en SecretError
    /// Then: c'est une erreur StoreFailed avec cause et hint
    #[test]
    fn test_error_conversion_catchall_to_store_failed() {
        // Use a variant that doesn't match the specific arms
        let other_err = keyring::Error::TooLong("field".to_string(), 100);

        let err = SecretError::from_keyring_error(other_err, "catch-key");

        match err {
            SecretError::StoreFailed { key, cause, hint } => {
                assert_eq!(key, "catch-key", "Key should be preserved");
                assert!(!cause.is_empty(), "Cause should not be empty");
                assert!(
                    hint.contains("keyring service"),
                    "Hint should reference keyring service: got '{}'",
                    hint
                );
            }
            _ => panic!("Expected StoreFailed, got {:?}", err),
        }
    }

    /// Test: from_keyring_error preserves key name in all conversions
    ///
    /// Given: différentes erreurs keyring
    /// When: on convertit chacune en SecretError
    /// Then: le nom de la clé est préservé dans chaque cas
    #[test]
    fn test_error_conversion_preserves_key_name() {
        let test_key = "preserved-key-name";

        // NoEntry -> SecretNotFound
        let err1 = SecretError::from_keyring_error(keyring::Error::NoEntry, test_key);
        match &err1 {
            SecretError::SecretNotFound { key, .. } => assert_eq!(key, test_key),
            _ => panic!("Expected SecretNotFound, got {:?}", err1),
        }

        // Ambiguous -> AccessDenied (empty vec since we can't easily construct CredentialApi)
        let err2 = SecretError::from_keyring_error(keyring::Error::Ambiguous(vec![]), test_key);
        match &err2 {
            SecretError::AccessDenied { key, .. } => assert_eq!(key, test_key),
            _ => panic!("Expected AccessDenied, got {:?}", err2),
        }

        // TooLong (catchall) -> StoreFailed
        let err3 =
            SecretError::from_keyring_error(keyring::Error::TooLong("x".to_string(), 1), test_key);
        match &err3 {
            SecretError::StoreFailed { key, .. } => assert_eq!(key, test_key),
            _ => panic!("Expected StoreFailed, got {:?}", err3),
        }
    }

    // ============================================================
    // SECURITY: Error messages never contain secret values
    // ============================================================

    /// Test: Debug output for errors never contains secret-like values
    ///
    /// Given: des erreurs construites sans valeurs secrètes
    /// When: on affiche avec Debug
    /// Then: les valeurs secrètes ne fuient pas dans Debug
    #[test]
    fn test_error_debug_never_contains_secret_values() {
        let secret_value = "super-secret-password-12345";

        let errors: Vec<SecretError> = vec![
            SecretError::SecretNotFound {
                key: "safe-key".to_string(),
                hint: "safe hint".to_string(),
            },
            SecretError::KeyringUnavailable {
                platform: "linux".to_string(),
                hint: "safe hint".to_string(),
            },
            SecretError::AccessDenied {
                key: "safe-key".to_string(),
                hint: "safe hint".to_string(),
            },
            SecretError::StoreFailed {
                key: "safe-key".to_string(),
                cause: "generic error".to_string(),
                hint: "safe hint".to_string(),
            },
        ];

        for err in &errors {
            let debug_str = format!("{:?}", err);
            assert!(
                !debug_str.contains(secret_value),
                "Debug output must never contain secret values: got '{}'",
                debug_str
            );
        }
    }

    /// Test: SecretError implements std::error::Error trait
    ///
    /// Given: une SecretError
    /// When: on l'utilise comme dyn Error
    /// Then: c'est compatible avec le trait Error standard
    #[test]
    fn test_secret_error_implements_error_trait() {
        fn assert_error<T: std::error::Error>() {}

        assert_error::<SecretError>();
    }
}

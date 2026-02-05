//! tf-security: Security module for test-framework
//!
//! This crate provides security-related functionality:
//!
//! - **Secret Management** (this story): Store and retrieve secrets via OS keyring
//! - **Anonymization** (story 0.7): Anonymize sensitive data before cloud operations
//! - **Redaction** (future): Redact sensitive data from logs
//!
//! # Secret Management
//!
//! Use `SecretStore` to securely store credentials:
//!
//! ```no_run
//! use tf_security::{SecretStore, SecretError};
//!
//! fn main() -> Result<(), SecretError> {
//!     let store = SecretStore::new("test-framework");
//!
//!     // Store a secret (e.g., Jira API token)
//!     store.store_secret("jira-token", "your-secret-token")?;
//!
//!     // Retrieve it later
//!     let token = store.get_secret("jira-token")?;
//!
//!     // Check if a secret exists before using
//!     if store.has_secret("optional-secret") {
//!         let value = store.get_secret("optional-secret")?;
//!     }
//!
//!     // Delete when no longer needed
//!     store.delete_secret("jira-token")?;
//!     Ok(())
//! }
//! ```
//!
//! # Configuration Integration
//!
//! Secrets can be referenced in configuration files using the pattern:
//!
//! ```yaml
//! # config.yaml
//! jira:
//!   token: ${SECRET:jira-token}
//!   url: https://jira.example.com
//! ```
//!
//! The `${SECRET:key_name}` syntax indicates that the value should be resolved
//! from the OS keyring at runtime. This pattern is **documented but not yet
//! implemented** - resolution will be added in a future story integrating
//! tf-config with tf-security.
//!
//! To store the secret referenced above:
//! ```bash
//! tf secret set jira-token
//! # You will be prompted to enter the secret value securely
//! ```
//!
//! # Platform Support
//!
//! | Platform | Backend |
//! |----------|---------|
//! | Linux | gnome-keyring, kwallet (Secret Service D-Bus API) |
//! | macOS | Keychain Access |
//! | Windows | Credential Manager |
//!
//! # Testing
//!
//! Most tests in this crate require a working OS keyring and are marked with
//! `#[ignore]` by default. To run all tests including integration tests:
//!
//! ```bash
//! # On a system with keyring available (native Linux/macOS/Windows, not WSL)
//! cargo test -p tf-security -- --include-ignored
//! ```
//!
//! In CI environments, ensure the keyring service is available:
//! - **Linux CI**: Install and start `gnome-keyring` with `dbus-run-session`
//! - **macOS CI**: Keychain is available by default
//! - **Windows CI**: Credential Manager is available by default
//!
//! For local development in WSL or environments without keyring, the unit tests
//! (error handling, Debug impl) run without the `--include-ignored` flag.
//!
//! # Security Guarantees
//!
//! - Secrets are stored in the OS keyring, never in files or environment variables
//! - Secret values are NEVER logged (Debug implementations are safe)
//! - All errors include actionable hints without exposing secret values
//! - Zero secrets in clair in the repository or configuration
//!
//! # Error Handling
//!
//! All operations return `Result<T, SecretError>` with actionable hints:
//!
//! ```no_run
//! use tf_security::{SecretStore, SecretError};
//!
//! let store = SecretStore::new("test-framework");
//!
//! match store.get_secret("api-token") {
//!     Ok(token) => println!("Got token"),
//!     Err(SecretError::SecretNotFound { key, hint }) => {
//!         eprintln!("Secret '{}' not found. {}", key, hint);
//!         // hint: "Use 'tf secret set api-token' to store this secret."
//!     }
//!     Err(SecretError::KeyringUnavailable { platform, hint }) => {
//!         eprintln!("Keyring unavailable on {}. {}", platform, hint);
//!         // hint: Platform-specific instructions
//!     }
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! ```

mod error;
mod keyring;

// Re-export public API
pub use error::SecretError;
pub use keyring::SecretStore;

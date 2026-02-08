//! Error types for configuration handling
//!
//! # Design Note
//!
//! The variant `InvalidValue` was chosen over the Dev Notes' suggested `ValidationError`
//! for semantic clarity: `InvalidValue` specifically describes the error condition (a value
//! that doesn't meet constraints), while `ValidationError` is more generic. This naming
//! aligns better with Rust conventions (e.g., `ParseError`, `IoError`) where variant names
//! describe the specific failure mode.

use std::path::PathBuf;

/// Formats the available profiles list for user-friendly error messages.
///
/// Returns a helpful message when no profiles are defined, or lists
/// the available profiles when some exist.
fn format_available_profiles(available: &[String]) -> String {
    if available.is_empty() {
        "No profiles defined in configuration. Add a 'profiles' section to config.yaml.".to_string()
    } else {
        format!("Available profiles: {}", available.join(", "))
    }
}

/// Errors that can occur when loading or validating configuration
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Configuration file not found at the specified path
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: PathBuf },

    /// A required field is missing from the configuration
    #[error("Invalid configuration: field '{field}' is missing. Expected: {hint}")]
    MissingField { field: String, hint: String },

    /// A field has an invalid value (named `InvalidValue` instead of `ValidationError`
    /// for semantic precision - see module-level documentation)
    #[error("Invalid configuration: field '{field}' {reason}. Expected: {hint}")]
    InvalidValue {
        field: String,
        reason: String,
        hint: String,
    },

    /// Requested profile does not exist in the configuration.
    ///
    /// This error occurs when `with_profile()` is called with a profile name
    /// that is not defined in the `profiles` section of the configuration.
    /// The error includes the list of available profiles to help the user
    /// select a valid one.
    #[error(
        "Profile '{requested}' not found. {}",
        format_available_profiles(available)
    )]
    ProfileNotFound {
        /// The profile name that was requested
        requested: String,
        /// List of available profile names from the configuration
        available: Vec<String>,
    },

    /// Failed to read the configuration file.
    ///
    /// Note: This variant is tested implicitly through the `From<std::io::Error>` derive.
    /// When `std::fs::read_to_string` fails in `load_config`, the `?` operator
    /// automatically converts the `std::io::Error` to `ConfigError::IoError`.
    /// The `FileNotFound` variant handles the explicit file existence check,
    /// while this variant catches other I/O errors (permission denied, etc.).
    #[error("Failed to read configuration file: {0}")]
    IoError(#[from] std::io::Error),

    /// Failed to parse the YAML configuration
    #[error("Failed to parse configuration: {0}")]
    ParseError(#[from] serde_yaml::Error),
}

impl ConfigError {
    /// Create a MissingField error
    pub fn missing_field(field: impl Into<String>, hint: impl Into<String>) -> Self {
        ConfigError::MissingField {
            field: field.into(),
            hint: hint.into(),
        }
    }

    /// Create an InvalidValue error
    pub fn invalid_value(
        field: impl Into<String>,
        reason: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        ConfigError::InvalidValue {
            field: field.into(),
            reason: reason.into(),
            hint: hint.into(),
        }
    }
}

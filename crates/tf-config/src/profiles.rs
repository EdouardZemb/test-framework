//! Configuration profiles management for environment-specific overrides.
//!
//! This module provides support for multiple configuration profiles,
//! allowing users to switch between different environments (dev, staging, prod).
//!
//! # Overview
//!
//! Profiles allow partial overrides of the base configuration. When a profile is applied:
//! - Fields specified in the profile override the base configuration values
//! - Fields not specified in the profile inherit from the base configuration
//! - The `active_profile` field is set to the selected profile name
//!
//! # Example
//!
//! ```yaml
//! project_name: "my-project"
//! output_folder: "./output"
//!
//! jira:
//!   endpoint: "https://jira.prod.example.com"
//!   token: "${JIRA_TOKEN}"
//!
//! profiles:
//!   dev:
//!     output_folder: "./dev-output"
//!     jira:
//!       endpoint: "https://jira.dev.example.com"
//!   staging:
//!     jira:
//!       endpoint: "https://jira.staging.example.com"
//! ```
//!
//! # Security
//!
//! All sensitive fields (tokens, passwords, API keys) are automatically redacted
//! in `Debug` output to prevent accidental logging of secrets.
//!
//! Use the [`Redact`] trait's `.redacted()` method for safe logging:
//!
//! ```
//! use tf_config::{ProfileOverride, JiraConfig, Redact};
//!
//! let profile = ProfileOverride {
//!     jira: Some(JiraConfig {
//!         endpoint: "https://jira.example.com".to_string(),
//!         token: Some("secret-token".to_string()),
//!     }),
//!     ..Default::default()
//! };
//!
//! // Safe for logging - secrets are redacted
//! let safe_output = profile.redacted();
//! assert!(safe_output.contains("[REDACTED]"));
//! assert!(!safe_output.contains("secret-token"));
//! ```

use crate::config::{JiraConfig, LlmConfig, Redact, SquashConfig, TemplatesConfig};
use serde::Deserialize;
use std::fmt;

/// Profile identifier type alias for clarity.
pub type ProfileId = String;

/// Configuration overrides for a specific profile.
///
/// Each field is optional - only specified fields will override the base configuration.
/// Fields set to `None` preserve the base configuration value (partial override pattern).
///
/// # Security
///
/// This struct implements a custom `Debug` trait that redacts sensitive information
/// (tokens, passwords, API keys) to prevent accidental logging of secrets.
///
/// # Example
///
/// ```
/// use tf_config::{ProfileOverride, JiraConfig};
///
/// let profile = ProfileOverride {
///     output_folder: Some("./dev-output".to_string()),
///     jira: Some(JiraConfig {
///         endpoint: "https://jira.dev.example.com".to_string(),
///         token: Some("secret-token".to_string()),
///     }),
///     ..Default::default()
/// };
/// ```
#[derive(Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileOverride {
    /// Override for output folder path.
    ///
    /// When set, replaces the base `output_folder` configuration.
    #[serde(default)]
    pub output_folder: Option<String>,

    /// Override for Jira integration configuration.
    ///
    /// When set, completely replaces the base Jira configuration.
    /// To partially update Jira settings, specify all desired values.
    #[serde(default)]
    pub jira: Option<JiraConfig>,

    /// Override for Squash integration configuration.
    ///
    /// When set, completely replaces the base Squash configuration.
    #[serde(default)]
    pub squash: Option<SquashConfig>,

    /// Override for LLM configuration.
    ///
    /// When set, completely replaces the base LLM configuration.
    #[serde(default)]
    pub llm: Option<LlmConfig>,

    /// Override for template file paths.
    ///
    /// When set, completely replaces the base templates configuration.
    #[serde(default)]
    pub templates: Option<TemplatesConfig>,
}

/// Custom Debug implementation that redacts sensitive information.
///
/// This ensures that tokens, passwords, and API keys are never accidentally
/// logged when a ProfileOverride is formatted with `{:?}`.
impl fmt::Debug for ProfileOverride {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProfileOverride")
            .field("output_folder", &self.output_folder)
            .field("jira", &self.jira) // JiraConfig has its own redacting Debug
            .field("squash", &self.squash) // SquashConfig has its own redacting Debug
            .field("llm", &self.llm) // LlmConfig has its own redacting Debug
            .field("templates", &self.templates)
            .finish()
    }
}

impl Redact for ProfileOverride {
    fn redacted(&self) -> String {
        format!("{:?}", self)
    }
}

// Note: Unit tests for ProfileOverride are in tests/profile_unit_tests.rs
// to avoid duplication and keep the source file focused on implementation.

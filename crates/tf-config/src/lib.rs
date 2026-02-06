#![forbid(unsafe_code)]
//! Configuration management for test-framework
//!
//! This crate provides configuration loading and validation for the test-framework CLI tool.
//!
//! # Features
//!
//! - **YAML Configuration Loading**: Load project configuration from YAML files with schema validation
//! - **Explicit Error Messages**: Validation errors include field name, reason, and correction hints
//! - **Sensitive Data Protection**: Secrets (tokens, passwords, API keys) are automatically redacted in logs
//! - **Flexible Schema**: Support for Jira, Squash, templates, and LLM integrations (all optional)
//!
//! # Quick Start
//!
//! ```no_run
//! use std::path::Path;
//! use tf_config::{load_config, ConfigError};
//!
//! fn main() -> Result<(), ConfigError> {
//!     // Load configuration from file
//!     let config = load_config(Path::new("config.yaml"))?;
//!
//!     println!("Project: {}", config.project_name);
//!     println!("Output: {}", config.output_folder);
//!
//!     // Check optional integrations
//!     if let Some(jira) = &config.jira {
//!         println!("Jira endpoint: {}", jira.endpoint);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Configuration Schema
//!
//! See [`ProjectConfig`] for the full schema. A minimal config requires only:
//!
//! ```yaml
//! project_name: "my-project"
//! output_folder: "./output"
//! ```
//!
//! # Redacting Sensitive Data
//!
//! Use the [`Redact`] trait to safely log configurations:
//!
//! ```no_run
//! use std::path::Path;
//! use tf_config::{load_config, Redact};
//!
//! let config = load_config(Path::new("config.yaml")).unwrap();
//! if let Some(jira) = &config.jira {
//!     // Safe for logging - tokens are replaced with [REDACTED]
//!     println!("{}", jira.redacted());
//! }
//! ```

pub mod config;
pub mod error;
pub mod profiles;
pub mod template;

pub use config::{
    load_config, redact_url_sensitive_params, JiraConfig, LlmConfig, LlmMode, ProjectConfig, Redact,
    SquashConfig, TemplatesConfig,
};
pub use error::ConfigError;

// Profile types for Story 0.2
pub use profiles::{ProfileId, ProfileOverride};

// Template types for Story 0.4
pub use template::{validate_content, LoadedTemplate, TemplateError, TemplateKind, TemplateLoader};

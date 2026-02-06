#![deny(unsafe_code)]
//! Structured logging for test-framework with automatic sensitive field redaction.
//!
//! This crate provides JSON-structured logging with:
//! - Structured JSON output (timestamp, level, message, target, fields)
//! - Automatic redaction of sensitive fields (tokens, passwords, API keys)
//! - File-based logging with daily rotation
//! - Non-blocking I/O for performance
//! - LogGuard lifecycle for guaranteed flush on shutdown
//!
//! # Quick Start
//!
//! ```no_run
//! use tf_logging::{init_logging, LoggingConfig};
//!
//! let config = LoggingConfig {
//!     log_level: "info".to_string(),
//!     log_dir: "./logs".to_string(),
//!     log_to_stdout: false,
//! };
//!
//! // Keep _guard alive for the application lifetime!
//! let _guard = init_logging(&config).unwrap();
//!
//! tracing::info!(command = "triage", status = "success", "Command executed");
//! // Sensitive fields are automatically redacted:
//! tracing::info!(token = "secret", "This token value will appear as [REDACTED]");
//! ```

pub mod config;
pub mod error;
pub mod init;
pub mod redact;

pub use config::LoggingConfig;
pub use error::LoggingError;
pub use init::{init_logging, LogGuard};

#[cfg(test)]
pub(crate) mod test_helpers {
    use std::fs;
    use std::path::{Path, PathBuf};

    /// Find the first log file in a directory.
    ///
    /// tracing-appender creates files with date-based names (e.g., "app.log.2026-02-06"),
    /// so we search for any file in the directory rather than a fixed name.
    pub fn find_log_file(log_dir: &Path) -> PathBuf {
        fs::read_dir(log_dir)
            .expect("Failed to read log directory")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .find(|p| p.is_file())
            .unwrap_or_else(|| panic!("No log file found in {}", log_dir.display()))
    }
}

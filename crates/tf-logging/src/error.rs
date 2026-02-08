//! Error types for the logging subsystem.

use thiserror::Error;

/// Errors that can occur during logging initialization and operation.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum LoggingError {
    /// Failed to initialize the tracing subscriber.
    ///
    /// Reserved for future use by tf-cli when subscriber initialization can fail
    /// (e.g., global subscriber already set). Currently not returned by `init_logging()`
    /// which uses thread-local dispatch (`set_default`) that cannot fail.
    #[error("Failed to initialize logging: {cause}. {hint}")]
    InitFailed { cause: String, hint: String },

    /// Failed to create the log output directory.
    #[error("Failed to create log directory '{path}': {cause}. {hint}")]
    DirectoryCreationFailed {
        path: String,
        cause: String,
        hint: String,
    },

    /// An invalid log level string was provided.
    #[error("Invalid log level '{level}'. {hint}")]
    InvalidLogLevel { level: String, hint: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    // Test 0.5-UNIT-008: LoggingError contains actionable hints
    #[test]
    fn test_logging_error_init_failed_has_actionable_hint() {
        let error = LoggingError::InitFailed {
            cause: "tracing subscriber already set".to_string(),
            hint: "Check that the log directory is writable and tracing is not already initialized"
                .to_string(),
        };

        let display = error.to_string();

        // Verify cause and hint appear in display
        assert!(
            display.contains("tracing subscriber already set"),
            "Display missing cause"
        );
        assert!(
            display.contains("Check that the log directory is writable"),
            "Display missing actionable hint"
        );

        // Verify variant structure
        assert_matches!(error, LoggingError::InitFailed { ref hint, .. } => {
            assert!(!hint.trim().is_empty(), "InitFailed hint must not be empty");
        });
    }

    #[test]
    fn test_logging_error_directory_creation_failed_has_actionable_hint() {
        let error = LoggingError::DirectoryCreationFailed {
            path: "/invalid/path/logs".to_string(),
            cause: "permission denied".to_string(),
            hint: "Verify permissions on the parent directory or set a different output_folder in config.yaml".to_string(),
        };

        let display = error.to_string();

        assert!(
            display.contains("/invalid/path/logs"),
            "Display missing path"
        );
        assert!(
            display.contains("permission denied"),
            "Display missing cause"
        );
        assert!(
            display.contains("Verify permissions on the parent directory"),
            "Display missing actionable hint"
        );

        assert_matches!(error, LoggingError::DirectoryCreationFailed { ref hint, .. } => {
            assert!(!hint.trim().is_empty(), "DirectoryCreationFailed hint must not be empty");
        });
    }

    #[test]
    fn test_logging_error_invalid_log_level_has_actionable_hint() {
        let error = LoggingError::InvalidLogLevel {
            level: "invalid_level".to_string(),
            hint: "Valid values: a level (trace, debug, info, warn, error) or a filter expression (e.g. \"info,tf_logging=debug\"). Set via config or RUST_LOG env var for diagnostics.".to_string(),
        };

        let display = error.to_string();

        assert!(display.contains("invalid_level"), "Display missing level");
        assert!(
            display.contains("Valid values"),
            "Display missing actionable hint"
        );

        assert_matches!(error, LoggingError::InvalidLogLevel { ref hint, .. } => {
            assert!(!hint.trim().is_empty(), "InvalidLogLevel hint must not be empty");
        });
    }
}

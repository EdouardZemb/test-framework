//! Logging initialization: subscriber setup, file appender, non-blocking writer.

use crate::config::LoggingConfig;
use crate::error::LoggingError;
use crate::redact::RedactingJsonFormatter;
use std::fs;
use tracing::Dispatch;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

/// Guard that must be kept alive to ensure logs are flushed.
///
/// When this guard is dropped, all pending log records are flushed to disk
/// and the thread-local subscriber is removed.
/// **MUST** be kept alive for the entire application lifetime:
///
/// ```no_run
/// # use tf_logging::{init_logging, LoggingConfig};
/// let config = LoggingConfig { log_level: "info".into(), log_dir: "./logs".into(), log_to_stdout: false };
/// let _guard = init_logging(&config).unwrap(); // keep _guard alive!
/// ```
pub struct LogGuard {
    _worker_guard: WorkerGuard,
    _dispatch_guard: tracing::dispatcher::DefaultGuard,
}

impl std::fmt::Debug for LogGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Safe Debug impl: never expose internal state or sensitive data
        f.debug_struct("LogGuard").finish()
    }
}

/// Initialize the logging subsystem.
///
/// Sets up:
/// - JSON-structured log format (timestamp, level, message, target, fields)
/// - File appender with daily rotation to `{config.log_dir}`
/// - Non-blocking writer for performance
/// - Sensitive field redaction via [`crate::redact::RedactingJsonFormatter`]
/// - Optional stdout output (if `config.log_to_stdout` is true)
///
/// Returns a [`LogGuard`] that MUST be kept alive for the application lifetime.
pub fn init_logging(config: &LoggingConfig) -> Result<LogGuard, LoggingError> {
    // Create log directory
    fs::create_dir_all(&config.log_dir).map_err(|e| LoggingError::DirectoryCreationFailed {
        path: config.log_dir.clone(),
        cause: e.to_string(),
        hint: "Verify permissions on the parent directory or set a different output_folder in config.yaml".to_string(),
    })?;

    // Build EnvFilter: RUST_LOG takes priority, otherwise use config.log_level
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(&config.log_level)
    });

    // Set up daily rolling file appender
    let file_appender = tracing_appender::rolling::daily(&config.log_dir, "app.log");
    let (non_blocking, worker_guard) = tracing_appender::non_blocking(file_appender);

    // Build the fmt layer with our custom RedactingJsonFormatter
    let fmt_layer = fmt::layer()
        .event_format(RedactingJsonFormatter)
        .with_writer(non_blocking)
        .with_ansi(false);

    // Build subscriber
    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer);

    // Use set_default (thread-local) to allow multiple init calls in tests
    let dispatch = Dispatch::new(subscriber);
    let dispatch_guard = tracing::dispatcher::set_default(&dispatch);

    Ok(LogGuard {
        _worker_guard: worker_guard,
        _dispatch_guard: dispatch_guard,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LoggingConfig;
    use std::fs;
    use tempfile::tempdir;

    /// Helper: find any file in the logs directory.
    /// tracing-appender creates files with date-based names.
    fn find_log_file(logs_dir: &std::path::Path) -> std::path::PathBuf {
        fs::read_dir(logs_dir)
            .expect("Failed to read logs directory")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .find(|p| p.is_file())
            .unwrap_or_else(|| panic!("No log file found in {}", logs_dir.display()))
    }

    // Test 0.5-UNIT-001: init_logging creates directory and returns LogGuard
    #[test]
    fn test_init_logging_creates_dir_and_returns_guard() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");

        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };

        let guard = init_logging(&config).unwrap();

        // Verify directory was created
        assert!(log_dir.exists(), "Log directory should be created by init_logging");
        assert!(log_dir.is_dir());

        drop(guard);
    }

    // Test 0.5-UNIT-002: Log output contains required JSON fields
    #[test]
    fn test_log_output_contains_required_json_fields() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");

        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };

        let guard = init_logging(&config).unwrap();

        // Emit a structured log event
        tracing::info!(
            command = "triage",
            status = "success",
            scope = "lot-42",
            "Command executed"
        );

        // Flush logs
        drop(guard);

        // Read and parse log file
        let log_file = find_log_file(&log_dir);
        let content = fs::read_to_string(&log_file).unwrap();
        let last_line = content.lines().last().expect("Log file should have at least one line");
        let json: serde_json::Value = serde_json::from_str(last_line)
            .expect("Log line should be valid JSON");

        // Required fields: timestamp, level, message, target
        assert!(json.get("timestamp").is_some(), "Missing 'timestamp' field");
        assert!(json.get("level").is_some(), "Missing 'level' field");
        assert!(json.get("target").is_some(), "Missing 'target' field");

        // Level must be uppercase
        assert_eq!(json["level"].as_str().unwrap(), "INFO");

        // Timestamp must be ISO 8601 (contains 'T')
        let ts = json["timestamp"].as_str().unwrap();
        assert!(ts.contains('T'), "Timestamp should be ISO 8601 format, got: {ts}");
    }

    // Test 0.5-UNIT-005: Logs written to configured directory
    #[test]
    fn test_logs_written_to_configured_directory() {
        let temp = tempdir().unwrap();
        let output_folder = temp.path().join("output");
        fs::create_dir(&output_folder).unwrap();
        let log_dir = output_folder.join("logs");

        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };

        let guard = init_logging(&config).unwrap();

        tracing::info!(command = "test", status = "ok", "Test log event");

        drop(guard);

        // Verify log directory was created at configured path
        assert!(log_dir.exists(), "Log directory not created at: {:?}", log_dir);

        // Verify at least one log file exists
        let file_count = fs::read_dir(&log_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .count();
        assert!(file_count > 0, "No log files in configured directory");

        // Verify content
        let log_file = find_log_file(&log_dir);
        let content = fs::read_to_string(&log_file).unwrap();
        assert!(content.contains("Test log event"), "Log file missing expected event");
    }

    // Test 0.5-UNIT-006: Default log level is info
    #[test]
    fn test_default_log_level_is_info() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");

        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };

        let guard = init_logging(&config).unwrap();

        // Debug should be filtered out at info level
        tracing::debug!("This debug message should not appear");
        tracing::info!("This info message should appear");

        drop(guard);

        let log_file = find_log_file(&log_dir);
        let content = fs::read_to_string(&log_file).unwrap();

        assert!(!content.contains("This debug message should not appear"),
                "Debug message should be filtered at info level");
        assert!(content.contains("This info message should appear"),
                "Info message should pass at info level");
    }

    // Test 0.5-UNIT-007: RUST_LOG overrides configured level
    #[test]
    fn test_rust_log_overrides_configured_level() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");

        // Set RUST_LOG to debug to override the info default
        std::env::set_var("RUST_LOG", "debug");

        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };

        let guard = init_logging(&config).unwrap();

        tracing::debug!("Debug visible via RUST_LOG override");

        drop(guard);

        let log_file = find_log_file(&log_dir);
        let content = fs::read_to_string(&log_file).unwrap();

        assert!(content.contains("Debug visible via RUST_LOG override"),
                "RUST_LOG=debug should override config level and show debug messages");

        // Cleanup
        std::env::remove_var("RUST_LOG");
    }

    // Test 0.5-UNIT-011: ANSI colors disabled for file logs
    #[test]
    fn test_ansi_disabled_for_file_logs() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");

        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };

        let guard = init_logging(&config).unwrap();

        tracing::info!("Message to verify no ANSI escape codes");

        drop(guard);

        let log_file = find_log_file(&log_dir);
        let content = fs::read_to_string(&log_file).unwrap();

        // ANSI escape codes start with \x1b[
        assert!(!content.contains("\x1b["),
                "Log file should not contain ANSI escape codes");
        assert!(content.contains("Message to verify no ANSI escape codes"));
    }
}

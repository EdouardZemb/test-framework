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
///
/// # Thread-local limitation
///
/// This function uses `tracing::dispatcher::set_default` which installs the
/// subscriber on the **current thread only**. Events emitted from other threads
/// or async workers will **not** be captured unless they are running on the same
/// thread. Before tf-cli integration, consider switching to `set_global_default`
/// for process-wide logging (at the cost of single-init-only semantics).
pub fn init_logging(config: &LoggingConfig) -> Result<LogGuard, LoggingError> {
    // Create log directory
    fs::create_dir_all(&config.log_dir).map_err(|e| LoggingError::DirectoryCreationFailed {
        path: config.log_dir.clone(),
        cause: e.to_string(),
        hint: "Verify permissions on the parent directory or set a different output_folder in config.yaml".to_string(),
    })?;

    // Validate log level before building filter
    const VALID_LEVELS: &[&str] = &["trace", "debug", "info", "warn", "error"];
    if !VALID_LEVELS.contains(&config.log_level.to_lowercase().as_str()) {
        return Err(LoggingError::InvalidLogLevel {
            level: config.log_level.clone(),
            hint: "Valid levels are: trace, debug, info, warn, error. Set via RUST_LOG env var (or future dedicated logging config when available).".to_string(),
        });
    }

    // Build EnvFilter: RUST_LOG takes priority, otherwise use config.log_level
    let filter = match EnvFilter::try_from_default_env() {
        Ok(f) => f,
        Err(e) => {
            // If RUST_LOG is set but malformed, emit a diagnostic to stderr
            if std::env::var("RUST_LOG").is_ok() {
                eprintln!(
                    "tf-logging: ignoring malformed RUST_LOG value ({}), falling back to '{}'",
                    e, config.log_level
                );
            }
            EnvFilter::new(&config.log_level)
        }
    };

    // Set up daily rolling file appender
    let file_appender = tracing_appender::rolling::daily(&config.log_dir, "app.log");
    let (non_blocking, worker_guard) = tracing_appender::non_blocking(file_appender);

    // Build the fmt layer with our custom RedactingJsonFormatter
    let fmt_layer = fmt::layer()
        .event_format(RedactingJsonFormatter)
        .with_writer(non_blocking)
        .with_ansi(false);

    // Build subscriber with optional stdout layer
    if config.log_to_stdout {
        let stdout_layer = fmt::layer()
            .event_format(RedactingJsonFormatter)
            .with_writer(std::io::stdout)
            .with_ansi(false);

        let subscriber = tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .with(stdout_layer);

        let dispatch = Dispatch::new(subscriber);
        let dispatch_guard = tracing::dispatcher::set_default(&dispatch);

        return Ok(LogGuard {
            _worker_guard: worker_guard,
            _dispatch_guard: dispatch_guard,
        });
    }

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
    use assert_matches::assert_matches;
    use crate::config::LoggingConfig;
    use std::fs;
    use tempfile::tempdir;

    use crate::test_helpers::find_log_file;

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
    //
    // Uses a mutex to serialize env-var-dependent tests, plus an RAII guard to
    // ensure RUST_LOG is always cleaned up — even if an assertion panics.
    //
    // Note: other concurrent `init_logging()` calls in parallel tests *could*
    // observe the temporary RUST_LOG value. This is an inherent limitation of
    // process-wide environment variables. The mutex prevents other env-mutating
    // tests from conflicting, and `set_default` (thread-local subscriber) limits
    // the blast radius to this test's thread.
    #[test]
    #[allow(unsafe_code)]
    fn test_rust_log_overrides_configured_level() {
        use std::sync::Mutex;
        static ENV_MUTEX: Mutex<()> = Mutex::new(());

        /// RAII guard that removes RUST_LOG on drop (including panic unwind).
        struct EnvGuard;
        impl Drop for EnvGuard {
            fn drop(&mut self) {
                // Safety: protected by ENV_MUTEX; no other thread modifies
                // RUST_LOG concurrently.
                unsafe { std::env::remove_var("RUST_LOG") };
            }
        }

        let _lock = ENV_MUTEX.lock().unwrap();
        // Safety: protected by ENV_MUTEX to avoid race with other env-mutating tests
        unsafe { std::env::set_var("RUST_LOG", "debug") };
        let _env_guard = EnvGuard;

        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");

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
        // _env_guard dropped here, cleaning up RUST_LOG
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

    // --- P0: LogGuard Debug impl + lifecycle tests ---

    // Test that Debug output of LogGuard shows opaque representation
    // (no internal state leaked, just "LogGuard" struct name)
    #[test]
    fn test_log_guard_debug_shows_opaque_struct() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");

        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };

        let guard = init_logging(&config).unwrap();
        let debug_output = format!("{:?}", guard);

        // Must contain the struct name
        assert!(debug_output.contains("LogGuard"),
                "Debug output should contain 'LogGuard', got: {debug_output}");

        // Must NOT expose internal field names
        assert!(!debug_output.contains("_worker_guard"),
                "Debug output must not expose _worker_guard field");
        assert!(!debug_output.contains("_dispatch_guard"),
                "Debug output must not expose _dispatch_guard field");
        assert!(!debug_output.contains("WorkerGuard"),
                "Debug output must not expose WorkerGuard type");
        assert!(!debug_output.contains("DefaultGuard"),
                "Debug output must not expose DefaultGuard type");

        drop(guard);
    }

    // Test that LogGuard can be successfully created and that it is a valid
    // object that survives being moved and dropped
    #[test]
    fn test_log_guard_lifecycle_create_and_drop() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");

        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };

        // Create guard
        let guard = init_logging(&config).unwrap();

        // Emit a log event while guard is alive
        tracing::info!("lifecycle test message");

        // Move the guard to a new binding (tests Send-like behavior)
        let moved_guard = guard;

        // Emit another event after move
        tracing::info!("after move message");

        // Drop flushes logs
        drop(moved_guard);

        // After drop, verify logs were flushed to disk
        let log_file = find_log_file(&log_dir);
        let content = fs::read_to_string(&log_file).unwrap();
        assert!(content.contains("lifecycle test message"),
                "Log should contain message emitted before guard move");
        assert!(content.contains("after move message"),
                "Log should contain message emitted after guard move");
    }

    // Test [AI-Review-R3 M2]: init_logging returns DirectoryCreationFailed on unwritable path
    #[test]
    fn test_init_logging_directory_creation_failed() {
        // Use a path under /proc which cannot have directories created inside it
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: "/proc/nonexistent/impossible/logs".to_string(),
            log_to_stdout: false,
        };

        let result = init_logging(&config);
        assert!(result.is_err(), "Should fail on unwritable directory");

        let err = result.unwrap_err();
        assert_matches!(err, LoggingError::DirectoryCreationFailed { ref path, ref hint, .. } => {
            assert_eq!(path, "/proc/nonexistent/impossible/logs");
            assert!(hint.contains("Verify permissions"), "Hint should be actionable");
        });
    }

    // Test [AI-Review]: invalid log level returns InvalidLogLevel error
    #[test]
    fn test_invalid_log_level_returns_error() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");

        let config = LoggingConfig {
            log_level: "invalid_level".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };

        let result = init_logging(&config);
        assert!(result.is_err(), "Invalid log level should return an error");

        let err = result.unwrap_err();
        assert_matches!(err, LoggingError::InvalidLogLevel { ref level, ref hint } => {
            assert_eq!(level, "invalid_level");
            assert!(hint.contains("Valid levels are"), "Hint should list valid levels");
        });
    }

    // Test [AI-Review]: log_to_stdout=true creates stdout layer
    #[test]
    fn test_log_to_stdout_creates_guard() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");

        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: true,
        };

        let guard = init_logging(&config);
        assert!(guard.is_ok(), "init_logging with log_to_stdout=true should succeed");

        // Emit a log and verify it reaches the file (stdout is harder to test)
        tracing::info!("stdout test message");
        drop(guard.unwrap());

        let log_file = find_log_file(&log_dir);
        let content = fs::read_to_string(&log_file).unwrap();
        assert!(content.contains("stdout test message"),
                "Log should still reach file when log_to_stdout=true");
    }
}

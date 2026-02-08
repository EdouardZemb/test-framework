//! Integration tests for tf-logging crate.
//!
//! These tests verify the complete logging lifecycle:
//! - Initialization → log emission → flush → file verification
//! - JSON structure compliance
//! - Sensitive field redaction in end-to-end scenario
//! - Workspace integration (crate compiles and is accessible)

mod common;

use common::find_log_file;
use std::fs;
use std::process::Command;
use tf_logging::{init_logging, LoggingConfig, LoggingError};

// Test 0.5-INT-001: Full logging lifecycle
//
// End-to-end test covering:
// 1. Initialization from LoggingConfig
// 2. Structured JSON log emission with sensitive + normal fields
// 3. Guard drop → flush
// 4. File content verification (JSON structure, redaction, preserved fields)
#[test]
fn test_full_logging_lifecycle() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let log_dir = temp_dir.path().join("logs");

    // Initialize logging
    let config = LoggingConfig {
        log_level: "info".to_string(),
        log_dir: log_dir.to_string_lossy().to_string(),
        log_to_stdout: false,
    };

    let guard = init_logging(&config).expect("Failed to initialize logging");

    // Emit log with both sensitive and normal fields
    tracing::info!(
        command = "triage",
        token = "secret123",
        status = "success",
        scope = "lot-42",
        "Pipeline complete"
    );

    // Flush by dropping guard
    drop(guard);

    // Verify log file exists
    let log_file = find_log_file(&log_dir);
    let content = fs::read_to_string(&log_file).expect("Failed to read log file");

    // Parse as JSON
    let lines: Vec<&str> = content.lines().collect();
    assert!(
        !lines.is_empty(),
        "Log file should contain at least one line"
    );

    let json: serde_json::Value =
        serde_json::from_str(lines[0]).expect("First log line should be valid JSON");

    // Verify required JSON fields
    assert!(json.get("timestamp").is_some(), "Missing 'timestamp'");
    assert!(json.get("level").is_some(), "Missing 'level'");
    assert!(json.get("target").is_some(), "Missing 'target'");

    // Verify sensitive value is redacted
    assert!(
        !content.contains("secret123"),
        "Sensitive value 'secret123' should be redacted"
    );
    assert!(
        content.contains("[REDACTED]"),
        "[REDACTED] placeholder should appear"
    );

    // Verify normal fields are preserved
    assert!(
        content.contains("triage"),
        "Normal field 'command=triage' should be preserved"
    );
    assert!(
        content.contains("Pipeline complete"),
        "Log message should be preserved"
    );
}

// Test 0.5-INT-002: Workspace integration
//
// Verifies that tf-logging is properly integrated in the workspace:
// - Crate compiles
// - Types are accessible from external crate
// - Basic struct construction works
#[test]
fn test_tf_logging_crate_compiles_and_types_accessible() {
    // Verify LoggingConfig is constructible
    let config = LoggingConfig {
        log_level: "debug".to_string(),
        log_dir: "/tmp/test-logs".to_string(),
        log_to_stdout: true,
    };

    assert_eq!(config.log_level, "debug");
    assert_eq!(config.log_dir, "/tmp/test-logs");
    assert!(config.log_to_stdout);

    // Verify LoggingError variants exist
    let _error = LoggingError::InvalidLogLevel {
        level: "bad".to_string(),
        hint: "test".to_string(),
    };
}

// Additional integration test: multiple sensitive fields in single event
#[test]
fn test_multiple_sensitive_fields_redacted_in_single_event() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let log_dir = temp_dir.path().join("logs");

    let config = LoggingConfig {
        log_level: "info".to_string(),
        log_dir: log_dir.to_string_lossy().to_string(),
        log_to_stdout: false,
    };

    let guard = init_logging(&config).expect("Failed to initialize logging");

    // Emit with multiple sensitive fields
    tracing::info!(
        api_key = "key_abc",
        password = "pass_def",
        secret = "secret_ghi",
        normal_field = "visible_value",
        "Multi-sensitive fields test"
    );

    drop(guard);

    let content = fs::read_to_string(find_log_file(&log_dir)).unwrap();

    // All sensitive values must be redacted
    assert!(
        !content.contains("key_abc"),
        "api_key value should be redacted"
    );
    assert!(
        !content.contains("pass_def"),
        "password value should be redacted"
    );
    assert!(
        !content.contains("secret_ghi"),
        "secret value should be redacted"
    );

    // Normal field must be preserved
    assert!(
        content.contains("visible_value"),
        "Normal field should be visible"
    );
}

#[test]
fn test_log_output_includes_parent_spans() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let log_dir = temp_dir.path().join("logs");

    let config = LoggingConfig {
        log_level: "info".to_string(),
        log_dir: log_dir.to_string_lossy().to_string(),
        log_to_stdout: false,
    };

    let guard = init_logging(&config).expect("Failed to initialize logging");
    let span = tracing::info_span!("cli_command", command = "triage", scope = "lot-42");
    let _entered = span.enter();
    tracing::info!(status = "success", "Command completed");
    drop(_entered);
    drop(guard);

    let log_file = find_log_file(&log_dir);
    let content = fs::read_to_string(&log_file).expect("Failed to read log file");
    let json: serde_json::Value = serde_json::from_str(
        content
            .lines()
            .last()
            .expect("Log file should contain at least one line"),
    )
    .expect("Log line should be valid JSON");

    let spans = json
        .get("spans")
        .and_then(|v| v.as_array())
        .expect("Expected 'spans' array in JSON output");

    assert!(
        spans.iter().any(|span| span["name"] == "cli_command"),
        "Expected cli_command span to be present"
    );

    // Verify span fields are structured JSON objects (not opaque strings)
    let cli_span = spans.iter().find(|s| s["name"] == "cli_command").unwrap();
    let fields = cli_span.get("fields").expect("Expected 'fields' in span");
    assert!(
        fields.is_object(),
        "Span fields should be a JSON object, got: {fields}"
    );
    let fields_map = fields.as_object().unwrap();
    assert_eq!(fields_map.get("command").unwrap(), "triage");
    assert_eq!(fields_map.get("scope").unwrap(), "lot-42");
}

// Test 0.5-INT-004: Simulate a full CLI command execution in a subprocess.
//
// This verifies a command-style process lifecycle:
// 1. Child process starts (simulated CLI entrypoint)
// 2. init_logging() is called with configured log directory
// 3. "command + scope + exit_code" event is emitted
// 4. Process exits and logs are flushed
// 5. Parent process validates JSON content
#[test]
fn test_cli_command_simulation_via_subprocess() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let log_dir = temp_dir.path().join("logs");

    let exe = std::env::current_exe().expect("Failed to resolve current test binary");
    let output = Command::new(exe)
        .arg("--ignored")
        .arg("--exact")
        .arg("cli_subprocess_entrypoint")
        .env("TF_LOGGING_RUN_CLI_SUBPROCESS", "1")
        .env("TF_LOGGING_CLI_COMMAND", "triage")
        .env("TF_LOGGING_CLI_SCOPE", "lot-42")
        .env(
            "TF_LOGGING_CLI_LOG_DIR",
            log_dir.to_string_lossy().to_string(),
        )
        .output()
        .expect("Failed to execute subprocess test entrypoint");

    assert!(
        output.status.success(),
        "Subprocess CLI simulation failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log_file = find_log_file(&log_dir);
    let content = fs::read_to_string(&log_file).expect("Failed to read subprocess log file");
    let json: serde_json::Value = serde_json::from_str(
        content
            .lines()
            .last()
            .expect("Subprocess log file should contain at least one line"),
    )
    .expect("Subprocess log line should be valid JSON");

    assert_eq!(json["level"], "INFO");
    assert_eq!(json["fields"]["command"], "triage");
    assert_eq!(json["fields"]["scope"], "lot-42");
    assert_eq!(json["fields"]["status"], "success");
    assert_eq!(json["fields"]["exit_code"], 0);
}

#[test]
#[ignore]
fn cli_subprocess_entrypoint() {
    if std::env::var("TF_LOGGING_RUN_CLI_SUBPROCESS").as_deref() != Ok("1") {
        return;
    }

    let log_dir =
        std::env::var("TF_LOGGING_CLI_LOG_DIR").expect("TF_LOGGING_CLI_LOG_DIR must be set");
    let command =
        std::env::var("TF_LOGGING_CLI_COMMAND").expect("TF_LOGGING_CLI_COMMAND must be set");
    let scope = std::env::var("TF_LOGGING_CLI_SCOPE").expect("TF_LOGGING_CLI_SCOPE must be set");

    let config = LoggingConfig {
        log_level: "info".to_string(),
        log_dir,
        log_to_stdout: false,
    };

    let guard = init_logging(&config).expect("Failed to initialize logging in CLI subprocess");
    tracing::info!(
        command = command.as_str(),
        scope = scope.as_str(),
        status = "success",
        exit_code = 0_i64,
        "CLI command executed"
    );
    drop(guard);
}

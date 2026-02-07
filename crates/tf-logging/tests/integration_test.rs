//! Integration tests for tf-logging crate.
//!
//! These tests verify the complete logging lifecycle:
//! - Initialization → log emission → flush → file verification
//! - JSON structure compliance
//! - Sensitive field redaction in end-to-end scenario
//! - Workspace integration (crate compiles and is accessible)

mod common;

use std::fs;
use common::find_log_file;
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
    assert!(!lines.is_empty(), "Log file should contain at least one line");

    let json: serde_json::Value = serde_json::from_str(lines[0])
        .expect("First log line should be valid JSON");

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
    assert!(content.contains("triage"), "Normal field 'command=triage' should be preserved");
    assert!(content.contains("Pipeline complete"), "Log message should be preserved");
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
    assert!(!content.contains("key_abc"), "api_key value should be redacted");
    assert!(!content.contains("pass_def"), "password value should be redacted");
    assert!(!content.contains("secret_ghi"), "secret value should be redacted");

    // Normal field must be preserved
    assert!(content.contains("visible_value"), "Normal field should be visible");
}

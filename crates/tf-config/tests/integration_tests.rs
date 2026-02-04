//! Integration tests using fixture files
//!
//! These tests verify configuration loading against real YAML fixture files.
//! Uses CARGO_MANIFEST_DIR to ensure paths work regardless of working directory.

use std::path::PathBuf;
use tf_config::{load_config, ConfigError, LlmMode};

/// Returns the path to a test fixture file, using CARGO_MANIFEST_DIR for robustness.
fn fixture_path(name: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("tests")
        .join("fixtures")
        .join(name)
}

/// Test loading a valid configuration from fixture file
#[test]
fn test_load_valid_config_from_fixture() {
    let config = load_config(&fixture_path("valid_config.yaml")).unwrap();

    assert_eq!(config.project_name, "test-project");
    assert_eq!(config.output_folder, "./output");

    let jira = config.jira.as_ref().unwrap();
    assert_eq!(jira.endpoint, "https://jira.example.com");
    assert_eq!(jira.token.as_ref().unwrap(), "secret-token");

    let squash = config.squash.as_ref().unwrap();
    assert_eq!(squash.endpoint, "https://squash.example.com");
    assert_eq!(squash.username.as_ref().unwrap(), "testuser");

    let templates = config.templates.as_ref().unwrap();
    assert_eq!(templates.cr.as_ref().unwrap(), "./templates/cr.md");

    let llm = config.llm.as_ref().unwrap();
    assert_eq!(llm.mode, LlmMode::Auto);
}

/// Test loading a minimal configuration from fixture file
#[test]
fn test_load_minimal_config_from_fixture() {
    let config = load_config(&fixture_path("minimal_config.yaml")).unwrap();

    assert_eq!(config.project_name, "minimal-project");
    assert_eq!(config.output_folder, "./output");
    assert!(config.jira.is_none());
    assert!(config.squash.is_none());
    assert!(config.templates.is_none());
    assert!(config.llm.is_none());
}

/// Test that missing required field produces helpful error with hint
#[test]
fn test_missing_project_name_from_fixture() {
    let result = load_config(&fixture_path("missing_project_name.yaml"));

    assert!(result.is_err());
    let err = result.unwrap_err();

    // We intercept serde's missing field error and return a user-friendly MissingField
    match &err {
        ConfigError::MissingField { field, hint } => {
            assert_eq!(field, "project_name");
            assert!(
                hint.contains("project name"),
                "Hint should mention 'project name', got: {hint}"
            );
        }
        other => panic!(
            "Expected MissingField for missing required field, got: {:?}",
            other
        ),
    }

    // Verify error message is user-friendly (AC #2)
    let err_msg = err.to_string();
    assert!(err_msg.contains("project_name"));
    assert!(err_msg.contains("missing"));
    assert!(err_msg.contains("Expected"));
}

/// Test that invalid URL (scheme only) is rejected
#[test]
fn test_invalid_jira_url_from_fixture() {
    let result = load_config(&fixture_path("invalid_jira_url.yaml"));

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(err_msg.contains("jira.endpoint"));
    assert!(err_msg.contains("valid URL"));
}

/// Test that invalid LLM mode is rejected with helpful message
#[test]
fn test_invalid_llm_mode_from_fixture() {
    let result = load_config(&fixture_path("invalid_llm_mode.yaml"));

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(err_msg.contains("llm.mode"));
    assert!(err_msg.contains("not a valid mode"));
    assert!(err_msg.contains("Expected"));
}

/// Test that file not found error is returned for non-existent file
#[test]
fn test_file_not_found() {
    let result = load_config(&fixture_path("nonexistent.yaml"));

    assert!(result.is_err());
    match result.unwrap_err() {
        ConfigError::FileNotFound { path } => {
            assert!(path.to_string_lossy().contains("nonexistent.yaml"));
        }
        other => panic!("Expected FileNotFound, got: {:?}", other),
    }
}

/// Test that secrets are redacted in debug output
#[test]
fn test_secrets_redacted_in_debug() {
    let config = load_config(&fixture_path("valid_config.yaml")).unwrap();

    let debug_output = format!("{:?}", config);

    // Verify secrets are NOT in debug output
    assert!(!debug_output.contains("secret-token"));
    assert!(!debug_output.contains("secret-password"));

    // Verify [REDACTED] placeholder is present
    assert!(debug_output.contains("[REDACTED]"));

    // But the actual values are still accessible programmatically
    let jira = config.jira.as_ref().unwrap();
    assert_eq!(jira.token.as_ref().unwrap(), "secret-token");
}

/// Test that IoError is returned when reading a directory as a file
///
/// This explicitly tests the IoError variant which is implicitly covered via
/// the `From<std::io::Error>` derive. On Unix, attempting to read a directory
/// as a file produces an I/O error (not "file not found").
#[test]
fn test_io_error_reading_directory_as_file() {
    // Create a temporary directory
    let temp_dir = tempfile::tempdir().unwrap();
    let dir_path = temp_dir.path().join("not_a_file");
    std::fs::create_dir(&dir_path).unwrap();

    // Attempting to load a directory as config should produce IoError
    let result = load_config(&dir_path);

    assert!(result.is_err());
    match result.unwrap_err() {
        ConfigError::IoError(io_err) => {
            // On Unix: "Is a directory" error
            // On Windows: might be different but still an I/O error
            assert!(
                io_err.kind() == std::io::ErrorKind::IsADirectory
                    || io_err.kind() == std::io::ErrorKind::PermissionDenied
                    || io_err.kind() == std::io::ErrorKind::Other,
                "Expected I/O error when reading directory, got: {:?}",
                io_err.kind()
            );
        }
        other => panic!("Expected IoError when reading directory, got: {:?}", other),
    }
}

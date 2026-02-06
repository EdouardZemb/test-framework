//! Sensitive field redaction layer for tracing events.
//!
//! Provides a [`RedactingLayer`] that intercepts tracing events and replaces
//! sensitive field values with `[REDACTED]` before they reach the JSON formatter.

/// Field names considered sensitive. Values of these fields will be replaced
/// with `[REDACTED]` in log output.
pub(crate) const SENSITIVE_FIELDS: &[&str] = &[
    "token",
    "api_key",
    "apikey",
    "key",
    "secret",
    "password",
    "passwd",
    "pwd",
    "auth",
    "authorization",
    "credential",
    "credentials",
];

// RED phase: RedactingLayer, RedactingVisitor, and integration with
// tracing-subscriber will be implemented here.
// See story subtasks 3.1-3.5 for implementation details.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LoggingConfig;
    use crate::init::init_logging;
    use std::fs;
    use tempfile::tempdir;

    /// Helper: find any file in the logs directory.
    fn find_log_file(logs_dir: &std::path::Path) -> std::path::PathBuf {
        fs::read_dir(logs_dir)
            .expect("Failed to read logs directory")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .find(|p| p.is_file())
            .unwrap_or_else(|| panic!("No log file found in {}", logs_dir.display()))
    }

    // Test 0.5-UNIT-003: All 12 sensitive fields are redacted
    //
    // This test verifies exhaustively that each sensitive field name in
    // SENSITIVE_FIELDS is masked by [REDACTED] in log output.
    // Also verifies that normal fields (command, status, scope) are NOT masked.
    #[test]
    fn test_sensitive_field_token_redacted() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };
        let guard = init_logging(&config).unwrap();
        tracing::info!(token = "secret_value_123", "test");
        drop(guard);
        let content = fs::read_to_string(find_log_file(&log_dir)).unwrap();
        assert!(!content.contains("secret_value_123"), "Field 'token' was not redacted");
        assert!(content.contains("[REDACTED]"), "'token' should show [REDACTED]");
    }

    #[test]
    fn test_sensitive_field_api_key_redacted() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };
        let guard = init_logging(&config).unwrap();
        tracing::info!(api_key = "secret_value_123", "test");
        drop(guard);
        let content = fs::read_to_string(find_log_file(&log_dir)).unwrap();
        assert!(!content.contains("secret_value_123"), "Field 'api_key' was not redacted");
        assert!(content.contains("[REDACTED]"), "'api_key' should show [REDACTED]");
    }

    #[test]
    fn test_sensitive_field_apikey_redacted() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };
        let guard = init_logging(&config).unwrap();
        tracing::info!(apikey = "secret_value_123", "test");
        drop(guard);
        let content = fs::read_to_string(find_log_file(&log_dir)).unwrap();
        assert!(!content.contains("secret_value_123"), "Field 'apikey' was not redacted");
        assert!(content.contains("[REDACTED]"), "'apikey' should show [REDACTED]");
    }

    #[test]
    fn test_sensitive_field_key_redacted() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };
        let guard = init_logging(&config).unwrap();
        tracing::info!(key = "secret_value_123", "test");
        drop(guard);
        let content = fs::read_to_string(find_log_file(&log_dir)).unwrap();
        assert!(!content.contains("secret_value_123"), "Field 'key' was not redacted");
        assert!(content.contains("[REDACTED]"), "'key' should show [REDACTED]");
    }

    #[test]
    fn test_sensitive_field_secret_redacted() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };
        let guard = init_logging(&config).unwrap();
        tracing::info!(secret = "secret_value_123", "test");
        drop(guard);
        let content = fs::read_to_string(find_log_file(&log_dir)).unwrap();
        assert!(!content.contains("secret_value_123"), "Field 'secret' was not redacted");
        assert!(content.contains("[REDACTED]"), "'secret' should show [REDACTED]");
    }

    #[test]
    fn test_sensitive_field_password_redacted() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };
        let guard = init_logging(&config).unwrap();
        tracing::info!(password = "secret_value_123", "test");
        drop(guard);
        let content = fs::read_to_string(find_log_file(&log_dir)).unwrap();
        assert!(!content.contains("secret_value_123"), "Field 'password' was not redacted");
        assert!(content.contains("[REDACTED]"), "'password' should show [REDACTED]");
    }

    #[test]
    fn test_sensitive_field_passwd_redacted() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };
        let guard = init_logging(&config).unwrap();
        tracing::info!(passwd = "secret_value_123", "test");
        drop(guard);
        let content = fs::read_to_string(find_log_file(&log_dir)).unwrap();
        assert!(!content.contains("secret_value_123"), "Field 'passwd' was not redacted");
        assert!(content.contains("[REDACTED]"), "'passwd' should show [REDACTED]");
    }

    #[test]
    fn test_sensitive_field_pwd_redacted() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };
        let guard = init_logging(&config).unwrap();
        tracing::info!(pwd = "secret_value_123", "test");
        drop(guard);
        let content = fs::read_to_string(find_log_file(&log_dir)).unwrap();
        assert!(!content.contains("secret_value_123"), "Field 'pwd' was not redacted");
        assert!(content.contains("[REDACTED]"), "'pwd' should show [REDACTED]");
    }

    #[test]
    fn test_sensitive_field_auth_redacted() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };
        let guard = init_logging(&config).unwrap();
        tracing::info!(auth = "secret_value_123", "test");
        drop(guard);
        let content = fs::read_to_string(find_log_file(&log_dir)).unwrap();
        assert!(!content.contains("secret_value_123"), "Field 'auth' was not redacted");
        assert!(content.contains("[REDACTED]"), "'auth' should show [REDACTED]");
    }

    #[test]
    fn test_sensitive_field_authorization_redacted() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };
        let guard = init_logging(&config).unwrap();
        tracing::info!(authorization = "secret_value_123", "test");
        drop(guard);
        let content = fs::read_to_string(find_log_file(&log_dir)).unwrap();
        assert!(!content.contains("secret_value_123"), "Field 'authorization' was not redacted");
        assert!(content.contains("[REDACTED]"), "'authorization' should show [REDACTED]");
    }

    #[test]
    fn test_sensitive_field_credential_redacted() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };
        let guard = init_logging(&config).unwrap();
        tracing::info!(credential = "secret_value_123", "test");
        drop(guard);
        let content = fs::read_to_string(find_log_file(&log_dir)).unwrap();
        assert!(!content.contains("secret_value_123"), "Field 'credential' was not redacted");
        assert!(content.contains("[REDACTED]"), "'credential' should show [REDACTED]");
    }

    #[test]
    fn test_sensitive_field_credentials_redacted() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };
        let guard = init_logging(&config).unwrap();
        tracing::info!(credentials = "secret_value_123", "test");
        drop(guard);
        let content = fs::read_to_string(find_log_file(&log_dir)).unwrap();
        assert!(!content.contains("secret_value_123"), "Field 'credentials' was not redacted");
        assert!(content.contains("[REDACTED]"), "'credentials' should show [REDACTED]");
    }

    // Negative test: normal fields must NOT be redacted
    #[test]
    fn test_normal_fields_are_not_redacted() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };
        let guard = init_logging(&config).unwrap();
        tracing::info!(
            command = "triage",
            status = "success",
            scope = "lot-42",
            "Normal fields test"
        );
        drop(guard);
        let content = fs::read_to_string(find_log_file(&log_dir)).unwrap();
        assert!(content.contains("triage"), "command field was incorrectly redacted");
        assert!(content.contains("success"), "status field was incorrectly redacted");
        assert!(content.contains("lot-42"), "scope field was incorrectly redacted");
    }

    // Test 0.5-UNIT-004: URLs with sensitive params are redacted
    #[test]
    fn test_urls_with_sensitive_params_are_redacted() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };
        let guard = init_logging(&config).unwrap();
        tracing::info!(
            endpoint = "https://api.example.com?token=abc123&user=john",
            "API call"
        );
        drop(guard);
        let content = fs::read_to_string(find_log_file(&log_dir)).unwrap();
        assert!(!content.contains("abc123"),
                "URL token parameter value should be redacted");
        assert!(content.contains("[REDACTED]"),
                "Redacted URL should contain [REDACTED]");
        assert!(content.contains("user"),
                "Non-sensitive URL parameter name should be preserved");
    }

    // Test 0.5-UNIT-009: Debug impl of LogGuard does not leak sensitive data
    #[test]
    fn test_log_guard_debug_no_sensitive_data() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };
        let guard = init_logging(&config).unwrap();
        let debug_output = format!("{:?}", guard);

        // Debug output must not contain sensitive patterns
        assert!(!debug_output.to_lowercase().contains("secret"),
                "Debug output should not contain 'secret'");
        assert!(!debug_output.to_lowercase().contains("password"),
                "Debug output should not contain 'password'");
        assert!(!debug_output.to_lowercase().contains("token"),
                "Debug output should not contain 'token'");
        assert!(!debug_output.to_lowercase().contains("key"),
                "Debug output should not contain 'key'");
    }
}

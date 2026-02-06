//! Sensitive field redaction layer for tracing events.
//!
//! Provides a custom JSON formatter that intercepts tracing events and replaces
//! sensitive field values with `[REDACTED]` before they are written to output.

use serde_json::Value;
use tracing::{Event, Subscriber};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;

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

/// A custom JSON event formatter that redacts sensitive fields.
///
/// This formatter produces JSON log lines with the structure:
/// ```json
/// {"timestamp":"...","level":"INFO","target":"...","message":"...","fields":{...}}
/// ```
///
/// Sensitive fields (listed in [`SENSITIVE_FIELDS`]) have their values replaced
/// with `[REDACTED]`. Fields containing URLs have sensitive URL parameters redacted
/// via [`tf_config::redact_url_sensitive_params`].
pub(crate) struct RedactingJsonFormatter;

/// Visitor that collects event fields into a serde_json map,
/// redacting sensitive values as it goes.
struct RedactingVisitor {
    fields: serde_json::Map<String, Value>,
    message: String,
}

impl RedactingVisitor {
    fn new() -> Self {
        Self {
            fields: serde_json::Map::new(),
            message: String::new(),
        }
    }

    fn is_sensitive(name: &str) -> bool {
        let lower = name.to_lowercase();
        SENSITIVE_FIELDS.iter().any(|&f| f == lower)
    }

    fn looks_like_url(value: &str) -> bool {
        value.starts_with("http://") || value.starts_with("https://")
    }

    fn redact_value(&self, name: &str, value: &str) -> String {
        if Self::is_sensitive(name) {
            "[REDACTED]".to_string()
        } else if Self::looks_like_url(value) {
            tf_config::redact_url_sensitive_params(value)
        } else {
            value.to_string()
        }
    }
}

impl tracing::field::Visit for RedactingVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let name = field.name();
        if name == "message" {
            self.message = format!("{:?}", value);
            // Remove surrounding quotes if present (Debug adds them for &str)
            if self.message.starts_with('"') && self.message.ends_with('"') {
                self.message = self.message[1..self.message.len() - 1].to_string();
            }
            return;
        }

        let raw = format!("{:?}", value);
        let cleaned = if raw.starts_with('"') && raw.ends_with('"') {
            raw[1..raw.len() - 1].to_string()
        } else {
            raw
        };

        let redacted = self.redact_value(name, &cleaned);
        self.fields
            .insert(name.to_string(), Value::String(redacted));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        let name = field.name();
        if name == "message" {
            self.message = value.to_string();
            return;
        }
        let redacted = self.redact_value(name, value);
        self.fields
            .insert(name.to_string(), Value::String(redacted));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        let name = field.name();
        if Self::is_sensitive(name) {
            self.fields
                .insert(name.to_string(), Value::String("[REDACTED]".to_string()));
        } else {
            self.fields
                .insert(name.to_string(), Value::Number(value.into()));
        }
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        let name = field.name();
        if Self::is_sensitive(name) {
            self.fields
                .insert(name.to_string(), Value::String("[REDACTED]".to_string()));
        } else {
            self.fields
                .insert(name.to_string(), Value::Number(value.into()));
        }
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        let name = field.name();
        if Self::is_sensitive(name) {
            self.fields
                .insert(name.to_string(), Value::String("[REDACTED]".to_string()));
        } else {
            self.fields
                .insert(name.to_string(), Value::Bool(value));
        }
    }
}

impl<S, N> FormatEvent<S, N> for RedactingJsonFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        // Collect fields via our redacting visitor
        let mut visitor = RedactingVisitor::new();
        event.record(&mut visitor);

        // Build the JSON object
        let mut obj = serde_json::Map::new();

        // Timestamp in RFC 3339 / ISO 8601 UTC
        let now = std::time::SystemTime::now();
        let duration = now
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let secs = duration.as_secs();
        let nanos = duration.subsec_nanos();
        // Manual RFC 3339 formatting to avoid chrono dependency
        let timestamp = format_rfc3339(secs, nanos);
        obj.insert("timestamp".to_string(), Value::String(timestamp));

        // Level (uppercase)
        let level = event.metadata().level();
        obj.insert(
            "level".to_string(),
            Value::String(level.to_string().to_uppercase()),
        );

        // Target
        obj.insert(
            "target".to_string(),
            Value::String(event.metadata().target().to_string()),
        );

        // Message
        if !visitor.message.is_empty() {
            obj.insert(
                "message".to_string(),
                Value::String(visitor.message),
            );
        }

        // Fields
        if !visitor.fields.is_empty() {
            obj.insert("fields".to_string(), Value::Object(visitor.fields));
        }

        let json_str = serde_json::to_string(&obj).map_err(|_| std::fmt::Error)?;
        write!(writer, "{}", json_str)?;
        writeln!(writer)?;

        Ok(())
    }
}

/// Format a Unix timestamp as RFC 3339 (e.g., "2026-02-06T10:30:45.123Z").
fn format_rfc3339(secs: u64, nanos: u32) -> String {
    // Calculate date components from Unix timestamp
    let days = secs / 86400;
    let time_of_day = secs % 86400;

    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;
    let millis = nanos / 1_000_000;

    // Convert days since epoch to year-month-day
    let (year, month, day) = days_to_ymd(days);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        year, month, day, hours, minutes, seconds, millis
    )
}

/// Convert days since Unix epoch (1970-01-01) to (year, month, day).
fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    // Algorithm from Howard Hinnant's date algorithms
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LoggingConfig;
    use crate::init::init_logging;
    use std::fs;
    use tempfile::tempdir;

    use crate::test_helpers::find_log_file;

    // Test 0.5-UNIT-003: All 12 sensitive fields are redacted
    //
    // Uses a macro to generate one test per sensitive field name, avoiding
    // ~200 lines of copy-paste duplication.

    macro_rules! test_sensitive_field_redacted {
        ($test_name:ident, $field:ident) => {
            #[test]
            fn $test_name() {
                let temp = tempdir().unwrap();
                let log_dir = temp.path().join("logs");
                let config = LoggingConfig {
                    log_level: "info".to_string(),
                    log_dir: log_dir.to_string_lossy().to_string(),
                    log_to_stdout: false,
                };
                let guard = init_logging(&config).unwrap();
                tracing::info!($field = "secret_value_123", "test");
                drop(guard);
                let content = fs::read_to_string(find_log_file(&log_dir)).unwrap();
                assert!(!content.contains("secret_value_123"),
                        "Field '{}' was not redacted", stringify!($field));
                assert!(content.contains("[REDACTED]"),
                        "'{}' should show [REDACTED]", stringify!($field));
            }
        };
    }

    test_sensitive_field_redacted!(test_sensitive_field_token_redacted, token);
    test_sensitive_field_redacted!(test_sensitive_field_api_key_redacted, api_key);
    test_sensitive_field_redacted!(test_sensitive_field_apikey_redacted, apikey);
    test_sensitive_field_redacted!(test_sensitive_field_key_redacted, key);
    test_sensitive_field_redacted!(test_sensitive_field_secret_redacted, secret);
    test_sensitive_field_redacted!(test_sensitive_field_password_redacted, password);
    test_sensitive_field_redacted!(test_sensitive_field_passwd_redacted, passwd);
    test_sensitive_field_redacted!(test_sensitive_field_pwd_redacted, pwd);
    test_sensitive_field_redacted!(test_sensitive_field_auth_redacted, auth);
    test_sensitive_field_redacted!(test_sensitive_field_authorization_redacted, authorization);
    test_sensitive_field_redacted!(test_sensitive_field_credential_redacted, credential);
    test_sensitive_field_redacted!(test_sensitive_field_credentials_redacted, credentials);

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

    #[test]
    fn test_format_rfc3339_basic() {
        // 2026-01-01T00:00:00.000Z = 1767225600 seconds since epoch
        let result = format_rfc3339(1767225600, 0);
        assert_eq!(result, "2026-01-01T00:00:00.000Z");
    }

    #[test]
    fn test_format_rfc3339_with_millis() {
        let result = format_rfc3339(1767225600, 123_000_000);
        assert_eq!(result, "2026-01-01T00:00:00.123Z");
    }

    #[test]
    fn test_redacting_visitor_sensitive_detection() {
        assert!(RedactingVisitor::is_sensitive("token"));
        assert!(RedactingVisitor::is_sensitive("password"));
        assert!(RedactingVisitor::is_sensitive("api_key"));
        assert!(!RedactingVisitor::is_sensitive("command"));
        assert!(!RedactingVisitor::is_sensitive("status"));
    }

    #[test]
    fn test_redacting_visitor_url_detection() {
        assert!(RedactingVisitor::looks_like_url("https://example.com"));
        assert!(RedactingVisitor::looks_like_url("http://example.com"));
        assert!(!RedactingVisitor::looks_like_url("not a url"));
        assert!(!RedactingVisitor::looks_like_url("ftp://example.com"));
    }

    // --- P0: format_rfc3339() tests ---

    #[test]
    fn test_format_rfc3339_unix_epoch() {
        // Unix epoch: 0 seconds, 0 nanos
        let result = format_rfc3339(0, 0);
        assert_eq!(result, "1970-01-01T00:00:00.000Z");
    }

    #[test]
    fn test_format_rfc3339_known_timestamp_2024() {
        // 1704067200 = 2024-01-01T00:00:00Z
        let result = format_rfc3339(1704067200, 0);
        assert_eq!(result, "2024-01-01T00:00:00.000Z");
    }

    #[test]
    fn test_format_rfc3339_leap_year_feb29() {
        // 1709209845 = 2024-02-29T12:30:45Z (2024 is a leap year)
        let result = format_rfc3339(1709209845, 0);
        assert_eq!(result, "2024-02-29T12:30:45.000Z");
    }

    #[test]
    fn test_format_rfc3339_end_of_year_boundary() {
        // 1735689599 = 2024-12-31T23:59:59Z
        let result = format_rfc3339(1735689599, 0);
        assert_eq!(result, "2024-12-31T23:59:59.000Z");
    }

    #[test]
    fn test_format_rfc3339_end_of_year_with_millis() {
        // 1735689599 = 2024-12-31T23:59:59Z with 999 ms
        let result = format_rfc3339(1735689599, 999_000_000);
        assert_eq!(result, "2024-12-31T23:59:59.999Z");
    }

    // --- P0: days_to_ymd() tests ---

    #[test]
    fn test_days_to_ymd_epoch() {
        // Day 0 = 1970-01-01
        let (y, m, d) = days_to_ymd(0);
        assert_eq!((y, m, d), (1970, 1, 1));
    }

    #[test]
    fn test_days_to_ymd_known_date_2024() {
        // 19723 days since epoch = 2024-01-01
        let (y, m, d) = days_to_ymd(19723);
        assert_eq!((y, m, d), (2024, 1, 1));
    }

    #[test]
    fn test_days_to_ymd_leap_year_feb29() {
        // 19782 days since epoch = 2024-02-29 (leap year)
        let (y, m, d) = days_to_ymd(19782);
        assert_eq!((y, m, d), (2024, 2, 29));
    }

    #[test]
    fn test_days_to_ymd_after_leap_day() {
        // 11017 days since epoch = 2000-03-01 (day after Feb 29, 2000)
        let (y, m, d) = days_to_ymd(11017);
        assert_eq!((y, m, d), (2000, 3, 1));
    }
}

//! Sensitive field redaction layer for tracing events.
//!
//! Provides a custom JSON formatter that intercepts tracing events and replaces
//! sensitive field values with `[REDACTED]` before they are written to output.

use serde_json::Value;
use tracing::{Event, Subscriber};
use tracing_subscriber::fmt::FormattedFields;
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

/// Pre-computed suffixes for compound field detection (e.g., `_token`, `-key`).
/// Avoids per-call `format!` allocations in `is_sensitive()`.
const SENSITIVE_SUFFIXES: &[&str] = &[
    "_token", "-token",
    "_api_key", "-api_key",
    "_apikey", "-apikey",
    "_key", "-key",
    "_secret", "-secret",
    "_password", "-password",
    "_passwd", "-passwd",
    "_pwd", "-pwd",
    "_auth", "-auth",
    "_authorization", "-authorization",
    "_credential", "-credential",
    "_credentials", "-credentials",
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
///
/// # Limitation
///
/// Only **named fields** (e.g., `tracing::info!(token = "x", ...)`) are scanned
/// for sensitive data. Free-text message content (the format string) is **not**
/// inspected — callers must avoid embedding secrets directly in log messages.
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
        // Exact match first
        if SENSITIVE_FIELDS.contains(&lower.as_str()) {
            return true;
        }
        // Suffix match for compound field names like access_token,
        // auth_token, session_key, api_secret, etc.
        // Uses pre-computed SENSITIVE_SUFFIXES to avoid per-call allocations.
        SENSITIVE_SUFFIXES.iter().any(|suffix| lower.ends_with(suffix))
    }

    fn looks_like_url(value: &str) -> bool {
        let lower = value.to_ascii_lowercase();
        lower.starts_with("http://") || lower.starts_with("https://")
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

        // Note: outer quotes are stripped but inner Debug-escaped sequences (e.g., `\"`,
        // `\\`) are NOT unescaped. This is intentional — a full unescape would require
        // replicating Rust's Debug parser and could introduce bugs on non-standard Debug
        // impls. The raw escaped content is safe and lossless for log consumers.
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

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        let name = field.name();
        if Self::is_sensitive(name) {
            self.fields
                .insert(name.to_string(), Value::String("[REDACTED]".to_string()));
        } else if let Some(n) = serde_json::Number::from_f64(value) {
            self.fields
                .insert(name.to_string(), Value::Number(n));
        } else {
            // NaN/Infinity cannot be represented as JSON numbers
            self.fields
                .insert(name.to_string(), Value::Null);
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
        ctx: &FmtContext<'_, S, N>,
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

        // Parent spans (from root to leaf), when available.
        // Span fields are re-parsed from their pre-rendered format and redacted
        // through the same `is_sensitive` / URL-redaction pipeline as event fields,
        // ensuring AC #2 compliance (no sensitive data leaks via spans).
        if let Some(scope) = ctx.event_scope() {
            let mut spans = Vec::new();
            for span in scope.from_root() {
                let mut span_obj = serde_json::Map::new();
                span_obj.insert(
                    "name".to_string(),
                    Value::String(span.metadata().name().to_string()),
                );

                let ext = span.extensions();
                if let Some(fields) = ext.get::<FormattedFields<N>>() {
                    let rendered = fields.fields.as_str().trim();
                    if !rendered.is_empty() {
                        let span_fields = parse_and_redact_span_fields(rendered);
                        if !span_fields.is_empty() {
                            span_obj.insert(
                                "fields".to_string(),
                                Value::Object(span_fields),
                            );
                        }
                    }
                }

                spans.push(Value::Object(span_obj));
            }

            if !spans.is_empty() {
                obj.insert("spans".to_string(), Value::Array(spans));
            }
        }

        let json_str = serde_json::to_string(&obj).map_err(|_| std::fmt::Error)?;
        writeln!(writer, "{}", json_str)?;

        Ok(())
    }
}

/// Parse pre-rendered span fields (format: `key=value key2="string"`) and redact
/// sensitive values. Returns a structured JSON map instead of an opaque flat string.
///
/// `DefaultFields` renders span fields as space-separated `key=debug_value` pairs:
/// - String values: `key="value"` (Debug-formatted with surrounding quotes)
/// - Numbers: `key=42`
/// - Booleans: `key=true`
///
/// This function splits on `key=` boundaries, applies `is_sensitive()` and URL
/// redaction, and returns individual key-value entries as a `serde_json::Map`.
fn parse_and_redact_span_fields(rendered: &str) -> serde_json::Map<String, Value> {
    let mut result = serde_json::Map::new();

    // Split into key=value segments. We scan for patterns where a word followed
    // by '=' starts a new field.
    let mut remaining = rendered.trim();

    while !remaining.is_empty() {
        // Find the next '=' to extract the key
        let eq_pos = match remaining.find('=') {
            Some(p) => p,
            None => break,
        };

        let key = &remaining[..eq_pos];
        remaining = &remaining[eq_pos + 1..];

        // Parse the value: either quoted string or bare token
        let (value_str, rest) = if let Some(after_quote) = remaining.strip_prefix('"') {
            // Quoted value: find matching close quote (handling escaped quotes)
            parse_quoted_value(after_quote)
        } else {
            // Bare value: read until next space or end
            match remaining.find(' ') {
                Some(sp) => (&remaining[..sp], remaining[sp..].trim_start()),
                None => (remaining, ""),
            }
        };

        // Apply redaction and preserve JSON types where possible
        let value = if RedactingVisitor::is_sensitive(key) {
            Value::String("[REDACTED]".to_string())
        } else if RedactingVisitor::looks_like_url(value_str) {
            Value::String(tf_config::redact_url_sensitive_params(value_str))
        } else {
            // Try to preserve numeric/bool types from bare (unquoted) values
            parse_typed_value(value_str)
        };
        result.insert(key.to_string(), value);

        remaining = rest;
    }

    result
}

/// Parse a Debug-formatted quoted string value, returning `(value_content, remaining)`.
/// Input starts AFTER the opening quote.
fn parse_quoted_value(input: &str) -> (&str, &str) {
    let mut chars = input.char_indices();
    while let Some((i, ch)) = chars.next() {
        match ch {
            '\\' => {
                // Skip escaped character
                chars.next();
            }
            '"' => {
                // Found closing quote
                let value = &input[..i];
                let rest = &input[i + 1..];
                return (value, rest.trim_start());
            }
            _ => {}
        }
    }
    // No closing quote found — treat rest as value
    (input, "")
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

/// Attempt to parse a bare (unquoted) span field value into its JSON type.
/// - Integers → `Value::Number`
/// - `true`/`false` → `Value::Bool`
/// - Everything else → `Value::String`
///
/// Quoted values (already stripped of quotes by the caller) are always treated
/// as strings since `DefaultFields` quotes string-typed span fields.
fn parse_typed_value(s: &str) -> Value {
    if s == "true" {
        return Value::Bool(true);
    }
    if s == "false" {
        return Value::Bool(false);
    }
    // Try integer first (most common numeric span field type)
    if let Ok(n) = s.parse::<i64>() {
        return Value::Number(n.into());
    }
    if let Ok(n) = s.parse::<u64>() {
        return Value::Number(n.into());
    }
    // Try float
    if let Ok(f) = s.parse::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            return Value::Number(n);
        }
    }
    Value::String(s.to_string())
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

    // Test [AI-Review-R3 M1]: compound field names detected via suffix matching
    #[test]
    fn test_redacting_visitor_sensitive_compound_fields() {
        // Underscore-separated compound names
        assert!(RedactingVisitor::is_sensitive("access_token"));
        assert!(RedactingVisitor::is_sensitive("auth_token"));
        assert!(RedactingVisitor::is_sensitive("session_key"));
        assert!(RedactingVisitor::is_sensitive("api_secret"));
        assert!(RedactingVisitor::is_sensitive("user_password"));
        assert!(RedactingVisitor::is_sensitive("db_credential"));
        // Hyphen-separated compound names
        assert!(RedactingVisitor::is_sensitive("access-token"));
        assert!(RedactingVisitor::is_sensitive("api-key"));
        assert!(RedactingVisitor::is_sensitive("session-secret"));
        // Non-sensitive compound fields must NOT match
        assert!(!RedactingVisitor::is_sensitive("token_count"));
        assert!(!RedactingVisitor::is_sensitive("password_length"));
        assert!(!RedactingVisitor::is_sensitive("secret_level"));
    }

    // Test [AI-Review-R3 M1]: compound sensitive fields redacted in log output
    #[test]
    fn test_compound_sensitive_field_redacted_in_output() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };
        let guard = init_logging(&config).unwrap();
        tracing::info!(access_token = "my_secret_tok_123", "compound field test");
        drop(guard);
        let content = fs::read_to_string(find_log_file(&log_dir)).unwrap();
        assert!(!content.contains("my_secret_tok_123"),
                "Compound field 'access_token' value should be redacted");
        assert!(content.contains("[REDACTED]"));
    }

    #[test]
    fn test_redacting_visitor_url_detection() {
        assert!(RedactingVisitor::looks_like_url("https://example.com"));
        assert!(RedactingVisitor::looks_like_url("http://example.com"));
        assert!(!RedactingVisitor::looks_like_url("not a url"));
        assert!(!RedactingVisitor::looks_like_url("ftp://example.com"));
    }

    // Test [AI-Review-R3 L3]: case-insensitive URL detection
    #[test]
    fn test_redacting_visitor_url_detection_case_insensitive() {
        assert!(RedactingVisitor::looks_like_url("HTTP://example.com"));
        assert!(RedactingVisitor::looks_like_url("HTTPS://example.com"));
        assert!(RedactingVisitor::looks_like_url("Http://example.com"));
        assert!(RedactingVisitor::looks_like_url("hTtPs://example.com"));
    }

    // Test [AI-Review-R3 L1]: float values stored as JSON numbers, not strings
    #[test]
    fn test_float_values_stored_as_json_numbers() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };
        let guard = init_logging(&config).unwrap();
        tracing::info!(duration = 42.5, "float test");
        drop(guard);
        let content = fs::read_to_string(find_log_file(&log_dir)).unwrap();
        let line = content.lines().last().unwrap();
        let json: serde_json::Value = serde_json::from_str(line).unwrap();
        let fields = json.get("fields").expect("Missing fields");
        let duration = fields.get("duration").expect("Missing duration field");
        assert!(duration.is_number(), "Float should be stored as JSON number, got: {duration}");
    }

    // Test [AI-Review-R4 M2]: numeric and bool sensitive fields are redacted
    #[test]
    fn test_numeric_sensitive_fields_redacted() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };
        let guard = init_logging(&config).unwrap();
        tracing::info!(token = 42_i64, api_key = 99_u64, secret = true, "numeric sensitive test");
        drop(guard);
        let content = fs::read_to_string(find_log_file(&log_dir)).unwrap();
        let line = content.lines().last().unwrap();
        let json: serde_json::Value = serde_json::from_str(line).unwrap();
        let fields = json.get("fields").expect("Missing fields");
        // All three sensitive fields should be "[REDACTED]", not their numeric/bool values
        assert_eq!(fields.get("token").unwrap(), "[REDACTED]",
            "i64 sensitive field 'token' should be redacted");
        assert_eq!(fields.get("api_key").unwrap(), "[REDACTED]",
            "u64 sensitive field 'api_key' should be redacted");
        assert_eq!(fields.get("secret").unwrap(), "[REDACTED]",
            "bool sensitive field 'secret' should be redacted");
        // Ensure the raw numeric values don't appear
        assert!(!content.contains("\"42\"") && !content.contains(":42,") && !content.contains(":42}"),
            "Numeric value 42 should not appear in output");
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

    // --- parse_and_redact_span_fields() tests ---

    #[test]
    fn test_parse_and_redact_span_fields_sensitive_redacted() {
        let rendered = "command=\"triage\" token=\"secret123\"";
        let result = parse_and_redact_span_fields(rendered);
        assert_eq!(result.get("command").unwrap(), "triage");
        assert_eq!(result.get("token").unwrap(), "[REDACTED]");
    }

    #[test]
    fn test_parse_and_redact_span_fields_bare_values() {
        let rendered = "count=42 enabled=true";
        let result = parse_and_redact_span_fields(rendered);
        assert_eq!(result.get("count").unwrap(), 42);
        assert_eq!(result.get("enabled").unwrap(), true);
    }

    #[test]
    fn test_parse_and_redact_span_fields_url_redacted() {
        let rendered = "endpoint=\"https://api.example.com?token=abc123\"";
        let result = parse_and_redact_span_fields(rendered);
        let endpoint = result.get("endpoint").unwrap().as_str().unwrap();
        assert!(!endpoint.contains("abc123"), "URL token should be redacted");
        assert!(endpoint.contains("[REDACTED]"));
    }

    #[test]
    fn test_parse_and_redact_span_fields_compound_sensitive() {
        let rendered = "access_token=\"mysecret\" scope=\"lot-42\"";
        let result = parse_and_redact_span_fields(rendered);
        assert_eq!(result.get("access_token").unwrap(), "[REDACTED]");
        assert_eq!(result.get("scope").unwrap(), "lot-42");
    }

    // Test [AI-Review-R7 M3]: span fields preserve numeric and boolean types
    #[test]
    fn test_parse_and_redact_span_fields_preserves_types() {
        let rendered = "count=42 enabled=true ratio=3.14 name=\"alice\"";
        let result = parse_and_redact_span_fields(rendered);
        assert!(result.get("count").unwrap().is_number(),
            "Integer span field should be parsed as JSON number");
        assert_eq!(result.get("count").unwrap(), 42);
        assert!(result.get("enabled").unwrap().is_boolean(),
            "Boolean span field should be parsed as JSON boolean");
        assert_eq!(result.get("enabled").unwrap(), true);
        assert!(result.get("ratio").unwrap().is_number(),
            "Float span field should be parsed as JSON number");
        assert!(result.get("name").unwrap().is_string(),
            "Quoted span field should remain a JSON string");
        assert_eq!(result.get("name").unwrap(), "alice");
    }

    // Test [AI-Review-R7 M3]: span typed fields in full log output
    #[test]
    fn test_span_typed_fields_in_log_output() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };
        let guard = init_logging(&config).unwrap();
        let span = tracing::info_span!("batch", count = 42_i64, active = true);
        let _entered = span.enter();
        tracing::info!("typed span fields test");
        drop(_entered);
        drop(guard);
        let content = fs::read_to_string(find_log_file(&log_dir)).unwrap();
        let line = content.lines().last().unwrap();
        let json: serde_json::Value = serde_json::from_str(line).unwrap();
        let spans = json.get("spans").and_then(|v| v.as_array())
            .expect("Expected 'spans' array");
        let span_obj = &spans[0];
        let fields = span_obj.get("fields").expect("Expected 'fields' in span");
        let count = fields.get("count").expect("Missing count field");
        let active = fields.get("active").expect("Missing active field");
        assert!(count.is_number(), "count should be a JSON number, got: {count}");
        assert_eq!(count, 42);
        assert!(active.is_boolean(), "active should be a JSON boolean, got: {active}");
        assert_eq!(active, true);
    }

    // Test [AI-Review-R6 H2]: span fields with sensitive data are redacted in log output
    #[test]
    fn test_span_sensitive_fields_redacted_in_log_output() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };
        let guard = init_logging(&config).unwrap();
        let span = tracing::info_span!("auth", token = "super_secret_value");
        let _entered = span.enter();
        tracing::info!("inside span with sensitive field");
        drop(_entered);
        drop(guard);
        let content = fs::read_to_string(find_log_file(&log_dir)).unwrap();
        assert!(!content.contains("super_secret_value"),
                "Span sensitive field 'token' value should be redacted in log output");
        assert!(content.contains("[REDACTED]"),
                "Span field should show [REDACTED]");
    }

    // Test [AI-Review-R7 M2]: free-text message is NOT scanned for secrets
    // This test documents the known limitation and serves as a guardrail reminder:
    // callers MUST use named fields (e.g., `token = "x"`) for sensitive data,
    // never embed secrets in the message format string.
    #[test]
    fn test_free_text_message_not_scanned_for_secrets() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };
        let guard = init_logging(&config).unwrap();
        // WRONG pattern (do not do this!): secret embedded in message text
        // This test proves the limitation exists and is documented.
        tracing::info!("Connecting to service with token=secret_in_message_abc");
        // CORRECT pattern: secret in a named field (gets redacted)
        tracing::info!(token = "secret_in_field_xyz", "Connecting to service");
        drop(guard);
        let content = fs::read_to_string(find_log_file(&log_dir)).unwrap();
        // Named field IS redacted (correct behavior)
        assert!(!content.contains("secret_in_field_xyz"),
                "Named field secret should be redacted");
        // Free-text message is NOT scanned (known limitation, documented)
        assert!(content.contains("secret_in_message_abc"),
                "Free-text message is NOT scanned — this is a documented limitation. \
                 Callers must use named fields for sensitive data.");
    }

    // Test [AI-Review-R6 L3]: span fields rendered as structured JSON, not opaque string
    #[test]
    fn test_span_fields_rendered_as_structured_json() {
        let temp = tempdir().unwrap();
        let log_dir = temp.path().join("logs");
        let config = LoggingConfig {
            log_level: "info".to_string(),
            log_dir: log_dir.to_string_lossy().to_string(),
            log_to_stdout: false,
        };
        let guard = init_logging(&config).unwrap();
        let span = tracing::info_span!("cli_cmd", command = "triage", scope = "lot-42");
        let _entered = span.enter();
        tracing::info!("test structured spans");
        drop(_entered);
        drop(guard);
        let content = fs::read_to_string(find_log_file(&log_dir)).unwrap();
        let line = content.lines().last().unwrap();
        let json: serde_json::Value = serde_json::from_str(line).unwrap();
        let spans = json.get("spans").and_then(|v| v.as_array())
            .expect("Expected 'spans' array");
        let span_obj = &spans[0];
        let fields = span_obj.get("fields").expect("Expected 'fields' in span");
        // Fields should be a JSON object, not a string
        assert!(fields.is_object(),
            "Span fields should be a JSON object, got: {fields}");
        let fields_map = fields.as_object().unwrap();
        assert_eq!(fields_map.get("command").unwrap(), "triage");
        assert_eq!(fields_map.get("scope").unwrap(), "lot-42");
    }
}

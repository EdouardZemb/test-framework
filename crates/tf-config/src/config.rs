//! Configuration structures and loading logic

use crate::error::ConfigError;
use serde::Deserialize;
use std::fmt;
use std::path::Path;

/// Main project configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    /// Name of the project
    pub project_name: String,

    /// Jira integration configuration
    #[serde(default)]
    pub jira: Option<JiraConfig>,

    /// Squash integration configuration
    #[serde(default)]
    pub squash: Option<SquashConfig>,

    /// Output folder for generated files
    pub output_folder: String,

    /// Template file paths
    #[serde(default)]
    pub templates: Option<TemplatesConfig>,

    /// LLM configuration
    #[serde(default)]
    pub llm: Option<LlmConfig>,
}

/// Jira integration configuration
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JiraConfig {
    /// Jira server endpoint URL
    pub endpoint: String,

    /// API token (sensitive - will be redacted in logs via custom Debug impl)
    #[serde(default)]
    pub token: Option<String>,
}

/// Squash integration configuration
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SquashConfig {
    /// Squash server endpoint URL
    pub endpoint: String,

    /// Username for authentication
    #[serde(default)]
    pub username: Option<String>,

    /// Password (sensitive - will be redacted in logs via custom Debug impl)
    #[serde(default)]
    pub password: Option<String>,
}

/// Template file paths configuration for document generation.
///
/// All template paths are optional. When provided, they should point to
/// valid template files that will be used for generating reports and documents.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplatesConfig {
    /// Path to CR (compte-rendu/daily report) template file.
    ///
    /// Used for generating daily status reports. Typically a Markdown file
    /// with placeholders for test execution data.
    /// Example: `"./templates/cr.md"`
    #[serde(default)]
    pub cr: Option<String>,

    /// Path to PPT (PowerPoint presentation) template file.
    ///
    /// Used for generating weekly status presentations and TNR reports.
    /// Should be a `.pptx` file with placeholder slides.
    /// Example: `"./templates/report.pptx"`
    #[serde(default)]
    pub ppt: Option<String>,

    /// Path to anomaly report template file.
    ///
    /// Used for generating standardized bug/anomaly reports.
    /// Typically a Markdown file with sections for reproduction steps,
    /// expected vs actual behavior, and evidence links.
    /// Example: `"./templates/anomaly.md"`
    #[serde(default)]
    pub anomaly: Option<String>,
}

/// LLM operation mode
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LlmMode {
    /// Automatically select local or cloud based on availability
    #[default]
    Auto,
    /// Use only local LLM (e.g., Ollama)
    Local,
    /// Use cloud LLM provider
    Cloud,
}

impl std::fmt::Display for LlmMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmMode::Auto => write!(f, "auto"),
            LlmMode::Local => write!(f, "local"),
            LlmMode::Cloud => write!(f, "cloud"),
        }
    }
}

/// LLM (Large Language Model) configuration
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LlmConfig {
    /// LLM mode: auto, local, or cloud
    #[serde(default)]
    pub mode: LlmMode,

    /// Local LLM endpoint (e.g., Ollama at `http://localhost:11434`)
    #[serde(default)]
    pub local_endpoint: Option<String>,

    /// Local model name (e.g., `mistral:7b-instruct`, `codellama:13b`)
    #[serde(default)]
    pub local_model: Option<String>,

    /// Whether cloud LLM is enabled (required for cloud/auto mode to use cloud)
    #[serde(default)]
    pub cloud_enabled: bool,

    /// Cloud LLM endpoint (e.g., `https://api.openai.com/v1`)
    #[serde(default)]
    pub cloud_endpoint: Option<String>,

    /// Cloud model name (e.g., `gpt-4o-mini`)
    #[serde(default)]
    pub cloud_model: Option<String>,

    /// API key for cloud LLM (sensitive - will be redacted in logs via custom Debug impl)
    #[serde(default)]
    pub api_key: Option<String>,

    /// Request timeout in seconds (default: 120)
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u32,

    /// Maximum tokens for LLM response (default: 4096)
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

fn default_timeout_seconds() -> u32 {
    120
}

fn default_max_tokens() -> u32 {
    4096
}

/// Redact sensitive information from a URL for safe logging.
///
/// This function redacts:
/// 1. Userinfo credentials (`scheme://user:pass@host` -> `scheme://[REDACTED]@host`)
/// 2. Sensitive query parameters (token, api_key, password, etc.)
///
/// Sensitive query parameters that will be redacted:
/// - token, api_key, apikey, key, secret, password, passwd, pwd, auth, authorization
///
/// Example:
/// - `https://user:secret@jira.example.com` -> `https://[REDACTED]@jira.example.com`
/// - `https://jira.example.com?token=secret123` -> `https://jira.example.com?token=[REDACTED]`
/// - `https://api.example.com?api_key=sk-123&foo=bar` -> `https://api.example.com?api_key=[REDACTED]&foo=bar`
fn redact_url_sensitive_params(url: &str) -> String {
    // List of sensitive parameter names (case-insensitive matching)
    // Includes both snake_case and camelCase variants
    const SENSITIVE_PARAMS: &[&str] = &[
        // Common names
        "token", "api_key", "apikey", "key", "secret",
        "password", "passwd", "pwd", "auth", "authorization",
        // snake_case variants
        "access_token", "refresh_token", "bearer", "credentials",
        "client_secret", "private_key", "session_token", "auth_token",
        // camelCase variants
        "accesstoken", "refreshtoken", "clientsecret", "privatekey",
        "sessiontoken", "authtoken", "apitoken", "secretkey",
        "accesskey", "secretaccesskey",
        // kebab-case variants (with hyphens)
        "api-key", "access-token", "refresh-token", "client-secret",
        "private-key", "session-token", "auth-token", "secret-key",
        "access-key",
    ];

    /// URL percent-decoding for parameter names with double-encoding support.
    /// Decodes %XX sequences recursively (up to 3 iterations) to handle double-encoded
    /// param names like `api%255Fkey` (double-encoded api_key: api_key -> api%5Fkey -> api%255Fkey).
    fn percent_decode(s: &str) -> String {
        fn decode_once(input: &str) -> String {
            let mut result = String::with_capacity(input.len());
            let mut chars = input.chars().peekable();
            while let Some(c) = chars.next() {
                if c == '%' {
                    // Try to read two hex digits
                    let hex: String = chars.by_ref().take(2).collect();
                    if hex.len() == 2 {
                        if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                            result.push(byte as char);
                            continue;
                        }
                    }
                    // Invalid encoding, keep original
                    result.push('%');
                    result.push_str(&hex);
                } else if c == '+' {
                    // + is space in application/x-www-form-urlencoded
                    result.push(' ');
                } else {
                    result.push(c);
                }
            }
            result
        }

        // Decode iteratively until stable or max iterations reached
        // This handles double-encoding (api%255Fkey -> api%5Fkey -> api_key)
        let mut current = s.to_string();
        for _ in 0..3 {
            let decoded = decode_once(&current);
            if decoded == current {
                break; // Stable, no more encoding
            }
            current = decoded;
        }
        current
    }

    /// Helper to redact sensitive params in a key=value string (query or fragment).
    /// Handles:
    /// - URL-encoded parameter names (e.g., `api%5Fkey` for `api_key`)
    /// - Both `&` and `;` as parameter separators (RFC 1866, HTML forms)
    /// - Whitespace around parameter names (e.g., `token =value` or ` api_key=value`)
    fn redact_params(params: &str, sensitive: &[&str]) -> String {
        // Split by both & and ; separators, but track which separator was used
        // to reconstruct the string with original separators
        let mut result = String::with_capacity(params.len());
        let mut current_pos = 0;

        // Find all separator positions
        let mut separators: Vec<(usize, char)> = Vec::new();
        for (i, c) in params.char_indices() {
            if c == '&' || c == ';' {
                separators.push((i, c));
            }
        }
        separators.push((params.len(), '\0')); // Sentinel for last segment

        for (sep_pos, sep_char) in separators {
            let param = &params[current_pos..sep_pos];

            let redacted_param = if let Some(eq_pos) = param.find('=') {
                let (name, _value) = param.split_at(eq_pos);
                // Trim whitespace from parameter name to handle "token = value" cases
                let trimmed_name = name.trim();
                // Decode the parameter name to handle URL-encoded keys like api%5Fkey
                let decoded_name = percent_decode(trimmed_name);
                let name_lower = decoded_name.to_lowercase();
                if sensitive.iter().any(|&s| name_lower == s) {
                    format!("{}=[REDACTED]", name)
                } else {
                    param.to_string()
                }
            } else {
                param.to_string()
            };

            result.push_str(&redacted_param);
            if sep_char != '\0' {
                result.push(sep_char);
            }
            current_pos = sep_pos + 1;
        }

        result
    }

    // First, redact userinfo (user:pass@host) if present
    // URL format: scheme://[userinfo@]host[:port][/path][?query][#fragment]
    let url = redact_url_userinfo(url);

    /// Helper to redact secrets in path and return (scheme://host, redacted_path)
    fn redact_base_path(base: &str) -> String {
        // Find the path portion (after scheme://host)
        if let Some(scheme_end) = base.find("://") {
            let after_scheme = &base[scheme_end + 3..];
            if let Some(path_start) = after_scheme.find('/') {
                let host = &after_scheme[..path_start];
                let path = &after_scheme[path_start..];
                let redacted_path = redact_url_path_secrets(path);
                return format!("{}://{}{}", &base[..scheme_end], host, redacted_path);
            }
        }
        base.to_string()
    }

    // Find query and fragment positions
    let query_start = url.find('?');
    let fragment_start = url.find('#');

    match (query_start, fragment_start) {
        // Both query and fragment present
        (Some(q_pos), Some(f_pos)) if q_pos < f_pos => {
            // Query comes before fragment: ?query#fragment
            let base = redact_base_path(&url[..q_pos]);
            let query = &url[q_pos + 1..f_pos];
            let fragment = &url[f_pos + 1..];

            let redacted_query = if query.is_empty() {
                String::new()
            } else {
                redact_params(query, SENSITIVE_PARAMS)
            };
            let redacted_fragment = if fragment.is_empty() {
                String::new()
            } else {
                redact_params(fragment, SENSITIVE_PARAMS)
            };

            format!("{}?{}#{}", base, redacted_query, redacted_fragment)
        }
        // Only query present (no fragment)
        (Some(q_pos), None) => {
            let base = redact_base_path(&url[..q_pos]);
            let query = &url[q_pos + 1..];
            if query.is_empty() {
                return format!("{}?", base);
            }
            let redacted_query = redact_params(query, SENSITIVE_PARAMS);
            format!("{}?{}", base, redacted_query)
        }
        // Only fragment present (no query)
        (None, Some(f_pos)) => {
            let base = redact_base_path(&url[..f_pos]);
            let fragment = &url[f_pos + 1..];
            if fragment.is_empty() {
                return format!("{}#", base);
            }
            let redacted_fragment = redact_params(fragment, SENSITIVE_PARAMS);
            format!("{}#{}", base, redacted_fragment)
        }
        // Fragment before query (invalid URL but handle gracefully)
        (Some(q_pos), Some(f_pos)) if f_pos < q_pos => {
            // Fragment before query is unusual - just redact the fragment part
            let base = redact_base_path(&url[..f_pos]);
            let rest = &url[f_pos + 1..];
            let redacted_rest = redact_params(rest, SENSITIVE_PARAMS);
            format!("{}#{}", base, redacted_rest)
        }
        // No query or fragment
        _ => redact_base_path(&url),
    }
}

/// Redact potential secrets in URL path segments.
///
/// Some APIs embed secrets in URL paths like `/api/token/{token}/resource`.
/// This function looks for path segments that follow sensitive segment names.
///
/// Example:
/// - `/api/token/sk-12345/resource` -> `/api/token/[REDACTED]/resource`
/// - `/key/abcdef123/data` -> `/key/[REDACTED]/data`
fn redact_url_path_secrets(path: &str) -> String {
    // Sensitive path segment indicators - the segment AFTER these will be redacted
    const SENSITIVE_PATH_SEGMENTS: &[&str] = &[
        "token", "tokens", "api_key", "apikey", "key", "keys", "secret", "secrets",
        "password", "auth", "credential", "credentials", "access_token",
    ];

    let segments: Vec<&str> = path.split('/').collect();
    if segments.len() < 2 {
        return path.to_string();
    }

    let mut result = Vec::with_capacity(segments.len());
    let mut redact_next = false;

    for segment in segments {
        if redact_next && !segment.is_empty() {
            // Only redact if it looks like a secret (not a simple ID or number)
            // Secrets typically have: length > 8, mix of chars, or long hex-like patterns
            let looks_like_secret = segment.len() > 8
                || (segment.len() > 4 && segment.chars().any(|c| c.is_ascii_digit())
                    && segment.chars().any(|c| c.is_ascii_alphabetic()))
                || segment.starts_with("sk-")
                || segment.starts_with("pk-")
                // Hex strings only count as secrets if they're reasonably long (> 8 chars)
                || (segment.len() > 8 && segment.chars().all(|c| c.is_ascii_hexdigit()));

            if looks_like_secret {
                result.push("[REDACTED]");
            } else {
                result.push(segment);
            }
            redact_next = false;
        } else {
            result.push(segment);
            let lower = segment.to_lowercase();
            redact_next = SENSITIVE_PATH_SEGMENTS.iter().any(|&s| lower == s);
        }
    }

    result.join("/")
}

/// Redact userinfo (user:password) from a URL.
///
/// RFC 3986 defines userinfo as: `userinfo = user [ ":" password ]`
/// located between `scheme://` and `@host`
///
/// The userinfo delimiter is the LAST `@` before the host, because passwords
/// may contain unencoded `@` characters (e.g., `user:p@ss@host`).
///
/// Example:
/// - `https://admin:secret@example.com` -> `https://[REDACTED]@example.com`
/// - `https://user@example.com` -> `https://[REDACTED]@example.com`
/// - `https://admin:p@ss@example.com` -> `https://[REDACTED]@example.com`
/// - `https://example.com` -> `https://example.com` (unchanged)
fn redact_url_userinfo(url: &str) -> String {
    // Find the scheme separator
    let Some(scheme_end) = url.find("://") else {
        return url.to_string();
    };

    let after_scheme = &url[scheme_end + 3..];

    // Find the end of the authority section (before path, query, or fragment)
    let path_start = after_scheme.find('/').unwrap_or(after_scheme.len());
    let query_start = after_scheme.find('?').unwrap_or(after_scheme.len());
    let fragment_start = after_scheme.find('#').unwrap_or(after_scheme.len());
    let end_of_authority = path_start.min(query_start).min(fragment_start);

    // Extract just the authority section
    let authority = &after_scheme[..end_of_authority];

    // Find the LAST @ in authority - this is the userinfo/host delimiter
    // Passwords may contain unencoded @ characters (e.g., user:p@ss@host)
    let Some(at_pos) = authority.rfind('@') else {
        return url.to_string();
    };

    // Reconstruct URL with redacted userinfo
    let scheme = &url[..scheme_end + 3]; // includes "://"
    let host_part = &authority[at_pos + 1..]; // after the @
    let rest = &after_scheme[end_of_authority..]; // path, query, fragment

    format!("{}[REDACTED]@{}{}", scheme, host_part, rest)
}

/// Trait for redacting sensitive information in display output
pub trait Redact {
    /// Returns a redacted version suitable for logging
    fn redacted(&self) -> String;
}

impl Redact for JiraConfig {
    fn redacted(&self) -> String {
        format!(
            "JiraConfig {{ endpoint: {:?}, token: [REDACTED] }}",
            redact_url_sensitive_params(&self.endpoint)
        )
    }
}

impl Redact for SquashConfig {
    fn redacted(&self) -> String {
        format!(
            "SquashConfig {{ endpoint: {:?}, username: {:?}, password: [REDACTED] }}",
            redact_url_sensitive_params(&self.endpoint), self.username
        )
    }
}

impl Redact for LlmConfig {
    fn redacted(&self) -> String {
        // Redact sensitive query params in endpoints if present
        let redacted_local_endpoint = self
            .local_endpoint
            .as_ref()
            .map(|ep| redact_url_sensitive_params(ep));
        let redacted_cloud_endpoint = self
            .cloud_endpoint
            .as_ref()
            .map(|ep| redact_url_sensitive_params(ep));
        format!(
            "LlmConfig {{ mode: {:?}, local_endpoint: {:?}, local_model: {:?}, \
             cloud_enabled: {}, cloud_endpoint: {:?}, cloud_model: {:?}, \
             api_key: [REDACTED], timeout_seconds: {}, max_tokens: {} }}",
            self.mode, redacted_local_endpoint, self.local_model,
            self.cloud_enabled, redacted_cloud_endpoint, self.cloud_model,
            self.timeout_seconds, self.max_tokens
        )
    }
}

// Custom Debug implementations to prevent accidental logging of secrets
impl ProjectConfig {
    /// Check if output_folder exists and is a directory on the filesystem.
    ///
    /// Returns `Some(warning_message)` if:
    /// - The folder does not exist (will need to be created)
    /// - The path exists but is not a directory (e.g., it's a file)
    ///
    /// Returns `None` if the path exists and is a directory.
    ///
    /// This is an optional validation - the caller decides whether to
    /// warn the user, create the folder, or treat it as an error.
    pub fn check_output_folder_exists(&self) -> Option<String> {
        let path = Path::new(&self.output_folder);
        if !path.exists() {
            Some(format!(
                "output_folder '{}' does not exist - it will be created when needed",
                self.output_folder
            ))
        } else if !path.is_dir() {
            Some(format!(
                "output_folder '{}' exists but is not a directory",
                self.output_folder
            ))
        } else {
            None
        }
    }
}

impl fmt::Debug for JiraConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JiraConfig")
            .field("endpoint", &redact_url_sensitive_params(&self.endpoint))
            .field("token", &"[REDACTED]")
            .finish()
    }
}

impl fmt::Debug for SquashConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SquashConfig")
            .field("endpoint", &redact_url_sensitive_params(&self.endpoint))
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .finish()
    }
}

impl fmt::Debug for LlmConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Redact sensitive query params in endpoints if present
        let redacted_local_endpoint = self
            .local_endpoint
            .as_ref()
            .map(|ep| redact_url_sensitive_params(ep));
        let redacted_cloud_endpoint = self
            .cloud_endpoint
            .as_ref()
            .map(|ep| redact_url_sensitive_params(ep));
        f.debug_struct("LlmConfig")
            .field("mode", &self.mode)
            .field("local_endpoint", &redacted_local_endpoint)
            .field("local_model", &self.local_model)
            .field("cloud_enabled", &self.cloud_enabled)
            .field("cloud_endpoint", &redacted_cloud_endpoint)
            .field("cloud_model", &self.cloud_model)
            .field("api_key", &"[REDACTED]")
            .field("timeout_seconds", &self.timeout_seconds)
            .field("max_tokens", &self.max_tokens)
            .finish()
    }
}

/// Load and validate configuration from a YAML file
///
/// # Arguments
///
/// * `path` - Path to the configuration YAML file
///
/// # Returns
///
/// * `Ok(ProjectConfig)` - Successfully loaded and validated configuration
/// * `Err(ConfigError)` - Error during loading or validation
///
/// # Example
///
/// ```no_run
/// // Note: This example requires a config.yaml file to exist.
/// // See tests/fixtures/valid_config.yaml for a complete example.
/// use std::path::Path;
/// use tf_config::load_config;
///
/// let config = load_config(Path::new("config.yaml")).unwrap();
/// println!("Project: {}", config.project_name);
/// ```
///
/// For a working test example, see `test_load_valid_config` in the test module.
pub fn load_config(path: &Path) -> Result<ProjectConfig, ConfigError> {
    // Check if file exists
    if !path.exists() {
        return Err(ConfigError::FileNotFound {
            path: path.to_path_buf(),
        });
    }

    // Read file content
    let content = std::fs::read_to_string(path)?;

    // Parse YAML with enhanced error handling for missing required fields and invalid enum values
    let config: ProjectConfig = match serde_yaml::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            // Check if the error can be transformed into a user-friendly message
            let err_msg = e.to_string();
            if let Some(error_kind) = parse_serde_error(&err_msg) {
                return Err(match error_kind {
                    SerdeErrorKind::MissingField { field, hint } => {
                        ConfigError::missing_field(field, hint)
                    }
                    SerdeErrorKind::InvalidEnumValue { field, reason, hint } => {
                        ConfigError::invalid_value(field, reason, hint)
                    }
                    SerdeErrorKind::InvalidEnumValueDynamic { field, reason, hint } => {
                        ConfigError::invalid_value(field, reason, hint)
                    }
                    SerdeErrorKind::UnknownField { field, location, hint } => {
                        ConfigError::invalid_value(
                            format!("{}.{}", location, field),
                            "is not a recognized configuration field",
                            hint,
                        )
                    }
                });
            }
            return Err(ConfigError::ParseError(e));
        }
    };

    // Validate required fields
    validate_config(&config)?;

    Ok(config)
}

/// Result of parsing a serde error for user-friendly transformation
enum SerdeErrorKind {
    /// Missing required field
    MissingField { field: &'static str, hint: &'static str },
    /// Invalid enum variant (static field name)
    InvalidEnumValue { field: &'static str, reason: &'static str, hint: &'static str },
    /// Invalid enum variant (dynamic field name extracted from error message)
    InvalidEnumValueDynamic { field: String, reason: &'static str, hint: &'static str },
    /// Unknown field (when deny_unknown_fields is active)
    UnknownField { field: String, location: &'static str, hint: &'static str },
}

/// Extract field path from serde_yaml error message patterns.
///
/// Attempts to find field names in various formats:
/// - Backtick-quoted: `field_name`
/// - Path prefix: "jira.token: invalid type"
/// - Context suffix: "while parsing jira.token"
///
/// Returns an owned String to avoid memory leaks from static lifetime requirements.
fn extract_field_from_error(err_msg: &str) -> Option<String> {
    // Pattern 1: Look for field path like "jira.token:" at start or after newline
    // serde_yaml sometimes formats nested errors as "section.field: error"
    for pattern in ["jira.", "squash.", "llm.", "templates."] {
        if let Some(start) = err_msg.find(pattern) {
            let rest = &err_msg[start..];
            // Find end of field path (colon, space, or end of string)
            if let Some(end) = rest.find([':', ' ', '\n']) {
                let field_path = &rest[..end];
                // Validate it looks like a field path (only alphanumeric, dots, underscores)
                if field_path.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '_') {
                    return Some(field_path.to_string());
                }
            }
        }
    }

    // Pattern 2: Look for backtick-quoted field names
    if let Some(start) = err_msg.find('`') {
        let after_backtick = &err_msg[start + 1..];
        if let Some(end) = after_backtick.find('`') {
            let field_name = &after_backtick[..end];
            // Skip if it's a value like `auto` or `123` (these are enum values, not fields)
            if !field_name.chars().all(|c| c.is_ascii_digit())
                && !matches!(field_name, "auto" | "local" | "cloud" | "true" | "false")
            {
                return Some(field_name.to_string());
            }
        }
    }

    None
}

/// Extract line and column location from serde_yaml error message if available.
///
/// serde_yaml typically includes location as "at line X column Y" or similar.
/// Returns a formatted string like " (at line 5, column 3)" or empty string if not found.
fn extract_location_from_error(err_msg: &str) -> String {
    // Pattern: "at line X column Y" or "line X, column Y"
    if let Some(line_pos) = err_msg.find("line ") {
        let after_line = &err_msg[line_pos..];
        // Extract until end of location info
        // Stop at period (sentence end) or comma (but not the comma in "line X, column Y")
        // Using a closure for clarity instead of complex boolean expression
        let is_location_end = |c: char, pos: usize| {
            c == '.' || (c == ',' && pos > 0 && !after_line[..pos].ends_with("column"))
        };
        let end_pos = after_line
            .char_indices()
            .find(|&(pos, c)| is_location_end(c, pos))
            .map(|(pos, _)| pos);
        if let Some(end) = end_pos {
            let location = after_line[..end].trim();
            if !location.is_empty() && location.len() < 50 {
                return format!(" (at {})", location);
            }
        }
        // Fallback: take a reasonable chunk
        let location_end = after_line.len().min(30);
        let location = after_line[..location_end].trim();
        if location.chars().any(|c| c.is_ascii_digit()) {
            return format!(" (at {})", location.trim_end_matches(|c: char| !c.is_ascii_digit()));
        }
    }
    String::new()
}

/// Parse serde error message to extract field info and provide helpful hints.
///
/// # Supported Error Types
///
/// - Missing required fields (project_name, output_folder, endpoint)
/// - Invalid enum variants (llm.mode)
/// - Unknown fields (when deny_unknown_fields is active)
/// - Invalid type errors for nested sections (templates, llm, jira, squash)
/// - Invalid type errors for scalar fields (timeout_seconds, max_tokens, etc.)
///
/// # Limitations
///
/// This function uses string pattern matching to identify known fields. Unknown or
/// deeply nested fields not explicitly mapped will fall through to the generic
/// ParseError. This is a trade-off: explicit hints for common errors vs. generic
/// messages for edge cases. The patterns are based on serde_yaml error message format.
///
/// Section detection for `unknown field` errors uses the `expected one of` list from
/// serde_yaml which enumerates valid fields, allowing reliable section identification.
fn parse_serde_error(err_msg: &str) -> Option<SerdeErrorKind> {
    // Handle missing field errors: "missing field `field_name`"
    if err_msg.contains("missing field") {
        if err_msg.contains("`project_name`") {
            return Some(SerdeErrorKind::MissingField {
                field: "project_name",
                hint: "a non-empty project name string (e.g., 'my-project')",
            });
        }
        if err_msg.contains("`output_folder`") {
            return Some(SerdeErrorKind::MissingField {
                field: "output_folder",
                hint: "an output folder path (e.g., './output' or '/var/data')",
            });
        }
        if err_msg.contains("`endpoint`") {
            // Could be jira.endpoint or squash.endpoint
            if err_msg.contains("jira") || err_msg.to_lowercase().contains("jira") {
                return Some(SerdeErrorKind::MissingField {
                    field: "jira.endpoint",
                    hint: "a Jira server URL (e.g., 'https://jira.example.com')",
                });
            }
            if err_msg.contains("squash") || err_msg.to_lowercase().contains("squash") {
                return Some(SerdeErrorKind::MissingField {
                    field: "squash.endpoint",
                    hint: "a Squash server URL (e.g., 'https://squash.example.com')",
                });
            }
            // Generic endpoint
            return Some(SerdeErrorKind::MissingField {
                field: "endpoint",
                hint: "a valid URL endpoint",
            });
        }
    }

    // Handle invalid enum variant errors for LlmMode
    // serde_yaml format: "unknown variant `invalid`, expected one of `auto`, `local`, `cloud`"
    if err_msg.contains("unknown variant") && (err_msg.contains("auto") || err_msg.contains("local") || err_msg.contains("cloud")) {
        return Some(SerdeErrorKind::InvalidEnumValue {
            field: "llm.mode",
            reason: "is not a valid mode",
            hint: "one of: 'auto', 'local', or 'cloud'",
        });
    }

    // Handle unknown field errors (deny_unknown_fields): "unknown field `field_name`, expected one of ..."
    // Use the "expected one of" list to reliably determine which section we're in
    if err_msg.contains("unknown field") {
        // Extract the field name from backticks
        if let Some(start) = err_msg.find("unknown field `") {
            let after_prefix = &err_msg[start + 15..]; // len("unknown field `") = 15
            if let Some(end) = after_prefix.find('`') {
                let field_name = after_prefix[..end].to_string();

                // Determine the location based on "expected one of" list (more reliable than heuristic)
                // serde_yaml includes valid field names in the error message
                let location = detect_section_from_expected_fields(err_msg);

                let hint = match location {
                    "jira" => "valid jira fields are: endpoint, token",
                    "squash" => "valid squash fields are: endpoint, username, password",
                    "llm" => "valid llm fields are: mode, local_endpoint, local_model, cloud_enabled, cloud_endpoint, cloud_model, api_key, timeout_seconds, max_tokens",
                    "templates" => "valid templates fields are: cr, ppt, anomaly",
                    _ => "valid root fields are: project_name, output_folder, jira, squash, templates, llm",
                };

                return Some(SerdeErrorKind::UnknownField {
                    field: field_name,
                    location,
                    hint,
                });
            }
        }
    }

    // Handle type errors: "invalid type: found X, expected Y"
    if err_msg.contains("invalid type") {
        // Check for nested section type mismatches (e.g., "templates: 123" instead of "templates: {}")
        // serde_yaml format: "invalid type: integer `123`, expected struct TemplatesConfig"
        // or: "invalid type: string \"yes\", expected struct LlmConfig"

        if err_msg.contains("TemplatesConfig") || (err_msg.contains("templates") && err_msg.contains("expected struct")) {
            return Some(SerdeErrorKind::InvalidEnumValue {
                field: "templates",
                reason: "has invalid type (expected a section with fields, not a scalar value)",
                hint: "a templates section with optional fields: cr, ppt, anomaly (e.g., templates:\\n  cr: \"./templates/cr.md\")",
            });
        }

        if err_msg.contains("LlmConfig") || (err_msg.contains("llm") && err_msg.contains("expected struct")) {
            return Some(SerdeErrorKind::InvalidEnumValue {
                field: "llm",
                reason: "has invalid type (expected a section with fields, not a scalar value)",
                hint: "an llm section with optional fields: mode, local_endpoint, api_key, etc. (e.g., llm:\\n  mode: \"auto\")",
            });
        }

        if err_msg.contains("JiraConfig") || (err_msg.contains("jira") && err_msg.contains("expected struct")) {
            return Some(SerdeErrorKind::InvalidEnumValue {
                field: "jira",
                reason: "has invalid type (expected a section with fields, not a scalar value)",
                hint: "a jira section with endpoint field (e.g., jira:\\n  endpoint: \"https://jira.example.com\")",
            });
        }

        if err_msg.contains("SquashConfig") || (err_msg.contains("squash") && err_msg.contains("expected struct")) {
            return Some(SerdeErrorKind::InvalidEnumValue {
                field: "squash",
                reason: "has invalid type (expected a section with fields, not a scalar value)",
                hint: "a squash section with endpoint field (e.g., squash:\\n  endpoint: \"https://squash.example.com\")",
            });
        }

        // Handle scalar field type errors within LlmConfig
        // serde_yaml format: "invalid type: string \"abc\", expected u32"
        // These occur for timeout_seconds, max_tokens fields
        if err_msg.contains("expected u32") || err_msg.contains("expected u64") || err_msg.contains("expected i32") || err_msg.contains("expected i64") {
            // Check if this is within llm section by looking for llm-specific field names
            if err_msg.contains("timeout") || err_msg.contains("max_token") {
                if err_msg.contains("timeout") {
                    return Some(SerdeErrorKind::InvalidEnumValue {
                        field: "llm.timeout_seconds",
                        reason: "has invalid type (expected an integer)",
                        hint: "a positive integer for timeout in seconds (e.g., timeout_seconds: 120)",
                    });
                }
                if err_msg.contains("max_token") {
                    return Some(SerdeErrorKind::InvalidEnumValue {
                        field: "llm.max_tokens",
                        reason: "has invalid type (expected an integer)",
                        hint: "a positive integer for maximum tokens (e.g., max_tokens: 4096)",
                    });
                }
            }
            // Try to extract field name from error message for better attribution
            if let Some(field_name) = extract_field_from_error(err_msg) {
                return Some(SerdeErrorKind::InvalidEnumValueDynamic {
                    field: field_name,
                    reason: "has invalid type (expected an integer)",
                    hint: "a valid positive integer value",
                });
            }
            // Fallback with line/column info if available for actionable guidance
            let location_hint = extract_location_from_error(err_msg);
            return Some(SerdeErrorKind::InvalidEnumValueDynamic {
                field: format!("integer field{}", location_hint),
                reason: "has invalid type (expected an integer)",
                hint: "a valid positive integer - check timeout_seconds, max_tokens, or port numbers",
            });
        }

        // Check for type mismatches on explicitly named string fields only
        // Do NOT guess the field name from generic "expected a string" errors
        // to avoid incorrect attribution (Issue: HIGH - Review 15)
        if err_msg.contains("project_name") {
            return Some(SerdeErrorKind::InvalidEnumValue {
                field: "project_name",
                reason: "has invalid type",
                hint: "a string value (e.g., project_name: \"my-project\")",
            });
        }

        // Handle output_folder type errors ONLY when explicitly mentioned in error
        // serde_yaml format: "invalid type: sequence, expected a string at output_folder"
        if err_msg.contains("output_folder") && err_msg.contains("expected a string") {
            return Some(SerdeErrorKind::InvalidEnumValue {
                field: "output_folder",
                reason: "has invalid type (expected a string path)",
                hint: "a string path value (e.g., output_folder: \"./output\" or output_folder: \"/var/data\")",
            });
        }

        // Handle string type errors with field extraction
        // serde_yaml may include field path in various formats
        if err_msg.contains("expected a string") && !err_msg.contains("project_name") && !err_msg.contains("output_folder") {
            // Try to extract field name from serde_yaml error message patterns:
            // Pattern 1: "jira.token: invalid type" (field path prefix)
            // Pattern 2: "at line X, column Y, while parsing jira.token" (path in context)
            // Pattern 3: backtick-quoted field names
            if let Some(field_name) = extract_field_from_error(err_msg) {
                return Some(SerdeErrorKind::InvalidEnumValueDynamic {
                    field: field_name,
                    reason: "has invalid type (expected a string)",
                    hint: "a valid string value (e.g., \"example-value\")",
                });
            }

            // Check for known nested string fields by context
            if err_msg.contains("token") {
                let section = if err_msg.to_lowercase().contains("jira") { "jira" } else { "configuration" };
                return Some(SerdeErrorKind::InvalidEnumValueDynamic {
                    field: format!("{}.token", section),
                    reason: "has invalid type (expected a string)",
                    hint: "a valid token string",
                });
            }
            if err_msg.contains("endpoint") {
                return Some(SerdeErrorKind::InvalidEnumValue {
                    field: "endpoint",
                    reason: "has invalid type (expected a string URL)",
                    hint: "a valid URL string (e.g., \"https://example.com\")",
                });
            }
            if err_msg.contains("username") {
                return Some(SerdeErrorKind::InvalidEnumValue {
                    field: "squash.username",
                    reason: "has invalid type (expected a string)",
                    hint: "a valid username string",
                });
            }
            if err_msg.contains("password") {
                return Some(SerdeErrorKind::InvalidEnumValue {
                    field: "squash.password",
                    reason: "has invalid type (expected a string)",
                    hint: "a valid password string",
                });
            }
            if err_msg.contains("api_key") {
                return Some(SerdeErrorKind::InvalidEnumValue {
                    field: "llm.api_key",
                    reason: "has invalid type (expected a string)",
                    hint: "a valid API key string",
                });
            }

            // Fallback with line/column info if available for actionable guidance
            let location_hint = extract_location_from_error(err_msg);
            return Some(SerdeErrorKind::InvalidEnumValueDynamic {
                field: format!("string field{}", location_hint),
                reason: "has invalid type (expected a string)",
                hint: "a valid string value - check for arrays [] or maps {} where a string \"value\" is expected",
            });
        }

        // Handle boolean type errors (e.g., cloud_enabled: "nope" instead of true/false)
        // serde_yaml format: "invalid type: string \"nope\", expected a boolean"
        if err_msg.contains("expected a boolean") || err_msg.contains("expected bool") {
            // Check which boolean field this is likely for
            if err_msg.contains("cloud_enabled") {
                return Some(SerdeErrorKind::InvalidEnumValue {
                    field: "llm.cloud_enabled",
                    reason: "has invalid type (expected a boolean)",
                    hint: "true or false (e.g., cloud_enabled: true)",
                });
            }
            // Try to extract field name from error message
            if let Some(field_name) = extract_field_from_error(err_msg) {
                return Some(SerdeErrorKind::InvalidEnumValueDynamic {
                    field: field_name,
                    reason: "has invalid type (expected a boolean)",
                    hint: "true or false (not \"yes\", \"no\", or other strings)",
                });
            }
            // Fallback with line/column info if available
            let location_hint = extract_location_from_error(err_msg);
            return Some(SerdeErrorKind::InvalidEnumValueDynamic {
                field: format!("boolean field{}", location_hint),
                reason: "has invalid type (expected a boolean)",
                hint: "true or false - check cloud_enabled or similar boolean settings",
            });
        }
    }

    // Handle YAML syntax errors to reduce generic ParseError responses
    // serde_yaml format: "did not find expected key"
    if err_msg.contains("did not find expected") {
        return Some(SerdeErrorKind::InvalidEnumValue {
            field: "YAML syntax",
            reason: "has invalid structure",
            hint: "check indentation and YAML key-value format (key: value)",
        });
    }

    // Handle duplicate key errors
    // serde_yaml format: "duplicate key"
    if err_msg.contains("duplicate key") {
        return Some(SerdeErrorKind::InvalidEnumValue {
            field: "YAML structure",
            reason: "contains duplicate keys",
            hint: "remove duplicate configuration keys",
        });
    }

    // Handle anchor/alias errors
    if err_msg.contains("anchor") || err_msg.contains("alias") {
        return Some(SerdeErrorKind::InvalidEnumValue {
            field: "YAML anchors",
            reason: "has invalid anchor or alias reference",
            hint: "check YAML anchor (&name) and alias (*name) syntax",
        });
    }

    // Handle recursive/deeply nested structure errors
    if err_msg.contains("recursion limit") || err_msg.contains("too deeply nested") {
        return Some(SerdeErrorKind::InvalidEnumValue {
            field: "YAML structure",
            reason: "is too deeply nested",
            hint: "simplify configuration structure (avoid excessive nesting)",
        });
    }

    // Handle EOF/incomplete document errors
    if err_msg.contains("end of stream") || err_msg.contains("unexpected end") {
        return Some(SerdeErrorKind::InvalidEnumValue {
            field: "YAML document",
            reason: "is incomplete or truncated",
            hint: "ensure the YAML file is complete and properly closed",
        });
    }

    // Handle tab character errors (YAML doesn't allow tabs for indentation)
    if err_msg.contains("tab") && err_msg.contains("indent") {
        return Some(SerdeErrorKind::InvalidEnumValue {
            field: "YAML indentation",
            reason: "contains tab characters",
            hint: "use spaces instead of tabs for YAML indentation",
        });
    }

    None
}

/// Detect which configuration section an unknown field belongs to based on serde_yaml's
/// "expected one of" list in the error message.
///
/// This is more reliable than simple string contains checks because it uses the
/// actual valid field names enumerated by serde.
fn detect_section_from_expected_fields(err_msg: &str) -> &'static str {
    // Look for "expected one of" pattern which lists valid fields
    if let Some(expected_pos) = err_msg.find("expected one of") {
        let expected_section = &err_msg[expected_pos..];

        // Check for jira-specific fields (endpoint, token)
        // Jira has only endpoint and token, so if we see both, it's jira
        if expected_section.contains("`endpoint`") && expected_section.contains("`token`")
           && !expected_section.contains("`username`") && !expected_section.contains("`mode`") {
            return "jira";
        }

        // Check for squash-specific fields (endpoint, username, password)
        if expected_section.contains("`endpoint`") && expected_section.contains("`username`")
           && expected_section.contains("`password`") {
            return "squash";
        }

        // Check for llm-specific fields (mode, local_endpoint, api_key, etc.)
        if expected_section.contains("`mode`") && expected_section.contains("`local_endpoint`") {
            return "llm";
        }

        // Check for templates-specific fields (cr, ppt, anomaly)
        if expected_section.contains("`cr`") && expected_section.contains("`ppt`")
           && expected_section.contains("`anomaly`") {
            return "templates";
        }

        // Check for root-level fields (project_name, output_folder, jira, squash, etc.)
        if expected_section.contains("`project_name`") && expected_section.contains("`output_folder`") {
            return "root";
        }
    }

    // Fallback: if no "expected one of", use simple heuristics as backup
    // but be more conservative to avoid mis-attribution
    "root"
}

/// Validate that a URL has a valid format (scheme + valid host at minimum)
///
/// A valid host must be either:
/// - "localhost" (with optional port)
/// - A single-label internal hostname (e.g., "jira", "squash", "server1") - RFC 1123 compliant
/// - A multi-label domain name (e.g., "example.com", "api.example.com")
/// - An IPv4 address (contains dots)
/// - An IPv6 address in brackets (e.g., "[::1]", "[2001:db8::1]")
///
/// Single-label hostnames (without dots) are accepted for internal/corporate networks
/// where DNS resolution handles short names (e.g., "http://jira:8080").
///
/// Invalid hosts like "...", "-start", "end-" are rejected.
/// Ports must be valid (1-65535).
fn is_valid_url(url: &str) -> bool {
    // Must start with http:// or https:// (case-insensitive per RFC 3986)
    // RFC 3986 §3.1: "schemes are case-insensitive"
    let url_lower = url.to_lowercase();
    let without_scheme = if url_lower.starts_with("https://") {
        // Return the original URL's remainder (preserving case)
        &url[8..] // "https://".len() == 8
    } else if url_lower.starts_with("http://") {
        &url[7..] // "http://".len() == 7
    } else {
        return false;
    };

    // Reject URLs with whitespace immediately after schema (e.g., "https:// example.com")
    // This is invalid per RFC 3986 - authority must follow directly after "://"
    if without_scheme.starts_with(char::is_whitespace) {
        return false;
    }

    // Must have at least one character after the scheme (the host)
    // and the host must not be empty or just whitespace
    let trimmed = without_scheme.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Extract host and port (before path, query, or fragment)
    // URL format: host[:port][/path][?query][#fragment]
    // We need to handle URLs without path like "https://host?query" or "https://host#frag"
    let host_port_end = trimmed
        .find('/')
        .unwrap_or_else(|| {
            // No path - check for query or fragment
            trimmed.find('?')
                .or_else(|| trimmed.find('#'))
                .unwrap_or(trimmed.len())
        });
    let host_port = &trimmed[..host_port_end];

    // Handle IPv6 addresses: [::1] or [::1]:8080
    if host_port.starts_with('[') {
        // Find closing bracket
        if let Some(bracket_end) = host_port.find(']') {
            let ipv6_addr = &host_port[1..bracket_end];

            // Basic IPv6 validation: must contain colons
            if ipv6_addr.is_empty() || !ipv6_addr.contains(':') {
                return false;
            }

            // Validate IPv6 address characters and structure
            // IPv6 addresses can contain: hex digits (0-9, a-f, A-F), colons, and optionally
            // a zone ID suffix starting with % (e.g., fe80::1%eth0)
            let (addr_part, zone_part) = if let Some(zone_pos) = ipv6_addr.find('%') {
                (&ipv6_addr[..zone_pos], Some(&ipv6_addr[zone_pos + 1..]))
            } else {
                (ipv6_addr, None)
            };

            // Validate zone ID if present: must not be empty (e.g., "fe80::1%" is invalid)
            if let Some(zone) = zone_part {
                if zone.is_empty() {
                    return false; // Empty zone ID is invalid
                }
                // Zone ID should contain only alphanumeric characters and common interface name chars
                // Allow: alphanumeric, hyphen, underscore, dot (common in interface names like eth0, wlan-0, etc.)
                if !zone.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.') {
                    return false;
                }
            }

            // The address part must only contain hex digits and colons
            if !addr_part.chars().all(|c| c.is_ascii_hexdigit() || c == ':') {
                return false;
            }

            // Must have at least 2 colons (minimum valid IPv6 like ::1)
            if addr_part.matches(':').count() < 2 {
                return false;
            }

            // Reject invalid IPv6 forms:
            // - More than 2 consecutive colons only allowed once (::) for zero compression
            // - Cannot have more than 7 colons (8 groups max)
            // - Cannot be all colons (like ::::)
            let colon_count = addr_part.matches(':').count();
            if colon_count > 7 {
                return false; // Too many colons
            }

            // Check for invalid consecutive colons (more than 2 in a row, or multiple :: groups)
            let double_colon_count = addr_part.matches("::").count();
            if double_colon_count > 1 {
                return false; // Multiple :: groups not allowed
            }

            // Check for triple+ colons which are never valid (:::, ::::, etc.)
            if addr_part.contains(":::") {
                return false;
            }

            // Ensure there's at least one hex digit (not just colons like "::")
            // Exception: "::" alone is valid (represents all zeros)
            if addr_part != "::" && !addr_part.chars().any(|c| c.is_ascii_hexdigit()) {
                return false;
            }

            // Check for port after bracket
            let after_bracket = &host_port[bracket_end + 1..];
            if !after_bracket.is_empty() {
                // Must be :port
                if let Some(port_str) = after_bracket.strip_prefix(':') {
                    if port_str.is_empty() {
                        return false; // Empty port is invalid (e.g., "[::1]:")
                    }
                    match port_str.parse::<u32>() {
                        Ok(p) if (1..=65535).contains(&p) => {}
                        _ => return false, // Invalid port number
                    }
                } else {
                    return false; // Invalid format after bracket
                }
            }
            return true;
        } else {
            return false; // Unclosed bracket
        }
    }

    // Non-IPv6: Check for port and validate it
    let (host, port_str) = if let Some(colon_pos) = host_port.rfind(':') {
        (&host_port[..colon_pos], Some(&host_port[colon_pos + 1..]))
    } else {
        (host_port, None)
    };

    // Validate port if present (1-65535 range)
    // Reject empty port (e.g., "https://example.com:")
    if let Some(port) = port_str {
        if port.is_empty() {
            return false; // Empty port is invalid (e.g., "https://example.com:")
        }
        match port.parse::<u32>() {
            Ok(p) if (1..=65535).contains(&p) => {}
            _ => return false, // Invalid port number
        }
    }

    // Host must not be empty after extraction
    if host.is_empty() {
        return false;
    }

    // Accept "localhost" as valid
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }

    // For hosts without dots (internal hostnames like "jira", "squash", "server1"):
    // Accept if it's a valid RFC 1123 label (alphanumeric + hyphens, 1-63 chars,
    // cannot start/end with hyphen)
    if !host.contains('.') {
        // Single-word internal hostname: validate as a single label
        if host.is_empty() || host.len() > 63 {
            return false;
        }
        if host.starts_with('-') || host.ends_with('-') {
            return false;
        }
        // Must be alphanumeric + hyphens only
        return host.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    }

    // Reject hosts that are just dots (e.g., "...", ".")
    if host.chars().all(|c| c == '.') {
        return false;
    }

    // Reject hosts starting or ending with dot
    if host.starts_with('.') || host.ends_with('.') {
        return false;
    }

    // Reject consecutive dots (e.g., "example..com")
    if host.contains("..") {
        return false;
    }

    // Validate each label in the hostname (RFC 1123 compliant):
    // - Labels separated by dots
    // - Each label: 1-63 chars, alphanumeric + hyphens
    // - Labels cannot start or end with hyphen
    let labels: Vec<&str> = host.split('.').collect();

    for label in &labels {
        if label.is_empty() || label.len() > 63 {
            return false;
        }
        // Label cannot start or end with hyphen
        if label.starts_with('-') || label.ends_with('-') {
            return false;
        }
        // Label must contain only alphanumeric and hyphens
        // Exception: allow all-numeric labels for IP addresses
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return false;
        }
    }

    // Check if this looks like an IPv4 address (all labels are purely numeric)
    // If so, validate each octet is in range 0-255
    let all_numeric = labels.iter().all(|label| label.chars().all(|c| c.is_ascii_digit()));
    if all_numeric && labels.len() == 4 {
        // This is an IPv4 address - validate each octet
        for label in &labels {
            // Check for leading zeros (e.g., "01" is invalid in strict IPv4)
            // But allow single "0"
            if label.len() > 1 && label.starts_with('0') {
                return false; // Leading zeros not allowed
            }
            match label.parse::<u32>() {
                Ok(octet) if octet <= 255 => {}
                _ => return false, // Invalid IPv4 octet (> 255 or parse error)
            }
        }
    }

    true
}

/// Validate that a path string has valid format (not empty, no null bytes)
fn is_valid_path_format(path: &str) -> bool {
    let trimmed = path.trim();
    // Basic checks: non-empty and no null bytes
    if trimmed.is_empty() || path.contains('\0') {
        return false;
    }
    // Reject values that look like they were coerced from YAML boolean/null scalars
    // These are clearly not valid paths and indicate user error
    // Examples: "true", "false", "null", "~" (but NOT numeric strings like "123" or "2026")
    if is_coerced_scalar(trimmed) {
        return false;
    }
    true
}

/// Check if a string looks like it was coerced from a YAML boolean or null scalar.
///
/// This catches cases where the user wrote `output_folder: true` or `output_folder: null`
/// which are clearly not valid paths.
///
/// **Important:** Only rejects `true`, `false`, `null`, and `~` (the actual coerced values
/// after serde_yaml parsing). YAML 1.1 booleans like `yes`/`no`/`on`/`off` are coerced
/// to "true"/"false" by serde_yaml, so we don't need to check for them explicitly.
/// This allows intentionally quoted paths like `"yes"` or `"on"` to be valid folder names.
///
/// Note: Pure numeric strings like "2026" are NOT rejected because they can be valid
/// folder names (e.g., year-based directories). While `output_folder: 123` (unquoted)
/// and `output_folder: "123"` (quoted) both become "123" after serde parsing,
/// numeric folder names are legitimate use cases.
fn is_coerced_scalar(s: &str) -> bool {
    // Only reject actual coerced values from serde_yaml:
    // - Boolean: true/false (YAML 1.1 yes/no/on/off become true/false)
    // - Null: null/~ (tilde is YAML null shorthand)
    // This allows intentional folder names like "yes", "no", "on", "off"
    let lower = s.to_lowercase();
    matches!(lower.as_str(), "true" | "false" | "null" | "~")
}

/// Validate that a project name contains only valid characters
/// Valid: alphanumeric, hyphens (-), underscores (_)
fn is_valid_project_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Validate that a path does not contain path traversal sequences (..)
///
/// This checks for ".." as a path component, not just any occurrence of "..".
/// For example:
/// - "../parent" is unsafe (traversal at start)
/// - "./data/../secrets" is unsafe (traversal in middle)
/// - "file..txt" is safe (not a path component)
/// - "my..folder/data" is safe (not a path component)
fn is_safe_path(path: &str) -> bool {
    // Split by both Unix and Windows path separators
    for component in path.split(['/', '\\']) {
        if component == ".." {
            return false;
        }
    }
    true
}

/// Validate template file extension
fn has_valid_extension(path: &str, expected_extensions: &[&str]) -> bool {
    let lower = path.to_lowercase();
    expected_extensions.iter().any(|ext| lower.ends_with(ext))
}

/// Validate configuration fields
fn validate_config(config: &ProjectConfig) -> Result<(), ConfigError> {
    // Validate project_name is not empty
    if config.project_name.trim().is_empty() {
        return Err(ConfigError::invalid_value(
            "project_name",
            "cannot be empty",
            "a non-empty project name string",
        ));
    }

    // Validate project_name format (alphanumeric + hyphens + underscores only)
    if !is_valid_project_name(&config.project_name) {
        return Err(ConfigError::invalid_value(
            "project_name",
            "contains invalid characters",
            "alphanumeric characters, hyphens (-), and underscores (_) only (e.g., 'my-project' or 'test_framework')",
        ));
    }

    // Validate output_folder is not empty and has valid format
    // This rejects YAML boolean/null coercions like "true" or "null" (but allows numeric strings)
    if !is_valid_path_format(&config.output_folder) {
        // Provide specific error message for coerced scalars
        let reason = if is_coerced_scalar(&config.output_folder) {
            "has invalid type (boolean or null values are not valid paths)"
        } else {
            "cannot be empty or contain invalid characters (null bytes)"
        };
        return Err(ConfigError::invalid_value(
            "output_folder",
            reason,
            "a valid output folder path string (e.g., output_folder: \"./output\" or output_folder: \"/var/data\")",
        ));
    }

    // Validate output_folder does not contain path traversal
    if !is_safe_path(&config.output_folder) {
        return Err(ConfigError::invalid_value(
            "output_folder",
            "cannot contain path traversal sequences (..)",
            "a direct path without '..' (e.g., './output' or '/var/data')",
        ));
    }

    // Validate LLM config if present
    if let Some(ref llm) = config.llm {
        // If mode is local, local_endpoint should be provided
        if llm.mode == LlmMode::Local && llm.local_endpoint.is_none() {
            return Err(ConfigError::missing_field(
                "llm.local_endpoint",
                "a local LLM endpoint URL (e.g., http://localhost:11434) when mode is 'local'",
            ));
        }

        // If mode is cloud, api_key is required and must not be empty
        match &llm.api_key {
            None => {
                if llm.mode == LlmMode::Cloud {
                    return Err(ConfigError::missing_field(
                        "llm.api_key",
                        "an API key for cloud LLM when mode is 'cloud'",
                    ));
                }
            }
            Some(key) if key.trim().is_empty() => {
                if llm.mode == LlmMode::Cloud {
                    return Err(ConfigError::invalid_value(
                        "llm.api_key",
                        "cannot be empty when mode is 'cloud'",
                        "a non-empty API key string",
                    ));
                }
            }
            _ => {}
        }

        // If mode is cloud and cloud_enabled is false, that's a configuration error
        if llm.mode == LlmMode::Cloud && !llm.cloud_enabled {
            return Err(ConfigError::invalid_value(
                "llm.cloud_enabled",
                "must be true when mode is 'cloud'",
                "set cloud_enabled: true when using cloud mode",
            ));
        }

        // If mode is cloud, cloud_endpoint is required
        if llm.mode == LlmMode::Cloud && llm.cloud_endpoint.is_none() {
            return Err(ConfigError::missing_field(
                "llm.cloud_endpoint",
                "a cloud LLM endpoint URL (e.g., https://api.openai.com/v1) when mode is 'cloud'",
            ));
        }

        // If mode is cloud, cloud_model is required and must not be empty
        match &llm.cloud_model {
            None => {
                if llm.mode == LlmMode::Cloud {
                    return Err(ConfigError::missing_field(
                        "llm.cloud_model",
                        "a cloud model name (e.g., 'gpt-4o-mini', 'claude-3-sonnet') when mode is 'cloud'",
                    ));
                }
            }
            Some(model) if model.trim().is_empty() => {
                if llm.mode == LlmMode::Cloud {
                    return Err(ConfigError::invalid_value(
                        "llm.cloud_model",
                        "cannot be empty when mode is 'cloud'",
                        "a non-empty cloud model name (e.g., 'gpt-4o-mini')",
                    ));
                }
            }
            _ => {}
        }

        // If mode is auto AND cloud_enabled=true, require cloud prerequisites
        // This ensures that when the user intends to use cloud fallback, the config is valid
        if llm.mode == LlmMode::Auto && llm.cloud_enabled {
            // Require cloud_endpoint when cloud is enabled in auto mode
            if llm.cloud_endpoint.is_none() {
                return Err(ConfigError::missing_field(
                    "llm.cloud_endpoint",
                    "a cloud LLM endpoint URL when cloud_enabled is true in 'auto' mode (e.g., https://api.openai.com/v1)",
                ));
            }

            // Require cloud_model when cloud is enabled in auto mode
            match &llm.cloud_model {
                None => {
                    return Err(ConfigError::missing_field(
                        "llm.cloud_model",
                        "a cloud model name when cloud_enabled is true in 'auto' mode (e.g., 'gpt-4o-mini')",
                    ));
                }
                Some(model) if model.trim().is_empty() => {
                    return Err(ConfigError::invalid_value(
                        "llm.cloud_model",
                        "cannot be empty when cloud_enabled is true in 'auto' mode",
                        "a non-empty cloud model name (e.g., 'gpt-4o-mini')",
                    ));
                }
                _ => {}
            }

            // Require api_key when cloud is enabled in auto mode
            match &llm.api_key {
                None => {
                    return Err(ConfigError::missing_field(
                        "llm.api_key",
                        "an API key when cloud_enabled is true in 'auto' mode",
                    ));
                }
                Some(key) if key.trim().is_empty() => {
                    return Err(ConfigError::invalid_value(
                        "llm.api_key",
                        "cannot be empty when cloud_enabled is true in 'auto' mode",
                        "a non-empty API key string",
                    ));
                }
                _ => {}
            }
        }

        // Validate local_endpoint URL format if provided
        if let Some(ref endpoint) = llm.local_endpoint {
            // Reject URLs with leading/trailing whitespace (strict validation)
            if endpoint != endpoint.trim() {
                return Err(ConfigError::invalid_value(
                    "llm.local_endpoint",
                    "must not contain leading or trailing whitespace",
                    "a URL without extra spaces (e.g., 'http://localhost:11434')",
                ));
            }
            if !is_valid_url(endpoint) {
                return Err(ConfigError::invalid_value(
                    "llm.local_endpoint",
                    "must be a valid URL with host",
                    "a URL like http://localhost:11434",
                ));
            }
        }

        // Validate cloud_endpoint URL format if provided
        if let Some(ref endpoint) = llm.cloud_endpoint {
            // Reject URLs with leading/trailing whitespace (strict validation)
            if endpoint != endpoint.trim() {
                return Err(ConfigError::invalid_value(
                    "llm.cloud_endpoint",
                    "must not contain leading or trailing whitespace",
                    "a URL without extra spaces (e.g., 'https://api.openai.com/v1')",
                ));
            }
            if !is_valid_url(endpoint) {
                return Err(ConfigError::invalid_value(
                    "llm.cloud_endpoint",
                    "must be a valid URL with host",
                    "a URL like https://api.openai.com/v1",
                ));
            }
        }

        // Validate timeout_seconds is positive (must be > 0)
        if llm.timeout_seconds == 0 {
            return Err(ConfigError::invalid_value(
                "llm.timeout_seconds",
                "must be a positive integer (greater than 0)",
                "a positive integer for timeout in seconds (e.g., 120)",
            ));
        }

        // Validate max_tokens is positive (must be > 0)
        if llm.max_tokens == 0 {
            return Err(ConfigError::invalid_value(
                "llm.max_tokens",
                "must be a positive integer (greater than 0)",
                "a positive integer for maximum tokens (e.g., 4096)",
            ));
        }
    }

    // Validate Jira endpoint URL format if present
    if let Some(ref jira) = config.jira {
        // Reject URLs with leading/trailing whitespace (strict validation)
        if jira.endpoint != jira.endpoint.trim() {
            return Err(ConfigError::invalid_value(
                "jira.endpoint",
                "must not contain leading or trailing whitespace",
                "a URL without extra spaces (e.g., 'https://jira.example.com')",
            ));
        }
        if !is_valid_url(&jira.endpoint) {
            return Err(ConfigError::invalid_value(
                "jira.endpoint",
                "must be a valid URL with host",
                "a URL like https://jira.example.com",
            ));
        }
    }

    // Validate Squash endpoint URL format if present
    if let Some(ref squash) = config.squash {
        // Reject URLs with leading/trailing whitespace (strict validation)
        if squash.endpoint != squash.endpoint.trim() {
            return Err(ConfigError::invalid_value(
                "squash.endpoint",
                "must not contain leading or trailing whitespace",
                "a URL without extra spaces (e.g., 'https://squash.example.com')",
            ));
        }
        if !is_valid_url(&squash.endpoint) {
            return Err(ConfigError::invalid_value(
                "squash.endpoint",
                "must be a valid URL with host",
                "a URL like https://squash.example.com",
            ));
        }
    }

    // Validate template paths if provided
    if let Some(ref templates) = config.templates {
        if let Some(ref cr) = templates.cr {
            if !is_valid_path_format(cr) {
                return Err(ConfigError::invalid_value(
                    "templates.cr",
                    "must be a valid file path",
                    "a path like './templates/cr.md'",
                ));
            }
            if !is_safe_path(cr) {
                return Err(ConfigError::invalid_value(
                    "templates.cr",
                    "cannot contain path traversal sequences (..)",
                    "a direct path without '..' (e.g., './templates/cr.md')",
                ));
            }
            if !has_valid_extension(cr, &[".md"]) {
                return Err(ConfigError::invalid_value(
                    "templates.cr",
                    "must be a Markdown file",
                    "a .md file path like './templates/cr.md'",
                ));
            }
        }
        if let Some(ref ppt) = templates.ppt {
            if !is_valid_path_format(ppt) {
                return Err(ConfigError::invalid_value(
                    "templates.ppt",
                    "must be a valid file path",
                    "a path like './templates/ppt.pptx'",
                ));
            }
            if !is_safe_path(ppt) {
                return Err(ConfigError::invalid_value(
                    "templates.ppt",
                    "cannot contain path traversal sequences (..)",
                    "a direct path without '..' (e.g., './templates/report.pptx')",
                ));
            }
            if !has_valid_extension(ppt, &[".pptx"]) {
                return Err(ConfigError::invalid_value(
                    "templates.ppt",
                    "must be a PowerPoint file",
                    "a .pptx file path like './templates/report.pptx'",
                ));
            }
        }
        if let Some(ref anomaly) = templates.anomaly {
            if !is_valid_path_format(anomaly) {
                return Err(ConfigError::invalid_value(
                    "templates.anomaly",
                    "must be a valid file path",
                    "a path like './templates/anomaly.md'",
                ));
            }
            if !is_safe_path(anomaly) {
                return Err(ConfigError::invalid_value(
                    "templates.anomaly",
                    "cannot contain path traversal sequences (..)",
                    "a direct path without '..' (e.g., './templates/anomaly.md')",
                ));
            }
            if !has_valid_extension(anomaly, &[".md"]) {
                return Err(ConfigError::invalid_value(
                    "templates.anomaly",
                    "must be a Markdown file",
                    "a .md file path like './templates/anomaly.md'",
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_config(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_load_valid_config() {
        let yaml = r#"
project_name: "test-project"
output_folder: "./output"
jira:
  endpoint: "https://jira.example.com"
squash:
  endpoint: "https://squash.example.com"
llm:
  mode: "auto"
"#;
        let file = create_temp_config(yaml);
        let config = load_config(file.path()).unwrap();

        assert_eq!(config.project_name, "test-project");
        assert_eq!(config.output_folder, "./output");
        assert!(config.jira.is_some());
        assert!(config.squash.is_some());
    }

    #[test]
    fn test_load_minimal_config() {
        let yaml = r#"
project_name: "minimal"
output_folder: "./out"
"#;
        let file = create_temp_config(yaml);
        let config = load_config(file.path()).unwrap();

        assert_eq!(config.project_name, "minimal");
        assert!(config.jira.is_none());
        assert!(config.squash.is_none());
    }

    #[test]
    fn test_file_not_found_error() {
        let result = load_config(Path::new("/nonexistent/config.yaml"));
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("not found"));
    }

    #[test]
    fn test_empty_project_name_error() {
        let yaml = r#"
project_name: ""
output_folder: "./output"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("project_name"));
        assert!(err_msg.contains("cannot be empty"));
    }

    #[test]
    fn test_invalid_llm_mode_error() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "invalid_mode"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("llm.mode"));
        assert!(err_msg.contains("not a valid mode"));
        assert!(err_msg.contains("Expected"));
    }

    #[test]
    fn test_local_mode_requires_endpoint() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "local"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("llm.local_endpoint"));
        assert!(err_msg.contains("missing"));
    }

    #[test]
    fn test_invalid_jira_url_error() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
jira:
  endpoint: "not-a-url"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("jira.endpoint"));
        assert!(err_msg.contains("valid URL"));
    }

    #[test]
    fn test_parse_error_invalid_yaml() {
        // Use truly malformed YAML that can't be parsed at all
        let yaml = "[[[invalid yaml structure";
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        // Truly invalid YAML should result in a parsing error
        assert!(
            err_msg.to_lowercase().contains("parse") || err_msg.contains("expected"),
            "Expected parse error, got: {}", err_msg
        );
    }

    #[test]
    fn test_debug_redacts_jira_token() {
        let jira = JiraConfig {
            endpoint: "https://jira.example.com".to_string(),
            token: Some("super_secret_token".to_string()),
        };

        let debug_output = format!("{:?}", jira);
        assert!(!debug_output.contains("super_secret_token"));
        assert!(debug_output.contains("[REDACTED]"));
        assert!(debug_output.contains("jira.example.com"));
    }

    #[test]
    fn test_debug_redacts_squash_password() {
        let squash = SquashConfig {
            endpoint: "https://squash.example.com".to_string(),
            username: Some("user".to_string()),
            password: Some("secret_password".to_string()),
        };

        let debug_output = format!("{:?}", squash);
        assert!(!debug_output.contains("secret_password"));
        assert!(debug_output.contains("[REDACTED]"));
        assert!(debug_output.contains("squash.example.com"));
    }

    #[test]
    fn test_debug_redacts_llm_api_key() {
        let llm = LlmConfig {
            mode: LlmMode::Cloud,
            local_endpoint: None,
            local_model: None,
            cloud_enabled: true,
            cloud_endpoint: None,
            cloud_model: None,
            api_key: Some("sk-secret-api-key-12345".to_string()),
            timeout_seconds: 120,
            max_tokens: 4096,
        };

        let debug_output = format!("{:?}", llm);
        assert!(!debug_output.contains("sk-secret-api-key-12345"));
        assert!(debug_output.contains("[REDACTED]"));
    }

    #[test]
    fn test_redact_trait_jira() {
        let jira = JiraConfig {
            endpoint: "https://jira.example.com".to_string(),
            token: Some("my_token".to_string()),
        };

        let redacted = jira.redacted();
        assert!(!redacted.contains("my_token"));
        assert!(redacted.contains("[REDACTED]"));
        assert!(redacted.contains("jira.example.com"));
    }

    #[test]
    fn test_redact_trait_squash() {
        let squash = SquashConfig {
            endpoint: "https://squash.example.com".to_string(),
            username: Some("testuser".to_string()),
            password: Some("secret_password".to_string()),
        };

        let redacted = squash.redacted();
        assert!(!redacted.contains("secret_password"));
        assert!(redacted.contains("[REDACTED]"));
        assert!(redacted.contains("squash.example.com"));
        assert!(redacted.contains("testuser")); // username should be visible
    }

    #[test]
    fn test_redact_trait_llm() {
        let llm = LlmConfig {
            mode: LlmMode::Cloud,
            local_endpoint: Some("http://localhost:11434".to_string()),
            local_model: None,
            cloud_enabled: true,
            cloud_endpoint: None,
            cloud_model: None,
            api_key: Some("sk-super-secret-key".to_string()),
            timeout_seconds: 120,
            max_tokens: 4096,
        };

        let redacted = llm.redacted();
        assert!(!redacted.contains("sk-super-secret-key"));
        assert!(redacted.contains("[REDACTED]"));
        assert!(redacted.contains("Cloud"));
        assert!(redacted.contains("localhost:11434"));
    }

    #[test]
    fn test_full_config_with_all_fields() {
        let yaml = r#"
project_name: "full-project"
output_folder: "./output"
jira:
  endpoint: "https://jira.example.com"
  token: "jira-token-secret"
squash:
  endpoint: "https://squash.example.com"
  username: "testuser"
  password: "squash-password-secret"
templates:
  cr: "./templates/cr.md"
  ppt: "./templates/ppt.pptx"
  anomaly: "./templates/anomaly.md"
llm:
  mode: "local"
  local_endpoint: "http://localhost:11434"
  api_key: "llm-api-key-secret"
"#;
        let file = create_temp_config(yaml);
        let config = load_config(file.path()).unwrap();

        assert_eq!(config.project_name, "full-project");

        // Verify secrets are loaded but not exposed in debug
        let jira = config.jira.as_ref().unwrap();
        assert_eq!(jira.token.as_ref().unwrap(), "jira-token-secret");

        let debug_output = format!("{:?}", config);
        assert!(!debug_output.contains("jira-token-secret"));
        assert!(!debug_output.contains("squash-password-secret"));
        assert!(!debug_output.contains("llm-api-key-secret"));
    }

    // === NEW TESTS FOR REVIEW FOLLOW-UPS ===

    #[test]
    fn test_url_scheme_only_rejected() {
        // "https://" alone should be rejected as invalid
        let yaml = r#"
project_name: "test"
output_folder: "./output"
jira:
  endpoint: "https://"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("jira.endpoint"));
        assert!(err_msg.contains("valid URL"));
    }

    #[test]
    fn test_url_scheme_with_whitespace_rejected() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
squash:
  endpoint: "https://   "
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("squash.endpoint"));
    }

    #[test]
    fn test_url_space_after_schema_rejected_jira() {
        // Test that "https:// example.com" (space after scheme) is rejected
        let yaml = r#"
project_name: "test"
output_folder: "./output"
jira:
  endpoint: "https:// jira.example.com"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("jira.endpoint"));
    }

    #[test]
    fn test_url_space_after_schema_rejected_squash() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
squash:
  endpoint: "http:// squash.example.com:8080"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("squash.endpoint"));
    }

    #[test]
    fn test_url_tab_after_schema_rejected() {
        // Tab character immediately after scheme
        let yaml = "project_name: \"test\"\noutput_folder: \"./output\"\njira:\n  endpoint: \"https://\texample.com\"\n";
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("jira.endpoint"));
    }

    #[test]
    fn test_valid_url_helper() {
        // Valid URLs
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("http://localhost:8080"));
        assert!(is_valid_url("http://localhost"));
        assert!(is_valid_url("https://jira.example.com/api"));
        assert!(is_valid_url("https://api.example.com:443/path"));
        assert!(is_valid_url("http://192.168.1.1"));
        assert!(is_valid_url("http://192.168.1.1:8080"));

        // Valid: uppercase schemes (RFC 3986 §3.1 schemes are case-insensitive)
        assert!(is_valid_url("HTTP://example.com"));
        assert!(is_valid_url("HTTPS://example.com"));
        assert!(is_valid_url("Http://example.com"));
        assert!(is_valid_url("Https://Example.COM:8080/Path"));

        // Invalid: empty or whitespace after scheme
        assert!(!is_valid_url("https://"));
        assert!(!is_valid_url("http://"));
        assert!(!is_valid_url("https://   "));

        // Invalid: whitespace immediately after scheme (space before host)
        assert!(!is_valid_url("https:// example.com"));
        assert!(!is_valid_url("http:// localhost:8080"));
        assert!(!is_valid_url("https://  jira.example.com"));
        assert!(!is_valid_url("http://\texample.com")); // tab after scheme
        assert!(!is_valid_url("https://\nexample.com")); // newline after scheme

        // Invalid: wrong scheme or no scheme
        assert!(!is_valid_url("not-a-url"));
        assert!(!is_valid_url("ftp://example.com"));

        // Valid: internal hostnames without dots (RFC 1123 compliant labels)
        assert!(is_valid_url("https://a"));
        assert!(is_valid_url("https://abc"));
        assert!(is_valid_url("http://x:8080"));
        assert!(is_valid_url("http://jira:8080"));
        assert!(is_valid_url("http://squash"));
        assert!(is_valid_url("http://server1"));
        assert!(is_valid_url("http://my-internal-host:3000"));

        // Invalid: dot-only hosts
        assert!(!is_valid_url("https://..."));
        assert!(!is_valid_url("https://."));
        assert!(!is_valid_url("https://.."));

        // Invalid: dots at start/end or consecutive
        assert!(!is_valid_url("https://.example.com"));
        assert!(!is_valid_url("https://example.com."));
        assert!(!is_valid_url("https://example..com"));
    }

    #[test]
    fn test_valid_path_helper() {
        assert!(is_valid_path_format("./output"));
        assert!(is_valid_path_format("/var/data"));
        assert!(is_valid_path_format("relative/path"));
        assert!(!is_valid_path_format(""));
        assert!(!is_valid_path_format("   "));
        assert!(!is_valid_path_format("path\0with\0null"));
    }

    #[test]
    fn test_template_path_validation() {
        // Valid template paths should work
        let yaml = r#"
project_name: "test"
output_folder: "./output"
templates:
  cr: "./templates/cr.md"
  ppt: "./templates/report.pptx"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_template_path_rejected() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
templates:
  cr: ""
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("templates.cr"));
    }

    #[test]
    fn test_llm_local_endpoint_url_validation() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "local"
  local_endpoint: "not-a-valid-url"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("llm.local_endpoint"));
    }

    #[test]
    fn test_output_folder_null_bytes_rejected() {
        // Test with actual null byte character using Rust escape sequence
        let yaml = "project_name: \"test\"\noutput_folder: \"path\0withnull\"";
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        // The null byte should cause either a parse error or validation failure
        match result {
            Err(_) => {
                // Expected: YAML parser or our validation caught the null byte
            }
            Ok(config) => {
                // If somehow loaded, our validation helper should still flag it
                assert!(
                    !is_valid_path_format(&config.output_folder),
                    "Path with null byte should be invalid"
                );
            }
        }
    }

    #[test]
    fn test_is_valid_path_format_null_byte() {
        // Direct test of the helper function with actual null bytes
        assert!(!is_valid_path_format("path\0with\0null"));
        assert!(!is_valid_path_format("\0"));
        assert!(!is_valid_path_format("before\0after"));
    }

    #[test]
    fn test_check_output_folder_exists_nonexistent() {
        let yaml = r#"
project_name: "test"
output_folder: "/nonexistent/path/that/does/not/exist"
"#;
        let file = create_temp_config(yaml);
        let config = load_config(file.path()).unwrap();

        let warning = config.check_output_folder_exists();
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("does not exist"));
    }

    #[test]
    fn test_check_output_folder_exists_existing() {
        let yaml = r#"
project_name: "test"
output_folder: "."
"#;
        let file = create_temp_config(yaml);
        let config = load_config(file.path()).unwrap();

        let warning = config.check_output_folder_exists();
        assert!(warning.is_none());
    }

    // === REVIEW 5 TESTS ===

    #[test]
    fn test_path_traversal_rejected() {
        let yaml = r#"
project_name: "test"
output_folder: "../../../etc"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("output_folder"));
        assert!(err_msg.contains("path traversal"));
    }

    #[test]
    fn test_path_traversal_in_middle_rejected() {
        let yaml = r#"
project_name: "test"
output_folder: "./data/../secrets"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("path traversal"));
    }

    #[test]
    fn test_safe_path_helper() {
        assert!(is_safe_path("./output"));
        assert!(is_safe_path("/var/data"));
        assert!(is_safe_path("relative/path"));
        assert!(!is_safe_path("../parent"));
        assert!(!is_safe_path("./data/../secrets"));
        assert!(!is_safe_path(".."));
    }

    #[test]
    fn test_port_validation_valid() {
        assert!(is_valid_url("http://localhost:8080"));
        assert!(is_valid_url("http://localhost:1"));
        assert!(is_valid_url("http://localhost:65535"));
        assert!(is_valid_url("https://example.com:443"));
    }

    #[test]
    fn test_port_validation_invalid() {
        assert!(!is_valid_url("http://localhost:0"));
        assert!(!is_valid_url("http://localhost:65536"));
        assert!(!is_valid_url("http://localhost:99999"));
        assert!(!is_valid_url("http://localhost:abc"));
        assert!(!is_valid_url("http://example.com:999999"));
    }

    #[test]
    fn test_template_cr_wrong_extension_rejected() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
templates:
  cr: "./templates/cr.txt"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("templates.cr"));
        assert!(err_msg.contains("Markdown"));
    }

    #[test]
    fn test_template_ppt_wrong_extension_rejected() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
templates:
  ppt: "./templates/report.pdf"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("templates.ppt"));
        assert!(err_msg.contains("PowerPoint"));
    }

    #[test]
    fn test_template_anomaly_wrong_extension_rejected() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
templates:
  anomaly: "./templates/anomaly.docx"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("templates.anomaly"));
        assert!(err_msg.contains("Markdown"));
    }

    #[test]
    fn test_extension_helper() {
        assert!(has_valid_extension("file.md", &[".md"]));
        assert!(has_valid_extension("file.MD", &[".md"]));
        assert!(has_valid_extension("path/to/file.pptx", &[".pptx"]));
        assert!(!has_valid_extension("file.txt", &[".md"]));
        assert!(!has_valid_extension("file.ppt", &[".pptx"]));
    }

    #[test]
    fn test_missing_project_name_serde_error() {
        let yaml = r#"
output_folder: "./output"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("project_name"));
        assert!(err_msg.contains("missing"));
    }

    #[test]
    fn test_missing_output_folder_serde_error() {
        let yaml = r#"
project_name: "test"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("output_folder"));
        assert!(err_msg.contains("missing"));
    }

    #[test]
    fn test_llm_mode_enum_values() {
        // Test that enum deserializes correctly
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "auto"
"#;
        let file = create_temp_config(yaml);
        let config = load_config(file.path()).unwrap();
        assert_eq!(config.llm.unwrap().mode, LlmMode::Auto);

        // Cloud mode requires cloud_enabled, api_key, cloud_endpoint, and cloud_model
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "cloud"
  cloud_enabled: true
  api_key: "sk-test-key"
  cloud_endpoint: "https://api.openai.com/v1"
  cloud_model: "gpt-4o-mini"
"#;
        let file = create_temp_config(yaml);
        let config = load_config(file.path()).unwrap();
        assert_eq!(config.llm.unwrap().mode, LlmMode::Cloud);
    }

    #[test]
    fn test_llm_mode_display() {
        assert_eq!(format!("{}", LlmMode::Auto), "auto");
        assert_eq!(format!("{}", LlmMode::Local), "local");
        assert_eq!(format!("{}", LlmMode::Cloud), "cloud");
    }

    // === REVIEW 6 TESTS: IPv6 URL validation ===

    #[test]
    fn test_ipv6_url_valid() {
        // IPv6 localhost
        assert!(is_valid_url("http://[::1]:8080"));
        assert!(is_valid_url("http://[::1]"));
        // Full IPv6 address
        assert!(is_valid_url("http://[2001:db8::1]:8080"));
        assert!(is_valid_url("https://[2001:db8:85a3::8a2e:370:7334]"));
        // IPv6 with port
        assert!(is_valid_url("http://[fe80::1%25eth0]:3000"));
    }

    #[test]
    fn test_ipv6_url_invalid_port() {
        // IPv6 with invalid port
        assert!(!is_valid_url("http://[::1]:0"));
        assert!(!is_valid_url("http://[::1]:65536"));
        assert!(!is_valid_url("http://[::1]:abc"));
    }

    #[test]
    fn test_llm_mode_default() {
        // Test that Default derive works correctly
        let mode = LlmMode::default();
        assert_eq!(mode, LlmMode::Auto);
    }

    // === REVIEW 7 TESTS ===

    #[test]
    fn test_llm_config_full_architecture_fields() {
        // Test all LlmConfig fields from architecture spec
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "cloud"
  local_endpoint: "http://localhost:11434"
  local_model: "mistral:7b-instruct"
  cloud_enabled: true
  cloud_endpoint: "https://api.openai.com/v1"
  cloud_model: "gpt-4o-mini"
  api_key: "sk-secret-key"
  timeout_seconds: 60
  max_tokens: 2048
"#;
        let file = create_temp_config(yaml);
        let config = load_config(file.path()).unwrap();

        let llm = config.llm.unwrap();
        assert_eq!(llm.mode, LlmMode::Cloud);
        assert_eq!(llm.local_endpoint.as_deref(), Some("http://localhost:11434"));
        assert_eq!(llm.local_model.as_deref(), Some("mistral:7b-instruct"));
        assert!(llm.cloud_enabled);
        assert_eq!(llm.cloud_endpoint.as_deref(), Some("https://api.openai.com/v1"));
        assert_eq!(llm.cloud_model.as_deref(), Some("gpt-4o-mini"));
        assert_eq!(llm.api_key.as_deref(), Some("sk-secret-key"));
        assert_eq!(llm.timeout_seconds, 60);
        assert_eq!(llm.max_tokens, 2048);
    }

    #[test]
    fn test_llm_config_default_timeout_and_max_tokens() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "auto"
"#;
        let file = create_temp_config(yaml);
        let config = load_config(file.path()).unwrap();

        let llm = config.llm.unwrap();
        assert_eq!(llm.timeout_seconds, 120); // default
        assert_eq!(llm.max_tokens, 4096); // default
    }

    #[test]
    fn test_cloud_mode_requires_api_key() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "cloud"
  cloud_enabled: true
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("llm.api_key"));
        assert!(err_msg.contains("missing"));
    }

    #[test]
    fn test_cloud_mode_requires_cloud_enabled() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "cloud"
  cloud_enabled: false
  api_key: "sk-key"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("cloud_enabled"));
        assert!(err_msg.contains("must be true"));
    }

    #[test]
    fn test_cloud_endpoint_url_validation() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "cloud"
  cloud_enabled: true
  api_key: "sk-key"
  cloud_endpoint: "not-a-url"
  cloud_model: "gpt-4o-mini"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("llm.cloud_endpoint"));
        assert!(err_msg.contains("valid URL"));
    }

    #[test]
    fn test_project_name_valid_formats() {
        // Valid: alphanumeric, hyphens, underscores
        assert!(is_valid_project_name("my-project"));
        assert!(is_valid_project_name("test_framework"));
        assert!(is_valid_project_name("MyProject123"));
        assert!(is_valid_project_name("a"));
        assert!(is_valid_project_name("test-project_v2"));
    }

    #[test]
    fn test_project_name_invalid_formats() {
        // Invalid: spaces, special chars, etc.
        assert!(!is_valid_project_name("my project"));
        assert!(!is_valid_project_name("test.project"));
        assert!(!is_valid_project_name("project@name"));
        assert!(!is_valid_project_name("project/name"));
        assert!(!is_valid_project_name(""));
        assert!(!is_valid_project_name("项目")); // non-ASCII
    }

    #[test]
    fn test_project_name_validation_in_config() {
        let yaml = r#"
project_name: "my project with spaces"
output_folder: "./output"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("project_name"));
        assert!(err_msg.contains("invalid characters"));
    }

    #[test]
    fn test_safe_path_allows_double_dots_in_filename() {
        // Double dots in filename (not as path component) should be allowed
        assert!(is_safe_path("file..txt"));
        assert!(is_safe_path("my..folder/data"));
        assert!(is_safe_path("test..data..file.txt"));
        assert!(is_safe_path("./output/file..backup"));
    }

    #[test]
    fn test_safe_path_rejects_traversal_sequences() {
        // Actual path traversal should still be rejected
        assert!(!is_safe_path(".."));
        assert!(!is_safe_path("../parent"));
        assert!(!is_safe_path("./data/../secrets"));
        assert!(!is_safe_path("folder/../../etc"));
        assert!(!is_safe_path("..\\windows\\style"));
    }

    #[test]
    fn test_llm_debug_redacts_all_sensitive_fields() {
        let llm = LlmConfig {
            mode: LlmMode::Cloud,
            local_endpoint: Some("http://localhost:11434".to_string()),
            local_model: Some("mistral".to_string()),
            cloud_enabled: true,
            cloud_endpoint: Some("https://api.openai.com/v1".to_string()),
            cloud_model: Some("gpt-4o-mini".to_string()),
            api_key: Some("sk-super-secret-key-12345".to_string()),
            timeout_seconds: 120,
            max_tokens: 4096,
        };

        let debug_output = format!("{:?}", llm);
        assert!(!debug_output.contains("sk-super-secret-key-12345"));
        assert!(debug_output.contains("[REDACTED]"));
        assert!(debug_output.contains("Cloud"));
        assert!(debug_output.contains("mistral"));
        assert!(debug_output.contains("gpt-4o-mini"));
    }

    #[test]
    fn test_llm_redact_trait_all_fields() {
        let llm = LlmConfig {
            mode: LlmMode::Auto,
            local_endpoint: Some("http://localhost:11434".to_string()),
            local_model: Some("codellama:13b".to_string()),
            cloud_enabled: false,
            cloud_endpoint: None,
            cloud_model: None,
            api_key: Some("secret-api-key".to_string()),
            timeout_seconds: 60,
            max_tokens: 2048,
        };

        let redacted = llm.redacted();
        assert!(!redacted.contains("secret-api-key"));
        assert!(redacted.contains("[REDACTED]"));
        assert!(redacted.contains("Auto"));
        assert!(redacted.contains("codellama:13b"));
        assert!(redacted.contains("timeout_seconds: 60"));
        assert!(redacted.contains("max_tokens: 2048"));
    }

    // === REVIEW 8 TESTS ===

    #[test]
    fn test_deny_unknown_fields_root_level() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
unknown_field: "should fail"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("unknown_field") || err_msg.contains("not a recognized"));
    }

    #[test]
    fn test_deny_unknown_fields_jira() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
jira:
  endpoint: "https://jira.example.com"
  unknown_jira_field: "should fail"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("unknown") || err_msg.contains("not a recognized"));
    }

    #[test]
    fn test_deny_unknown_fields_llm() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "auto"
  invalid_llm_option: true
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("unknown") || err_msg.contains("not a recognized"));
    }

    #[test]
    fn test_deny_unknown_fields_templates() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
templates:
  cr: "./templates/cr.md"
  extra_template: "./templates/extra.md"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("unknown") || err_msg.contains("not a recognized"));
    }

    #[test]
    fn test_hostname_validation_rfc1123_labels() {
        // Valid hostnames
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("https://my-domain.example.com"));
        assert!(is_valid_url("https://sub1.sub2.example.com"));
        assert!(is_valid_url("https://a1.b2.c3.example.com"));

        // Invalid: label starts with hyphen
        assert!(!is_valid_url("https://-example.com"));
        assert!(!is_valid_url("https://sub.-example.com"));

        // Invalid: label ends with hyphen
        assert!(!is_valid_url("https://example-.com"));
        assert!(!is_valid_url("https://sub.example-.com"));

        // Invalid: special characters in label
        assert!(!is_valid_url("https://exam_ple.com"));
        assert!(!is_valid_url("https://exam+ple.com"));
        assert!(!is_valid_url("https://exam@ple.com"));
    }

    #[test]
    fn test_hostname_label_length() {
        // Valid: 63 char label (max allowed)
        let label_63 = "a".repeat(63);
        assert!(is_valid_url(&format!("https://{}.com", label_63)));

        // Invalid: 64 char label (too long)
        let label_64 = "a".repeat(64);
        assert!(!is_valid_url(&format!("https://{}.com", label_64)));
    }

    #[test]
    fn test_unknown_field_error_message_quality() {
        // Test that unknown field errors provide helpful hints
        let yaml = r#"
project_name: "test"
output_folder: "./output"
typo_field: "value"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // Should mention valid fields or indicate it's not recognized
        assert!(
            err_msg.contains("not a recognized") || err_msg.contains("unknown"),
            "Error should indicate field is unknown: {}", err_msg
        );
    }

    // === REVIEW 9 TESTS: Cloud mode complete validation ===

    #[test]
    fn test_cloud_mode_requires_cloud_endpoint() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "cloud"
  cloud_enabled: true
  api_key: "sk-key"
  cloud_model: "gpt-4o-mini"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("llm.cloud_endpoint"));
        assert!(err_msg.contains("missing"));
    }

    #[test]
    fn test_cloud_mode_requires_cloud_model() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "cloud"
  cloud_enabled: true
  api_key: "sk-key"
  cloud_endpoint: "https://api.openai.com/v1"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("llm.cloud_model"));
        assert!(err_msg.contains("missing"));
    }

    #[test]
    fn test_cloud_mode_valid_complete_config() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "cloud"
  cloud_enabled: true
  api_key: "sk-key"
  cloud_endpoint: "https://api.openai.com/v1"
  cloud_model: "gpt-4o-mini"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_ok());
        let config = result.unwrap();
        let llm = config.llm.unwrap();
        assert_eq!(llm.mode, LlmMode::Cloud);
        assert_eq!(llm.cloud_endpoint.as_deref(), Some("https://api.openai.com/v1"));
        assert_eq!(llm.cloud_model.as_deref(), Some("gpt-4o-mini"));
    }

    // === REVIEW 9 TESTS: Internal hostnames without dots ===

    #[test]
    fn test_internal_hostname_jira() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
jira:
  endpoint: "http://jira:8080"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_ok());
    }

    #[test]
    fn test_internal_hostname_squash() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
squash:
  endpoint: "http://squash"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_ok());
    }

    #[test]
    fn test_internal_hostname_with_hyphen() {
        assert!(is_valid_url("http://my-internal-server:3000"));
        assert!(is_valid_url("http://test-jira"));
        assert!(is_valid_url("https://prod-api:443"));
    }

    #[test]
    fn test_internal_hostname_invalid_formats() {
        // Cannot start or end with hyphen
        assert!(!is_valid_url("http://-jira"));
        assert!(!is_valid_url("http://jira-"));
        assert!(!is_valid_url("http://-"));

        // Empty hostname
        assert!(!is_valid_url("http://"));

        // Too long (> 63 chars)
        let long_host = "a".repeat(64);
        assert!(!is_valid_url(&format!("http://{}", long_host)));

        // Invalid characters in hostname
        assert!(!is_valid_url("http://jira_server"));  // underscore not allowed

        // Note: "http://jira.server" IS valid - it's a dotted hostname
        // and goes through the standard dot validation path
        assert!(is_valid_url("http://jira.server")); // valid hostname with dot
    }

    // === REVIEW 9 TESTS: Nested section type errors ===

    #[test]
    fn test_templates_wrong_type_scalar() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
templates: 123
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("templates"));
        assert!(err_msg.contains("invalid type") || err_msg.contains("expected a section"));
    }

    #[test]
    fn test_llm_wrong_type_string() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm: "yes"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("llm"));
        assert!(err_msg.contains("invalid type") || err_msg.contains("expected a section"));
    }

    #[test]
    fn test_jira_wrong_type_boolean() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
jira: true
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("jira"));
        assert!(err_msg.contains("invalid type") || err_msg.contains("expected a section"));
    }

    #[test]
    fn test_squash_wrong_type_array() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
squash:
  - item1
  - item2
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        // Should fail either at parse or validation
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("squash") || err_msg.contains("invalid type") || err_msg.contains("expected"));
    }

    // === REVIEW 10 TESTS: Scalar field type errors ===

    #[test]
    fn test_timeout_seconds_wrong_type_string() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "auto"
  timeout_seconds: "not a number"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // Should provide field-specific hint
        assert!(
            err_msg.contains("timeout") || err_msg.contains("invalid type") || err_msg.contains("integer"),
            "Expected timeout-related error, got: {}", err_msg
        );
    }

    #[test]
    fn test_max_tokens_wrong_type_string() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "auto"
  max_tokens: "four thousand"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // Should provide field-specific hint
        assert!(
            err_msg.contains("max_token") || err_msg.contains("invalid type") || err_msg.contains("integer"),
            "Expected max_tokens-related error, got: {}", err_msg
        );
    }

    #[test]
    fn test_timeout_seconds_wrong_type_float() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "auto"
  timeout_seconds: 120.5
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        // Floats might be coerced to int or fail - either is acceptable
        // The key is that we don't panic and provide a reasonable response
        match result {
            Ok(config) => {
                // If it loaded, the value should be truncated or rounded
                let llm = config.llm.unwrap();
                assert!(llm.timeout_seconds > 0);
            }
            Err(e) => {
                // If it failed, should have a meaningful error
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("timeout") || err_msg.contains("invalid") || err_msg.contains("type"),
                    "Expected timeout-related error, got: {}", err_msg
                );
            }
        }
    }

    #[test]
    fn test_section_detection_reliability() {
        // Test that unknown field in llm section is correctly attributed to llm
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "auto"
  unknown_llm_field: "value"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // Should mention llm fields in the hint
        assert!(
            err_msg.contains("llm") || err_msg.contains("mode") || err_msg.contains("local_endpoint"),
            "Expected llm section hint, got: {}", err_msg
        );
    }

    #[test]
    fn test_unknown_field_in_jira_section() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
jira:
  endpoint: "https://jira.example.com"
  invalid_jira_option: "value"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // Should provide jira-specific hint
        assert!(
            err_msg.contains("jira") || err_msg.contains("endpoint") || err_msg.contains("token"),
            "Expected jira section hint, got: {}", err_msg
        );
    }

    #[test]
    fn test_single_label_hostname_documented_behavior() {
        // Document that single-label hostnames (like "a", "jira") ARE valid
        // This aligns documentation with actual implementation
        assert!(is_valid_url("https://a"));
        assert!(is_valid_url("http://jira"));
        assert!(is_valid_url("http://squash:8080"));

        // But invalid single-label formats are still rejected
        assert!(!is_valid_url("http://-invalid"));
        assert!(!is_valid_url("http://invalid-"));
    }

    // === REVIEW 11 TESTS: Empty cloud fields, zero timeout/max_tokens, template traversal ===

    #[test]
    fn test_cloud_mode_empty_api_key_rejected() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "cloud"
  cloud_enabled: true
  api_key: ""
  cloud_endpoint: "https://api.openai.com/v1"
  cloud_model: "gpt-4o-mini"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("llm.api_key"));
        assert!(err_msg.contains("cannot be empty") || err_msg.contains("empty"));
    }

    #[test]
    fn test_cloud_mode_whitespace_api_key_rejected() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "cloud"
  cloud_enabled: true
  api_key: "   "
  cloud_endpoint: "https://api.openai.com/v1"
  cloud_model: "gpt-4o-mini"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("llm.api_key"));
        assert!(err_msg.contains("cannot be empty") || err_msg.contains("empty"));
    }

    #[test]
    fn test_cloud_mode_empty_cloud_model_rejected() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "cloud"
  cloud_enabled: true
  api_key: "sk-valid-key"
  cloud_endpoint: "https://api.openai.com/v1"
  cloud_model: ""
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("llm.cloud_model"));
        assert!(err_msg.contains("cannot be empty") || err_msg.contains("empty"));
    }

    #[test]
    fn test_cloud_mode_whitespace_cloud_model_rejected() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "cloud"
  cloud_enabled: true
  api_key: "sk-valid-key"
  cloud_endpoint: "https://api.openai.com/v1"
  cloud_model: "   "
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("llm.cloud_model"));
        assert!(err_msg.contains("cannot be empty") || err_msg.contains("empty"));
    }

    #[test]
    fn test_timeout_seconds_zero_rejected() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "auto"
  timeout_seconds: 0
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("llm.timeout_seconds"));
        assert!(err_msg.contains("positive") || err_msg.contains("greater than 0"));
    }

    #[test]
    fn test_max_tokens_zero_rejected() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "auto"
  max_tokens: 0
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("llm.max_tokens"));
        assert!(err_msg.contains("positive") || err_msg.contains("greater than 0"));
    }

    #[test]
    fn test_template_cr_path_traversal_rejected() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
templates:
  cr: "../../../etc/passwd.md"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("templates.cr"));
        assert!(err_msg.contains("path traversal"));
    }

    #[test]
    fn test_template_ppt_path_traversal_rejected() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
templates:
  ppt: "./templates/../../../secrets/report.pptx"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("templates.ppt"));
        assert!(err_msg.contains("path traversal"));
    }

    #[test]
    fn test_template_anomaly_path_traversal_rejected() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
templates:
  anomaly: "../parent/../grandparent/anomaly.md"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("templates.anomaly"));
        assert!(err_msg.contains("path traversal"));
    }

    #[test]
    fn test_valid_cloud_config_with_all_required_fields() {
        // Verify valid cloud config still works after all validations
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "cloud"
  cloud_enabled: true
  api_key: "sk-valid-key-12345"
  cloud_endpoint: "https://api.openai.com/v1"
  cloud_model: "gpt-4o-mini"
  timeout_seconds: 60
  max_tokens: 2048
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_ok());
        let config = result.unwrap();
        let llm = config.llm.unwrap();
        assert_eq!(llm.mode, LlmMode::Cloud);
        assert_eq!(llm.api_key.as_deref(), Some("sk-valid-key-12345"));
        assert_eq!(llm.cloud_model.as_deref(), Some("gpt-4o-mini"));
        assert_eq!(llm.timeout_seconds, 60);
        assert_eq!(llm.max_tokens, 2048);
    }

    #[test]
    fn test_valid_templates_without_traversal() {
        // Verify valid template paths still work
        let yaml = r#"
project_name: "test"
output_folder: "./output"
templates:
  cr: "./templates/cr.md"
  ppt: "./templates/report.pptx"
  anomaly: "./templates/anomaly.md"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_ok());
    }

    // === REVIEW 12 TESTS: Boolean type errors, URL sensitive params, empty port, IPv6 validation ===

    #[test]
    fn test_boolean_type_error_cloud_enabled() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "auto"
  cloud_enabled: "nope"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // Should provide a helpful message about boolean type
        assert!(
            err_msg.contains("boolean") || err_msg.contains("true") || err_msg.contains("false") || err_msg.contains("cloud_enabled"),
            "Expected boolean-related error, got: {}", err_msg
        );
    }

    #[test]
    fn test_boolean_type_error_string_yes() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "auto"
  cloud_enabled: "yes"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("boolean") || err_msg.contains("true") || err_msg.contains("false"),
            "Expected boolean-related error, got: {}", err_msg
        );
    }

    #[test]
    fn test_redact_url_sensitive_params_token() {
        let url = "https://jira.example.com/api?token=secret123&project=PROJ";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("secret123"));
        assert!(redacted.contains("token=[REDACTED]"));
        assert!(redacted.contains("project=PROJ"));
    }

    #[test]
    fn test_redact_url_sensitive_params_api_key() {
        let url = "https://api.example.com?api_key=sk-12345&foo=bar";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("sk-12345"));
        assert!(redacted.contains("api_key=[REDACTED]"));
        assert!(redacted.contains("foo=bar"));
    }

    #[test]
    fn test_redact_url_sensitive_params_password() {
        let url = "https://example.com?user=admin&password=hunter2";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("hunter2"));
        assert!(redacted.contains("password=[REDACTED]"));
        assert!(redacted.contains("user=admin"));
    }

    #[test]
    fn test_redact_url_sensitive_params_no_query() {
        let url = "https://example.com/api/v1";
        let redacted = redact_url_sensitive_params(url);
        assert_eq!(redacted, url); // Unchanged
    }

    #[test]
    fn test_redact_url_sensitive_params_no_sensitive() {
        let url = "https://example.com?page=1&limit=10";
        let redacted = redact_url_sensitive_params(url);
        assert_eq!(redacted, url); // Unchanged
    }

    #[test]
    fn test_redact_url_path_token_secret() {
        // Test redacting secrets in URL path segments (AC #3 - no secrets in logs)
        let url = "https://api.example.com/token/sk-abc123def456/resource";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("sk-abc123def456"));
        assert!(redacted.contains("/token/[REDACTED]/"));
    }

    #[test]
    fn test_redact_url_path_key_secret() {
        let url = "https://api.example.com/api/key/abcdef123456789/data";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("abcdef123456789"));
        assert!(redacted.contains("/key/[REDACTED]/"));
    }

    #[test]
    fn test_redact_url_path_no_false_positive() {
        // Normal path segments should not be redacted
        let url = "https://api.example.com/api/v1/users";
        let redacted = redact_url_sensitive_params(url);
        assert_eq!(redacted, url); // Unchanged - no secret-looking segments
    }

    #[test]
    fn test_redact_url_path_combined_with_query() {
        // Test that both path secrets and query secrets are redacted
        let url = "https://api.example.com/token/sk-12345678/resource?api_key=secret";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("sk-12345678"));
        assert!(!redacted.contains("secret"));
        assert!(redacted.contains("/token/[REDACTED]/"));
        assert!(redacted.contains("api_key=[REDACTED]"));
    }

    #[test]
    fn test_redact_url_path_short_segment_not_redacted() {
        // Short segments after sensitive keywords should not be redacted
        // (they're likely IDs or other non-secret data)
        let url = "https://api.example.com/token/123/resource";
        let redacted = redact_url_sensitive_params(url);
        // "123" is too short to look like a secret, should not be redacted
        assert!(redacted.contains("/token/123/"));
    }

    #[test]
    fn test_redact_url_userinfo_with_password() {
        // Test redacting user:password@host format (AC #3 - no secrets in logs)
        let url = "https://admin:secret123@jira.example.com/api";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("admin"));
        assert!(!redacted.contains("secret123"));
        assert!(redacted.contains("[REDACTED]@jira.example.com"));
        assert_eq!(redacted, "https://[REDACTED]@jira.example.com/api");
    }

    #[test]
    fn test_redact_url_userinfo_without_password() {
        // Test redacting user@host format (username only, still sensitive)
        let url = "https://apiuser@api.example.com/v1";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("apiuser"));
        assert!(redacted.contains("[REDACTED]@api.example.com"));
        assert_eq!(redacted, "https://[REDACTED]@api.example.com/v1");
    }

    #[test]
    fn test_redact_url_userinfo_with_port() {
        // Test redacting userinfo when port is present
        let url = "https://user:pass@example.com:8080/path";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("user"));
        assert!(!redacted.contains("pass"));
        assert!(redacted.contains(":8080"));
        assert_eq!(redacted, "https://[REDACTED]@example.com:8080/path");
    }

    #[test]
    fn test_redact_url_userinfo_with_query() {
        // Test redacting both userinfo AND sensitive query params
        let url = "https://user:pass@example.com?token=secret";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("user"));
        assert!(!redacted.contains("pass"));
        assert!(!redacted.contains("secret"));
        assert!(redacted.contains("[REDACTED]@example.com"));
        assert!(redacted.contains("token=[REDACTED]"));
    }

    #[test]
    fn test_redact_url_no_userinfo() {
        // Test that URLs without userinfo are not modified (except query params)
        let url = "https://example.com/path?foo=bar";
        let redacted = redact_url_sensitive_params(url);
        assert_eq!(redacted, url); // No userinfo to redact, no sensitive params
    }

    #[test]
    fn test_redact_url_at_in_path_not_userinfo() {
        // Test that @ in path is not treated as userinfo
        let url = "https://example.com/user@domain/resource";
        let redacted = redact_url_sensitive_params(url);
        assert_eq!(redacted, url); // @ is in path, not userinfo
    }

    #[test]
    fn test_redact_url_at_in_query_not_userinfo() {
        // Test that @ in query params is not treated as userinfo
        let url = "https://example.com?email=user@domain.com";
        let redacted = redact_url_sensitive_params(url);
        assert_eq!(redacted, url); // @ is in query value, not userinfo
    }

    // === Fragment param redaction tests (AC #3 - sensitive data in URL fragments) ===

    #[test]
    fn test_redact_url_fragment_token() {
        // OAuth implicit flow puts tokens in fragments - must be redacted (AC #3)
        let url = "https://example.com/callback#access_token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"), "Fragment token should be redacted");
        assert!(redacted.contains("access_token=[REDACTED]"));
    }

    #[test]
    fn test_redact_url_fragment_api_key() {
        // API key in fragment should be redacted
        let url = "https://example.com#api_key=sk-secret123";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("sk-secret123"), "Fragment api_key should be redacted");
        assert!(redacted.contains("api_key=[REDACTED]"));
    }

    #[test]
    fn test_redact_url_fragment_multiple_params() {
        // Multiple params in fragment - redact only sensitive ones
        let url = "https://example.com#state=abc&token=secret&redirect=/home";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("secret"), "Fragment token should be redacted");
        assert!(redacted.contains("state=abc"), "Non-sensitive param should remain");
        assert!(redacted.contains("token=[REDACTED]"));
        assert!(redacted.contains("redirect=/home"), "Non-sensitive param should remain");
    }

    #[test]
    fn test_redact_url_fragment_with_query() {
        // Both query and fragment params should be redacted
        let url = "https://example.com?api_key=query_secret#token=frag_secret";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("query_secret"), "Query api_key should be redacted");
        assert!(!redacted.contains("frag_secret"), "Fragment token should be redacted");
        assert!(redacted.contains("api_key=[REDACTED]"));
        assert!(redacted.contains("token=[REDACTED]"));
    }

    #[test]
    fn test_redact_url_fragment_no_sensitive() {
        // Fragment without sensitive params should remain unchanged
        let url = "https://example.com#section=intro&page=2";
        let redacted = redact_url_sensitive_params(url);
        assert_eq!(redacted, url, "Non-sensitive fragment should remain unchanged");
    }

    #[test]
    fn test_redact_url_fragment_only_identifier() {
        // Simple fragment identifier (no key=value) should remain unchanged
        let url = "https://example.com/docs#introduction";
        let redacted = redact_url_sensitive_params(url);
        assert_eq!(redacted, url, "Simple fragment identifier should remain unchanged");
    }

    #[test]
    fn test_jira_debug_redacts_endpoint_token_param() {
        let jira = JiraConfig {
            endpoint: "https://jira.example.com?token=secret".to_string(),
            token: Some("other_secret".to_string()),
        };

        let debug_output = format!("{:?}", jira);
        assert!(!debug_output.contains("secret"), "Endpoint token should be redacted: {}", debug_output);
        assert!(debug_output.contains("[REDACTED]"));
    }

    #[test]
    fn test_squash_debug_redacts_endpoint_password_param() {
        let squash = SquashConfig {
            endpoint: "https://squash.example.com?password=secret123".to_string(),
            username: Some("user".to_string()),
            password: Some("other_secret".to_string()),
        };

        let debug_output = format!("{:?}", squash);
        assert!(!debug_output.contains("secret123"), "Endpoint password should be redacted: {}", debug_output);
        assert!(debug_output.contains("[REDACTED]"));
    }

    #[test]
    fn test_jira_redact_trait_redacts_endpoint_token_param() {
        let jira = JiraConfig {
            endpoint: "https://jira.example.com?api_key=sk-12345".to_string(),
            token: Some("token_value".to_string()),
        };

        let redacted = jira.redacted();
        assert!(!redacted.contains("sk-12345"), "Endpoint api_key should be redacted: {}", redacted);
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn test_url_empty_port_rejected() {
        // Empty port (trailing colon without port number)
        assert!(!is_valid_url("https://jira.example.com:"));
        assert!(!is_valid_url("http://localhost:"));
        assert!(!is_valid_url("https://api.example.com:/api"));
    }

    #[test]
    fn test_url_empty_port_in_config_rejected() {
        let yaml = r#"
project_name: "test"
output_folder: "./output"
jira:
  endpoint: "https://jira.example.com:"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("jira.endpoint"));
        assert!(err_msg.contains("valid URL"));
    }

    #[test]
    fn test_ipv6_invalid_chars_rejected() {
        // Invalid IPv6 with non-hex characters
        assert!(!is_valid_url("http://[abc%def]")); // Invalid IPv6 (no colons, wrong chars)
        assert!(!is_valid_url("http://[xyz::1]")); // Invalid hex chars 'x', 'y', 'z'
        assert!(!is_valid_url("http://[ghij::1]")); // Invalid hex chars
        assert!(!is_valid_url("http://[::g]")); // Invalid hex char 'g'
    }

    #[test]
    fn test_ipv6_too_few_colons_rejected() {
        // IPv6 must have at least 2 colons
        assert!(!is_valid_url("http://[1:2]")); // Only 1 colon
        assert!(!is_valid_url("http://[abc]")); // No colons at all
    }

    #[test]
    fn test_ipv6_valid_addresses() {
        // Valid IPv6 addresses should still work
        assert!(is_valid_url("http://[::1]:8080"));
        assert!(is_valid_url("http://[2001:db8::1]"));
        assert!(is_valid_url("https://[fe80::1]"));
        assert!(is_valid_url("http://[::]:80"));
        assert!(is_valid_url("http://[a:b:c:d:e:f:0:1]"));
    }

    #[test]
    fn test_ipv6_empty_port_rejected() {
        // IPv6 with empty port should be rejected
        assert!(!is_valid_url("http://[::1]:"));
        assert!(!is_valid_url("https://[2001:db8::1]:"));
    }

    #[test]
    fn test_ipv6_with_zone_id_valid() {
        // IPv6 with zone ID (link-local)
        assert!(is_valid_url("http://[fe80::1%25eth0]:8080"));
    }

    // ===== Review 13 Tests =====

    #[test]
    fn test_ipv6_invalid_forms_rejected() {
        // Invalid IPv6 with too many consecutive colons (::::)
        assert!(!is_valid_url("http://[::::]"), ":::: should be rejected");
        assert!(!is_valid_url("http://[::::]:8080"), ":::: with port should be rejected");
        assert!(!is_valid_url("http://[1:::2]"), "::: (triple colon) should be rejected");
        assert!(!is_valid_url("http://[:::1]"), "::: at start should be rejected");
        assert!(!is_valid_url("http://[1:::]"), "::: at end should be rejected");
    }

    #[test]
    fn test_ipv6_multiple_double_colon_rejected() {
        // Multiple :: groups are not allowed
        assert!(!is_valid_url("http://[::1::2]"), "Multiple :: should be rejected");
        assert!(!is_valid_url("http://[1::2::3]"), "Multiple :: should be rejected");
    }

    #[test]
    fn test_ipv6_too_many_colons_rejected() {
        // More than 7 colons (8 groups) is invalid
        assert!(!is_valid_url("http://[1:2:3:4:5:6:7:8:9]"), "More than 8 groups should be rejected");
    }

    #[test]
    fn test_ipv6_double_colon_alone_valid() {
        // :: alone is valid (represents all zeros - 0:0:0:0:0:0:0:0)
        assert!(is_valid_url("http://[::]"), ":: alone should be valid");
        assert!(is_valid_url("http://[::]:8080"), ":: with port should be valid");
    }

    #[test]
    fn test_llm_config_debug_redacts_cloud_endpoint_params() {
        let llm = LlmConfig {
            mode: LlmMode::Cloud,
            local_endpoint: None,
            local_model: None,
            cloud_enabled: true,
            cloud_endpoint: Some("https://api.example.com?api_key=secret123&foo=bar".to_string()),
            cloud_model: Some("gpt-4".to_string()),
            api_key: Some("other_secret".to_string()),
            timeout_seconds: 120,
            max_tokens: 4096,
        };

        let debug_output = format!("{:?}", llm);
        assert!(!debug_output.contains("secret123"), "cloud_endpoint api_key should be redacted in Debug: {}", debug_output);
        assert!(debug_output.contains("[REDACTED]"));
        assert!(debug_output.contains("foo=bar")); // Non-sensitive params should remain
    }

    #[test]
    fn test_llm_config_redact_trait_redacts_cloud_endpoint_params() {
        let llm = LlmConfig {
            mode: LlmMode::Cloud,
            local_endpoint: None,
            local_model: None,
            cloud_enabled: true,
            cloud_endpoint: Some("https://api.example.com?token=mysecret&version=v1".to_string()),
            cloud_model: Some("gpt-4".to_string()),
            api_key: Some("other_secret".to_string()),
            timeout_seconds: 120,
            max_tokens: 4096,
        };

        let redacted = llm.redacted();
        assert!(!redacted.contains("mysecret"), "cloud_endpoint token should be redacted in Redact: {}", redacted);
        assert!(redacted.contains("[REDACTED]"));
        assert!(redacted.contains("version=v1")); // Non-sensitive params should remain
    }

    #[test]
    fn test_output_folder_numeric_string_accepted() {
        // YAML coerces 123 to string "123" - numeric paths are valid folder names
        // Examples: year-based directories like "2026", version folders like "123"
        // This was changed in Review 16 to accept numeric strings as valid paths
        let yaml = r#"
project_name: "test"
output_folder: 123
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        // Should succeed - numeric folder names are legitimate
        assert!(result.is_ok(), "Numeric folder names should be accepted: {:?}", result.err());
        let config = result.unwrap();
        assert_eq!(config.output_folder, "123");
    }

    #[test]
    fn test_output_folder_year_string_accepted() {
        // Year-based folder names like "2026" should be accepted
        let yaml = r#"
project_name: "test"
output_folder: "2026"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_ok(), "Year folder names should be accepted: {:?}", result.err());
        let config = result.unwrap();
        assert_eq!(config.output_folder, "2026");
    }

    #[test]
    fn test_output_folder_boolean_scalar_rejected() {
        // YAML coerces true to string "true", but this is NOT a valid path
        // We explicitly reject coerced scalar values for output_folder
        let yaml = r#"
project_name: "test"
output_folder: true
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        // Should fail - boolean scalars are rejected even after YAML coercion
        assert!(result.is_err(), "Boolean scalars should be rejected for output_folder");
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("output_folder"), "Error should mention field name: {}", err_msg);
    }

    #[test]
    fn test_output_folder_null_scalar_rejected() {
        // YAML null (~ or null) should also be rejected
        let yaml = r#"
project_name: "test"
output_folder: null
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        // Should fail - null scalar is rejected
        // Note: serde_yaml may fail at parsing level for required field, or our validation catches it
        assert!(result.is_err(), "Null scalars should be rejected for output_folder");
    }

    #[test]
    fn test_output_folder_intentional_yaml11_names_accepted() {
        // YAML 1.1 boolean names like "yes", "no", "on", "off" should be accepted
        // when intentionally quoted as folder names
        for folder_name in &["yes", "no", "on", "off"] {
            let yaml = format!(
                r#"
project_name: "test"
output_folder: "{}"
"#,
                folder_name
            );
            let file = create_temp_config(&yaml);
            let result = load_config(file.path());
            assert!(
                result.is_ok(),
                "Intentionally quoted '{}' should be accepted as folder name: {:?}",
                folder_name,
                result.err()
            );
            let config = result.unwrap();
            assert_eq!(config.output_folder, *folder_name);
        }
    }

    #[test]
    fn test_output_folder_array_type_error() {
        // Arrays cannot be coerced to strings - this should fail
        let yaml = r#"
project_name: "test"
output_folder:
  - item1
  - item2
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err(), "Arrays should not be coercible to string");
    }

    #[test]
    fn test_output_folder_map_type_error() {
        // Maps cannot be coerced to strings - this should fail
        let yaml = r#"
project_name: "test"
output_folder:
  key: value
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err(), "Maps should not be coercible to string");
    }

    #[test]
    fn test_string_type_error_not_attributed_to_output_folder() {
        // When other string fields have type errors, they should NOT be attributed to output_folder
        // This test ensures parse_serde_error correctly identifies the actual field (Review 15+16 fix)
        let yaml = r#"
project_name: "test"
output_folder: "./valid-path"
jira:
  endpoint: "https://jira.example.com"
  token:
    - should
    - be
    - string
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err(), "Array for jira.token should fail");
        let err_msg = result.unwrap_err().to_string();
        // The error should NOT mention output_folder
        // It should either mention "token", "jira.token", "configuration field", or be a ParseError
        assert!(!err_msg.contains("output_folder"),
            "Error for jira.token should not be attributed to output_folder: {}", err_msg);
        // Verify it identifies the correct field or provides actionable guidance
        assert!(err_msg.contains("token") || err_msg.contains("configuration field") || err_msg.contains("Parse"),
            "Error should mention token field or provide guidance: {}", err_msg);
    }

    #[test]
    fn test_auto_mode_cloud_enabled_requires_cloud_endpoint() {
        // When mode=auto and cloud_enabled=true, cloud_endpoint is required
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "auto"
  cloud_enabled: true
  cloud_model: "gpt-4"
  api_key: "sk-123"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("llm.cloud_endpoint") && err_msg.contains("missing"),
            "Should require cloud_endpoint in auto mode with cloud_enabled: {}", err_msg);
    }

    #[test]
    fn test_auto_mode_cloud_enabled_requires_cloud_model() {
        // When mode=auto and cloud_enabled=true, cloud_model is required
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "auto"
  cloud_enabled: true
  cloud_endpoint: "https://api.example.com"
  api_key: "sk-123"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("llm.cloud_model") && err_msg.contains("missing"),
            "Should require cloud_model in auto mode with cloud_enabled: {}", err_msg);
    }

    #[test]
    fn test_auto_mode_cloud_enabled_requires_api_key() {
        // When mode=auto and cloud_enabled=true, api_key is required
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "auto"
  cloud_enabled: true
  cloud_endpoint: "https://api.example.com"
  cloud_model: "gpt-4"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("llm.api_key") && err_msg.contains("missing"),
            "Should require api_key in auto mode with cloud_enabled: {}", err_msg);
    }

    #[test]
    fn test_auto_mode_cloud_enabled_empty_api_key_rejected() {
        // When mode=auto and cloud_enabled=true, api_key cannot be empty
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "auto"
  cloud_enabled: true
  cloud_endpoint: "https://api.example.com"
  cloud_model: "gpt-4"
  api_key: "   "
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("llm.api_key") && err_msg.contains("empty"),
            "Should reject empty api_key in auto mode with cloud_enabled: {}", err_msg);
    }

    #[test]
    fn test_auto_mode_cloud_enabled_empty_cloud_model_rejected() {
        // When mode=auto and cloud_enabled=true, cloud_model cannot be empty
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "auto"
  cloud_enabled: true
  cloud_endpoint: "https://api.example.com"
  cloud_model: ""
  api_key: "sk-123"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("llm.cloud_model") && err_msg.contains("empty"),
            "Should reject empty cloud_model in auto mode with cloud_enabled: {}", err_msg);
    }

    #[test]
    fn test_auto_mode_cloud_enabled_valid_config() {
        // Valid config with mode=auto and cloud_enabled=true
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "auto"
  cloud_enabled: true
  cloud_endpoint: "https://api.openai.com/v1"
  cloud_model: "gpt-4o-mini"
  api_key: "sk-valid-key"
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_ok(), "Valid auto+cloud_enabled config should work: {:?}", result);
    }

    #[test]
    fn test_auto_mode_cloud_disabled_no_requirements() {
        // When mode=auto but cloud_enabled=false, no cloud requirements
        let yaml = r#"
project_name: "test"
output_folder: "./output"
llm:
  mode: "auto"
  cloud_enabled: false
"#;
        let file = create_temp_config(yaml);
        let result = load_config(file.path());

        assert!(result.is_ok(), "auto mode with cloud_enabled=false should not require cloud fields: {:?}", result);
    }

    // ==================== Review 14 Tests ====================

    #[test]
    fn test_ipv4_invalid_octet_rejected() {
        // IPv4 addresses with octets > 255 should be rejected
        assert!(!is_valid_url("http://999.999.999.999"), "999.999.999.999 should be invalid");
        assert!(!is_valid_url("http://256.1.1.1"), "256 octet should be invalid");
        assert!(!is_valid_url("http://1.256.1.1"), "256 octet should be invalid");
        assert!(!is_valid_url("http://1.1.256.1"), "256 octet should be invalid");
        assert!(!is_valid_url("http://1.1.1.256"), "256 octet should be invalid");
        assert!(!is_valid_url("http://300.200.100.50"), "300 octet should be invalid");
    }

    #[test]
    fn test_ipv4_valid_addresses() {
        // Valid IPv4 addresses should be accepted
        assert!(is_valid_url("http://192.168.1.1"), "192.168.1.1 should be valid");
        assert!(is_valid_url("http://10.0.0.1"), "10.0.0.1 should be valid");
        assert!(is_valid_url("http://255.255.255.255"), "255.255.255.255 should be valid");
        assert!(is_valid_url("http://0.0.0.0"), "0.0.0.0 should be valid");
        assert!(is_valid_url("http://127.0.0.1:8080"), "127.0.0.1 with port should be valid");
    }

    #[test]
    fn test_ipv4_leading_zeros_rejected() {
        // Leading zeros in IPv4 octets should be rejected (strict validation)
        assert!(!is_valid_url("http://01.1.1.1"), "Leading zero should be invalid");
        assert!(!is_valid_url("http://1.01.1.1"), "Leading zero should be invalid");
        assert!(!is_valid_url("http://192.168.001.001"), "Leading zeros should be invalid");
    }

    #[test]
    fn test_ipv4_single_zero_valid() {
        // Single zero octets should be valid
        assert!(is_valid_url("http://0.0.0.1"), "0.0.0.1 should be valid");
        assert!(is_valid_url("http://10.0.0.0"), "10.0.0.0 should be valid");
    }

    #[test]
    fn test_redact_url_camelcase_params() {
        // camelCase sensitive params should be redacted
        let url_accesstoken = "https://api.example.com?accessToken=secret123&version=v1";
        let redacted = redact_url_sensitive_params(url_accesstoken);
        assert!(!redacted.contains("secret123"), "accessToken should be redacted: {}", redacted);
        assert!(redacted.contains("accessToken=[REDACTED]"));
        assert!(redacted.contains("version=v1"));
    }

    #[test]
    fn test_redact_url_more_camelcase_params() {
        // Additional camelCase params
        assert!(redact_url_sensitive_params("http://x?refreshToken=abc").contains("[REDACTED]"));
        assert!(redact_url_sensitive_params("http://x?clientSecret=abc").contains("[REDACTED]"));
        assert!(redact_url_sensitive_params("http://x?privateKey=abc").contains("[REDACTED]"));
        assert!(redact_url_sensitive_params("http://x?sessionToken=abc").contains("[REDACTED]"));
        assert!(redact_url_sensitive_params("http://x?authToken=abc").contains("[REDACTED]"));
        assert!(redact_url_sensitive_params("http://x?apiToken=abc").contains("[REDACTED]"));
        assert!(redact_url_sensitive_params("http://x?secretKey=abc").contains("[REDACTED]"));
        assert!(redact_url_sensitive_params("http://x?accessKey=abc").contains("[REDACTED]"));
    }

    #[test]
    fn test_parse_serde_error_yaml_syntax_error() {
        // Test that YAML syntax errors are handled with helpful messages
        let err = parse_serde_error("did not find expected key at line 5");
        assert!(err.is_some(), "YAML syntax errors should be handled");
        let err = err.unwrap();
        match err {
            SerdeErrorKind::InvalidEnumValue { field, reason, hint } => {
                assert!(field.contains("YAML"));
                assert!(reason.contains("invalid"));
                assert!(hint.contains("indentation") || hint.contains("format"));
            }
            _ => panic!("Expected InvalidEnumValue for syntax error"),
        }
    }

    #[test]
    fn test_parse_serde_error_duplicate_key() {
        // Test that duplicate key errors are handled
        let err = parse_serde_error("duplicate key at line 3");
        assert!(err.is_some(), "Duplicate key errors should be handled");
        let err = err.unwrap();
        match err {
            SerdeErrorKind::InvalidEnumValue { field, reason, .. } => {
                assert!(field.contains("YAML"));
                assert!(reason.contains("duplicate"));
            }
            _ => panic!("Expected InvalidEnumValue for duplicate key"),
        }
    }

    #[test]
    fn test_parse_serde_error_end_of_stream() {
        // Test that EOF errors are handled
        let err = parse_serde_error("unexpected end of stream");
        assert!(err.is_some(), "End of stream errors should be handled");
    }

    #[test]
    fn test_ipv4_not_four_octets_treated_as_hostname() {
        // IPv4-like strings with wrong number of octets are treated as hostnames
        // These should be valid hostnames (if they pass label validation)
        assert!(is_valid_url("http://192.168.1"), "3-octet should be valid hostname");
        assert!(is_valid_url("http://192.168.1.1.1"), "5-octet should be valid hostname");
    }

    // ==================== Review 18 Tests ====================

    #[test]
    fn test_llm_config_debug_redacts_local_endpoint_params() {
        // Test that local_endpoint with sensitive query params is redacted in Debug output
        let llm = LlmConfig {
            mode: LlmMode::Local,
            local_endpoint: Some("http://localhost:11434?api_key=secret-local-key&model=llama".to_string()),
            local_model: Some("llama2".to_string()),
            cloud_enabled: false,
            cloud_endpoint: None,
            cloud_model: None,
            api_key: None,
            timeout_seconds: 120,
            max_tokens: 4096,
        };

        let debug_output = format!("{:?}", llm);
        assert!(!debug_output.contains("secret-local-key"),
            "local_endpoint api_key should be redacted in Debug: {}", debug_output);
        assert!(debug_output.contains("[REDACTED]"),
            "Debug output should contain [REDACTED]: {}", debug_output);
        assert!(debug_output.contains("model=llama"),
            "Non-sensitive params should remain: {}", debug_output);
    }

    #[test]
    fn test_llm_config_redact_trait_redacts_local_endpoint_params() {
        // Test that local_endpoint with sensitive query params is redacted via Redact trait
        let llm = LlmConfig {
            mode: LlmMode::Local,
            local_endpoint: Some("http://localhost:11434?token=mysecret&format=json".to_string()),
            local_model: Some("codellama".to_string()),
            cloud_enabled: false,
            cloud_endpoint: None,
            cloud_model: None,
            api_key: None,
            timeout_seconds: 60,
            max_tokens: 2048,
        };

        let redacted = llm.redacted();
        assert!(!redacted.contains("mysecret"),
            "local_endpoint token should be redacted in Redact: {}", redacted);
        assert!(redacted.contains("[REDACTED]"),
            "Redacted output should contain [REDACTED]: {}", redacted);
        assert!(redacted.contains("format=json"),
            "Non-sensitive params should remain: {}", redacted);
    }

    #[test]
    fn test_llm_config_redacts_local_endpoint_userinfo() {
        // Test that local_endpoint with userinfo (user:pass@host) is redacted
        let llm = LlmConfig {
            mode: LlmMode::Local,
            local_endpoint: Some("http://admin:secret@localhost:11434".to_string()),
            local_model: None,
            cloud_enabled: false,
            cloud_endpoint: None,
            cloud_model: None,
            api_key: None,
            timeout_seconds: 120,
            max_tokens: 4096,
        };

        let debug_output = format!("{:?}", llm);
        assert!(!debug_output.contains("secret"),
            "local_endpoint userinfo password should be redacted: {}", debug_output);
        assert!(!debug_output.contains("admin:"),
            "local_endpoint userinfo should be fully redacted: {}", debug_output);
        assert!(debug_output.contains("[REDACTED]@"),
            "Redacted userinfo should be visible: {}", debug_output);

        let redacted = llm.redacted();
        assert!(!redacted.contains("secret"),
            "Redact trait should also redact userinfo: {}", redacted);
    }

    #[test]
    fn test_url_with_query_no_path_valid() {
        // URLs with query string but no path should be valid
        // Review 18 fix: is_valid_url should accept these
        assert!(is_valid_url("https://example.com?foo=bar"),
            "URL with query but no path should be valid");
        assert!(is_valid_url("http://localhost?param=value"),
            "localhost with query but no path should be valid");
        assert!(is_valid_url("https://api.example.com?key=value&other=123"),
            "URL with multiple query params but no path should be valid");
    }

    #[test]
    fn test_url_with_fragment_no_path_valid() {
        // URLs with fragment but no path should be valid
        assert!(is_valid_url("https://example.com#section"),
            "URL with fragment but no path should be valid");
        assert!(is_valid_url("http://localhost#anchor"),
            "localhost with fragment but no path should be valid");
    }

    #[test]
    fn test_url_with_query_and_fragment_no_path_valid() {
        // URLs with both query and fragment but no path should be valid
        assert!(is_valid_url("https://example.com?foo=bar#section"),
            "URL with query and fragment but no path should be valid");
    }

    #[test]
    fn test_url_with_path_query_fragment_still_valid() {
        // Existing URLs with path, query, and fragment should still work
        assert!(is_valid_url("https://example.com/path?foo=bar#section"),
            "Full URL with path, query, and fragment should be valid");
        assert!(is_valid_url("http://localhost:8080/api/v1?key=value"),
            "localhost with path and query should be valid");
    }

    #[test]
    fn test_check_output_folder_exists_file_not_directory() {
        // Test that check_output_folder_exists detects when path is a file, not directory
        use std::io::Write;

        // Create a unique temporary file using thread ID and timestamp to avoid collisions
        let unique_id = format!("tf_config_test_{:?}_{}",
            std::thread::current().id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join(format!("{}.txt", unique_id));

        // Create the file
        {
            let mut file = std::fs::File::create(&test_file).expect("Failed to create test file");
            writeln!(file, "test content").expect("Failed to write to test file");
        }

        // Create config pointing to the file (not a directory)
        let yaml = format!(r#"
project_name: "test"
output_folder: "{}"
"#, test_file.to_string_lossy().replace('\\', "/"));

        let file = create_temp_config(&yaml);
        let config = load_config(file.path()).unwrap();

        // check_output_folder_exists should return warning about not being a directory
        let warning = config.check_output_folder_exists();
        assert!(warning.is_some(), "Should warn when output_folder is a file, not directory");
        let warning_msg = warning.unwrap();
        assert!(warning_msg.contains("not a directory"),
            "Warning should mention 'not a directory': {}", warning_msg);

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
    }

    #[test]
    fn test_redact_url_userinfo_fragment_boundary() {
        // Test that @ in fragment is not treated as userinfo
        // RFC 3986: authority ends at first /, ?, or #
        let url = "https://example.com#user@mention";
        let redacted = redact_url_userinfo(url);
        assert_eq!(redacted, url, "@ in fragment should not be treated as userinfo");

        let url2 = "https://example.com/path#section@ref";
        let redacted2 = redact_url_userinfo(url2);
        assert_eq!(redacted2, url2, "@ in fragment after path should not be treated as userinfo");
    }

    // ===== Review 21 fixes =====

    #[test]
    fn test_redact_url_userinfo_password_contains_at() {
        // HIGH: Password containing unencoded @ should still be fully redacted
        // The @ in p@ssword is part of the password, not the userinfo delimiter
        let url = "https://admin:p@ssword@example.com/api";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("admin"), "Username should be redacted");
        assert!(!redacted.contains("p@ssword"), "Password with @ should be redacted");
        assert!(redacted.contains("[REDACTED]@example.com"));
        assert_eq!(redacted, "https://[REDACTED]@example.com/api");
    }

    #[test]
    fn test_redact_url_userinfo_password_multiple_at() {
        // Password with multiple @ characters
        let url = "https://user:p@ss@word@host.example.com:8080/path";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("user"), "Username should be redacted");
        assert!(!redacted.contains("p@ss@word"), "Password with multiple @ should be redacted");
        assert!(redacted.contains(":8080"));
        assert_eq!(redacted, "https://[REDACTED]@host.example.com:8080/path");
    }

    #[test]
    fn test_redact_url_userinfo_complex_password() {
        // Complex password with special chars including @
        // Note: # must be percent-encoded in userinfo per RFC 3986, so we use %23
        let url = "https://admin:C0mpl3x!P@ss%23123@api.example.com?token=abc";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("admin"));
        assert!(!redacted.contains("C0mpl3x!P@ss%23123"));
        assert!(redacted.contains("token=[REDACTED]"));
        assert_eq!(redacted, "https://[REDACTED]@api.example.com?token=[REDACTED]");
    }

    #[test]
    fn test_redact_url_encoded_param_api_key() {
        // MEDIUM: URL-encoded parameter name api%5Fkey (api_key) should be redacted
        let url = "https://example.com?api%5Fkey=secret123";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("secret123"), "Value of encoded api_key should be redacted");
        assert!(redacted.contains("api%5Fkey=[REDACTED]"));
    }

    #[test]
    fn test_redact_url_encoded_param_token() {
        // URL-encoded 'token' (%74%6F%6B%65%6E)
        let url = "https://example.com?%74%6F%6B%65%6E=mysecret";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("mysecret"), "Value of encoded token should be redacted");
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn test_redact_url_encoded_param_mixed() {
        // Mix of encoded and plain param names
        let url = "https://example.com?api%5Fkey=secret1&password=secret2&foo=bar";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("secret1"));
        assert!(!redacted.contains("secret2"));
        assert!(redacted.contains("foo=bar"), "Non-sensitive param should remain");
        assert!(redacted.contains("api%5Fkey=[REDACTED]"));
        assert!(redacted.contains("password=[REDACTED]"));
    }

    #[test]
    fn test_redact_url_encoded_param_in_fragment() {
        // URL-encoded param in fragment (OAuth implicit flow)
        let url = "https://example.com#access%5Ftoken=xyz123";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("xyz123"), "Value of encoded access_token should be redacted");
        assert!(redacted.contains("access%5Ftoken=[REDACTED]"));
    }

    #[test]
    fn test_redact_url_encoded_plus_as_space() {
        // Plus sign decodes to space in application/x-www-form-urlencoded
        // api+key would decode to "api key" which is not sensitive
        let url = "https://example.com?api+key=value";
        let redacted = redact_url_sensitive_params(url);
        // "api key" (with space) is not in our sensitive list, so it should remain
        assert_eq!(redacted, url);
    }

    #[test]
    fn test_redact_url_combined_userinfo_at_and_encoded_params() {
        // Combined: password with @, and encoded param names
        let url = "https://user:p@ss@example.com?api%5Fkey=secret&access%5Ftoken=mytoken123";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("user:"), "Username should be redacted");
        assert!(!redacted.contains("p@ss"), "Password with @ should be redacted");
        assert!(!redacted.contains("secret"), "api_key value should be redacted");
        assert!(!redacted.contains("mytoken123"), "access_token value should be redacted");
        assert_eq!(redacted, "https://[REDACTED]@example.com?api%5Fkey=[REDACTED]&access%5Ftoken=[REDACTED]");
    }

    // =====================================================================
    // Review 22 Tests: Double-encoded params, kebab-case params, URL whitespace
    // =====================================================================

    #[test]
    fn test_redact_url_double_encoded_api_key() {
        // Double-encoded: api_key -> api%5Fkey -> api%255Fkey
        // %25 is encoded %, so api%255Fkey decodes to api%5Fkey which decodes to api_key
        let url = "https://example.com?api%255Fkey=secret123";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("secret123"), "Double-encoded api_key value should be redacted");
        assert!(redacted.contains("=[REDACTED]"), "Should contain redacted value");
    }

    #[test]
    fn test_redact_url_double_encoded_token() {
        // Double-encoded token: token -> tok%65n won't match, but api%255Fkey case is realistic
        // Test that actual double-encoding of underscore works: api%255Fkey -> api%5Fkey -> api_key
        let url = "https://example.com?api%255Fkey=mysecret";
        let redacted = redact_url_sensitive_params(url);
        // After double-decode, api%255Fkey becomes api_key which is in sensitive list
        assert!(!redacted.contains("mysecret"), "Double-encoded api_key should be redacted");
    }

    #[test]
    fn test_redact_url_double_encoded_password() {
        // Double-encoded: password -> password (no underscore) but api_key case
        // api%255Fkey double-decodes to api_key which is in the sensitive list
        let url = "https://example.com?api%255Fkey=pass123&foo=bar";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("pass123"), "Double-encoded api_key value should be redacted");
        assert!(redacted.contains("foo=bar"), "Non-sensitive param should remain");
    }

    #[test]
    fn test_redact_url_kebab_case_api_key() {
        // Kebab-case: api-key (with hyphen)
        let url = "https://example.com?api-key=secret-value";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("secret-value"), "api-key value should be redacted");
        assert!(redacted.contains("api-key=[REDACTED]"), "api-key param should be redacted");
    }

    #[test]
    fn test_redact_url_kebab_case_access_token() {
        // Kebab-case: access-token (with hyphen)
        let url = "https://example.com?access-token=mytoken123";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("mytoken123"), "access-token value should be redacted");
        assert!(redacted.contains("access-token=[REDACTED]"), "access-token param should be redacted");
    }

    #[test]
    fn test_redact_url_kebab_case_client_secret() {
        let url = "https://example.com?client-secret=very-secret&other=value";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("very-secret"), "client-secret value should be redacted");
        assert!(redacted.contains("other=value"), "Non-sensitive param should remain");
    }

    #[test]
    fn test_redact_url_kebab_case_multiple() {
        // Multiple kebab-case sensitive params
        let url = "https://example.com?api-key=key1&access-token=tok1&session-token=sess1&foo=bar";
        let redacted = redact_url_sensitive_params(url);
        assert!(!redacted.contains("key1"), "api-key value should be redacted");
        assert!(!redacted.contains("tok1"), "access-token value should be redacted");
        assert!(!redacted.contains("sess1"), "session-token value should be redacted");
        assert!(redacted.contains("foo=bar"), "Non-sensitive param should remain");
    }

    #[test]
    fn test_jira_endpoint_trailing_whitespace_rejected() {
        let yaml = r#"
project_name: test-project
output_folder: ./output
jira:
  endpoint: "https://jira.example.com  "
"#;
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("test_jira_whitespace_{}.yaml", std::process::id()));
        std::fs::write(&file_path, yaml).unwrap();

        let result = load_config(&file_path);
        std::fs::remove_file(&file_path).ok();

        assert!(result.is_err(), "Should reject endpoint with trailing whitespace");
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("jira.endpoint"), "Error should mention jira.endpoint");
        assert!(err_msg.contains("whitespace"), "Error should mention whitespace");
    }

    #[test]
    fn test_jira_endpoint_leading_whitespace_rejected() {
        let yaml = r#"
project_name: test-project
output_folder: ./output
jira:
  endpoint: "  https://jira.example.com"
"#;
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("test_jira_leading_ws_{}.yaml", std::process::id()));
        std::fs::write(&file_path, yaml).unwrap();

        let result = load_config(&file_path);
        std::fs::remove_file(&file_path).ok();

        assert!(result.is_err(), "Should reject endpoint with leading whitespace");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("whitespace"), "Error should mention whitespace");
    }

    #[test]
    fn test_squash_endpoint_whitespace_rejected() {
        let yaml = r#"
project_name: test-project
output_folder: ./output
squash:
  endpoint: "https://squash.example.com   "
"#;
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("test_squash_ws_{}.yaml", std::process::id()));
        std::fs::write(&file_path, yaml).unwrap();

        let result = load_config(&file_path);
        std::fs::remove_file(&file_path).ok();

        assert!(result.is_err(), "Should reject squash endpoint with trailing whitespace");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("squash.endpoint"), "Error should mention squash.endpoint");
    }

    #[test]
    fn test_llm_local_endpoint_whitespace_rejected() {
        let yaml = r#"
project_name: test-project
output_folder: ./output
llm:
  mode: local
  local_endpoint: " http://localhost:11434 "
"#;
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("test_llm_local_ws_{}.yaml", std::process::id()));
        std::fs::write(&file_path, yaml).unwrap();

        let result = load_config(&file_path);
        std::fs::remove_file(&file_path).ok();

        assert!(result.is_err(), "Should reject local_endpoint with whitespace");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("llm.local_endpoint"), "Error should mention llm.local_endpoint");
    }

    #[test]
    fn test_llm_cloud_endpoint_whitespace_rejected() {
        let yaml = r#"
project_name: test-project
output_folder: ./output
llm:
  mode: cloud
  cloud_enabled: true
  cloud_endpoint: "https://api.openai.com/v1   "
  cloud_model: gpt-4
  api_key: sk-test
"#;
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("test_llm_cloud_ws_{}.yaml", std::process::id()));
        std::fs::write(&file_path, yaml).unwrap();

        let result = load_config(&file_path);
        std::fs::remove_file(&file_path).ok();

        assert!(result.is_err(), "Should reject cloud_endpoint with trailing whitespace");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("llm.cloud_endpoint"), "Error should mention llm.cloud_endpoint");
    }

    #[test]
    fn test_valid_endpoints_without_whitespace_accepted() {
        let yaml = r#"
project_name: test-project
output_folder: ./output
jira:
  endpoint: "https://jira.example.com"
squash:
  endpoint: "https://squash.example.com"
llm:
  mode: auto
  local_endpoint: "http://localhost:11434"
"#;
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("test_valid_endpoints_{}.yaml", std::process::id()));
        std::fs::write(&file_path, yaml).unwrap();

        let result = load_config(&file_path);
        std::fs::remove_file(&file_path).ok();

        assert!(result.is_ok(), "Should accept endpoints without whitespace");
    }

    // === REVIEW 23 TESTS: Query param separators, whitespace, and IPv6 zone-id ===

    #[test]
    fn test_redact_url_semicolon_separator() {
        // RFC 1866 allows semicolon as query parameter separator in HTML forms
        let url = "https://example.com?token=secret;foo=bar";
        let redacted = redact_url_sensitive_params(url);
        assert!(redacted.contains("token=[REDACTED]"), "Should redact token with semicolon separator");
        assert!(redacted.contains("foo=bar"), "Should preserve non-sensitive param");
        assert!(redacted.contains(";"), "Should preserve semicolon separator");
    }

    #[test]
    fn test_redact_url_semicolon_only_sensitive() {
        // All params separated by semicolons
        let url = "https://example.com?api_key=sk123;password=secret;user=john";
        let redacted = redact_url_sensitive_params(url);
        assert!(redacted.contains("api_key=[REDACTED]"), "Should redact api_key");
        assert!(redacted.contains("password=[REDACTED]"), "Should redact password");
        assert!(redacted.contains("user=john"), "Should preserve non-sensitive param");
    }

    #[test]
    fn test_redact_url_mixed_separators() {
        // Mix of & and ; separators
        let url = "https://example.com?token=abc&secret=def;api_key=ghi;foo=bar&auth=xyz";
        let redacted = redact_url_sensitive_params(url);
        assert!(redacted.contains("token=[REDACTED]"), "Should redact token");
        assert!(redacted.contains("secret=[REDACTED]"), "Should redact secret");
        assert!(redacted.contains("api_key=[REDACTED]"), "Should redact api_key");
        assert!(redacted.contains("auth=[REDACTED]"), "Should redact auth");
        assert!(redacted.contains("foo=bar"), "Should preserve non-sensitive param");
    }

    #[test]
    fn test_redact_url_whitespace_around_key() {
        // Whitespace around parameter key
        let url = "https://example.com?token =secret";
        let redacted = redact_url_sensitive_params(url);
        assert!(redacted.contains("[REDACTED]"), "Should redact token with trailing whitespace in key");
        assert!(!redacted.contains("secret"), "Should not expose secret value");
    }

    #[test]
    fn test_redact_url_leading_whitespace_key() {
        // Leading whitespace in parameter key
        let url = "https://example.com? api_key=secret";
        let redacted = redact_url_sensitive_params(url);
        assert!(redacted.contains("[REDACTED]"), "Should redact api_key with leading whitespace");
        assert!(!redacted.contains("secret"), "Should not expose secret value");
    }

    #[test]
    fn test_redact_url_whitespace_with_semicolon() {
        // Whitespace around key with semicolon separator
        let url = "https://example.com?foo=bar; token =mysecret;other=value";
        let redacted = redact_url_sensitive_params(url);
        assert!(redacted.contains("[REDACTED]"), "Should redact token with whitespace and semicolon");
        assert!(!redacted.contains("mysecret"), "Should not expose secret value");
        assert!(redacted.contains("foo=bar"), "Should preserve first param");
        assert!(redacted.contains("other=value"), "Should preserve last param");
    }

    #[test]
    fn test_ipv6_zone_id_empty_rejected() {
        // Empty zone ID is invalid (e.g., "fe80::1%" with nothing after %)
        assert!(!is_valid_url("http://[fe80::1%]:8080"), "Should reject empty zone ID");
        assert!(!is_valid_url("http://[fe80::1%]"), "Should reject empty zone ID without port");
        assert!(!is_valid_url("https://[::1%]"), "Should reject empty zone ID on localhost");
    }

    #[test]
    fn test_ipv6_zone_id_valid() {
        // Valid zone IDs (interface names)
        assert!(is_valid_url("http://[fe80::1%eth0]:8080"), "Should accept zone ID with interface name");
        assert!(is_valid_url("http://[fe80::1%eth0]"), "Should accept zone ID without port");
        assert!(is_valid_url("http://[fe80::1%wlan0]:3000"), "Should accept zone ID with wlan");
        assert!(is_valid_url("http://[fe80::1%en0]:80"), "Should accept zone ID with en0");
        assert!(is_valid_url("http://[fe80::1%lo]:8080"), "Should accept zone ID with lo");
        // URL-encoded % is %25
        assert!(is_valid_url("http://[fe80::1%25eth0]:8080"), "Should accept URL-encoded zone ID");
    }

    #[test]
    fn test_ipv6_zone_id_invalid_chars() {
        // Zone ID with invalid characters (only alphanumeric, hyphen, underscore, dot allowed)
        assert!(!is_valid_url("http://[fe80::1%eth/0]:8080"), "Should reject zone ID with slash");
        assert!(!is_valid_url("http://[fe80::1%eth@0]:8080"), "Should reject zone ID with @");
        assert!(!is_valid_url("http://[fe80::1%eth 0]:8080"), "Should reject zone ID with space");
    }

    #[test]
    fn test_redact_fragment_semicolon_separator() {
        // Semicolon separator in fragment (OAuth implicit flow with semicolon)
        let url = "https://app.com/callback#token=abc;access_token=secret";
        let redacted = redact_url_sensitive_params(url);
        assert!(redacted.contains("token=[REDACTED]"), "Should redact token in fragment");
        assert!(redacted.contains("access_token=[REDACTED]"), "Should redact access_token in fragment");
        assert!(!redacted.contains("abc"), "Should not expose token value");
        assert!(!redacted.contains("secret"), "Should not expose access_token value");
    }

    #[test]
    fn test_redact_both_query_and_fragment_semicolon() {
        // Both query and fragment with semicolon separators
        let url = "https://example.com?api_key=key1;foo=bar#token=tok1;baz=qux";
        let redacted = redact_url_sensitive_params(url);
        assert!(redacted.contains("api_key=[REDACTED]"), "Should redact api_key in query");
        assert!(redacted.contains("token=[REDACTED]"), "Should redact token in fragment");
        assert!(redacted.contains("foo=bar"), "Should preserve foo in query");
        assert!(redacted.contains("baz=qux"), "Should preserve baz in fragment");
    }
}

//! Template loading and validation for test-framework
//!
//! Provides loading and basic format validation of templates (CR, PPT, Anomaly)
//! from configured file paths. Templates are validated for existence, correct
//! file extension, and basic format integrity before being made available.
//!
//! # Usage
//!
//! ```no_run
//! use tf_config::{TemplateLoader, TemplateKind, TemplatesConfig};
//!
//! let config = TemplatesConfig {
//!     cr: Some("templates/cr.md".to_string()),
//!     ppt: Some("templates/report.pptx".to_string()),
//!     anomaly: None,
//! };
//! let loader = TemplateLoader::new(&config);
//!
//! // Load a single template
//! let cr = loader.load_template(TemplateKind::Cr).unwrap();
//! println!("CR template: {} bytes", cr.size_bytes());
//!
//! // Load all configured templates at once
//! let all = loader.load_all().unwrap();
//! println!("Loaded {} templates", all.len());
//! ```

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::TemplatesConfig;

/// ZIP magic bytes: PK\x03\x04
const ZIP_MAGIC: &[u8; 4] = b"PK\x03\x04";

/// Minimum size for a valid .pptx file in bytes.
///
/// A valid .pptx is an OOXML ZIP archive that must contain at least
/// `[Content_Types].xml` and basic relationship entries. This threshold
/// prevents truncated or corrupted files from being accepted. Full OOXML
/// structural validation is deferred to `tf-export`.
const MIN_PPTX_SIZE: usize = 100;

/// Maximum allowed file size for Markdown templates (10 MB).
///
/// CR and anomaly templates are plain-text Markdown files. 10 MB is generous
/// for any realistic report template while preventing accidental loading of
/// multi-gigabyte files that could exhaust memory. Typical templates are
/// well under 100 KB.
const MAX_MD_SIZE: u64 = 10 * 1024 * 1024;

/// Maximum allowed file size for PowerPoint templates (100 MB).
///
/// PPTX files are ZIP archives that can contain embedded images and media,
/// making them significantly larger than Markdown templates. 100 MB allows
/// for image-heavy presentation templates while still guarding against
/// unbounded allocation from device files or corrupted paths.
const MAX_PPTX_SIZE: u64 = 100 * 1024 * 1024;

/// Types of templates supported by the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateKind {
    /// Daily report template (CR quotidien) - Markdown format
    Cr,
    /// Weekly/TNR presentation template - PowerPoint format
    Ppt,
    /// Bug report template - Markdown format
    Anomaly,
}

impl fmt::Display for TemplateKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemplateKind::Cr => write!(f, "cr"),
            TemplateKind::Ppt => write!(f, "ppt"),
            TemplateKind::Anomaly => write!(f, "anomaly"),
        }
    }
}

impl TemplateKind {
    /// Returns the expected file extension for this template kind (e.g. `".md"`, `".pptx"`)
    pub fn expected_extension(&self) -> &'static str {
        match self {
            TemplateKind::Cr | TemplateKind::Anomaly => ".md",
            TemplateKind::Ppt => ".pptx",
        }
    }

    /// Returns all template kinds
    pub fn all() -> &'static [TemplateKind] {
        &[TemplateKind::Cr, TemplateKind::Ppt, TemplateKind::Anomaly]
    }
}

/// Errors that can occur when loading or validating templates
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum TemplateError {
    /// Template kind not configured in config.yaml
    #[error("Template {kind} not configured. {hint}")]
    NotConfigured { kind: TemplateKind, hint: String },

    /// Template file not found at configured path
    #[error("Template file not found: '{path}' ({kind}). {hint}")]
    FileNotFound {
        path: String,
        kind: TemplateKind,
        hint: String,
    },

    /// Template file has wrong extension
    #[error("Invalid extension for template '{path}': expected {expected}, got '{actual}'. {hint}")]
    InvalidExtension {
        path: String,
        expected: String,
        actual: String,
        hint: String,
    },

    /// Template file has invalid format
    #[error("Invalid format for template '{path}' ({kind}): {cause}. {hint}")]
    InvalidFormat {
        path: String,
        kind: TemplateKind,
        cause: String,
        hint: String,
    },

    /// Attempted to read binary template content as text
    #[error("Template '{path}' ({kind}) contains binary content. {hint}")]
    BinaryContent {
        path: String,
        kind: TemplateKind,
        hint: String,
    },

    /// Failed to read template file
    #[error("Failed to read template '{path}': {cause}. {hint}")]
    ReadError {
        path: String,
        cause: String,
        hint: String,
    },
}

/// A validated and loaded template ready for use
pub struct LoadedTemplate {
    kind: TemplateKind,
    path: PathBuf,
    content: Vec<u8>,
}

impl LoadedTemplate {
    /// Get the template kind
    pub fn kind(&self) -> TemplateKind {
        self.kind
    }

    /// Get the source file path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the raw content bytes
    pub fn content(&self) -> &[u8] {
        &self.content
    }

    /// Get content as UTF-8 string (for Markdown templates)
    ///
    /// Returns [`TemplateError::BinaryContent`] for PPTX templates (use
    /// [`content()`](Self::content) to access raw bytes for binary formats).
    /// Returns [`TemplateError::InvalidFormat`] for non-UTF-8 markdown templates.
    pub fn content_as_str(&self) -> Result<&str, TemplateError> {
        std::str::from_utf8(&self.content).map_err(|_| {
            if self.kind == TemplateKind::Ppt {
                TemplateError::BinaryContent {
                    path: self.path.display().to_string(),
                    kind: self.kind,
                    hint: "This template is binary (PPTX); use content() for raw bytes instead"
                        .to_string(),
                }
            } else {
                TemplateError::InvalidFormat {
                    path: self.path.display().to_string(),
                    kind: self.kind,
                    cause: "invalid UTF-8".to_string(),
                    hint: format!(
                        "Ensure the file is a valid {} template with UTF-8 encoding",
                        self.kind
                    ),
                }
            }
        })
    }

    /// Get the file size in bytes (computed from content length)
    pub fn size_bytes(&self) -> u64 {
        self.content.len() as u64
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl LoadedTemplate {
    /// Create a `LoadedTemplate` for testing purposes without loading from disk.
    ///
    /// Available in test builds (`#[cfg(test)]`) and when the `test-utils` feature
    /// is enabled. Allows downstream consumers to construct instances for unit tests
    /// without requiring real template files.
    ///
    /// # Enabling for downstream crates
    ///
    /// Add `tf-config = { workspace = true, features = ["test-utils"] }` to your
    /// `[dev-dependencies]`.
    pub fn new_for_test(kind: TemplateKind, path: impl Into<PathBuf>, content: Vec<u8>) -> Self {
        Self {
            kind,
            path: path.into(),
            content,
        }
    }
}

// Custom Debug implementation: never expose raw template content
impl fmt::Debug for LoadedTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoadedTemplate")
            .field("kind", &self.kind)
            .field("path", &self.path)
            .field("size_bytes", &self.size_bytes())
            .finish()
    }
}

/// Loads and validates templates from configured paths
#[derive(Debug)]
pub struct TemplateLoader<'a> {
    config: &'a TemplatesConfig,
}

impl<'a> TemplateLoader<'a> {
    /// Create a new template loader from configuration
    ///
    /// Borrows the configuration rather than cloning it, so the loader
    /// must not outlive the referenced `TemplatesConfig`.
    ///
    /// ```no_run
    /// use tf_config::{TemplateLoader, TemplatesConfig};
    ///
    /// let config = TemplatesConfig {
    ///     cr: Some("templates/cr.md".to_string()),
    ///     ppt: None,
    ///     anomaly: None,
    /// };
    /// let loader = TemplateLoader::new(&config);
    /// ```
    pub fn new(config: &'a TemplatesConfig) -> Self {
        Self { config }
    }

    /// Load a specific template by kind
    ///
    /// Resolves the configured path, validates the file extension, reads the file,
    /// and validates the format before returning the loaded template.
    ///
    /// **Note:** Relative paths are resolved against the current working directory,
    /// not the config file location. Use absolute paths in config for portability.
    ///
    /// ```no_run
    /// use tf_config::{TemplateLoader, TemplateKind, TemplatesConfig};
    ///
    /// let config = TemplatesConfig {
    ///     cr: Some("templates/cr.md".to_string()),
    ///     ppt: None,
    ///     anomaly: None,
    /// };
    /// let loader = TemplateLoader::new(&config);
    /// let template = loader.load_template(TemplateKind::Cr).unwrap();
    /// println!("Loaded {} ({} bytes)", template.kind(), template.size_bytes());
    /// ```
    pub fn load_template(&self, kind: TemplateKind) -> Result<LoadedTemplate, TemplateError> {
        let path_str = self.get_configured_path(kind)?;
        self.load_from_path(kind, path_str)
    }

    /// Load a template from a resolved path string.
    ///
    /// **Known limitation:** Relative paths are resolved against the current
    /// working directory (`std::env::current_dir`), not against the config file
    /// location. Callers running the CLI from a different directory may get
    /// unexpected `FileNotFound` errors. Use absolute paths in config to avoid
    /// ambiguity.
    fn load_from_path(&self, kind: TemplateKind, path_str: &str) -> Result<LoadedTemplate, TemplateError> {
        let path = PathBuf::from(path_str);

        // Validate extension before reading (avoids unnecessary I/O)
        validate_extension(&path, kind)?;

        // Pre-check file size to prevent unbounded memory allocation
        let max_size = match kind {
            TemplateKind::Cr | TemplateKind::Anomaly => MAX_MD_SIZE,
            TemplateKind::Ppt => MAX_PPTX_SIZE,
        };
        if let Ok(metadata) = fs::metadata(&path) {
            let file_size = metadata.len();
            if file_size > max_size {
                return Err(oversized_error(path_str, kind, file_size, max_size));
            }
        }
        // If metadata fails, proceed to fs::read which will surface the actual error

        // Read file content — handles NotFound directly to avoid TOCTOU race
        let content = fs::read(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                TemplateError::FileNotFound {
                    path: path_str.to_string(),
                    kind,
                    hint: format!(
                        "Check the path in config.yaml or create the template file at '{}'",
                        path_str
                    ),
                }
            } else {
                let hint = if path.is_dir() {
                    format!(
                        "The path '{}' is a directory, not a file. Update config.yaml to point to a template file",
                        path_str
                    )
                } else {
                    "Check file permissions and ensure the file is readable".to_string()
                };
                TemplateError::ReadError {
                    path: path_str.to_string(),
                    cause: e.to_string(),
                    hint,
                }
            }
        })?;

        // Post-read size check: guards against TOCTOU where file grows between
        // metadata check and read, or when metadata was unavailable above.
        let content_size = content.len() as u64;
        if content_size > max_size {
            return Err(oversized_error(path_str, kind, content_size, max_size));
        }

        // Validate format
        validate_content(kind, &content, &path)?;

        Ok(LoadedTemplate {
            kind,
            path,
            content,
        })
    }

    /// Load all configured templates
    ///
    /// Iterates over every [`TemplateKind`] in declaration order
    /// (`Cr`, `Ppt`, `Anomaly`) and loads each one that has a path
    /// set in the configuration. Skips unconfigured kinds. Uses **fail-fast**
    /// semantics: returns the first error encountered (in iteration order) and
    /// does not attempt to load remaining templates.
    pub fn load_all(&self) -> Result<HashMap<TemplateKind, LoadedTemplate>, TemplateError> {
        let mut templates = HashMap::with_capacity(TemplateKind::all().len());

        for &kind in TemplateKind::all() {
            // Single resolution: try to get the path, skip if not configured
            if let Some(path_str) = self.resolve_path(kind) {
                let template = self.load_from_path(kind, path_str)?;
                templates.insert(kind, template);
            }
        }

        Ok(templates)
    }

    /// Resolve the configured path for a template kind, returning `None` if not configured.
    fn resolve_path(&self, kind: TemplateKind) -> Option<&str> {
        match kind {
            TemplateKind::Cr => self.config.cr.as_deref(),
            TemplateKind::Ppt => self.config.ppt.as_deref(),
            TemplateKind::Anomaly => self.config.anomaly.as_deref(),
        }
    }

    /// Get the configured path for a template kind, returning an error if not configured.
    fn get_configured_path(&self, kind: TemplateKind) -> Result<&str, TemplateError> {
        self.resolve_path(kind).ok_or_else(|| TemplateError::NotConfigured {
            kind,
            hint: format!(
                "Add 'templates.{}: ./path/to/{template_file}' to your config.yaml",
                kind,
                template_file = match kind {
                    TemplateKind::Cr => "cr.md",
                    TemplateKind::Ppt => "report.pptx",
                    TemplateKind::Anomaly => "anomaly.md",
                }
            ),
        })
    }

}

/// Validate the file extension matches the expected format (case-insensitive)
fn validate_extension(path: &Path, kind: TemplateKind) -> Result<(), TemplateError> {
    let expected = kind.expected_extension();
    // Compare without dot prefix to avoid heap allocation on the happy path.
    // expected_extension() returns ".md" or ".pptx", so skip the leading dot.
    let expected_no_dot = &expected[1..];

    let ext_str = path.extension().and_then(|e| e.to_str());

    let matches = ext_str
        .map(|e| e.eq_ignore_ascii_case(expected_no_dot))
        .unwrap_or(false);

    if !matches {
        let actual = ext_str.map(|e| format!(".{}", e)).unwrap_or_else(|| "(none)".to_string());
        return Err(TemplateError::InvalidExtension {
            path: path.display().to_string(),
            expected: expected.to_string(),
            actual,
            hint: format!("Rename the file to use {} extension", expected),
        });
    }

    Ok(())
}

/// Build an `InvalidFormat` error for oversized files, used by both the
/// pre-read metadata check and the post-read TOCTOU guard.
fn oversized_error(path: &str, kind: TemplateKind, actual_size: u64, max_size: u64) -> TemplateError {
    TemplateError::InvalidFormat {
        path: path.to_string(),
        kind,
        cause: format!(
            "file is too large ({} bytes, maximum {} bytes)",
            actual_size, max_size
        ),
        hint: format!(
            "Reduce the file size or verify this is a valid {} template",
            kind
        ),
    }
}

/// Validate the format of a template based on its kind
///
/// Checks that `content` is well-formed for the given [`TemplateKind`]:
/// - Markdown (`.md`): non-empty, non-whitespace-only, valid UTF-8
/// - PowerPoint (`.pptx`): ZIP magic bytes and minimum size
///
/// The `path` parameter is used **only for error context** (included in error
/// messages to help the user locate the problematic file). It is not validated,
/// resolved, or read from — callers may pass any descriptive path.
pub fn validate_content(kind: TemplateKind, content: &[u8], path: &Path) -> Result<(), TemplateError> {
    let path_str = path.display().to_string();
    match kind {
        TemplateKind::Cr | TemplateKind::Anomaly => validate_markdown(content, &path_str, kind),
        TemplateKind::Ppt => validate_pptx(content, &path_str, kind),
    }
}

/// Validate Markdown template: must be non-empty, non-whitespace-only, valid UTF-8
fn validate_markdown(content: &[u8], path: &str, kind: TemplateKind) -> Result<(), TemplateError> {
    if content.is_empty() {
        return Err(TemplateError::InvalidFormat {
            path: path.to_string(),
            kind,
            cause: "file is empty".to_string(),
            hint: "Ensure the file is a valid Markdown template with content".to_string(),
        });
    }

    let text = std::str::from_utf8(content).map_err(|_| TemplateError::InvalidFormat {
        path: path.to_string(),
        kind,
        cause: "not valid UTF-8 text".to_string(),
        hint: "Ensure the file is a valid Markdown template with UTF-8 encoding".to_string(),
    })?;

    if text.trim().is_empty() {
        return Err(TemplateError::InvalidFormat {
            path: path.to_string(),
            kind,
            cause: "file contains only whitespace".to_string(),
            hint: "Ensure the file is a valid Markdown template with meaningful content".to_string(),
        });
    }

    Ok(())
}

/// Validate PowerPoint template: must have ZIP magic bytes and minimum size
fn validate_pptx(content: &[u8], path: &str, kind: TemplateKind) -> Result<(), TemplateError> {
    if content.is_empty() {
        return Err(TemplateError::InvalidFormat {
            path: path.to_string(),
            kind,
            cause: "file is empty".to_string(),
            hint: "Ensure the file is a valid .pptx template".to_string(),
        });
    }

    if content.len() < 4 || content[..4] != *ZIP_MAGIC {
        return Err(TemplateError::InvalidFormat {
            path: path.to_string(),
            kind,
            cause: "file does not have valid ZIP/OOXML signature".to_string(),
            hint: "Ensure the file is a valid .pptx PowerPoint template (OOXML format)".to_string(),
        });
    }

    if content.len() < MIN_PPTX_SIZE {
        return Err(TemplateError::InvalidFormat {
            path: path.to_string(),
            kind,
            cause: format!(
                "file is too small ({} bytes, minimum {} bytes)",
                content.len(),
                MIN_PPTX_SIZE
            ),
            hint: "Ensure the file is a complete .pptx template, not a truncated file".to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to get fixtures path relative to the crate root
    fn fixtures_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("templates")
    }

    // Helper to create a minimal valid pptx content (ZIP header + padding with invalid UTF-8)
    fn create_valid_pptx_bytes() -> Vec<u8> {
        let mut content = Vec::new();
        // ZIP magic bytes
        content.extend_from_slice(b"PK\x03\x04");
        // Padding with invalid UTF-8 sequences to simulate binary content
        content.resize(MIN_PPTX_SIZE + 10, 0xFF);
        content
    }

    // =========================================================================
    // Task 2: API de chargement — AC #1
    // =========================================================================

    #[test]
    fn test_load_cr_template_success() {
        let cr_path = fixtures_path().join("cr-test.md");
        let config = TemplatesConfig {
            cr: Some(cr_path.display().to_string()),
            ppt: None,
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let template = loader.load_template(TemplateKind::Cr).unwrap();
        assert_eq!(template.kind(), TemplateKind::Cr);
        assert!(!template.content().is_empty());
        assert!(template.content_as_str().is_ok());
        assert!(template.size_bytes() > 0);
    }

    #[test]
    fn test_load_anomaly_template_success() {
        let anomaly_path = fixtures_path().join("anomaly-test.md");
        let config = TemplatesConfig {
            cr: None,
            ppt: None,
            anomaly: Some(anomaly_path.display().to_string()),
        };
        let loader = TemplateLoader::new(&config);
        let template = loader.load_template(TemplateKind::Anomaly).unwrap();
        assert_eq!(template.kind(), TemplateKind::Anomaly);
        assert!(!template.content().is_empty());
        assert!(template.content_as_str().is_ok());
    }

    #[test]
    fn test_load_pptx_template_success() {
        let dir = tempfile::tempdir().unwrap();
        let pptx_path = dir.path().join("test.pptx");
        let content = create_valid_pptx_bytes();
        fs::write(&pptx_path, &content).unwrap();

        let config = TemplatesConfig {
            cr: None,
            ppt: Some(pptx_path.display().to_string()),
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let template = loader.load_template(TemplateKind::Ppt).unwrap();
        assert_eq!(template.kind(), TemplateKind::Ppt);
        assert_eq!(template.content(), content.as_slice());
    }

    #[test]
    fn test_load_all_with_complete_config() {
        let dir = tempfile::tempdir().unwrap();
        let cr_path = fixtures_path().join("cr-test.md");
        let anomaly_path = fixtures_path().join("anomaly-test.md");
        let pptx_path = dir.path().join("test.pptx");
        fs::write(&pptx_path, create_valid_pptx_bytes()).unwrap();

        let config = TemplatesConfig {
            cr: Some(cr_path.display().to_string()),
            ppt: Some(pptx_path.display().to_string()),
            anomaly: Some(anomaly_path.display().to_string()),
        };
        let loader = TemplateLoader::new(&config);
        let templates = loader.load_all().unwrap();
        assert_eq!(templates.len(), 3);
        assert!(templates.contains_key(&TemplateKind::Cr));
        assert!(templates.contains_key(&TemplateKind::Ppt));
        assert!(templates.contains_key(&TemplateKind::Anomaly));
    }

    #[test]
    fn test_load_all_with_partial_config() {
        let cr_path = fixtures_path().join("cr-test.md");
        let config = TemplatesConfig {
            cr: Some(cr_path.display().to_string()),
            ppt: None,
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let templates = loader.load_all().unwrap();
        assert_eq!(templates.len(), 1);
        assert!(templates.contains_key(&TemplateKind::Cr));
    }

    #[test]
    fn test_load_all_with_empty_config() {
        let config = TemplatesConfig {
            cr: None,
            ppt: None,
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let templates = loader.load_all().unwrap();
        assert!(templates.is_empty());
    }

    // =========================================================================
    // Task 3: Gestion des erreurs — AC #2
    // =========================================================================

    #[test]
    fn test_load_template_not_configured_has_hint() {
        let config = TemplatesConfig {
            cr: None,
            ppt: None,
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let err = loader.load_template(TemplateKind::Cr).unwrap_err();
        assert!(matches!(err, TemplateError::NotConfigured { .. }));
        assert!(err.to_string().contains("config.yaml"));
    }

    #[test]
    fn test_load_template_not_configured_ppt_has_hint() {
        let config = TemplatesConfig {
            cr: None,
            ppt: None,
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let err = loader.load_template(TemplateKind::Ppt).unwrap_err();
        assert!(matches!(err, TemplateError::NotConfigured { .. }));
        assert!(err.to_string().contains("config.yaml"));
        assert!(err.to_string().contains("ppt"));
    }

    #[test]
    fn test_load_template_not_configured_anomaly_has_hint() {
        let config = TemplatesConfig {
            cr: None,
            ppt: None,
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let err = loader.load_template(TemplateKind::Anomaly).unwrap_err();
        assert!(matches!(err, TemplateError::NotConfigured { .. }));
        assert!(err.to_string().contains("config.yaml"));
        assert!(err.to_string().contains("anomaly"));
    }

    #[test]
    fn test_load_template_file_not_found_has_hint() {
        let config = TemplatesConfig {
            cr: Some("/nonexistent/path/cr.md".to_string()),
            ppt: None,
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let err = loader.load_template(TemplateKind::Cr).unwrap_err();
        assert!(matches!(err, TemplateError::FileNotFound { .. }));
        assert!(err.to_string().contains("Check the path"));
    }

    #[test]
    fn test_load_template_invalid_extension() {
        let dir = tempfile::tempdir().unwrap();
        let wrong_ext = dir.path().join("template.txt");
        fs::write(&wrong_ext, "some content").unwrap();

        let config = TemplatesConfig {
            cr: Some(wrong_ext.display().to_string()),
            ppt: None,
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let err = loader.load_template(TemplateKind::Cr).unwrap_err();
        assert!(matches!(err, TemplateError::InvalidExtension { .. }));
        assert!(err.to_string().contains("Rename the file"));
        assert!(err.to_string().contains(".md"));
    }

    #[test]
    fn test_load_template_invalid_pptx_extension() {
        let dir = tempfile::tempdir().unwrap();
        let wrong_ext = dir.path().join("template.ppt");
        fs::write(&wrong_ext, create_valid_pptx_bytes()).unwrap();

        let config = TemplatesConfig {
            cr: None,
            ppt: Some(wrong_ext.display().to_string()),
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let err = loader.load_template(TemplateKind::Ppt).unwrap_err();
        assert!(matches!(err, TemplateError::InvalidExtension { .. }));
        assert!(err.to_string().contains(".pptx"));
    }

    // =========================================================================
    // Task 4: Sécurité des logs — AC #3
    // =========================================================================

    #[test]
    fn test_debug_does_not_expose_template_content() {
        let template = LoadedTemplate {
            kind: TemplateKind::Cr,
            path: PathBuf::from("test.md"),
            content: b"This is secret template content that should not appear in debug".to_vec(),
        };
        let debug_str = format!("{:?}", template);
        assert!(debug_str.contains("Cr"));
        assert!(debug_str.contains("test.md"));
        assert!(debug_str.contains("size_bytes: 63"));
        assert!(!debug_str.contains("secret template content"));
        assert!(!debug_str.contains("should not appear"));
    }

    #[test]
    fn test_error_messages_do_not_contain_content() {
        let err = TemplateError::InvalidFormat {
            path: "test.md".to_string(),
            kind: TemplateKind::Cr,
            cause: "not valid UTF-8 text".to_string(),
            hint: "Check the file".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("test.md"));
        assert!(msg.contains("cr"));
        // Error should not contain raw template bytes
        assert!(!msg.contains("\x00"));
    }

    // =========================================================================
    // Task 5: Validation de format — AC #1
    // =========================================================================

    #[test]
    fn test_validate_markdown_valid() {
        let content = b"# Hello World\n\nThis is valid markdown.";
        assert!(validate_content(TemplateKind::Cr, content, Path::new("test.md")).is_ok());
    }

    #[test]
    fn test_validate_markdown_empty_rejected() {
        let err = validate_content(TemplateKind::Cr, b"", Path::new("empty.md")).unwrap_err();
        assert!(matches!(err, TemplateError::InvalidFormat { .. }));
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn test_validate_markdown_binary_rejected() {
        let content: &[u8] = &[0x00, 0x01, 0x02, 0x80, 0x81, 0xFF];
        let err = validate_content(TemplateKind::Cr, content, Path::new("binary.md")).unwrap_err();
        assert!(matches!(err, TemplateError::InvalidFormat { .. }));
        assert!(err.to_string().contains("UTF-8"));
    }

    #[test]
    fn test_validate_pptx_valid() {
        let content = create_valid_pptx_bytes();
        assert!(validate_content(TemplateKind::Ppt, &content, Path::new("test.pptx")).is_ok());
    }

    #[test]
    fn test_validate_pptx_empty_rejected() {
        let err = validate_content(TemplateKind::Ppt, b"", Path::new("empty.pptx")).unwrap_err();
        assert!(matches!(err, TemplateError::InvalidFormat { .. }));
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn test_validate_pptx_no_magic_rejected() {
        let content = b"This is just text, not a ZIP file";
        let err = validate_content(TemplateKind::Ppt, content, Path::new("fake.pptx")).unwrap_err();
        assert!(matches!(err, TemplateError::InvalidFormat { .. }));
        assert!(err.to_string().contains("ZIP"));
    }

    #[test]
    fn test_validate_pptx_too_small_rejected() {
        // Has magic bytes but too small
        let mut content = Vec::new();
        content.extend_from_slice(b"PK\x03\x04");
        content.resize(50, 0x00); // Below MIN_PPTX_SIZE
        let err = validate_content(TemplateKind::Ppt, &content, Path::new("small.pptx")).unwrap_err();
        assert!(matches!(err, TemplateError::InvalidFormat { .. }));
        assert!(err.to_string().contains("too small"));
    }

    #[test]
    fn test_validate_pptx_boundary_at_min_size_minus_one_rejected() {
        // Exactly MIN_PPTX_SIZE - 1 bytes: should be rejected
        let mut content = Vec::new();
        content.extend_from_slice(b"PK\x03\x04");
        content.resize(MIN_PPTX_SIZE - 1, 0x00);
        let err = validate_content(TemplateKind::Ppt, &content, Path::new("boundary.pptx")).unwrap_err();
        assert!(matches!(err, TemplateError::InvalidFormat { .. }));
        assert!(err.to_string().contains("too small"));
    }

    #[test]
    fn test_validate_pptx_boundary_at_min_size_accepted() {
        // Exactly MIN_PPTX_SIZE bytes: should be accepted
        let mut content = Vec::new();
        content.extend_from_slice(b"PK\x03\x04");
        content.resize(MIN_PPTX_SIZE, 0x00);
        assert!(validate_content(TemplateKind::Ppt, &content, Path::new("boundary.pptx")).is_ok());
    }

    // =========================================================================
    // Task 6: Tests d'intégration avec fixtures — AC #1, #2, #3
    // =========================================================================

    #[test]
    fn test_load_empty_markdown_rejected() {
        let empty_path = fixtures_path().join("empty.md");
        let config = TemplatesConfig {
            cr: Some(empty_path.display().to_string()),
            ppt: None,
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let err = loader.load_template(TemplateKind::Cr).unwrap_err();
        assert!(matches!(err, TemplateError::InvalidFormat { .. }));
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn test_load_binary_as_markdown_rejected() {
        let binary_path = fixtures_path().join("binary-garbage.md");
        let config = TemplatesConfig {
            cr: Some(binary_path.display().to_string()),
            ppt: None,
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let err = loader.load_template(TemplateKind::Cr).unwrap_err();
        assert!(matches!(err, TemplateError::InvalidFormat { .. }));
        assert!(err.to_string().contains("UTF-8"));
    }

    #[test]
    fn test_load_text_as_pptx_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let text_path = dir.path().join("fake.pptx");
        fs::write(&text_path, "This is plain text, not a pptx").unwrap();

        let config = TemplatesConfig {
            cr: None,
            ppt: Some(text_path.display().to_string()),
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let err = loader.load_template(TemplateKind::Ppt).unwrap_err();
        assert!(matches!(err, TemplateError::InvalidFormat { .. }));
        assert!(err.to_string().contains("ZIP"));
    }

    #[test]
    fn test_template_kind_display() {
        assert_eq!(TemplateKind::Cr.to_string(), "cr");
        assert_eq!(TemplateKind::Ppt.to_string(), "ppt");
        assert_eq!(TemplateKind::Anomaly.to_string(), "anomaly");
    }

    #[test]
    fn test_content_as_str_for_markdown_template() {
        let cr_path = fixtures_path().join("cr-test.md");
        let config = TemplatesConfig {
            cr: Some(cr_path.display().to_string()),
            ppt: None,
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let template = loader.load_template(TemplateKind::Cr).unwrap();
        let text = template.content_as_str().unwrap();
        assert!(text.contains("Compte-Rendu"));
    }

    #[test]
    fn test_content_as_str_for_binary_template() {
        let dir = tempfile::tempdir().unwrap();
        let pptx_path = dir.path().join("test.pptx");
        fs::write(&pptx_path, create_valid_pptx_bytes()).unwrap();

        let config = TemplatesConfig {
            cr: None,
            ppt: Some(pptx_path.display().to_string()),
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let template = loader.load_template(TemplateKind::Ppt).unwrap();
        // Binary content should fail UTF-8 conversion with BinaryContent variant
        let err = template.content_as_str().unwrap_err();
        assert!(matches!(err, TemplateError::BinaryContent { .. }));
        assert!(err.to_string().contains("use content() for raw bytes instead"));
    }

    #[test]
    fn test_load_all_fails_on_invalid_template() {
        let cr_path = fixtures_path().join("cr-test.md");
        let config = TemplatesConfig {
            cr: Some(cr_path.display().to_string()),
            ppt: Some("/nonexistent/template.pptx".to_string()),
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let result = loader.load_all();
        assert!(matches!(
            result.unwrap_err(),
            TemplateError::FileNotFound { .. }
        ));
    }

    // =========================================================================
    // Round 3 Review: Case-insensitive extension validation
    // =========================================================================

    #[test]
    fn test_load_template_uppercase_md_extension_accepted() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("template.MD");
        fs::write(&path, "# Valid Markdown").unwrap();

        let config = TemplatesConfig {
            cr: Some(path.display().to_string()),
            ppt: None,
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let template = loader.load_template(TemplateKind::Cr).unwrap();
        assert_eq!(template.kind(), TemplateKind::Cr);
    }

    #[test]
    fn test_load_template_mixed_case_md_extension_accepted() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("template.Md");
        fs::write(&path, "# Valid Markdown").unwrap();

        let config = TemplatesConfig {
            cr: Some(path.display().to_string()),
            ppt: None,
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let template = loader.load_template(TemplateKind::Cr).unwrap();
        assert_eq!(template.kind(), TemplateKind::Cr);
    }

    #[test]
    fn test_load_template_uppercase_pptx_extension_accepted() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("template.PPTX");
        fs::write(&path, create_valid_pptx_bytes()).unwrap();

        let config = TemplatesConfig {
            cr: None,
            ppt: Some(path.display().to_string()),
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let template = loader.load_template(TemplateKind::Ppt).unwrap();
        assert_eq!(template.kind(), TemplateKind::Ppt);
    }

    // =========================================================================
    // Round 4 Review: File size guard
    // =========================================================================

    #[test]
    fn test_load_template_oversized_md_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("huge.md");
        // Create a file that exceeds MAX_MD_SIZE (10 MB)
        // We use fs::File and set_len to create a sparse file without allocating memory
        let file = fs::File::create(&path).unwrap();
        file.set_len(MAX_MD_SIZE + 1).unwrap();

        let config = TemplatesConfig {
            cr: Some(path.display().to_string()),
            ppt: None,
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let err = loader.load_template(TemplateKind::Cr).unwrap_err();
        assert!(matches!(err, TemplateError::InvalidFormat { .. }));
        assert!(err.to_string().contains("too large"));
    }

    #[test]
    fn test_load_template_oversized_pptx_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("huge.pptx");
        let file = fs::File::create(&path).unwrap();
        file.set_len(MAX_PPTX_SIZE + 1).unwrap();

        let config = TemplatesConfig {
            cr: None,
            ppt: Some(path.display().to_string()),
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let err = loader.load_template(TemplateKind::Ppt).unwrap_err();
        assert!(matches!(err, TemplateError::InvalidFormat { .. }));
        assert!(err.to_string().contains("too large"));
    }

    // =========================================================================
    // Round 4 Review: Serde representation alignment
    // =========================================================================

    #[test]
    fn test_template_kind_serde_lowercase() {
        // Serialize should produce lowercase matching Display output
        let cr_json = serde_json::to_string(&TemplateKind::Cr).unwrap();
        assert_eq!(cr_json, "\"cr\"");

        let ppt_json = serde_json::to_string(&TemplateKind::Ppt).unwrap();
        assert_eq!(ppt_json, "\"ppt\"");

        let anomaly_json = serde_json::to_string(&TemplateKind::Anomaly).unwrap();
        assert_eq!(anomaly_json, "\"anomaly\"");

        // Deserialize should accept lowercase
        let cr: TemplateKind = serde_json::from_str("\"cr\"").unwrap();
        assert_eq!(cr, TemplateKind::Cr);
    }

    // =========================================================================
    // Round 3 Review: Directory-as-path edge case
    // =========================================================================

    #[test]
    fn test_load_template_directory_as_path_gives_meaningful_error() {
        let dir = tempfile::tempdir().unwrap();
        // Create a directory with .md extension
        let dir_path = dir.path().join("fake-template.md");
        fs::create_dir(&dir_path).unwrap();

        let config = TemplatesConfig {
            cr: Some(dir_path.display().to_string()),
            ppt: None,
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let err = loader.load_template(TemplateKind::Cr).unwrap_err();
        // Should produce a ReadError (not FileNotFound) since the path exists but is a directory
        assert!(matches!(err, TemplateError::ReadError { .. }));
        assert!(err.to_string().contains("directory"));
    }

    // =========================================================================
    // Round 5 Review: Whitespace-only markdown rejection
    // =========================================================================

    #[test]
    fn test_validate_markdown_whitespace_only_rejected() {
        let content = b"   \n\t\n   \n";
        let err = validate_content(TemplateKind::Cr, content, Path::new("whitespace.md")).unwrap_err();
        assert!(matches!(err, TemplateError::InvalidFormat { .. }));
        assert!(err.to_string().contains("whitespace"));
    }

    #[test]
    fn test_load_whitespace_only_markdown_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("whitespace.md");
        fs::write(&path, "   \n\t\n   \n").unwrap();

        let config = TemplatesConfig {
            cr: Some(path.display().to_string()),
            ppt: None,
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let err = loader.load_template(TemplateKind::Cr).unwrap_err();
        assert!(matches!(err, TemplateError::InvalidFormat { .. }));
        assert!(err.to_string().contains("whitespace"));
    }

    // =========================================================================
    // Round 5 Review: LoadedTemplate test constructor
    // =========================================================================

    #[test]
    fn test_loaded_template_new_for_test() {
        let template = LoadedTemplate::new_for_test(
            TemplateKind::Cr,
            "test/path.md",
            b"# Test content".to_vec(),
        );
        assert_eq!(template.kind(), TemplateKind::Cr);
        assert_eq!(template.path(), Path::new("test/path.md"));
        assert_eq!(template.content(), b"# Test content");
        assert_eq!(template.size_bytes(), 14);
        assert!(template.content_as_str().is_ok());
    }

    // =========================================================================
    // Round 6 Review: TemplateError Clone, BinaryContent variant, type-safe kind
    // =========================================================================

    #[test]
    fn test_template_error_is_clone() {
        let err = TemplateError::NotConfigured {
            kind: TemplateKind::Cr,
            hint: "test hint".to_string(),
        };
        let cloned = err.clone();
        assert_eq!(err.to_string(), cloned.to_string());
    }

    #[test]
    fn test_template_error_kind_is_type_safe() {
        let config = TemplatesConfig {
            cr: None,
            ppt: None,
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let err = loader.load_template(TemplateKind::Ppt).unwrap_err();
        // Can match on TemplateKind directly instead of comparing strings
        match err {
            TemplateError::NotConfigured { kind, .. } => {
                assert_eq!(kind, TemplateKind::Ppt);
            }
            _ => panic!("Expected NotConfigured"),
        }
    }

    #[test]
    fn test_content_as_str_non_utf8_markdown_returns_invalid_format() {
        let template = LoadedTemplate::new_for_test(
            TemplateKind::Cr,
            "test.md",
            vec![0xFF, 0xFE, 0x80, 0x81],
        );
        let err = template.content_as_str().unwrap_err();
        match err {
            TemplateError::InvalidFormat { kind, cause, .. } => {
                assert_eq!(kind, TemplateKind::Cr);
                assert!(cause.contains("UTF-8"));
            }
            _ => panic!("Expected InvalidFormat for non-UTF-8 markdown, got {:?}", err),
        }
    }

    #[test]
    fn test_binary_content_variant_for_pptx() {
        let template = LoadedTemplate::new_for_test(
            TemplateKind::Ppt,
            "test.pptx",
            vec![0xFF; 100],
        );
        let err = template.content_as_str().unwrap_err();
        match err {
            TemplateError::BinaryContent { kind, hint, .. } => {
                assert_eq!(kind, TemplateKind::Ppt);
                assert!(hint.contains("use content() for raw bytes instead"));
            }
            _ => panic!("Expected BinaryContent, got {:?}", err),
        }
    }

    // =========================================================================
    // Round 7 Review: No-extension edge case
    // =========================================================================

    #[test]
    fn test_load_template_no_extension_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("README");
        fs::write(&path, "# Some content").unwrap();

        let config = TemplatesConfig {
            cr: Some(path.display().to_string()),
            ppt: None,
            anomaly: None,
        };
        let loader = TemplateLoader::new(&config);
        let err = loader.load_template(TemplateKind::Cr).unwrap_err();
        assert!(matches!(err, TemplateError::InvalidExtension { .. }));
        match &err {
            TemplateError::InvalidExtension { actual, .. } => {
                assert_eq!(actual, "(none)");
            }
            _ => panic!("Expected InvalidExtension"),
        }
        assert!(err.to_string().contains("(none)"));
    }

    #[test]
    fn test_validate_extension_as_free_function() {
        // validate_extension is now a free function, not a method on TemplateLoader
        assert!(validate_extension(Path::new("test.md"), TemplateKind::Cr).is_ok());
        assert!(validate_extension(Path::new("test.MD"), TemplateKind::Cr).is_ok());
        assert!(validate_extension(Path::new("test.pptx"), TemplateKind::Ppt).is_ok());
        assert!(validate_extension(Path::new("test.txt"), TemplateKind::Cr).is_err());
    }
}

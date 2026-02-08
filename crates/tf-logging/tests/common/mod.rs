//! Shared test utilities for tf-logging integration tests.

use std::fs;
use std::path::{Path, PathBuf};

/// Find the first log file in a directory.
///
/// tracing-appender creates files with date-based names (e.g., "app.log.2026-02-06"),
/// so we search for any file in the directory rather than a fixed name.
pub fn find_log_file(log_dir: &Path) -> PathBuf {
    fs::read_dir(log_dir)
        .expect("Failed to read log directory")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.is_file())
        .unwrap_or_else(|| panic!("No log file found in {}", log_dir.display()))
}

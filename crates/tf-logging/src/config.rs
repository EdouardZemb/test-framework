//! Logging configuration derived from project settings.

use tf_config::ProjectConfig;

/// Configuration for the logging subsystem.
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error). Default: "info"
    pub log_level: String,
    /// Directory for log files. Default: "{output_folder}/logs"
    pub log_dir: String,
    /// Also output logs to stdout (for interactive mode)
    pub log_to_stdout: bool,
}

impl LoggingConfig {
    /// Derive logging config from project configuration.
    ///
    /// - `log_dir` = `"{output_folder}/logs"`, fallback to `"./logs"` if output_folder is empty
    /// - `log_level` defaults to `"info"`
    /// - `log_to_stdout` defaults to `false`
    pub fn from_project_config(config: &ProjectConfig) -> Self {
        let log_dir = if config.output_folder.is_empty() {
            "./logs".to_string()
        } else {
            std::path::Path::new(&config.output_folder)
                .join("logs")
                .to_string_lossy()
                .to_string()
        };

        Self {
            log_level: "info".to_string(),
            log_dir,
            log_to_stdout: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    // Test 0.5-UNIT-010: LoggingConfig::from_project_config derives correctly with fallback
    #[test]
    fn test_logging_config_from_project_config_derives_log_dir() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("config.yaml");
        let mut file = fs::File::create(&config_path).unwrap();
        file.write_all(b"project_name: \"test-project\"\noutput_folder: \"/tmp/test-output\"\n").unwrap();
        file.flush().unwrap();

        let project_config = tf_config::load_config(&config_path).unwrap();
        let logging_config = LoggingConfig::from_project_config(&project_config);

        // log_dir should be derived from output_folder
        assert_eq!(logging_config.log_dir, "/tmp/test-output/logs");
        assert_eq!(logging_config.log_level, "info");
        assert!(!logging_config.log_to_stdout);
    }

    // Test [AI-Review-R2 M3]: trailing slash in output_folder should not produce double-slash
    #[test]
    fn test_logging_config_no_double_slash_with_trailing_slash() {
        let yaml = "project_name: \"test-project\"\noutput_folder: \"/tmp/test-output/\"\n";
        let project_config: tf_config::ProjectConfig = serde_yaml::from_str(yaml).unwrap();
        let logging_config = LoggingConfig::from_project_config(&project_config);
        assert_eq!(logging_config.log_dir, "/tmp/test-output/logs");
        assert!(!logging_config.log_dir.contains("//"),
                "log_dir should not contain double slashes, got: {}", logging_config.log_dir);
    }

    #[test]
    fn test_logging_config_fallback_when_output_folder_empty() {
        // Construct a ProjectConfig directly (bypassing load_config validation)
        // to test the defensive fallback in from_project_config
        let yaml = "project_name: \"test-project\"\noutput_folder: \"placeholder\"\n";
        let mut project_config: tf_config::ProjectConfig = serde_yaml::from_str(yaml).unwrap();
        // Manually set output_folder to empty to test fallback
        project_config.output_folder = String::new();

        let logging_config = LoggingConfig::from_project_config(&project_config);

        // Should fallback to "./logs" when output_folder is empty
        assert_eq!(logging_config.log_dir, "./logs");
    }
}

//! Integration tests for configuration profiles (Story 0-2)
//!
//! These tests verify the profile selection and merging functionality.
//!
//! # Acceptance Criteria Coverage
//!
//! - AC #1: Select valid profile -> config merged and displayed
//!   - `test_load_config_with_valid_profile`
//!   - `test_profile_merge_overrides_output_folder`
//!
//! - AC #2: Unknown profile -> error with available profiles list
//!   - `test_unknown_profile_lists_available`
//!   - `test_with_profile_on_config_without_profiles_section`
//!
//! - AC #3: Logs don't contain sensitive data after profile applied
//!   - `test_merged_config_redacts_profile_secrets`
//!
//! # Review Follow-up Round 2 Coverage
//!
//! - `test_with_profile_on_config_without_profiles_section` - [HIGH] profiles: None edge case
//! - `test_with_profile_rejects_invalid_jira_url_after_merge` - [HIGH] Invalid URL validation

use std::path::PathBuf;
use tf_config::{load_config, ConfigError};

/// Returns the path to a test fixture file, using CARGO_MANIFEST_DIR for robustness.
fn fixture_path(name: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("tests")
        .join("fixtures")
        .join(name)
}

/// AC #1: Load configuration with a valid profile name
///
/// When a valid profile is selected:
/// - Configuration loads successfully
/// - The profile field values are merged with base config
/// - The active_profile field reflects the selected profile
#[test]
fn test_load_config_with_valid_profile() {
    let config = load_config(&fixture_path("config_with_profiles.yaml")).unwrap();

    // Apply the "dev" profile
    let merged = config.with_profile("dev").unwrap();

    // Verify the profile was applied
    assert_eq!(merged.active_profile, Some("dev".to_string()));

    // Verify the profiles map is available
    assert!(merged.profiles.is_some());
    let profiles = merged.profiles.as_ref().unwrap();
    assert!(profiles.contains_key("dev"));
    assert!(profiles.contains_key("staging"));
    assert!(profiles.contains_key("prod"));
}

/// AC #1: Profile merge correctly overrides output_folder
///
/// The dev profile overrides output_folder from "./base-output" to "./dev-output"
#[test]
fn test_profile_merge_overrides_output_folder() {
    let config = load_config(&fixture_path("config_with_profiles.yaml")).unwrap();

    // Base config should have base output folder
    assert_eq!(config.output_folder, "./base-output");

    // After applying dev profile, output_folder should be overridden
    let merged = config.with_profile("dev").unwrap();
    assert_eq!(merged.output_folder, "./dev-output");

    // Jira endpoint should also be overridden by dev profile
    let jira = merged.jira.as_ref().expect("jira should be present");
    assert_eq!(jira.endpoint, "https://jira.dev.example.com");

    // Squash should remain from base config (not overridden by dev profile)
    let squash = merged.squash.as_ref().expect("squash should be present");
    assert_eq!(squash.endpoint, "https://squash.base.example.com");
}

/// AC #2: Unknown profile returns error with list of available profiles
///
/// When an unknown profile is requested:
/// - A ProfileNotFound error is returned
/// - The error message includes the requested profile name
/// - The error message lists all available profiles
#[test]
fn test_unknown_profile_lists_available() {
    let config = load_config(&fixture_path("config_with_profiles.yaml")).unwrap();

    // Try to apply a non-existent profile
    let result = config.with_profile("nonexistent");

    assert!(result.is_err());
    let err = result.unwrap_err();

    // Verify we get the ProfileNotFound variant
    match &err {
        ConfigError::ProfileNotFound {
            requested,
            available,
        } => {
            assert_eq!(requested, "nonexistent");
            // Available should contain all defined profiles
            assert!(available.contains(&"dev".to_string()));
            assert!(available.contains(&"staging".to_string()));
            assert!(available.contains(&"prod".to_string()));
        }
        other => panic!("Expected ProfileNotFound, got: {:?}", other),
    }

    // Verify the error message is user-friendly
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("nonexistent"),
        "Error should mention requested profile"
    );
    assert!(
        err_msg.contains("dev") && err_msg.contains("staging") && err_msg.contains("prod"),
        "Error should list available profiles"
    );
}

/// AC #3: Secrets from profile overrides are redacted in debug/display output
///
/// When a profile is applied that overrides secrets:
/// - The merged config's Debug output must not contain the secret values
/// - Both base secrets and profile-overridden secrets must be redacted
#[test]
fn test_merged_config_redacts_profile_secrets() {
    let config = load_config(&fixture_path("config_profile_override_jira.yaml")).unwrap();

    // Apply the profile that overrides jira token
    let merged = config.with_profile("override_secrets").unwrap();

    // Get debug output of merged config
    let debug_output = format!("{:?}", merged);

    // Verify neither the base token nor the override token appears in debug output
    assert!(
        !debug_output.contains("base-jira-token-12345"),
        "Base jira token should be redacted, found in: {}",
        debug_output
    );
    assert!(
        !debug_output.contains("override-jira-token-67890"),
        "Override jira token should be redacted, found in: {}",
        debug_output
    );

    // Verify [REDACTED] placeholder is present
    assert!(
        debug_output.contains("[REDACTED]"),
        "Debug output should contain [REDACTED] placeholder"
    );

    // But the actual override token should be accessible programmatically
    let jira = merged.jira.as_ref().unwrap();
    assert_eq!(
        jira.token.as_ref().unwrap(),
        "override-jira-token-67890",
        "Token should be accessible programmatically after profile merge"
    );
}

/// Test that staging profile correctly overrides both jira and squash
///
/// This is an additional test for AC #1 to verify complex merging scenarios
#[test]
fn test_staging_profile_overrides_multiple_sections() {
    let config = load_config(&fixture_path("config_with_profiles.yaml")).unwrap();

    let merged = config.with_profile("staging").unwrap();

    // Jira should be overridden
    let jira = merged.jira.as_ref().expect("jira should be present");
    assert_eq!(jira.endpoint, "https://jira.staging.example.com");

    // Squash should also be overridden (staging profile overrides both)
    let squash = merged.squash.as_ref().expect("squash should be present");
    assert_eq!(squash.endpoint, "https://squash.staging.example.com");
    assert_eq!(squash.username.as_ref().unwrap(), "staginguser");

    // output_folder should remain from base (staging doesn't override it)
    assert_eq!(merged.output_folder, "./base-output");
}

/// Test that prod profile (empty) uses all base values
///
/// An empty profile should not change any values from the base config
#[test]
fn test_empty_prod_profile_uses_base_values() {
    let config = load_config(&fixture_path("config_with_profiles.yaml")).unwrap();

    let merged = config.with_profile("prod").unwrap();

    // All values should match the base config
    assert_eq!(merged.output_folder, "./base-output");

    let jira = merged.jira.as_ref().expect("jira should be present");
    assert_eq!(jira.endpoint, "https://jira.base.example.com");

    let squash = merged.squash.as_ref().expect("squash should be present");
    assert_eq!(squash.endpoint, "https://squash.base.example.com");

    // active_profile should still reflect "prod" was selected
    assert_eq!(merged.active_profile, Some("prod".to_string()));
}

/// Test loading config without applying any profile
///
/// When no profile is applied, the config should work as before (backward compatibility)
#[test]
fn test_config_without_profile_is_backward_compatible() {
    let config = load_config(&fixture_path("config_with_profiles.yaml")).unwrap();

    // Without calling with_profile(), the config should still be usable
    assert_eq!(config.project_name, "profiles-test-project");
    assert_eq!(config.output_folder, "./base-output");

    // active_profile should be None when no profile is applied
    assert!(config.active_profile.is_none());

    // profiles field should be populated from YAML
    assert!(config.profiles.is_some());
}

// =============================================================================
// Tests for validation after merge (Review Follow-up HIGH)
// =============================================================================

/// AC #1 + Review Follow-up: with_profile() validates merged configuration
///
/// A profile that overrides output_folder with path traversal should fail
/// when with_profile() is called, because validate_config is called after merge.
#[test]
fn test_with_profile_rejects_path_traversal_in_output_folder() {
    let config = load_config(&fixture_path("config_profile_invalid_path.yaml")).unwrap();

    // The base config is valid
    assert_eq!(config.output_folder, "./valid-output");

    // Applying the "evil" profile should fail because it sets output_folder to "../../../etc/passwd"
    let result = config.with_profile("evil");

    assert!(
        result.is_err(),
        "with_profile should reject path traversal in output_folder"
    );

    let err = result.unwrap_err();
    let err_msg = err.to_string();

    // Error should mention output_folder and path traversal
    assert!(
        err_msg.contains("output_folder")
            || err_msg.contains("path traversal")
            || err_msg.contains(".."),
        "Error should mention output_folder or path traversal, got: {}",
        err_msg
    );
}

/// with_profile() should also reject empty output_folder from profile
#[test]
fn test_with_profile_rejects_empty_output_folder() {
    let config = load_config(&fixture_path("config_profile_invalid_path.yaml")).unwrap();

    // Applying the "empty_path" profile should fail
    let result = config.with_profile("empty_path");

    assert!(
        result.is_err(),
        "with_profile should reject empty output_folder"
    );

    let err = result.unwrap_err();
    let err_msg = err.to_string();

    // Error should mention output_folder
    assert!(
        err_msg.contains("output_folder")
            || err_msg.contains("empty")
            || err_msg.contains("invalid"),
        "Error should mention output_folder issue, got: {}",
        err_msg
    );
}

// =============================================================================
// Review Follow-up Round 2 - HIGH Priority Tests
// =============================================================================

/// AC #2 Edge Case: with_profile() on config without profiles section (profiles: None)
///
/// When a config has no profiles section at all, calling with_profile() should return
/// ProfileNotFound with a helpful message indicating no profiles are defined.
#[test]
fn test_with_profile_on_config_without_profiles_section() {
    // minimal_config.yaml has no profiles section
    let config = load_config(&fixture_path("minimal_config.yaml")).unwrap();

    // Verify the config has no profiles
    assert!(
        config.profiles.is_none(),
        "minimal_config should have no profiles section"
    );

    // Try to apply any profile
    let result = config.with_profile("dev");

    // Should return ProfileNotFound error
    assert!(
        result.is_err(),
        "with_profile on config without profiles should fail"
    );

    let err = result.unwrap_err();
    match &err {
        ConfigError::ProfileNotFound {
            requested,
            available,
        } => {
            assert_eq!(requested, "dev");
            // Available should be empty since no profiles defined
            assert!(
                available.is_empty(),
                "Available profiles should be empty when no profiles section exists"
            );
        }
        other => panic!("Expected ProfileNotFound, got: {:?}", other),
    }

    // Error message should be user-friendly even with empty available list
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("dev"),
        "Error should mention the requested profile, got: {}",
        err_msg
    );
}

/// Validation after merge: with_profile() should reject invalid Jira URL from profile
///
/// When a profile overrides jira.endpoint with an invalid URL, validate_config()
/// should reject it after the merge.
#[test]
fn test_with_profile_rejects_invalid_jira_url_after_merge() {
    let config = load_config(&fixture_path("config_profile_invalid_jira.yaml")).unwrap();

    // Base config has a valid Jira URL
    let base_jira = config.jira.as_ref().expect("base config should have jira");
    assert_eq!(base_jira.endpoint, "https://jira.valid.example.com");

    // Applying the "invalid_jira_url" profile should fail validation
    let result = config.with_profile("invalid_jira_url");

    assert!(
        result.is_err(),
        "with_profile should reject invalid Jira URL from profile"
    );

    let err = result.unwrap_err();
    let err_msg = err.to_string();

    // Error should mention jira endpoint or URL validation failure
    assert!(
        err_msg.contains("jira")
            || err_msg.contains("endpoint")
            || err_msg.contains("URL")
            || err_msg.contains("url"),
        "Error should mention jira/endpoint/URL issue, got: {}",
        err_msg
    );
}

// =============================================================================
// Review Follow-up Round 3 - MEDIUM Priority Tests
// =============================================================================

/// Validation after merge: with_profile() should reject invalid LlmConfig from profile
///
/// When a profile overrides llm with mode=cloud but cloud_enabled=false, this is
/// an invalid configuration that should be rejected by validate_config() after merge.
#[test]
fn test_with_profile_rejects_invalid_llm_config_after_merge() {
    let config = load_config(&fixture_path("config_profile_invalid_llm.yaml")).unwrap();

    // Base config has a valid LLM config (mode: auto, cloud_enabled: false)
    let base_llm = config.llm.as_ref().expect("base config should have llm");
    assert!(
        !base_llm.cloud_enabled,
        "base config should have cloud_enabled=false"
    );

    // Applying the "invalid_llm" profile should fail validation
    // because it sets mode=cloud but cloud_enabled=false
    let result = config.with_profile("invalid_llm");

    assert!(
        result.is_err(),
        "with_profile should reject invalid LLM config (mode=cloud, cloud_enabled=false)"
    );

    let err = result.unwrap_err();
    let err_msg = err.to_string();

    // Error should mention the LLM configuration issue
    assert!(
        err_msg.contains("llm") || err_msg.contains("cloud_enabled") || err_msg.contains("mode"),
        "Error should mention llm/cloud_enabled/mode issue, got: {}",
        err_msg
    );
}

/// Sanity check: a valid LLM profile should be accepted
#[test]
fn test_with_profile_accepts_valid_llm_config() {
    let config = load_config(&fixture_path("config_profile_invalid_llm.yaml")).unwrap();

    // Applying the "valid_llm" profile should succeed
    let result = config.with_profile("valid_llm");

    assert!(
        result.is_ok(),
        "with_profile should accept valid LLM config: {:?}",
        result
    );

    let merged = result.unwrap();
    let llm = merged.llm.as_ref().expect("merged config should have llm");
    assert_eq!(
        llm.local_endpoint.as_ref().unwrap(),
        "http://localhost:8080"
    );
}

// =============================================================================
// Review Follow-up Round 4 - MEDIUM Priority Tests
// =============================================================================

/// Test for templates profile override - the only ProfileOverride field without merge test
///
/// When a profile overrides the templates section, the merged config should use
/// the profile's template paths instead of the base config values.
#[test]
fn test_profile_merge_overrides_templates() {
    let config = load_config(&fixture_path("config_with_profiles.yaml")).unwrap();

    // Verify base config has templates (if present) or None
    // Apply the "with_templates" profile that overrides templates
    let merged = config.with_profile("with_templates").unwrap();

    // Templates should be overridden by the profile
    let templates = merged
        .templates
        .as_ref()
        .expect("templates should be present after profile merge");
    assert_eq!(
        templates.cr.as_ref().unwrap(),
        "./templates/dev/cr.md",
        "templates.cr should be overridden by profile"
    );
    assert_eq!(
        templates.ppt.as_ref().unwrap(),
        "./templates/dev/report.pptx",
        "templates.ppt should be overridden by profile"
    );
    assert_eq!(
        templates.anomaly.as_ref().unwrap(),
        "./templates/dev/anomaly.md",
        "templates.anomaly should be overridden by profile"
    );

    // active_profile should reflect the applied profile
    assert_eq!(merged.active_profile, Some("with_templates".to_string()));
}

/// Test for LLM profile override in the main fixture (Review Follow-up Round 4 - LOW)
///
/// When a profile overrides the llm section, the merged config should use
/// the profile's LLM settings instead of the base config values.
#[test]
fn test_profile_merge_overrides_llm_in_main_fixture() {
    let config = load_config(&fixture_path("config_with_profiles.yaml")).unwrap();

    // Base config has mode: auto
    let base_llm = config.llm.as_ref().expect("base config should have llm");
    assert_eq!(format!("{}", base_llm.mode), "auto");

    // Apply the "with_llm" profile that overrides llm
    let merged = config.with_profile("with_llm").unwrap();

    // LLM should be overridden by the profile
    let llm = merged
        .llm
        .as_ref()
        .expect("llm should be present after profile merge");
    assert_eq!(
        format!("{}", llm.mode),
        "local",
        "llm.mode should be overridden by profile"
    );
    assert_eq!(
        llm.local_endpoint.as_ref().unwrap(),
        "http://localhost:8080",
        "llm.local_endpoint should be overridden by profile"
    );
    assert_eq!(
        llm.local_model.as_ref().unwrap(),
        "codellama:7b",
        "llm.local_model should be overridden by profile"
    );
}

// =============================================================================
// Review Follow-up Round 5 - MEDIUM Priority Tests
// =============================================================================

/// Validation after merge: with_profile() should reject templates with path traversal
///
/// When a profile overrides templates with a path containing "..", validate_config()
/// should reject it after the merge.
#[test]
fn test_with_profile_rejects_templates_path_traversal() {
    let config = load_config(&fixture_path("config_profile_invalid_templates.yaml")).unwrap();

    // Base config has valid templates
    let base_templates = config
        .templates
        .as_ref()
        .expect("base config should have templates");
    assert_eq!(base_templates.cr.as_ref().unwrap(), "./templates/cr.md");

    // Applying the "evil_templates" profile should fail validation
    let result = config.with_profile("evil_templates");

    assert!(
        result.is_err(),
        "with_profile should reject templates with path traversal"
    );

    let err = result.unwrap_err();
    let err_msg = err.to_string();

    // Error should mention templates or path traversal
    assert!(
        err_msg.contains("templates")
            || err_msg.contains("cr")
            || err_msg.contains("path traversal")
            || err_msg.contains(".."),
        "Error should mention templates/path traversal issue, got: {}",
        err_msg
    );
}

/// Sanity check: a valid templates profile should be accepted
#[test]
fn test_with_profile_accepts_valid_templates() {
    let config = load_config(&fixture_path("config_profile_invalid_templates.yaml")).unwrap();

    // Applying the "valid_templates" profile should succeed
    let result = config.with_profile("valid_templates");

    assert!(
        result.is_ok(),
        "with_profile should accept valid templates: {:?}",
        result
    );

    let merged = result.unwrap();
    let templates = merged
        .templates
        .as_ref()
        .expect("merged config should have templates");
    assert_eq!(templates.cr.as_ref().unwrap(), "./templates/dev/cr.md");
}

/// Validation after merge: with_profile() should reject invalid Squash URL from profile
///
/// When a profile overrides squash.endpoint with an invalid URL, validate_config()
/// should reject it after the merge.
#[test]
fn test_with_profile_rejects_invalid_squash_url_after_merge() {
    let config = load_config(&fixture_path("config_profile_invalid_squash.yaml")).unwrap();

    // Base config has a valid Squash URL
    let base_squash = config
        .squash
        .as_ref()
        .expect("base config should have squash");
    assert_eq!(base_squash.endpoint, "https://squash.valid.example.com");

    // Applying the "invalid_squash_url" profile should fail validation
    let result = config.with_profile("invalid_squash_url");

    assert!(
        result.is_err(),
        "with_profile should reject invalid Squash URL from profile"
    );

    let err = result.unwrap_err();
    let err_msg = err.to_string();

    // Error should mention squash endpoint or URL validation failure
    assert!(
        err_msg.contains("squash")
            || err_msg.contains("endpoint")
            || err_msg.contains("URL")
            || err_msg.contains("url"),
        "Error should mention squash/endpoint/URL issue, got: {}",
        err_msg
    );
}

/// Sanity check: a valid Squash profile should be accepted
#[test]
fn test_with_profile_accepts_valid_squash_url() {
    let config = load_config(&fixture_path("config_profile_invalid_squash.yaml")).unwrap();

    // Applying the "valid_squash" profile should succeed
    let result = config.with_profile("valid_squash");

    assert!(
        result.is_ok(),
        "with_profile should accept valid Squash config: {:?}",
        result
    );

    let merged = result.unwrap();
    let squash = merged
        .squash
        .as_ref()
        .expect("merged config should have squash");
    assert_eq!(squash.endpoint, "https://squash.staging.example.com");
}

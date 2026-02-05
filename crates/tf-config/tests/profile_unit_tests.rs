//! Unit tests for ProfileOverride (Story 0-2)
//!
//! Story 0-2: Configuration Profiles (Définir et sélectionner des profils de configuration)
//!
//! These tests verify the ProfileOverride struct behavior including:
//! - Default trait implementation
//! - Clone trait implementation
//! - Debug output redaction of sensitive fields
//! - Profile merge logic (partial override pattern)
//! - active_profile_summary() method
//!
//! # Acceptance Criteria Covered:
//! - AC #1: Profile merge logic and summary display
//! - AC #3: ProfileOverride redacts secrets in Debug output

use tf_config::{JiraConfig, LlmConfig, LlmMode, ProjectConfig, SquashConfig, TemplatesConfig};
use tf_config::ProfileOverride;

// =============================================================================
// Test 1: ProfileOverride with jira override redacts token in Debug
// Acceptance Criteria #3: Secrets must be redacted in Debug output
// =============================================================================

/// ProfileOverride containing a JiraConfig with a token should redact
/// the token value when formatted with Debug ({:?}).
///
/// This ensures that if a ProfileOverride is accidentally logged,
/// sensitive credentials are not exposed in logs.
#[test]
fn test_profile_override_redacts_jira_token_in_debug() {
    let profile = ProfileOverride {
        jira: Some(JiraConfig {
            endpoint: "https://jira.dev.example.com".to_string(),
            token: Some("super-secret-jira-token-12345".to_string()),
        }),
        squash: None,
        llm: None,
        templates: None,
        output_folder: None,
    };

    let debug_str = format!("{:?}", profile);

    // The actual secret value must NOT appear in debug output
    assert!(
        !debug_str.contains("super-secret-jira-token-12345"),
        "Jira token should be redacted in Debug output, but found secret in: {}",
        debug_str
    );

    // The [REDACTED] placeholder must be present
    assert!(
        debug_str.contains("[REDACTED]"),
        "Debug output should contain [REDACTED] placeholder, got: {}",
        debug_str
    );
}

/// ProfileOverride containing LlmConfig with an API key should redact
/// the api_key value when formatted with Debug ({:?}).
#[test]
fn test_profile_override_redacts_llm_api_key_in_debug() {
    let profile = ProfileOverride {
        jira: None,
        squash: None,
        llm: Some(LlmConfig {
            mode: LlmMode::Cloud,
            local_endpoint: None,
            local_model: None,
            cloud_enabled: true,
            cloud_endpoint: Some("https://api.openai.com/v1".to_string()),
            cloud_model: Some("gpt-4".to_string()),
            api_key: Some("sk-secret-openai-key-xyz789".to_string()),
            timeout_seconds: 120,
            max_tokens: 4096,
        }),
        templates: None,
        output_folder: None,
    };

    let debug_str = format!("{:?}", profile);

    // The actual API key must NOT appear in debug output
    assert!(
        !debug_str.contains("sk-secret-openai-key-xyz789"),
        "LLM API key should be redacted in Debug output, but found secret in: {}",
        debug_str
    );

    // The [REDACTED] placeholder must be present
    assert!(
        debug_str.contains("[REDACTED]"),
        "Debug output should contain [REDACTED] placeholder, got: {}",
        debug_str
    );
}

/// ProfileOverride containing SquashConfig with a password should redact
/// the password value when formatted with Debug ({:?}).
#[test]
fn test_profile_override_redacts_squash_password_in_debug() {
    let profile = ProfileOverride {
        jira: None,
        squash: Some(SquashConfig {
            endpoint: "https://squash.staging.example.com".to_string(),
            username: Some("testuser".to_string()),
            password: Some("squash-secret-password-abc".to_string()),
        }),
        llm: None,
        templates: None,
        output_folder: None,
    };

    let debug_str = format!("{:?}", profile);

    // The actual password must NOT appear in debug output
    assert!(
        !debug_str.contains("squash-secret-password-abc"),
        "Squash password should be redacted in Debug output, but found secret in: {}",
        debug_str
    );

    // The [REDACTED] placeholder must be present
    assert!(
        debug_str.contains("[REDACTED]"),
        "Debug output should contain [REDACTED] placeholder, got: {}",
        debug_str
    );
}

/// Test ProfileOverride::redacted() trait method directly (not just Debug formatting).
///
/// The Redact trait provides a `.redacted()` method that should produce the same
/// output as `format!("{:?}")`, ensuring consistency between the two approaches.
#[test]
fn test_profile_override_redacted_trait_method() {
    use tf_config::Redact;

    let profile = ProfileOverride {
        jira: Some(JiraConfig {
            endpoint: "https://jira.example.com".to_string(),
            token: Some("super-secret-token-xyz".to_string()),
        }),
        squash: Some(SquashConfig {
            endpoint: "https://squash.example.com".to_string(),
            username: Some("user".to_string()),
            password: Some("secret-password-abc".to_string()),
        }),
        llm: Some(LlmConfig {
            mode: LlmMode::Cloud,
            local_endpoint: None,
            local_model: None,
            cloud_enabled: true,
            cloud_endpoint: Some("https://api.openai.com/v1".to_string()),
            cloud_model: Some("gpt-4".to_string()),
            api_key: Some("sk-api-key-12345".to_string()),
            timeout_seconds: 120,
            max_tokens: 4096,
        }),
        templates: None,
        output_folder: Some("./output".to_string()),
    };

    // Call .redacted() directly
    let redacted_output = profile.redacted();

    // Verify secrets are NOT in the redacted output
    assert!(
        !redacted_output.contains("super-secret-token-xyz"),
        "Jira token should be redacted in .redacted() output"
    );
    assert!(
        !redacted_output.contains("secret-password-abc"),
        "Squash password should be redacted in .redacted() output"
    );
    assert!(
        !redacted_output.contains("sk-api-key-12345"),
        "LLM API key should be redacted in .redacted() output"
    );

    // Verify [REDACTED] placeholder is present
    assert!(
        redacted_output.contains("[REDACTED]"),
        ".redacted() output should contain [REDACTED] placeholder, got: {}",
        redacted_output
    );

    // Verify non-sensitive data IS present
    assert!(
        redacted_output.contains("jira.example.com"),
        "Non-sensitive endpoint should be visible in .redacted() output"
    );
    assert!(
        redacted_output.contains("./output"),
        "Output folder should be visible in .redacted() output"
    );

    // Verify .redacted() produces same output as format!("{:?}")
    let debug_output = format!("{:?}", profile);
    assert_eq!(
        redacted_output, debug_output,
        ".redacted() should produce same output as Debug formatting"
    );
}

// =============================================================================
// Test 2: Empty profile preserves base config
// Acceptance Criteria #1: Profile merge logic
// =============================================================================

/// An empty ProfileOverride (all fields None) should not change any values
/// when applied to a ProjectConfig. This tests the identity property of merge.
#[test]
fn test_empty_profile_preserves_base_config() {
    // Create a fully populated base config
    let base_config = ProjectConfig {
        project_name: "original-project".to_string(),
        output_folder: "./original-output".to_string(),
        jira: Some(JiraConfig {
            endpoint: "https://jira.original.com".to_string(),
            token: Some("original-token".to_string()),
        }),
        squash: Some(SquashConfig {
            endpoint: "https://squash.original.com".to_string(),
            username: Some("original-user".to_string()),
            password: Some("original-pass".to_string()),
        }),
        llm: Some(LlmConfig {
            mode: LlmMode::Local,
            local_endpoint: Some("http://localhost:11434".to_string()),
            local_model: Some("mistral:7b".to_string()),
            cloud_enabled: false,
            cloud_endpoint: None,
            cloud_model: None,
            api_key: None,
            timeout_seconds: 60,
            max_tokens: 2048,
        }),
        templates: Some(TemplatesConfig {
            cr: Some("./templates/cr.md".to_string()),
            ppt: Some("./templates/report.pptx".to_string()),
            anomaly: Some("./templates/anomaly.md".to_string()),
        }),
        profiles: None,
        active_profile: None,
    };

    // Create an empty profile override (all None)
    let empty_profile = ProfileOverride::default();

    // Apply the empty profile to the base config
    let merged = base_config.apply_profile(&empty_profile);

    // All values should be unchanged
    assert_eq!(merged.project_name, "original-project");
    assert_eq!(merged.output_folder, "./original-output");

    let jira = merged.jira.as_ref().expect("jira should be preserved");
    assert_eq!(jira.endpoint, "https://jira.original.com");
    assert_eq!(jira.token.as_ref().unwrap(), "original-token");

    let squash = merged.squash.as_ref().expect("squash should be preserved");
    assert_eq!(squash.endpoint, "https://squash.original.com");
    assert_eq!(squash.username.as_ref().unwrap(), "original-user");

    let llm = merged.llm.as_ref().expect("llm should be preserved");
    assert_eq!(llm.mode, LlmMode::Local);
    assert_eq!(llm.local_endpoint.as_ref().unwrap(), "http://localhost:11434");

    let templates = merged.templates.as_ref().expect("templates should be preserved");
    assert_eq!(templates.cr.as_ref().unwrap(), "./templates/cr.md");
}

// =============================================================================
// Test 3: Partial override only changes specified fields
// Acceptance Criteria #1: Profile merge logic
// =============================================================================

/// A ProfileOverride with only output_folder set should change only that field,
/// leaving all other fields from the base config unchanged.
#[test]
fn test_partial_override_only_changes_specified_fields() {
    // Create a base config with all integrations
    let base_config = ProjectConfig {
        project_name: "my-project".to_string(),
        output_folder: "./output/default".to_string(),
        jira: Some(JiraConfig {
            endpoint: "https://jira.prod.com".to_string(),
            token: Some("prod-token".to_string()),
        }),
        squash: Some(SquashConfig {
            endpoint: "https://squash.prod.com".to_string(),
            username: Some("prod-user".to_string()),
            password: Some("prod-pass".to_string()),
        }),
        llm: Some(LlmConfig {
            mode: LlmMode::Cloud,
            local_endpoint: None,
            local_model: None,
            cloud_enabled: true,
            cloud_endpoint: Some("https://api.openai.com/v1".to_string()),
            cloud_model: Some("gpt-4".to_string()),
            api_key: Some("sk-prod-key".to_string()),
            timeout_seconds: 120,
            max_tokens: 4096,
        }),
        templates: Some(TemplatesConfig {
            cr: Some("./templates/cr.md".to_string()),
            ppt: None,
            anomaly: None,
        }),
        profiles: None,
        active_profile: None,
    };

    // Create a profile that only overrides output_folder
    let partial_profile = ProfileOverride {
        jira: None,
        squash: None,
        llm: None,
        templates: None,
        output_folder: Some("./output/staging".to_string()),
    };

    // Apply the partial profile
    let merged = base_config.apply_profile(&partial_profile);

    // output_folder should be changed
    assert_eq!(
        merged.output_folder, "./output/staging",
        "output_folder should be overridden"
    );

    // project_name should be unchanged (not part of ProfileOverride)
    assert_eq!(merged.project_name, "my-project");

    // jira should be unchanged
    let jira = merged.jira.as_ref().expect("jira should be preserved");
    assert_eq!(jira.endpoint, "https://jira.prod.com");
    assert_eq!(jira.token.as_ref().unwrap(), "prod-token");

    // squash should be unchanged
    let squash = merged.squash.as_ref().expect("squash should be preserved");
    assert_eq!(squash.endpoint, "https://squash.prod.com");

    // llm should be unchanged
    let llm = merged.llm.as_ref().expect("llm should be preserved");
    assert_eq!(llm.mode, LlmMode::Cloud);
    assert_eq!(llm.api_key.as_ref().unwrap(), "sk-prod-key");

    // templates should be unchanged
    let templates = merged.templates.as_ref().expect("templates should be preserved");
    assert_eq!(templates.cr.as_ref().unwrap(), "./templates/cr.md");
}

/// A ProfileOverride with only jira set should replace the entire jira config,
/// but leave other integrations unchanged.
#[test]
fn test_partial_override_replaces_entire_section() {
    let base_config = ProjectConfig {
        project_name: "my-project".to_string(),
        output_folder: "./output".to_string(),
        jira: Some(JiraConfig {
            endpoint: "https://jira.prod.com".to_string(),
            token: Some("prod-token".to_string()),
        }),
        squash: Some(SquashConfig {
            endpoint: "https://squash.prod.com".to_string(),
            username: Some("prod-user".to_string()),
            password: Some("prod-pass".to_string()),
        }),
        llm: None,
        templates: None,
        profiles: None,
        active_profile: None,
    };

    // Create a profile that overrides jira with a different endpoint and token
    let jira_override_profile = ProfileOverride {
        jira: Some(JiraConfig {
            endpoint: "https://jira.staging.com".to_string(),
            token: Some("staging-token".to_string()),
        }),
        squash: None,
        llm: None,
        templates: None,
        output_folder: None,
    };

    let merged = base_config.apply_profile(&jira_override_profile);

    // jira should be completely replaced
    let jira = merged.jira.as_ref().expect("jira should exist");
    assert_eq!(jira.endpoint, "https://jira.staging.com");
    assert_eq!(jira.token.as_ref().unwrap(), "staging-token");

    // squash should be unchanged
    let squash = merged.squash.as_ref().expect("squash should be preserved");
    assert_eq!(squash.endpoint, "https://squash.prod.com");
    assert_eq!(squash.username.as_ref().unwrap(), "prod-user");

    // output_folder should be unchanged
    assert_eq!(merged.output_folder, "./output");
}

// =============================================================================
// Test 4: ProfileOverride Default trait
// =============================================================================

/// ProfileOverride::default() should create an instance with all fields set to None.
/// This is essential for the partial override pattern where users only specify
/// the fields they want to change.
#[test]
fn test_profile_override_default_has_all_none() {
    let default_profile = ProfileOverride::default();

    assert!(
        default_profile.jira.is_none(),
        "default ProfileOverride.jira should be None"
    );
    assert!(
        default_profile.squash.is_none(),
        "default ProfileOverride.squash should be None"
    );
    assert!(
        default_profile.llm.is_none(),
        "default ProfileOverride.llm should be None"
    );
    assert!(
        default_profile.templates.is_none(),
        "default ProfileOverride.templates should be None"
    );
    assert!(
        default_profile.output_folder.is_none(),
        "default ProfileOverride.output_folder should be None"
    );
}

/// ProfileOverride should be Clone-able for flexibility in configuration management.
#[test]
fn test_profile_override_is_clone() {
    let profile = ProfileOverride {
        jira: Some(JiraConfig {
            endpoint: "https://jira.example.com".to_string(),
            token: Some("my-token".to_string()),
        }),
        squash: None,
        llm: None,
        templates: None,
        output_folder: Some("./custom-output".to_string()),
    };

    let cloned = profile.clone();

    // Verify the clone has the same values
    assert!(cloned.jira.is_some());
    assert_eq!(
        cloned.jira.as_ref().unwrap().endpoint,
        "https://jira.example.com"
    );
    assert!(cloned.squash.is_none());
    assert_eq!(cloned.output_folder.as_ref().unwrap(), "./custom-output");
}

// =============================================================================
// Additional edge case tests
// =============================================================================

/// Profile with None values preserves base config (partial override pattern).
///
/// When a ProfileOverride has a field set to None, the base configuration value
/// is preserved. This is the fundamental partial override behavior - only fields
/// explicitly set in the profile will change the configuration.
#[test]
fn test_profile_with_none_preserves_base_value() {
    let base_config = ProjectConfig {
        project_name: "my-project".to_string(),
        output_folder: "./output".to_string(),
        jira: Some(JiraConfig {
            endpoint: "https://jira.prod.com".to_string(),
            token: Some("prod-token".to_string()),
        }),
        squash: None,
        llm: None,
        templates: None,
        profiles: None,
        active_profile: None,
    };

    // A profile with jira = None means "don't change jira" (preserve base)
    // This is the default behavior for partial overrides
    let preserve_profile = ProfileOverride {
        jira: None, // None means "keep the base value"
        squash: None,
        llm: None,
        templates: None,
        output_folder: None,
    };

    let merged = base_config.apply_profile(&preserve_profile);

    // jira should be preserved (None in override means "keep base")
    assert!(
        merged.jira.is_some(),
        "jira should be preserved when profile.jira is None"
    );
}

// =============================================================================
// Test: active_profile_summary (Review Follow-up HIGH - AC #1)
// =============================================================================

/// Test that active_profile_summary() returns meaningful output when a profile is active.
/// This tests AC #1: "la configuration du profil est appliquée et affichée dans le résumé"
#[test]
fn test_active_profile_summary_shows_profile_and_values() {
    let config = ProjectConfig {
        project_name: "test-project".to_string(),
        output_folder: "./output".to_string(),
        jira: Some(JiraConfig {
            endpoint: "https://jira.prod.example.com".to_string(),
            token: Some("secret-token".to_string()),
        }),
        squash: None,
        llm: Some(LlmConfig {
            mode: LlmMode::Local,
            local_endpoint: Some("http://localhost:11434".to_string()),
            local_model: Some("mistral:7b".to_string()),
            cloud_enabled: false,
            cloud_endpoint: None,
            cloud_model: None,
            api_key: None,
            timeout_seconds: 60,
            max_tokens: 2048,
        }),
        templates: None,
        profiles: None,
        active_profile: Some("dev".to_string()),
    };

    let summary = config.active_profile_summary();

    // Summary should mention the active profile
    assert!(
        summary.contains("dev"),
        "Summary should mention active profile 'dev', got: {}",
        summary
    );

    // Summary should show effective output folder
    assert!(
        summary.contains("./output") || summary.contains("output"),
        "Summary should show output folder, got: {}",
        summary
    );

    // Summary should show LLM mode (exact format from active_profile_summary: "LLM: local")
    assert!(
        summary.contains("LLM: local"),
        "Summary should show LLM mode in format 'LLM: local', got: {}",
        summary
    );

    // Summary should NOT contain the secret token
    assert!(
        !summary.contains("secret-token"),
        "Summary should NOT contain secret token, got: {}",
        summary
    );
}

/// Test that active_profile_summary() handles no active profile gracefully.
#[test]
fn test_active_profile_summary_no_profile() {
    let config = ProjectConfig {
        project_name: "test-project".to_string(),
        output_folder: "./output".to_string(),
        jira: None,
        squash: None,
        llm: None,
        templates: None,
        profiles: None,
        active_profile: None,
    };

    let summary = config.active_profile_summary();

    // Summary should indicate no profile is active
    assert!(
        summary.contains("none") || summary.contains("None") || summary.contains("No profile") || summary.contains("default"),
        "Summary should indicate no active profile, got: {}",
        summary
    );
}

/// Multiple overrides can be applied in sequence (profile chaining).
#[test]
fn test_multiple_profile_overrides_chain() {
    let base_config = ProjectConfig {
        project_name: "my-project".to_string(),
        output_folder: "./output/base".to_string(),
        jira: Some(JiraConfig {
            endpoint: "https://jira.base.com".to_string(),
            token: Some("base-token".to_string()),
        }),
        squash: None,
        llm: None,
        templates: None,
        profiles: None,
        active_profile: None,
    };

    // First profile: change output folder
    let profile1 = ProfileOverride {
        jira: None,
        squash: None,
        llm: None,
        templates: None,
        output_folder: Some("./output/staging".to_string()),
    };

    // Second profile: change jira endpoint
    let profile2 = ProfileOverride {
        jira: Some(JiraConfig {
            endpoint: "https://jira.staging.com".to_string(),
            token: Some("staging-token".to_string()),
        }),
        squash: None,
        llm: None,
        templates: None,
        output_folder: None,
    };

    // Apply profiles in sequence
    let after_first = base_config.apply_profile(&profile1);
    let after_second = after_first.apply_profile(&profile2);

    // Both overrides should be applied
    assert_eq!(after_second.output_folder, "./output/staging");
    assert_eq!(
        after_second.jira.as_ref().unwrap().endpoint,
        "https://jira.staging.com"
    );
}

// =============================================================================
// Review Follow-up Round 5 - LOW Priority Tests
// =============================================================================

/// Test ProfileOverride PartialEq derive directly (Review Follow-up Round 5 - LOW)
///
/// Validates that the PartialEq derive works correctly by comparing identical
/// and different ProfileOverride instances.
#[test]
fn test_profile_override_partial_eq() {
    let profile1 = ProfileOverride {
        jira: Some(JiraConfig {
            endpoint: "https://jira.example.com".to_string(),
            token: Some("secret-token".to_string()),
        }),
        squash: None,
        llm: None,
        templates: None,
        output_folder: Some("./output".to_string()),
    };

    // Clone should be equal
    let profile2 = profile1.clone();
    assert_eq!(profile1, profile2, "Clone should be equal to original");

    // Different output_folder should not be equal
    let profile3 = ProfileOverride {
        jira: Some(JiraConfig {
            endpoint: "https://jira.example.com".to_string(),
            token: Some("secret-token".to_string()),
        }),
        squash: None,
        llm: None,
        templates: None,
        output_folder: Some("./different-output".to_string()),
    };
    assert_ne!(profile1, profile3, "Different output_folder should not be equal");

    // Default profiles should be equal
    let default1 = ProfileOverride::default();
    let default2 = ProfileOverride::default();
    assert_eq!(default1, default2, "Default profiles should be equal");
}

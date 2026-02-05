# ATDD Checklist - Epic 0, Story 0.2: Définir et sélectionner des profils de configuration

**Date:** 2026-02-05
**Author:** Edouard
**Primary Test Level:** Integration + Unit (Rust `cargo test`)

---

## Story Summary

Enable configuration profiles to allow QA testers to quickly switch between different environments (dev, staging, prod) without modifying the base config file.

**As a** QA tester (TRA)
**I want** définir et sélectionner des profils de configuration
**So that** basculer rapidement de contexte

---

## Acceptance Criteria

1. **AC #1:** Given une config.yaml avec des profils, When je sélectionne un profil, Then la configuration du profil est appliquée et affichée dans le résumé
2. **AC #2:** Given un profil inconnu, When je tente de le sélectionner, Then un message indique l'erreur et liste les profils disponibles
3. **AC #3:** Given un profil appliqué, When les logs sont écrits, Then ils ne contiennent aucune donnée sensible

---

## Failing Tests Created (RED Phase)

### Integration Tests (7 tests)

**File:** `crates/tf-config/tests/profile_tests.rs` (233 lines)

- ✅ **Test:** `test_load_config_with_valid_profile`
  - **Status:** RED - `ProfileOverride` type doesn't exist
  - **Verifies:** AC #1 - Profile selection and active_profile field

- ✅ **Test:** `test_profile_merge_overrides_output_folder`
  - **Status:** RED - `with_profile()` method doesn't exist
  - **Verifies:** AC #1 - Merge logic for output_folder and jira

- ✅ **Test:** `test_unknown_profile_lists_available`
  - **Status:** RED - `ConfigError::ProfileNotFound` variant doesn't exist
  - **Verifies:** AC #2 - Error message with available profiles list

- ✅ **Test:** `test_merged_config_redacts_profile_secrets`
  - **Status:** RED - `with_profile()` method doesn't exist
  - **Verifies:** AC #3 - Secrets redaction after merge

- ✅ **Test:** `test_staging_profile_overrides_multiple_sections`
  - **Status:** RED - `with_profile()` method doesn't exist
  - **Verifies:** AC #1 - Complex merge (jira + squash)

- ✅ **Test:** `test_empty_prod_profile_uses_base_values`
  - **Status:** RED - `with_profile()` method doesn't exist
  - **Verifies:** AC #1 - Empty profile behavior

- ✅ **Test:** `test_config_without_profile_is_backward_compatible`
  - **Status:** RED - `active_profile` field doesn't exist
  - **Verifies:** Backward compatibility

### Unit Tests (12 tests)

**File:** `crates/tf-config/tests/profile_unit_tests.rs` (~200 lines)

- ✅ **Test:** `test_profile_override_redacts_jira_token_in_debug`
  - **Status:** RED - `ProfileOverride` type doesn't exist
  - **Verifies:** AC #3 - JiraConfig token redaction

- ✅ **Test:** `test_profile_override_redacts_llm_api_key_in_debug`
  - **Status:** RED - `ProfileOverride` type doesn't exist
  - **Verifies:** AC #3 - LlmConfig API key redaction

- ✅ **Test:** `test_profile_override_redacts_squash_password_in_debug`
  - **Status:** RED - `ProfileOverride` type doesn't exist
  - **Verifies:** AC #3 - SquashConfig password redaction

- ✅ **Test:** `test_empty_profile_preserves_base_config`
  - **Status:** RED - `apply_profile()` method doesn't exist
  - **Verifies:** AC #1 - Empty profile merge

- ✅ **Test:** `test_partial_override_only_changes_specified_fields`
  - **Status:** RED - `apply_profile()` method doesn't exist
  - **Verifies:** AC #1 - Partial override logic

- ✅ **Test:** `test_partial_override_replaces_entire_section`
  - **Status:** RED - `apply_profile()` method doesn't exist
  - **Verifies:** AC #1 - Section replacement behavior

- ✅ **Test:** `test_profile_override_can_remove_section`
  - **Status:** RED - `apply_profile()` method doesn't exist
  - **Verifies:** AC #1 - None behavior (preserve base)

- ✅ **Test:** `test_multiple_profile_overrides_chain`
  - **Status:** RED - `apply_profile()` method doesn't exist
  - **Verifies:** AC #1 - Profile chaining

- ✅ **Test:** `test_profile_override_default_has_all_none`
  - **Status:** RED - `ProfileOverride` type doesn't exist
  - **Verifies:** Default trait implementation

- ✅ **Test:** `test_profile_override_is_clone`
  - **Status:** RED - `ProfileOverride` type doesn't exist
  - **Verifies:** Clone trait implementation

---

## Data Factories Created

Not applicable for Rust - using YAML fixture files instead.

---

## Fixtures Created

### Profiles Test Fixtures

**File:** `crates/tf-config/tests/fixtures/config_with_profiles.yaml`

**Provides:**
- Base config with project_name, output_folder, jira, squash, llm
- Three profiles: dev, staging, prod
- Dev: overrides output_folder, jira
- Staging: overrides jira, squash
- Prod: empty (uses base values)

**Example Usage:**

```rust
let config = load_config(&fixture_path("config_with_profiles.yaml")).unwrap();
let merged = config.with_profile("dev").unwrap();
```

### Jira Override Fixture

**File:** `crates/tf-config/tests/fixtures/config_profile_override_jira.yaml`

**Provides:**
- Base config with jira token
- Single profile (override_secrets) with different jira token
- Tests AC #3 - secrets redaction after merge

---

## Mock Requirements

Not applicable - this is a pure library crate with no external service calls.

---

## Required data-testid Attributes

Not applicable - no UI components in this crate.

---

## Implementation Checklist

### Test: test_load_config_with_valid_profile

**File:** `crates/tf-config/tests/profile_tests.rs`

**Tasks to make this test pass:**

- [ ] Add `profiles: Option<HashMap<String, ProfileOverride>>` field to `ProjectConfig`
- [ ] Add `active_profile: Option<String>` field to `ProjectConfig`
- [ ] Create `ProfileOverride` struct with optional fields (jira, squash, llm, templates, output_folder)
- [ ] Add `#[serde(deny_unknown_fields)]` to `ProfileOverride`
- [ ] Export `ProfileOverride` from lib.rs
- [ ] Run test: `cargo test -p tf-config test_load_config_with_valid_profile`
- [ ] ✅ Test passes (green phase)

---

### Test: test_profile_merge_overrides_output_folder

**File:** `crates/tf-config/tests/profile_tests.rs`

**Tasks to make this test pass:**

- [ ] Implement `ProjectConfig::with_profile(&self, profile_id: &str) -> Result<ProjectConfig, ConfigError>`
- [ ] Clone base config
- [ ] Get profile from profiles HashMap
- [ ] Merge output_folder if Some
- [ ] Merge jira if Some (full replacement)
- [ ] Set active_profile to profile_id
- [ ] Run test: `cargo test -p tf-config test_profile_merge_overrides_output_folder`
- [ ] ✅ Test passes (green phase)

---

### Test: test_unknown_profile_lists_available

**File:** `crates/tf-config/tests/profile_tests.rs`

**Tasks to make this test pass:**

- [ ] Add `ConfigError::ProfileNotFound { requested: String, available: Vec<String> }` variant
- [ ] Add thiserror message: "Profile '{requested}' not found. Available profiles: {available:?}"
- [ ] Return this error from `with_profile()` when profile not in HashMap
- [ ] Collect available profiles from HashMap keys
- [ ] Run test: `cargo test -p tf-config test_unknown_profile_lists_available`
- [ ] ✅ Test passes (green phase)

---

### Test: test_merged_config_redacts_profile_secrets

**File:** `crates/tf-config/tests/profile_tests.rs`

**Tasks to make this test pass:**

- [ ] Implement custom `Debug` for `ProfileOverride` that redacts secrets
- [ ] Ensure JiraConfig, SquashConfig, LlmConfig tokens are redacted
- [ ] Verify merged config Debug output doesn't contain secret values
- [ ] Run test: `cargo test -p tf-config test_merged_config_redacts_profile_secrets`
- [ ] ✅ Test passes (green phase)

---

### Test: test_profile_override_redacts_jira_token_in_debug

**File:** `crates/tf-config/tests/profile_unit_tests.rs`

**Tasks to make this test pass:**

- [ ] `ProfileOverride` Debug impl must redact jira.token
- [ ] Use existing JiraConfig Debug pattern (already redacts)
- [ ] Run test: `cargo test -p tf-config test_profile_override_redacts_jira_token_in_debug`
- [ ] ✅ Test passes (green phase)

---

## Running Tests

```bash
# Run all failing tests for this story (will fail to compile in RED phase)
cargo test -p tf-config --no-run 2>&1 | grep "error\["

# Run specific test file (after implementation)
cargo test -p tf-config --test profile_tests

# Run unit tests
cargo test -p tf-config --test profile_unit_tests

# Run all tf-config tests
cargo test -p tf-config

# Run with verbose output
cargo test -p tf-config -- --nocapture
```

---

## Red-Green-Refactor Workflow

### RED Phase (Complete) ✅

**TEA Agent Responsibilities:**

- ✅ All tests written and failing to compile
- ✅ Fixtures created with test data
- ✅ Implementation checklist created
- ✅ Missing types documented

**Verification:**

- All tests reference non-existent types (ProfileOverride, ConfigError::ProfileNotFound)
- All tests reference non-existent methods (with_profile, apply_profile)
- Compilation fails with 10+ errors (expected)

---

### GREEN Phase (DEV Team - Next Steps)

**DEV Agent Responsibilities:**

1. **Pick one failing test** from implementation checklist (start with `test_load_config_with_valid_profile`)
2. **Read the test** to understand expected behavior
3. **Implement minimal code** to make that specific test compile and pass
4. **Run the test** to verify it now passes (green)
5. **Check off the task** in implementation checklist
6. **Move to next test** and repeat

**Key Principles:**

- One test at a time (don't try to fix all at once)
- Minimal implementation (don't over-engineer)
- Run tests frequently (immediate feedback)
- Use implementation checklist as roadmap

**Progress Tracking:**

- Check off tasks as you complete them
- Run `cargo test -p tf-config` after each change

---

### REFACTOR Phase (DEV Team - After All Tests Pass)

**DEV Agent Responsibilities:**

1. **Verify all tests pass** (green phase complete)
2. **Review code for quality** (readability, maintainability)
3. **Extract duplications** (DRY principle)
4. **Ensure tests still pass** after each refactor
5. **Update documentation** (doc comments)

**Key Principles:**

- Tests provide safety net (refactor with confidence)
- Make small refactors (easier to debug if tests fail)
- Run tests after each change
- Don't change test behavior (only implementation)

**Completion:**

- All 19 tests pass
- Code quality meets Rust conventions
- No clippy warnings
- Ready for code review

---

## Next Steps

1. **Review this checklist** before starting implementation
2. **Run compilation** to confirm RED phase: `cargo test -p tf-config --no-run`
3. **Begin implementation** using the story file tasks and this checklist
4. **Work one test at a time** (red → green for each)
5. **When all tests pass**, refactor code for quality
6. **Run full test suite**: `cargo test -p tf-config`
7. **Update story status** to 'done' in sprint-status.yaml

---

## Knowledge Base References Applied

This ATDD workflow applied Rust testing best practices:

- **test-quality.md** - Given-When-Then structure, isolation, determinism
- **data-factories.md** - YAML fixtures with realistic test data
- **Rust conventions** - `#[test]`, `assert!`, `assert_eq!`, `matches!`

---

## Test Execution Evidence

### Initial Test Run (RED Phase Verification)

**Command:** `cargo test -p tf-config --no-run`

**Results:**

```
error[E0432]: unresolved import `tf_config::ProfileOverride`
error[E0599]: no method named `with_profile` found for struct `ProjectConfig`
error[E0599]: no variant named `ProfileNotFound` found for enum `ConfigError`
error[E0599]: no method named `apply_profile` found for struct `ProjectConfig`
```

**Summary:**

- Total tests: 19 (7 integration + 12 unit)
- Compiling: 0 (expected)
- Failing to compile: 19 (expected)
- Status: ✅ RED phase verified

---

## Notes

- Story 0-2 builds on Story 0-1 patterns (Redact trait, custom Debug, ConfigError)
- All 134 existing tests from Story 0-1 must continue to pass
- Profile merge should use full section replacement (not deep merge)
- Empty profile ({}) should preserve all base values

---

## Contact

**Questions or Issues?**

- Review Story 0-2 file: `_bmad-output/implementation-artifacts/0-2-definir-et-selectionner-des-profils-de-configuration.md`
- Run `/bmad-bmm-dev-story` to start implementation

---

**Generated by BMad TEA Agent** - 2026-02-05

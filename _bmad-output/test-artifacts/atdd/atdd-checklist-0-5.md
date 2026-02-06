# ATDD Checklist - Epic 0, Story 0.5: Journalisation baseline sans donnees sensibles

**Date:** 2026-02-06
**Author:** Edouard
**Primary Test Level:** Unit (cargo test)

---

## Story Summary

Implement baseline structured JSON logging for the test-framework CLI with automatic sensitive field redaction and configurable file output.

**As a** QA maintainer
**I want** une journalisation baseline sans donnees sensibles
**So that** garantir l'auditabilite minimale des executions des le debut

---

## Acceptance Criteria

1. **AC #1**: Given la journalisation activee, When une commande CLI s'execute, Then des logs JSON structures sont generes (timestamp, commande, statut, perimetre)
2. **AC #2**: Given des champs sensibles sont presents dans le contexte, When ils seraient journalises, Then ils sont masques automatiquement
3. **AC #3**: Given une execution terminee, When les logs sont ecrits, Then ils sont stockes dans le dossier de sortie configure

---

## Failing Tests Created (RED Phase)

### Unit Tests — init.rs (7 tests)

**File:** `crates/tf-logging/src/init.rs` (191 lines)

- **Test:** `test_init_logging_creates_dir_and_returns_guard`
  - **Status:** RED — `todo!()` panic in `init_logging`
  - **Verifies:** AC #1, AC #3 — init_logging creates log directory and returns valid LogGuard
  - **Priority:** P1

- **Test:** `test_log_output_contains_required_json_fields`
  - **Status:** RED — `todo!()` panic in `init_logging`
  - **Verifies:** AC #1 — JSON output has timestamp (ISO 8601), level (UPPERCASE), message, target
  - **Priority:** P0

- **Test:** `test_logs_written_to_configured_directory`
  - **Status:** RED — `todo!()` panic in `init_logging`
  - **Verifies:** AC #3 — Log files created in `{output_folder}/logs/`
  - **Priority:** P0

- **Test:** `test_default_log_level_is_info`
  - **Status:** RED — `todo!()` panic in `init_logging`
  - **Verifies:** AC #1 — Default level filters out debug, passes info
  - **Priority:** P1

- **Test:** `test_rust_log_overrides_configured_level`
  - **Status:** RED — `todo!()` panic in `init_logging`
  - **Verifies:** AC #1 — RUST_LOG env var overrides configured level
  - **Priority:** P1

- **Test:** `test_ansi_disabled_for_file_logs`
  - **Status:** RED — `todo!()` panic in `init_logging`
  - **Verifies:** AC #1 — No ANSI escape codes in file output
  - **Priority:** P2

### Unit Tests — redact.rs (15 tests)

**File:** `crates/tf-logging/src/redact.rs` (289 lines)

- **Tests:** `test_sensitive_field_{token,api_key,apikey,key,secret,password,passwd,pwd,auth,authorization,credential,credentials}_redacted` (12 tests)
  - **Status:** RED — `todo!()` panic in `init_logging`
  - **Verifies:** AC #2 — Each of the 12 sensitive field names is masked by `[REDACTED]`
  - **Priority:** P0

- **Test:** `test_normal_fields_are_not_redacted`
  - **Status:** RED — `todo!()` panic in `init_logging`
  - **Verifies:** AC #2 (negative) — Normal fields (command, status, scope) are NOT masked
  - **Priority:** P0

- **Test:** `test_urls_with_sensitive_params_are_redacted`
  - **Status:** RED — `todo!()` panic in `init_logging`
  - **Verifies:** AC #2 — URLs with `?token=abc123` have values redacted
  - **Priority:** P0

- **Test:** `test_log_guard_debug_no_sensitive_data`
  - **Status:** RED — `todo!()` panic in `init_logging`
  - **Verifies:** AC #2 — Debug impl of LogGuard does not leak secrets
  - **Priority:** P1

### Unit Tests — config.rs (2 tests)

**File:** `crates/tf-logging/src/config.rs` (58 lines)

- **Test:** `test_logging_config_from_project_config_derives_log_dir`
  - **Status:** RED — `todo!()` panic in `from_project_config`
  - **Verifies:** AC #3 — log_dir derived as `{output_folder}/logs`
  - **Priority:** P2

- **Test:** `test_logging_config_fallback_when_output_folder_empty`
  - **Status:** RED — `todo!()` panic in `from_project_config`
  - **Verifies:** AC #3 — Falls back to `./logs` when output_folder empty
  - **Priority:** P2

### Unit Tests — error.rs (3 tests) — GREEN

**File:** `crates/tf-logging/src/error.rs` (100 lines)

- **Test:** `test_logging_error_init_failed_has_actionable_hint`
  - **Status:** GREEN — Type definitions are complete
  - **Verifies:** AC #3 — InitFailed error includes cause + actionable hint
  - **Priority:** P1

- **Test:** `test_logging_error_directory_creation_failed_has_actionable_hint`
  - **Status:** GREEN — Type definitions are complete
  - **Verifies:** AC #3 — DirectoryCreationFailed error includes path + cause + hint
  - **Priority:** P1

- **Test:** `test_logging_error_invalid_log_level_has_actionable_hint`
  - **Status:** GREEN — Type definitions are complete
  - **Verifies:** AC #3 — InvalidLogLevel error includes level + hint
  - **Priority:** P1

### Integration Tests (3 tests)

**File:** `crates/tf-logging/tests/integration_test.rs` (141 lines)

- **Test:** `test_full_logging_lifecycle`
  - **Status:** RED — `todo!()` panic in `init_logging`
  - **Verifies:** AC #1, #2, #3 — Full lifecycle: init → log with sensitive+normal fields → flush → verify JSON + redaction
  - **Priority:** P1

- **Test:** `test_tf_logging_crate_compiles_and_types_accessible`
  - **Status:** GREEN — Types exist
  - **Verifies:** INT-002 — Workspace integration
  - **Priority:** P1

- **Test:** `test_multiple_sensitive_fields_redacted_in_single_event`
  - **Status:** RED — `todo!()` panic in `init_logging`
  - **Verifies:** AC #2 — Multiple sensitive fields redacted in one event
  - **Priority:** P1

---

## Data Factories Created

N/A — Rust tests use `tempfile::tempdir()` for isolated filesystem testing and direct struct construction for test data. No external factory crate needed.

---

## Fixtures Created

### Log File Helper

**File:** `crates/tf-logging/src/init.rs` (tests module) + `crates/tf-logging/tests/integration_test.rs`

**Function:** `find_log_file(logs_dir: &Path) -> PathBuf`
- **Purpose:** Find the first log file in a directory (tracing-appender uses date-based filenames)
- **Usage:** Called after dropping LogGuard to locate the written log file

---

## Mock Requirements

None — tf-logging is a pure library crate with no external service dependencies. Tests use real filesystem via tempdir.

---

## Required data-testid Attributes

N/A — No UI components in this story.

---

## Implementation Checklist

### Task 1: Create tf-logging crate structure

**Tests that verify this:** `test_tf_logging_crate_compiles_and_types_accessible` (already GREEN)

- [x] Crate directory `crates/tf-logging/` created
- [x] `Cargo.toml` with workspace dependencies
- [x] `src/lib.rs` with module declarations
- [x] Workspace `Cargo.toml` updated with tracing dependencies
- [x] **Already done by TEA (ATDD setup)**

---

### Task 2: Implement init_logging + LogGuard

**Tests that verify this:**
- `test_init_logging_creates_dir_and_returns_guard`
- `test_log_output_contains_required_json_fields`
- `test_logs_written_to_configured_directory`
- `test_default_log_level_is_info`
- `test_rust_log_overrides_configured_level`
- `test_ansi_disabled_for_file_logs`

**Tasks to make these tests pass:**

- [ ] Replace `todo!()` in `init_logging()` with real implementation
- [ ] Configure `tracing-subscriber` with JSON format
- [ ] Configure `tracing-appender::rolling::RollingFileAppender` with daily rotation
- [ ] Use `tracing_appender::non_blocking()` for performance
- [ ] Support `EnvFilter` (RUST_LOG priority, else config level, else `info`)
- [ ] Disable ANSI colors with `with_ansi(false)`
- [ ] Return `LogGuard` wrapping `WorkerGuard`
- [ ] Create log directory with `fs::create_dir_all`
- [ ] Run tests: `cargo test -p tf-logging init::tests`
- [ ] All 7 init tests pass (green phase)

**Estimated Effort:** 2-3 hours

---

### Task 3: Implement RedactingLayer

**Tests that verify this:**
- `test_sensitive_field_*_redacted` (12 tests)
- `test_normal_fields_are_not_redacted`
- `test_urls_with_sensitive_params_are_redacted`
- `test_log_guard_debug_no_sensitive_data`

**Tasks to make these tests pass:**

- [ ] Expose `redact_url_sensitive_params` as `pub` in tf-config (Subtask 3.0)
- [ ] Create `RedactingLayer` implementing `tracing_subscriber::Layer`
- [ ] Implement `RedactingVisitor` implementing `tracing::field::Visit`
- [ ] Replace sensitive field values with `[REDACTED]` based on SENSITIVE_FIELDS
- [ ] Detect URL-like values and apply `redact_url_sensitive_params()`
- [ ] Integrate RedactingLayer in subscriber stack (before JSON layer)
- [ ] Run tests: `cargo test -p tf-logging redact::tests`
- [ ] All 15 redact tests pass (green phase)

**Estimated Effort:** 3-5 hours (includes R-05-02 spike for immutable events)

---

### Task 4: Implement LoggingConfig::from_project_config

**Tests that verify this:**
- `test_logging_config_from_project_config_derives_log_dir`
- `test_logging_config_fallback_when_output_folder_empty`

**Tasks to make these tests pass:**

- [ ] Replace `todo!()` in `from_project_config()` with real implementation
- [ ] Derive `log_dir = format!("{}/logs", config.output_folder)`
- [ ] Fallback to `"./logs"` if `output_folder` is empty
- [ ] Default `log_level = "info"`, `log_to_stdout = false`
- [ ] Run tests: `cargo test -p tf-logging config::tests`
- [ ] Both config tests pass (green phase)

**Estimated Effort:** 0.5 hours

---

### Task 5: Integration verification

**Tests that verify this:**
- `test_full_logging_lifecycle`
- `test_multiple_sensitive_fields_redacted_in_single_event`

**Tasks to make these tests pass:**

- [ ] All previous tasks completed
- [ ] Run integration tests: `cargo test -p tf-logging --test integration_test`
- [ ] Both integration tests pass (green phase)
- [ ] Run full workspace: `cargo test --workspace`
- [ ] All 327+ existing tests still pass (non-regression)

**Estimated Effort:** 0.5 hours

---

## Running Tests

```bash
# Run all failing tests for this story
cargo test -p tf-logging

# Run specific test module
cargo test -p tf-logging init::tests
cargo test -p tf-logging redact::tests
cargo test -p tf-logging config::tests
cargo test -p tf-logging error::tests

# Run integration tests only
cargo test -p tf-logging --test integration_test

# Run specific test by name
cargo test -p tf-logging test_log_output_contains_required_json_fields

# Run with output visible
cargo test -p tf-logging -- --nocapture

# Run non-regression (full workspace)
cargo test --workspace

# Run clippy checks
cargo clippy -p tf-logging -- -D warnings
```

---

## Red-Green-Refactor Workflow

### RED Phase (Complete)

**TEA Agent Responsibilities:**

- All 29 tests written (25 failing + 4 passing)
- Crate structure created with stub implementations
- Workspace dependencies configured
- Integration test infrastructure ready
- ATDD checklist created

**Verification:**

- 25 tests fail with `todo!()` panic (expected behavior)
- 4 tests pass (type definitions + workspace integration)
- Failure message: `not yet implemented: RED phase: implement logging initialization...`
- Failures are due to missing implementation, not test bugs

---

### GREEN Phase (DEV Team - Next Steps)

**DEV Agent Responsibilities:**

1. **Start with Task 4** (LoggingConfig::from_project_config) — simplest, unblocks config tests
2. **Then Task 2** (init_logging) — core initialization, unblocks most tests
3. **Then Task 3** (RedactingLayer) — sensitive field redaction, hardest part
4. **Finally Task 5** (integration verification)

**Key Principles:**

- Replace `todo!()` stubs with real implementation
- Run `cargo test -p tf-logging` after each change
- Watch failing test count decrease
- Use `cargo test -p tf-logging <test_name>` to target specific tests

**Progress Tracking:**

- Check off tasks as you complete them
- Target: 0 failing tests = GREEN phase complete

---

### REFACTOR Phase (DEV Team - After All Tests Pass)

**DEV Agent Responsibilities:**

1. Verify all 29 tests pass
2. Run `cargo clippy -p tf-logging -- -D warnings`
3. Run `cargo fmt -- --check`
4. Review for code quality (DRY, naming, documentation)
5. Ensure `cargo test --workspace` passes (non-regression)

---

## Next Steps

1. **Share this checklist** with the dev workflow (manual handoff)
2. **Run failing tests** to confirm RED phase: `cargo test -p tf-logging`
3. **Begin implementation** using implementation checklist as guide
4. **Work one task at a time** (red -> green for each)
5. **When all tests pass**, refactor code for quality
6. **When refactoring complete**, update story status to 'done'

---

## Knowledge Base References Applied

- **data-factories.md** — Factory pattern principles (adapted to Rust: tempdir + direct construction)
- **test-quality.md** — Deterministic, isolated, explicit assertions, atomic tests
- **test-healing-patterns.md** — Failure catalog awareness for future debugging
- **component-tdd.md** — Red-Green-Refactor cycle applied to Rust crate
- **test-levels-framework.md** — Unit vs Integration level selection
- **test-priorities-matrix.md** — P0-P3 prioritization from test-design document

---

## Test Execution Evidence

### Initial Test Run (RED Phase Verification)

**Command:** `cargo test -p tf-logging`

**Results:**

```
test result: FAILED. 3 passed; 23 failed; 0 ignored; 0 measured; 0 filtered out
```

**Integration tests:**

```
test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
```

**Summary:**

- Total tests: 29
- Passing: 4 (type definitions + workspace integration)
- Failing: 25 (all require implementation)
- Status: RED phase verified

**Expected Failure Messages:**

- All 25 failing tests: `not yet implemented: RED phase: implement logging initialization with tracing-subscriber JSON format, file appender, redaction layer`
- 2 config tests: `not yet implemented: RED phase: implement LoggingConfig derivation from ProjectConfig`

---

## Risks and Assumptions

### High-Priority Risks

- **R-05-01 (Score 6)**: RedactingLayer incomplet — mitigated by 12 exhaustive per-field tests + negative test
- **R-05-02 (Score 6)**: tracing events immutables — DEV must spike the approach (custom FormatEvent vs Layer::on_event)

### Assumptions

- tf-config `redact_url_sensitive_params` will be exposed as `pub` (Subtask 3.0)
- tracing-subscriber 0.3.x supports a redaction mechanism
- Tests that call `init_logging` will need careful handling of global subscriber (one subscriber per test or thread-local approach)

### Known Limitation

- tracing's global subscriber can only be set once per process. The DEV implementation must handle this for test isolation (recommended: use `tracing::subscriber::with_default()` in tests instead of `set_global_default()`).

---

## Notes

- This is a **Rust crate** story, not a Playwright/UI story. Tests use `cargo test`, not Playwright.
- The RED phase uses `todo!()` stubs (Rust's equivalent of `test.skip()`) to ensure tests fail before implementation.
- Error type tests (UNIT-008) pass in RED phase because `LoggingError` is fully defined — this is intentional and correct.
- Test count (29) exceeds the test-design estimate (14) because each sensitive field gets its own test for exhaustive coverage.

---

**Generated by BMad TEA Agent** — 2026-02-06

# NFR Assessment - Journalisation Baseline sans Donnees Sensibles

**Date:** 2026-02-07
**Story:** 0-5 Journalisation baseline sans donnees sensibles
**Overall Status:** CONCERNS ⚠️

---

Note: This assessment summarizes existing evidence; it does not run tests or CI workflows.

## Executive Summary

**Assessment:** 2 PASS, 4 CONCERNS, 0 FAIL (6 applicable categories; 2 N/A for CLI tool)

**Blockers:** 0 — No release-blocking issues identified

**High Priority Issues:** 3 — cargo-audit not installed, no CI pipeline, no performance benchmarks

**Recommendation:** PROCEED WITH CONCERNS — Address cargo-audit installation and CI pipeline setup before Epic 1. All core functionality (structured logging, sensitive data redaction, error handling) meets quality standards. The CONCERNS are infrastructure gaps, not functional defects.

---

## Performance Assessment

### Response Time (p95)

- **Status:** CONCERNS ⚠️
- **Threshold:** CLI startup < 2s (NFR8)
- **Actual:** Not measured — no benchmarks exist
- **Evidence:** No benchmark suite; PRD NFR8 defines < 2s target
- **Findings:** Non-blocking I/O architecture (tracing-appender) is sound. `RedactingJsonFormatter` uses zero-allocation string matching for field redaction. However, no timed acceptance tests exist to validate NFR8 compliance. This is acceptable for Sprint 0 (library crate, not yet integrated into CLI binary).

### Throughput

- **Status:** PASS ✅
- **Threshold:** Logging should not block CLI execution
- **Actual:** Non-blocking writer via `tracing_appender::non_blocking` with `WorkerGuard`
- **Evidence:** `crates/tf-logging/src/lib.rs` — non-blocking appender wraps both file and stdout layers
- **Findings:** Architecture ensures log writes never block the main thread. Daily rolling file appender distributes I/O.

### Resource Usage

- **CPU Usage**
  - **Status:** PASS ✅
  - **Threshold:** Minimal overhead for logging operations
  - **Actual:** Zero-allocation field name matching in `RedactingJsonFormatter`; `SENSITIVE_FIELDS.contains()` uses compile-time constant array
  - **Evidence:** `crates/tf-logging/src/formatter.rs` — `SENSITIVE_FIELDS` const array, direct string comparison

- **Memory Usage**
  - **Status:** PASS ✅
  - **Threshold:** No unbounded allocations in logging path
  - **Actual:** Bounded buffer via `tracing_appender::non_blocking` (default 8192 events); no heap allocation for field name matching
  - **Evidence:** `crates/tf-logging/src/lib.rs` — standard non_blocking defaults

### Scalability

- **Status:** N/A
- **Threshold:** N/A — CLI tool, not a service
- **Actual:** N/A
- **Evidence:** N/A
- **Findings:** Scalability is not applicable for a local CLI library crate. Log volume is bounded by CLI execution duration.

---

## Security Assessment

### Authentication Strength

- **Status:** PASS ✅
- **Threshold:** No hardcoded secrets; secure credential storage
- **Actual:** OS keyring via `keyring` crate 3.6; zero hardcoded secrets found in codebase
- **Evidence:** `crates/tf-security/src/keyring.rs`, `Cargo.toml` (keyring = "3.6")
- **Findings:** Credentials stored in OS-native secure storage (macOS Keychain, Windows Credential Manager, Linux Secret Service). Custom `Debug` impl on `TokenConfig` hides sensitive values.

### Authorization Controls

- **Status:** PASS ✅
- **Threshold:** Token handling follows least-privilege
- **Actual:** `TokenConfig` uses custom `Debug` that redacts token values; `Redact` trait in tf-config
- **Evidence:** `crates/tf-config/src/redact.rs`, `crates/tf-security/src/lib.rs`
- **Findings:** Authorization is handled at the application level (API token). The framework correctly prevents token leakage through Debug output and logging.

### Data Protection

- **Status:** PASS ✅
- **Threshold:** All sensitive fields masked in logs; URL parameters redacted
- **Actual:** 12 sensitive field names + 26 compound suffixes automatically redacted to `[REDACTED]`; URL query parameters redacted via `redact_url_sensitive_params`
- **Evidence:** `crates/tf-logging/src/formatter.rs` (SENSITIVE_FIELDS, SENSITIVE_SUFFIXES), `crates/tf-config/src/redact.rs`
- **Findings:** Comprehensive redaction coverage verified by 68 tf-logging tests. Negative tests confirm normal fields (command, status, scope) are NOT redacted. URL parameter redaction handles `?token=abc&key=xyz` patterns.

### Vulnerability Management

- **Status:** CONCERNS ⚠️
- **Threshold:** 0 critical vulnerabilities in dependencies
- **Actual:** Unknown — `cargo audit` not installed
- **Evidence:** Running `cargo audit` returned "no such command: `audit`"
- **Findings:** Dependency vulnerability scanning is not available. The project uses well-known crates (tracing 0.1, serde 1.x, keyring 3.6) but has no automated verification. `serde_yaml` is deprecated upstream — should plan migration.
- **Recommendation:** Install `cargo-audit` (`cargo install cargo-audit`) and run before each release.

### Compliance (NFR4)

- **Status:** CONCERNS ⚠️
- **Threshold:** NFR4: Audit logs retained 90 days
- **Actual:** Daily rolling file appender creates dated log files; no automated retention/purge implemented
- **Evidence:** `crates/tf-logging/src/lib.rs` — `RollingFileAppender::new(Rotation::DAILY, ...)`
- **Findings:** Log files are created with daily rotation but old files are never cleaned up. NFR4 (90-day retention) requires both retention AND purge — currently only half-implemented. This is acceptable for Sprint 0 as the retention/purge feature is planned for a future story.

---

## Reliability Assessment

### Test Suite Health

- **Status:** PASS ✅
- **Threshold:** All tests pass, 0 flaky tests
- **Actual:** 417 passed, 0 failed, 18 ignored (workspace-wide); 68 tests in tf-logging specifically
- **Evidence:** `cargo test --workspace` output (2026-02-07)
- **Findings:** Comprehensive test suite with zero failures. Test isolation via thread-local subscriber dispatch (`set_default`) and `tempdir` prevents cross-test interference. 18 ignored tests are intentionally excluded (platform-specific or slow).

### Error Handling

- **Status:** PASS ✅
- **Threshold:** Structured errors with actionable hints
- **Actual:** `LoggingError` enum with `thiserror` — `InitFailed`, `DirectoryCreationFailed`, `InvalidLogLevel` variants; each includes `cause` and `hint` fields
- **Evidence:** `crates/tf-logging/src/error.rs`
- **Findings:** Error handling follows the workspace-wide pattern (cause + hint). All error variants are `#[non_exhaustive]` for future extensibility. Error messages guide users to resolution.

### Fault Tolerance

- **Status:** PASS ✅
- **Threshold:** Logging failures don't crash the application
- **Actual:** Non-blocking writer with `WorkerGuard` RAII pattern; `init_logging` returns `Result` for graceful handling
- **Evidence:** `crates/tf-logging/src/lib.rs` — `LogGuard` wraps `WorkerGuard`
- **Findings:** If log directory creation fails, a clear error with hint is returned. If the non-blocking writer's buffer is full, events are dropped (not blocking). Guard drop flushes remaining events.

### CI Burn-In (Stability)

- **Status:** CONCERNS ⚠️
- **Threshold:** Automated CI pipeline with repeated test runs
- **Actual:** No CI pipeline configured — tests run manually via `cargo test`
- **Evidence:** No `.github/workflows/` or CI configuration files found
- **Findings:** All 417 tests pass consistently in local execution. However, there is no automated CI to catch regressions on push/PR. No burn-in loop (repeated test runs) to detect flaky tests.
- **Recommendation:** Set up GitHub Actions with `cargo test --workspace`, `cargo clippy`, and `cargo fmt --check`.

### Availability / Disaster Recovery

- **Status:** N/A
- **Threshold:** N/A — CLI tool, not a service
- **Actual:** N/A
- **Evidence:** N/A
- **Findings:** Availability and disaster recovery are not applicable for a local CLI library. The tool runs on-demand and has no persistent state beyond log files and configuration.

---

## Maintainability Assessment

### Test Coverage

- **Status:** CONCERNS ⚠️
- **Threshold:** Meaningful coverage of all public APIs
- **Actual:** 417 tests across 3 crates; no line-coverage measurement tool (tarpaulin/llvm-cov not configured)
- **Evidence:** `cargo test --workspace` output; test files in `crates/*/src/` and `crates/*/tests/`
- **Findings:** Test count is strong (68 in tf-logging, ~300 in tf-config, ~49 in tf-security). However, actual line/branch coverage percentage is unknown. This is a measurement gap, not necessarily a coverage gap.

### Code Quality

- **Status:** PASS ✅
- **Threshold:** 0 clippy warnings; consistent formatting
- **Actual:** `cargo clippy --workspace --all-targets -- -D warnings` passes clean; `cargo fmt` consistent
- **Evidence:** Clippy run output (2026-02-07), `#![forbid(unsafe_code)]` in all crates
- **Findings:** Excellent code quality discipline. Safety enforced via `forbid(unsafe_code)`. All crates pass strict clippy lints. Code review went through 8 rounds with 52+ findings addressed.

### Technical Debt

- **Status:** CONCERNS ⚠️
- **Threshold:** Test files < 500 lines; no deprecated dependencies
- **Actual:** tf-config test file is 3231 lines (monolith); 80+ duplicated test patterns; `serde_yaml` deprecated
- **Evidence:** `_bmad-output/test-review.md` — maintainability score 45/100; test review score 81/100 (B)
- **Findings:** The tf-config test monolith is the largest technical debt item. Test quality review identified 80+ instances of duplicated setup/assertion patterns that should be extracted into test utilities. `serde_yaml` upstream deprecation requires planned migration.

### Documentation Completeness

- **Status:** PASS ✅
- **Threshold:** Public API documented; error variants documented
- **Actual:** All public functions and types have doc comments; error variants include hint text
- **Evidence:** `crates/tf-logging/src/lib.rs`, `crates/tf-logging/src/error.rs`
- **Findings:** Documentation is comprehensive for the tf-logging crate. Story file documents 8 rounds of code review. Test design covers 14 scenarios with risk assessment.

### Test Quality (from test-review)

- **Status:** CONCERNS ⚠️
- **Threshold:** Test quality score >= 85/100
- **Actual:** 81/100 (B) overall; maintainability 45/100
- **Evidence:** `_bmad-output/test-review.md`
- **Findings:** Good test quality overall but maintainability is the weak point. Key issues: test monolith in tf-config (3231 lines), 80+ duplicated patterns, no test helper extraction. tf-logging tests are well-structured (thread-local dispatch, tempdir isolation).

---

## Custom NFR Assessments

### Sensitive Data Redaction (NFR4 - Security Core)

- **Status:** PASS ✅
- **Threshold:** All 12 sensitive field names + compound variants redacted in log output
- **Actual:** 12 base fields (`token`, `password`, `api_key`, `secret`, `auth`, `authorization`, `credential`, `credentials`, `passwd`, `pwd`, `apikey`, `key`) + 26 compound suffixes (`_token`, `_password`, etc.) all redacted to `[REDACTED]`
- **Evidence:** `crates/tf-logging/src/formatter.rs` — exhaustive tests for each field name; negative tests for normal fields
- **Findings:** Core security requirement fully met. The `RedactingJsonFormatter` intercepts all fields before JSON serialization. URL parameters with sensitive names are also redacted. This was validated through P0 tests (0.5-UNIT-003, 0.5-UNIT-004) and integration test (0.5-INT-001).

### Non-Blocking I/O Architecture

- **Status:** PASS ✅
- **Threshold:** Logging must not block CLI execution
- **Actual:** `tracing_appender::non_blocking` wraps both file and stdout layers; `LogGuard` (RAII) ensures flush on drop
- **Evidence:** `crates/tf-logging/src/lib.rs` — non-blocking wrapping, `LogGuard` struct
- **Findings:** Architecture validated by test 0.5-UNIT-001 (lifecycle) and 0.5-UNIT-005 (file output). The `WorkerGuard` pattern ensures no log loss at program exit.

---

## Quick Wins

3 quick wins identified for immediate implementation:

1. **Install cargo-audit** (Security) - HIGH - 5 minutes
   - Run `cargo install cargo-audit && cargo audit`
   - No code changes needed

2. **Add GitHub Actions CI** (Reliability) - HIGH - 30 minutes
   - Create `.github/workflows/ci.yml` with `cargo test --workspace`, `cargo clippy`, `cargo fmt --check`
   - Minimal configuration needed for Rust workspace

3. **Add basic timing assertion** (Performance) - MEDIUM - 30 minutes
   - Add `#[test]` that measures `init_logging` duration with `std::time::Instant`
   - Assert < 100ms for initialization
   - No external benchmark crate needed

---

## Recommended Actions

### Immediate (Before Next Epic) - HIGH Priority

1. **Install cargo-audit for dependency scanning** - HIGH - 5 min - Dev
   - `cargo install cargo-audit && cargo audit`
   - Run before each release/PR merge
   - Validation: `cargo audit` returns 0 critical/high vulnerabilities

2. **Set up GitHub Actions CI pipeline** - HIGH - 30 min - Dev
   - Create workflow: `cargo test --workspace` + `cargo clippy` + `cargo fmt --check`
   - Trigger on push and PR to main
   - Validation: Green CI badge on repository

### Short-term (Next Sprint) - MEDIUM Priority

3. **Add criterion benchmarks for logging operations** - MEDIUM - 2 hours - Dev
   - Benchmark `init_logging`, log event emission, and redaction overhead
   - Establish baseline for NFR8 (CLI < 2s)

4. **Split tf-config test monolith** - MEDIUM - 4 hours - Dev
   - Break 3231-line test file into domain-specific modules (validation, redaction, loading, defaults)
   - Extract shared test utilities into test helper module

### Long-term (Backlog) - LOW Priority

5. **Migrate from serde_yaml (deprecated)** - LOW - 4 hours - Dev
   - Evaluate alternatives: `yaml-rust2`, TOML migration, or `serde_yml`
   - Affects tf-config crate

6. **Implement log retention/purge for NFR4** - LOW - 4 hours - Dev
   - Add configurable retention period (default 90 days)
   - Automatic cleanup of old log files on `init_logging`

7. **Extract duplicated test patterns into utilities** - LOW - 3 hours - Dev
   - Address 80+ duplicated setup/assertion patterns across test suite
   - Create `test_utils` module with builder helpers

---

## Monitoring Hooks

3 monitoring hooks recommended for ongoing quality:

### Dependency Security

- [ ] cargo-audit in CI — Run `cargo audit` on every PR to detect vulnerable dependencies
  - **Owner:** Dev
  - **Deadline:** Before Epic 1

### Test Stability

- [ ] CI burn-in — Run `cargo test --workspace` 5x on PR merge to detect flaky tests
  - **Owner:** Dev
  - **Deadline:** Sprint 1

### Code Quality Regression

- [ ] Clippy strict mode — `cargo clippy -- -D warnings` enforced in CI
  - **Owner:** Dev
  - **Deadline:** Before Epic 1

---

## Fail-Fast Mechanisms

2 fail-fast mechanisms recommended:

### Validation Gates (Security)

- [ ] Pre-commit hook: `cargo clippy -- -D warnings && cargo fmt -- --check`
  - **Owner:** Dev
  - **Estimated Effort:** 15 minutes

### Smoke Tests (Maintainability)

- [ ] CI smoke test: `cargo test --workspace --lib` (fast unit tests only, < 30s) on every push
  - **Owner:** Dev
  - **Estimated Effort:** 15 minutes

---

## Evidence Gaps

4 evidence gaps identified — action required:

- [ ] **Dependency vulnerability scan** (Security)
  - **Owner:** Dev
  - **Deadline:** Before Epic 1
  - **Suggested Evidence:** `cargo audit` output saved as CI artifact
  - **Impact:** Cannot confirm 0 known vulnerabilities in dependency tree

- [ ] **Performance benchmarks** (Performance)
  - **Owner:** Dev
  - **Deadline:** Sprint 1
  - **Suggested Evidence:** criterion benchmark results for `init_logging` and log event throughput
  - **Impact:** Cannot validate NFR8 (CLI < 2s) quantitatively

- [ ] **Line/branch coverage report** (Maintainability)
  - **Owner:** Dev
  - **Deadline:** Sprint 1
  - **Suggested Evidence:** `cargo tarpaulin` or `cargo llvm-cov` report
  - **Impact:** 417 tests exist but actual coverage percentage unknown

- [ ] **CI pipeline configuration** (Reliability)
  - **Owner:** Dev
  - **Deadline:** Before Epic 1
  - **Suggested Evidence:** `.github/workflows/ci.yml` with green runs
  - **Impact:** No automated regression detection on push/PR

---

## Findings Summary

**Based on ADR Quality Readiness Checklist (8 categories, 29 criteria)**

| Category                                         | Criteria Met | PASS | CONCERNS | FAIL | Overall Status   |
| ------------------------------------------------ | ------------ | ---- | -------- | ---- | ---------------- |
| 1. Testability & Automation                      | 3/4          | 3    | 1        | 0    | CONCERNS ⚠️      |
| 2. Test Data Strategy                            | 3/3          | 3    | 0        | 0    | PASS ✅           |
| 3. Scalability & Availability                    | N/A          | N/A  | N/A      | N/A  | N/A (CLI tool)   |
| 4. Disaster Recovery                             | N/A          | N/A  | N/A      | N/A  | N/A (CLI tool)   |
| 5. Security                                      | 3/5          | 3    | 2        | 0    | CONCERNS ⚠️      |
| 6. Monitorability, Debuggability & Manageability | 3/4          | 3    | 1        | 0    | CONCERNS ⚠️      |
| 7. QoS & QoE                                     | 3/4          | 3    | 1        | 0    | CONCERNS ⚠️      |
| 8. Deployability                                 | 3/3          | 3    | 0        | 0    | PASS ✅           |
| **Total**                                        | **18/23**    | **18** | **5**  | **0** | **CONCERNS ⚠️** |

**Criteria Met Scoring:**

- ≥21/23 (90%+) = Strong foundation
- 16-20/23 (70-87%) = Room for improvement ← **18/23 = 78%**
- <16/23 (<70%) = Significant gaps

*Note: 6 criteria in categories 3-4 excluded as N/A for CLI tool. Scoring adjusted to 23 applicable criteria.*

---

## Gate YAML Snippet

```yaml
nfr_assessment:
  date: '2026-02-07'
  story_id: '0-5'
  feature_name: 'Journalisation baseline sans donnees sensibles'
  adr_checklist_score: '18/23'
  categories:
    testability_automation: 'CONCERNS'
    test_data_strategy: 'PASS'
    scalability_availability: 'N/A'
    disaster_recovery: 'N/A'
    security: 'CONCERNS'
    monitorability: 'CONCERNS'
    qos_qoe: 'CONCERNS'
    deployability: 'PASS'
  overall_status: 'CONCERNS'
  critical_issues: 0
  high_priority_issues: 3
  medium_priority_issues: 4
  concerns: 5
  blockers: false
  quick_wins: 3
  evidence_gaps: 4
  recommendations:
    - 'Install cargo-audit for dependency vulnerability scanning'
    - 'Set up GitHub Actions CI pipeline (cargo test + clippy + fmt)'
    - 'Add performance benchmarks before CLI integration (Epic 1)'
```

---

## Related Artifacts

- **Story File:** `_bmad-output/implementation-artifacts/0-5-journalisation-baseline-sans-donnees-sensibles.md`
- **PRD:** `_bmad-output/planning-artifacts/prd.md` (FR30, NFR4, NFR8)
- **Architecture:** `_bmad-output/planning-artifacts/architecture.md` (tf-logging, tracing stack)
- **Test Design:** `_bmad-output/test-artifacts/test-design/test-design-epic-0-5.md`
- **Test Review:** `_bmad-output/test-review.md` (81/100, B)
- **Evidence Sources:**
  - Test Results: `cargo test --workspace` (417 passed, 0 failed, 18 ignored)
  - Code Quality: `cargo clippy --workspace --all-targets -- -D warnings` (clean)
  - Source Code: `crates/tf-logging/src/` (formatter.rs, lib.rs, error.rs)

---

## Recommendations Summary

**Release Blocker:** None — no FAIL status in any category

**High Priority:** Install cargo-audit (5 min), set up CI pipeline (30 min), address serde_yaml deprecation (backlog)

**Medium Priority:** Add performance benchmarks, split tf-config test monolith, implement log retention

**Next Steps:** Address the 2 immediate actions (cargo-audit + CI), then proceed to Epic 1. Re-run `*nfr-assess` after CI is operational to upgrade Testability and Reliability categories from CONCERNS to PASS.

---

## Sign-Off

**NFR Assessment:**

- Overall Status: CONCERNS ⚠️
- Critical Issues: 0
- High Priority Issues: 3
- Concerns: 5
- Evidence Gaps: 4

**Gate Status:** CONDITIONAL PASS ⚠️

**Next Actions:**

- CONCERNS ⚠️: Address HIGH priority issues (cargo-audit + CI pipeline), then re-run `*nfr-assess`
- The 4 CONCERNS categories have clear, actionable remediation paths
- Core functionality (logging, redaction, error handling) is production-ready
- Infrastructure gaps (CI, scanning, benchmarks) are expected for Sprint 0

**Generated:** 2026-02-07
**Workflow:** testarch-nfr v4.0

---

<!-- Powered by BMAD-CORE™ -->

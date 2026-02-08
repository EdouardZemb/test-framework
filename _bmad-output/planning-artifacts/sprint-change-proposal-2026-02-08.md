# Sprint Change Proposal - Test Quality Remediation

**Date:** 2026-02-08
**Trigger:** TEA/QA workflow recommendations post-Story 0-5
**Scope Classification:** Minor
**Status:** Approved (2026-02-08)
**Approved By:** Edouard

---

## Section 1: Issue Summary

### Problem Statement

The TEA quality workflows (test-review, traceability-trace, nfr-assess) executed after Story 0-5 completion identified significant test maintainability debt in the tf-config crate test suite. The primary issues are:

1. **Monolithic test module** (3231 lines, 211 tests) — 10x the recommended 300-line threshold
2. **Extreme copy-paste duplication** — 80+ tests follow identical YAML-load-assert-error pattern without parameterization
3. **Review-round organization** — Tests organized by AI review round numbers ("Review 5", "Review 12") instead of functional grouping (URL validation, path traversal, serde errors)

### Discovery Context

- **When:** 2026-02-07/08, after Story 0-5 merge
- **How:** Automated TEA workflows: `testarch-test-review`, `testarch-trace`, `testarch-nfr`
- **Story:** 0-5 "Journalisation baseline sans donnees sensibles" (status: done)

### Evidence

| Metric | Value | Threshold | Status |
|--------|-------|-----------|--------|
| Test Review Score | 81/100 (B) | >= 85 | Below target |
| Maintainability Score | 45/100 (F) | >= 70 | Critical |
| NFR Assessment | 18/23 (78%) | >= 90% | CONCERNS |
| High Priority Issues | 3 | 0 | Action needed |
| Evidence Gaps | 4 | 0 | Action needed |

**Source artifacts:**
- `_bmad-output/test-review.md` (Test Quality Review, 2026-02-07)
- `_bmad-output/traceability-matrix.md` (Traceability & Gate Decision, 2026-02-08)
- `_bmad-output/nfr-assessment.md` (NFR Assessment, 2026-02-07)

---

## Section 2: Impact Analysis

### Epic Impact

| Epic | Status | Impact | Detail |
|------|--------|--------|--------|
| **Epic 0: Foundation & Access** | in-progress | **Direct** | Insert technical story between 0-5 (done) and 0-6 (backlog) |
| Epic 1: Triage & readiness | backlog | Indirect | Benefits from improved test patterns for tf-connectors |
| Epic 7: Conformite & securite | backlog | Indirect | serde_yaml deprecation to address before stabilization |
| Epics 2-6 | backlog | None | No impact |

### Story Impact

**Current stories affected:**
- Story 0-6 (backlog): Delayed by ~1 day while technical story is executed
- Stories 0-1 through 0-5 (done): No retroactive changes to functionality

**New story required:**
- Story 0-5b: "Refactor tf-config test suite for maintainability" (see Section 4 for details)

### Artifact Conflicts

| Artifact | Conflict | Action Required |
|----------|----------|-----------------|
| PRD (`prd.md`) | None | No changes |
| Architecture (`architecture.md`) | None | No changes |
| UX Design | N/A | No changes |
| Epics (`epics.md`) | Minor | Add technical story to Epic 0 |
| Sprint Status (`sprint-status.yaml`) | Minor | Add story entry |
| CI Pipeline (`.github/workflows/test.yml`) | None now | Coverage step added later (backlog) |
| Test Design docs | None | Test behavior unchanged, only structure |

### Technical Impact

- **Code changes:** Test files only (`#[cfg(test)]` modules). Zero production code changes.
- **Risk:** Very low — `cargo test --workspace` validates at every step
- **Infrastructure:** None
- **Deployment:** None

---

## Section 3: Recommended Approach

### Selected Path: Direct Adjustment

Insert a technical story into Epic 0 to address P0/P1 test quality findings before proceeding with Story 0-6.

### Rationale

1. **Prevents debt accumulation:** Story 0-6 (checklist + scoring config) will add more tests to tf-config — better to clean up before adding more
2. **Contained effort:** ~1 day of work, well-defined scope (4 items)
3. **Near-zero risk:** Test refactoring only, validated by existing passing test suite (417 tests)
4. **Measurable improvement:** Maintainability 45/100 -> estimated 75-80/100
5. **No disruption:** Natural insertion point between completed and upcoming stories

### Alternatives Considered

| Option | Assessment | Reason for Rejection |
|--------|------------|---------------------|
| Rollback | Not viable | Tests work correctly; issue is structure, not correctness |
| PRD MVP Review | Not viable | Disproportionate — no functional impact |
| Do nothing | Risky | Debt worsens with every future story touching tf-config |
| Defer to post-Epic 0 | Suboptimal | 0-6 and 0-7 will add more tests, making refactoring harder |

### Effort Estimate

- **Story technical scope:** ~1 day (P0/P1 items)
- **Risk level:** Low
- **Timeline impact:** Minimal — 1 day delay on Story 0-6 start

---

## Section 4: Detailed Change Proposals

### Proposal 1: Split monolithic test module (P0)

```
Story: tf-config test suite
Section: crates/tf-config/src/config.rs (lines 2003-5233)

OLD:
#[cfg(test)]
mod tests {
    // 3231 lines, 211 tests
    // === REVIEW 5 TESTS ===
    // === REVIEW 6 TESTS ===
    // ... (18+ review-round sections)
}

NEW:
#[cfg(test)]
mod tests {
    mod url_validation;        // ~50 tests: URL scheme, IPv6, whitespace
    mod path_validation;       // ~30 tests: traversal, null bytes, formats
    mod serde_errors;          // ~40 tests: type errors, missing fields
    mod llm_config;            // ~25 tests: cloud mode, local mode, defaults
    mod redact_url;            // ~30 tests: URL parameter redaction
    mod config_loading;        // ~15 tests: load_config, fixtures
    mod profile_summary;       // ~10 tests: active_profile_summary, check_output_folder
    mod helpers;               // create_temp_config, common assertions
}

Rationale: Navigate, understand, and maintain tests by functional area
instead of by when they were written. Each sub-module stays under 500 lines.
```

### Proposal 2: Extract parameterized test macro (P0)

```
Story: tf-config test suite
Section: crates/tf-config/src/config.rs (80+ duplicated tests)

OLD (repeated 80+ times):
#[test]
fn test_url_scheme_only_rejected() {
    let yaml = "project_name: \"test\"\noutput_folder: \"./out\"\njira:\n  endpoint: \"http://\"";
    let result = load_config(&create_temp_config(yaml));
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("URL"), "Expected URL error: {}", err);
}

NEW:
macro_rules! test_config_rejects {
    ($name:ident, $yaml:expr, $($expected:expr),+) => {
        #[test]
        fn $name() {
            let result = load_config(&create_temp_config($yaml));
            assert!(result.is_err(), "Expected rejection for {}", stringify!($name));
            let err = result.unwrap_err().to_string();
            $(
                assert!(err.contains($expected),
                    "Error should contain '{}': got '{}'", $expected, err);
            )+
        }
    };
}

test_config_rejects!(test_url_scheme_only_rejected,
    "project_name: \"test\"\noutput_folder: \"./out\"\njira:\n  endpoint: \"http://\"",
    "URL"
);
// ... ~80 more invocations

Rationale: Eliminates ~2000 lines of boilerplate. Pattern already proven
in tf-logging/redact.rs (test_sensitive_field_redacted! macro).
```

### Proposal 3: Reorganize by functionality (P1)

```
Story: tf-config test suite
Section: All test section headers

OLD:
// === REVIEW 5 TESTS ===
// === REVIEW 6 TESTS: IPv6 URL validation ===
// === REVIEW 12 TESTS: Boolean type errors, URL sensitive params ===

NEW:
// === URL Validation ===
// === Path Traversal Protection ===
// === Cloud Mode Requirements ===
// === Serde Error Messages ===

Rationale: Developers can find all tests for a feature in one place.
Currently URL validation tests are scattered across Reviews 5, 6, 9, 12, 13, 14, 18, 22, 23.
```

### Proposal 4: Normalize temp file management (P1)

```
Story: tf-config test suite
Section: crates/tf-config/src/config.rs (6 whitespace endpoint tests, lines ~4778-4905)

OLD:
let path = std::env::temp_dir().join("test_ws_endpoint.yaml");
std::fs::write(&path, yaml).unwrap();
let result = load_config(&path);
std::fs::remove_file(&path).ok();

NEW:
let path = create_temp_config(yaml);
let result = load_config(&path);
// cleanup handled automatically by tempfile

Rationale: Manual cleanup not guaranteed on panic.
create_temp_config() helper already used by 200+ other tests.
```

### Follow-up Items (P2-P3, not in technical story)

| # | Item | Priority | Timing |
|---|------|----------|--------|
| 5 | Extract `test_logging_config()` helper in tf-logging | P2 | Next story |
| 6 | Replace timestamp with AtomicU64 in `unique_key()` | P2 | Next story |
| 7 | Add NaN/Infinity test for `record_f64` | P2 | Next story |
| 8 | Configure cargo-tarpaulin for coverage measurement | P2 | Backlog |
| 9 | RAII `KeyGuard` for keyring tests | P3 | Backlog |
| 10 | Migrate from serde_yaml (deprecated) | P3 | Pre-Epic 7 |
| 11 | Implement log retention/purge (NFR4) | P3 | Story 7-2 |

---

## Section 5: Implementation Handoff

### Change Scope: Minor

Direct implementation by development team. No backlog reorganization or architectural replan needed.

### Handoff Plan

| Role | Agent | Responsibility |
|------|-------|----------------|
| **SM** | `bmad-agent-bmm-sm` | Create story file, update sprint-status.yaml, update epics.md |
| **Dev** | `bmad-agent-bmm-dev` | Execute technical story (Proposals 1-4) |
| **TEA** | `bmad-agent-tea-tea` | Re-run `testarch-test-review` post-refactoring to validate improvement |

### Implementation Sequence

1. SM creates story file with acceptance criteria derived from Proposals 1-4
2. Dev executes story using `dev-story` workflow
3. Dev runs `cargo test --workspace` to validate 0 regressions
4. TEA runs `testarch-test-review` to measure new maintainability score
5. SM updates sprint-status.yaml to mark story as done

### Success Criteria

| Criterion | Target | Measurement |
|-----------|--------|-------------|
| Test regressions | 0 | `cargo test --workspace` — all 417+ tests pass |
| Max sub-module size | < 500 lines | Line count per test sub-module |
| Maintainability score | >= 70/100 (target 80) | TEA `testarch-test-review` re-run |
| Macro coverage | >= 80% of validation tests | Count of `test_config_rejects!` invocations vs total validation tests |
| clippy clean | 0 warnings | `cargo clippy --workspace -- -D warnings` |

### Definition of Done

- [ ] config.rs test module split into functional sub-modules
- [ ] `test_config_rejects!` macro extracted and applied to duplicated tests
- [ ] Test headers reorganized by functionality
- [ ] 6 whitespace endpoint tests normalized to `create_temp_config()`
- [ ] `cargo test --workspace` passes with 0 regressions
- [ ] `cargo clippy` passes clean
- [ ] TEA test-review re-run confirms maintainability improvement

---

## Appendix: Recommendation Traceability

| # | Recommendation | Source Document | Source Section | Proposal |
|---|---------------|-----------------|----------------|----------|
| 1 | Split monolithic test module | test-review.md | Critical Issue #1 | Proposal 1 |
| 2 | Extract parameterized test macro | test-review.md | Critical Issue #2 | Proposal 2 |
| 3 | Reorganize by functionality | test-review.md | Critical Issue #3 | Proposal 3 |
| 4 | Normalize temp file management | test-review.md | Recommendation #4 | Proposal 4 |
| 5 | Extract test_logging_config helper | test-review.md | Recommendation #1 | Follow-up P2 |
| 6 | Replace timestamp with AtomicU64 | test-review.md | Recommendation #2 | Follow-up P2 |
| 7 | Add NaN/Infinity record_f64 test | test-review.md | Recommendation #3 | Follow-up P2 |
| 8 | Configure coverage measurement | nfr-assessment.md | Evidence Gap #3 | Follow-up P2 |
| 9 | RAII KeyGuard for keyring | test-review.md | Recommendation #5 | Follow-up P3 |
| 10 | Migrate serde_yaml | nfr-assessment.md | Recommendation #5 | Follow-up P3 |
| 11 | Log retention/purge NFR4 | nfr-assessment.md | Recommendation #6 | Story 7-2 |

---

**Generated:** 2026-02-08
**Workflow:** correct-course (BMad Method)
**Trigger:** TEA quality workflow recommendations post-Story 0-5

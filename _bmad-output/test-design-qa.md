# Test Design for QA: QA Tester Process Optimization (TRA)

**Purpose:** Test execution recipe for QA team. Defines what to test, how to test it, and what QA needs from other teams.

**Date:** 2026-02-06
**Author:** Edouard Zemb (via TEA Test Architect)
**Status:** Draft
**Project:** test-framework

**Related:** See Architecture doc (test-design-architecture.md) for testability concerns and architectural blockers.

---

## Executive Summary

**Scope:** Complete test coverage for test-framework Rust CLI tool: 10 crates, 31 FRs, 16 NFRs, 8 epics across config, security, connectors, storage, LLM, export, core pipeline, and CLI.

**Risk Summary:**

- Total Risks: 12 (4 high-priority score >=6, 6 medium, 2 low)
- Critical Categories: SEC (anonymization, secrets), TECH (mockability), OPS (proxy fallback)

**Coverage Summary:**

- P0 tests: ~12 (security, anonymization, fallback CSV, LLM local-only)
- P1 tests: ~28 (config, logging, storage, connectors, LLM, export, core pipeline, CLI)
- P2 tests: ~5 (cache, grouping, template guards, benchmarks, diagnostics)
- P3 tests: ~1 (CLI cold start benchmark)
- **Total**: ~46 tests (~3-4 weeks with 1 QA)

---

## Dependencies & Test Blockers

**CRITICAL:** QA cannot proceed without these items from other teams.

### Backend/Architecture Dependencies (Sprint 0)

**Source:** See Architecture doc "Quick Guide" for detailed mitigation plans

1. **Trait abstractions for connectors (R-03)** - Dev - Sprint 0
   - QA needs `MockJiraClient`, `MockSquashClient`, `MockOllamaClient` via mockall
   - Blocks all connector and pipeline integration tests

2. **Test data seeding module (C1)** - Dev - Sprint 0
   - QA needs `tf-storage/test_helpers.rs` with `seed_tickets()`, `seed_config()` functions
   - Blocks integration tests requiring specific SQLite states

3. **PII verification scanner (R-01)** - Dev - Sprint 0
   - QA needs callable PII pattern scanner in tf-security
   - Blocks canary tests for anonymization validation

### QA Infrastructure Setup (Sprint 0)

1. **Test Helper Module** - QA
   - `tests/common/mod.rs` with shared test fixtures
   - Temporary SQLite database per test (parallel-safe)
   - Config builder for test scenarios

2. **Test Environments** - QA
   - Local: `cargo test` with in-memory/temp SQLite
   - CI/CD: GitHub Actions with `cargo test` + `cargo clippy`
   - No staging needed (single-user CLI)

**Example test pattern (Rust):**

```rust
// tests/common/mod.rs
use tf_config::Config;
use tf_storage::test_helpers::TestDb;

pub fn test_config() -> Config {
    Config::from_str(r#"
        project_name: "test-project"
        profile: "default"
        output_folder: "/tmp/test-output"
    "#).unwrap()
}

pub fn test_db() -> TestDb {
    TestDb::new_temp() // Auto-cleanup on drop
}

// tests/unit/config_test.rs
#[test]
fn valid_config_loads_all_fields() {
    let config = Config::from_file("tests/fixtures/valid_config.yaml").unwrap();
    assert_eq!(config.project_name, "test-project");
    assert!(config.output_folder.exists());
}

#[test]
fn invalid_config_returns_structured_error() {
    let err = Config::from_file("tests/fixtures/missing_field.yaml").unwrap_err();
    assert_eq!(err.code(), "VALIDATION_ERROR");
    assert!(err.message().contains("project_name"));
    assert!(err.hint().contains("add project_name"));
}
```

---

## Risk Assessment

**Note:** Full risk details in Architecture doc. This section summarizes risks relevant to QA test planning.

### High-Priority Risks (Score >=6)

| Risk ID | Category | Description | Score | QA Test Coverage |
|---------|----------|-------------|-------|-----------------|
| **R-01** | SEC | Fuite PII vers cloud | **6** | SEC-INT-001: canary test injectant PII connues, verifiant masquage |
| **R-02** | SEC | Secrets en clair | **6** | SEC-INT-002: scan repo/config + tests unitaires redact |
| **R-03** | TECH | Connecteurs non mockables | **6** | CON-UNIT-*: tous les tests connectors via mocks |
| **R-06** | OPS | Proxy bloque APIs | **6** | CON-INT-001: test fallback CSV end-to-end |

### Medium/Low-Priority Risks

| Risk ID | Category | Description | Score | QA Test Coverage |
|---------|----------|-------------|-------|-----------------|
| R-04 | DATA | Corruption SQLite | 4 | STO-INT-001: crash mid-write recovery |
| R-07 | BUS | Templates non alignes | 4 | EXP-INT-001: golden file Excel |
| R-08 | TECH | OOXML fragile | 4 | EXP-INT-002: golden file PPT |
| R-09 | SEC | Keyring indisponible CI | 4 | SEC-UNIT-006: mock keyring |
| R-10 | DATA | Mapping divergents | 4 | CON-UNIT-006: validation stricte |
| R-11 | OPS | Purge ne s'execute pas | 4 | STO-UNIT-003: test TTL/purge |
| R-05 | PERF | CLI >2s | 2 | CLI-E2E-005: benchmark |
| R-12 | TECH | Ollama indisponible | 2 | LLM-UNIT-004: erreur explicite |

---

## Test Coverage Plan

**IMPORTANT:** P0/P1/P2/P3 = **priority and risk level** (what to focus on if time-constrained), NOT execution timing. See "Execution Strategy" for when tests run.

### P0 (Critical)

**Criteria:** Blocks core functionality + High risk (>=6) + No workaround + Security/compliance critical

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| **SEC-UNIT-001** | NFR1: Anonymisation emails/noms/tokens | Unit | R-01 | Regex patterns PII |
| **SEC-UNIT-003** | NFR1: Anonymisation echoue -> envoi bloque | Unit | R-01 | Fail-safe |
| **SEC-UNIT-004** | NFR4: Logs ne contiennent aucun PII | Unit | R-02 | Redact patterns |
| **SEC-INT-001** | NFR1: Pipeline complet donnee -> anonymize -> cloud mock -> sortie masquee | Integration | R-01 | Test canary |
| **SEC-INT-002** | NFR2: Scan repo/config 0 secret en clair | Integration | R-02 | CI scan |
| **CON-UNIT-004** | NFR9: CSV fallback equivalent fonctionnel | Unit | R-06 | Mode degrade |
| **CON-INT-001** | NFR9: API Jira indisponible -> fallback CSV transparent | Integration | R-06 | End-to-end fallback |
| **LLM-UNIT-002** | NFR1: Mode cloud anonymisation appelee avant envoi | Unit | R-01 | Pre-envoi check |
| **LLM-UNIT-005** | FR29: Mode local uniquement 0 requete cloud | Unit | - | Epic 7 compliance |
| **LOG-UNIT-002** | NFR4: Champs sensibles masques dans logs | Unit | R-02 | ASR-4 |
| **CLI-E2E-004** | NFR9: API indisponible -> degrade CSV -> exit 0 + warning | E2E | R-06 | Journey complet |

**Total P0:** ~11 tests

---

### P1 (High)

**Criteria:** Core user journeys + Medium risk + Frequent usage

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| **CFG-UNIT-001** | FR23: Config.yaml valide -> struct Config | Unit | - | Fondation |
| **CFG-UNIT-002** | FR23: Config invalide -> erreur structuree | Unit | - | UX erreur |
| **CFG-UNIT-003** | FR24: Selection profil existant | Unit | - | Multi-profil |
| **CFG-UNIT-004** | FR24: Profil inconnu -> erreur + liste | Unit | - | UX erreur |
| **CFG-UNIT-005** | FR25: Chargement template existant | Unit | - | Templates |
| **CFG-UNIT-006** | FR25: Template manquant -> erreur | Unit | - | Guard |
| **CFG-UNIT-007** | FR2/3: Checklist + regles scoring | Unit | - | Prerequis triage |
| **CFG-UNIT-008** | FR2/3: Checklist manquante -> erreur | Unit | - | Guard |
| **CFG-INT-001** | Config + profil -> pipeline init | Integration | - | Chaine init |
| **SEC-UNIT-002** | Pas de double anonymisation | Unit | - | Correctness |
| **SEC-UNIT-005** | FR31: Store/retrieve secret keyring | Unit | R-09 | Keyring |
| **SEC-UNIT-006** | Keyring indisponible -> erreur | Unit | R-09 | Fallback |
| **LOG-UNIT-001** | FR30: Log JSON structure genere | Unit | - | Baseline |
| **LOG-UNIT-003** | FR30: Logs dans dossier configure | Unit | - | Output path |
| **LOG-INT-001** | Execution CLI -> log valide JSON sans PII | Integration | - | Bout-en-bout |
| **STO-UNIT-001** | CRUD SQLite avec transactions | Unit | R-04 | Fondation |
| **STO-UNIT-002** | Chiffrement applicatif au repos | Unit | - | Security |
| **STO-UNIT-003** | NFR5: TTL/purge donnees >24h | Unit | R-11 | Purge |
| **STO-INT-001** | Crash mid-write -> recovery | Integration | R-04 | Resilience |
| **STO-INT-002** | NFR10: Replay derniere execution | Integration | - | Reprise |
| **CON-UNIT-001** | FR1: Import Jira -> struct Ticket[] | Unit (mock) | - | Core |
| **CON-UNIT-002** | Jira token invalide -> erreur + hint | Unit | - | UX |
| **CON-UNIT-003** | FR12/16: Squash read campagnes | Unit (mock) | - | Core |
| **CON-UNIT-005** | Retry exponentiel 3 tentatives | Unit | - | Pattern |
| **CON-UNIT-006** | Validation stricte mapping | Unit | R-10 | Data integrity |
| **CORE-UNIT-001** | FR2: Checklist testabilite | Unit | - | Triage |
| **CORE-UNIT-002** | FR3: Scoring Go/Clarify/No-Go | Unit | - | Triage |
| **CORE-UNIT-003** | FR4: Marquage clarification + cause | Unit | - | Tracabilite |
| **CORE-UNIT-005** | FR14/15: Brouillon anomalie | Unit | - | Template |
| **CORE-INT-001** | Pipeline triage: import -> checklist -> score | Integration | - | Journey 1 |
| **CORE-INT-002** | NFR6: Pipeline CR < 2 min | Integration | - | Performance |
| **EXP-UNIT-001** | FR18/22: Export Excel CR valide | Unit | R-07 | Reporting |
| **EXP-UNIT-002** | FR20/21: Export PPT OOXML | Unit | R-08 | Reporting |
| **EXP-UNIT-003** | FR22: Multi-formats json/csv/md/html/yaml | Unit | - | Exports |
| **EXP-INT-001** | Golden file Excel | Integration | R-07 | Validation |
| **EXP-INT-002** | Golden file PPT ZIP/XML | Integration | R-08 | Validation |
| **LLM-UNIT-001** | FR29: Mode local Ollama mock | Unit | - | LLM |
| **LLM-UNIT-003** | Mode auto: local OK -> local, local KO -> cloud | Unit | - | Orchestrator |
| **LLM-UNIT-004** | Ollama indisponible + cloud disabled -> erreur | Unit | R-12 | UX |
| **CLI-E2E-001** | tf triage -> sortie structuree exit 0 | E2E | - | Journey 1 |
| **CLI-E2E-002** | tf report -> fichier genere exit 0 | E2E | - | Journey 1 |
| **CLI-E2E-003** | Config invalide -> exit 2 + message | E2E | - | UX erreur |

**Total P1:** ~31 tests

---

### P2 (Medium)

**Criteria:** Secondary features + Low risk + Edge cases

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| **STO-UNIT-004** | Cache hit/miss/expiration | Unit | - | Performance |
| **CORE-UNIT-004** | FR5: Regroupement tickets lot/perimetre | Unit | - | Secondary |
| **EXP-UNIT-004** | Template PPT manquant -> erreur | Unit | - | Guard |
| **CON-INT-002** | NFR7: Import perimetre < 5 min | Integration | - | Benchmark |
| **CLI-E2E-006** | tf diagnostics -> rapport sans PII | E2E | - | Ops |

**Total P2:** ~5 tests

---

### P3 (Low)

**Criteria:** Nice-to-have + Benchmarks

| Test ID | Requirement | Test Level | Notes |
|---------|-------------|------------|-------|
| **CLI-E2E-005** | NFR8: Commande simple < 2s | E2E (benchmark) | Cold start monitoring |

**Total P3:** ~1 test

---

## Execution Strategy

**Philosophy:** Run everything in PRs unless there's significant infrastructure overhead. Cargo test with parallel execution is fast (100+ tests in <5 min for Rust unit tests).

**Organized by TOOL TYPE:**

### Every PR: cargo test (~5-10 min)

**All functional tests** (from any priority level):

- All unit tests (`cargo test --lib`)
- All integration tests (`cargo test --test '*'`)
- Clippy + format check (`cargo clippy && cargo fmt --check`)
- Parallelized natively by cargo (per-crate + per-test)
- Total: ~46 tests (includes P0, P1, P2)

**Why run in PRs:** Rust tests are fast, no external infrastructure needed (all mocked)

### Nightly: E2E + Golden Files (~15-30 min)

**Full CLI E2E tests** (from any priority level):

- CLI end-to-end via `assert_cmd` (tf triage, tf report, tf diagnostics)
- Golden file comparisons (Excel, PPT structure)
- Total: ~8 tests (CLI-E2E-*, EXP-INT-*)

**Why defer to nightly:** File I/O intensive, longer setup, golden file comparisons

### Weekly: Benchmarks + Chaos (~30-60 min)

**Performance and resilience tests**:

- CLI cold start benchmark (< 2s)
- Pipeline performance (CR < 2min, import < 5min)
- Fault injection tests (--simulate-failure)
- Purge 24h validation (requires time manipulation)

**Why defer to weekly:** Benchmarks need stable environment, fault injection requires special setup

---

## QA Effort Estimate

**QA test development effort only:**

| Priority | Count | Effort Range | Notes |
|----------|-------|-------------|-------|
| P0 | ~11 | ~2-3 weeks | Complex setup (PII canary, fallback chains, security) |
| P1 | ~31 | ~3-5 weeks | Standard unit/integration (config, storage, connectors, core) |
| P2 | ~5 | ~2-4 days | Edge cases, benchmarks |
| P3 | ~1 | ~0.5-1 day | Single benchmark |
| **Total** | **~48** | **~6-9 weeks** | **1 QA engineer, full-time** |

**Assumptions:**

- Includes test design, implementation, debugging, CI integration
- Excludes ongoing maintenance (~10% effort)
- Assumes test infrastructure (trait mocks, test helpers) ready from Sprint 0
- P0 effort front-loaded in Sprint 0; P1 spread across Sprint 0-2

---

## Appendix A: Code Examples & Tagging

**Rust Test Tagging for Selective Execution:**

```rust
// P0 critical test - security
#[test]
fn anonymize_masks_email_patterns() {
    let input = "Contact user@example.com for details";
    let result = anonymize(input);
    assert!(!result.contains("user@example.com"));
    assert!(result.contains("[EMAIL_REDACTED]"));
}

// P0 critical test - fallback
#[test]
fn jira_unavailable_triggers_csv_fallback() {
    let mock_client = MockJiraClient::new();
    mock_client.expect_import().returning(|_| Err(ConnectorError::Unavailable));

    let result = import_tickets(&mock_client, &csv_fallback_path);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().source, DataSource::CsvFallback);
}

// P1 integration test - pipeline
#[test]
fn triage_pipeline_produces_scored_tickets() {
    let db = TestDb::new_temp();
    let config = test_config();
    let mock_jira = MockJiraClient::with_fixtures("tests/fixtures/tickets.json");

    let result = run_triage_pipeline(&config, &mock_jira, &db);
    assert!(result.is_ok());

    let scored = result.unwrap();
    assert!(scored.iter().all(|t| t.score.is_some()));
    assert!(scored.iter().any(|t| t.score == Some(Score::Go)));
}
```

**Run tests by scope:**

```bash
# All tests (PR default, <10 min)
cargo test

# Only unit tests (fast feedback)
cargo test --lib

# Only integration tests
cargo test --test '*'

# Specific crate
cargo test -p tf-security

# Ignored tests (nightly/weekly benchmarks)
cargo test -- --ignored

# With output for debugging
cargo test -- --nocapture
```

---

## Appendix B: Knowledge Base References

- **Risk Governance**: `risk-governance.md` - Risk scoring methodology (probability x impact, 1-9)
- **Test Priorities Matrix**: `test-priorities-matrix.md` - P0-P3 criteria and decision tree
- **Test Levels Framework**: `test-levels-framework.md` - Unit vs Integration vs E2E selection
- **Test Quality**: `test-quality.md` - Definition of Done (deterministic, isolated, <300 lines, <1.5 min)
- **ADR Quality Readiness**: `adr-quality-readiness-checklist.md` - 8-category 29-criteria NFR assessment

---

**Generated by:** BMad TEA Agent
**Workflow:** `_bmad/tea/testarch/test-design`
**Version:** 5.0 (BMad v6)

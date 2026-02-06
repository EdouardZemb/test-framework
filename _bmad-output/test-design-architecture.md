# Test Design for Architecture: QA Tester Process Optimization (TRA)

**Purpose:** Architectural concerns, testability gaps, and NFR requirements for review by Architecture/Dev teams. Serves as a contract between QA and Engineering on what must be addressed before test development begins.

**Date:** 2026-02-06
**Author:** Edouard Zemb (via TEA Test Architect)
**Status:** Architecture Review Pending
**Project:** test-framework
**PRD Reference:** `_bmad-output/planning-artifacts/prd.md`
**ADR Reference:** `_bmad-output/planning-artifacts/architecture.md`

---

## Executive Summary

**Scope:** CLI tool (Rust) for QA process automation — ingestion Jira/Squash, checklist testability, scoring Go/Clarify/No-Go, test design assistance (LLM), anomaly management, reporting (CR/PPT/exports), with mandatory anonymization and audit logging.

**Business Context** (from PRD):

- **Impact:** Gain 2-3h/semaine, reduction erreurs reporting -50%, feedback client >= 4/5
- **Problem:** Workflow QA TRA manuel, repetitif, fragile, sans standardisation
- **Target:** MVP operationnel multi-projets

**Architecture** (from ADR):

- **Key Decision 1:** Rust workspace multi-crates (10 crates: tf-cli, tf-core, tf-connectors, tf-storage, tf-security, tf-config, tf-logging, tf-diagnostics, tf-llm, tf-export)
- **Key Decision 2:** SQLite local chiffre + keyring OS pour secrets
- **Key Decision 3:** Anonymisation inline obligatoire avant tout envoi cloud LLM

**Expected Scale:** Single-user CLI, volume standard: ~50-200 tickets/lot, ~10-20 cas de test/ticket, pipelines < 5 min.

**Risk Summary:**

- **Total risks**: 12
- **High-priority (>=6)**: 4 risks requiring immediate mitigation
- **Test effort**: ~46 tests (~3-4 semaines pour 1 QA)

---

## Quick Guide

### BLOCKERS - Team Must Decide (Can't Proceed Without)

**Sprint 0 Critical Path** - These MUST be completed before QA can write integration tests:

1. **C2: Trait abstractions for connectors** - Define `trait Connector` in tf-connectors with mock implementations for Jira/Squash/SharePoint/Ollama (recommended owner: Dev)
2. **C1: Test data seeding module** - Implement `tf-storage/test_helpers.rs` with deterministic SQLite seeding functions (recommended owner: Dev)
3. **C5: Anonymization verification** - Implement PII canary test infrastructure in tf-security (recommended owner: Dev + QA)

**What we need from team:** Complete these 3 items in Sprint 0 or test development is blocked.

---

### HIGH PRIORITY - Team Should Validate (We Provide Recommendation, You Approve)

1. **R-01: PII leak prevention** - Add regex-based PII scanner in tf-security/redact.rs + CI scan step (Sprint 0)
2. **R-06: Proxy/IT block fallback** - Validate CSV fallback is functionally equivalent end-to-end, not just a data format (Sprint 0-1)
3. **C3: Fault injection mechanism** - Add `--simulate-failure` flag for resilience testing of degraded mode (Sprint 1)

**What we need from team:** Review recommendations and approve (or suggest changes).

---

### INFO ONLY - Solutions Provided (Review, No Decisions Needed)

1. **Test strategy**: 70% Unit (cargo test + mockall), 25% Integration (assert_cmd + test SQLite), 5% E2E CLI (assert_cmd full pipeline)
2. **Tooling**: cargo test, mockall 0.13, assert_cmd 2.0, predicates 3.1
3. **Tiered CI/CD**: PR (<10 min all unit+integration), Nightly (E2E + golden files), Weekly (benchmarks + chaos)
4. **Coverage**: ~46 test scenarios prioritized P0-P3 with risk-based classification
5. **Quality gates**: P0=100%, P1>=95%, coverage >=80% on core crates

**What we need from team:** Just review and acknowledge.

---

## For Architects and Devs - Open Topics

### Risk Assessment

**Total risks identified**: 12 (4 high-priority score >=6, 6 medium, 2 low)

#### High-Priority Risks (Score >=6) - IMMEDIATE ATTENTION

| Risk ID | Category | Description | Prob | Impact | Score | Mitigation | Owner | Timeline |
|---------|----------|-------------|------|--------|-------|------------|-------|----------|
| **R-01** | **SEC** | Fuite PII vers LLM cloud (anonymisation incomplete) | 2 | 3 | **6** | Test canary PII + revue pipeline anonymize.rs | tf-security | Sprint 0 |
| **R-02** | **SEC** | Secrets en clair dans logs/config/repo | 2 | 3 | **6** | Scan CI automatique + tests unitaires redact.rs | tf-security | Sprint 0 |
| **R-03** | **TECH** | Connecteurs non mockables - tests lents/flaky | 3 | 2 | **6** | Trait abstrait + impl mock par connecteur | tf-connectors | Sprint 0 |
| **R-06** | **OPS** | Proxy IT bloque appels API Jira/Squash | 2 | 3 | **6** | Mode degrade CSV automatique + diagnostic explicite | tf-connectors | Sprint 0-1 |

#### Medium-Priority Risks (Score 3-5)

| Risk ID | Category | Description | Prob | Impact | Score | Mitigation | Owner |
|---------|----------|-------------|------|--------|-------|------------|-------|
| R-04 | DATA | Corruption SQLite mid-pipeline | 2 | 2 | 4 | Transactions + test recovery | tf-storage |
| R-07 | BUS | Templates non alignes client | 2 | 2 | 4 | Golden file tests | tf-export |
| R-08 | TECH | Generation OOXML fragile | 2 | 2 | 4 | Golden file + validation ZIP/XML | tf-export |
| R-09 | SEC | Keyring indisponible en CI | 2 | 2 | 4 | Mock keyring + fallback env var | tf-security |
| R-10 | DATA | Mapping Jira/Squash divergents | 2 | 2 | 4 | Validation stricte au load | tf-connectors |
| R-11 | OPS | Purge locale ne s'execute pas | 2 | 2 | 4 | Test purge integre + alerte | tf-storage |

#### Low-Priority Risks (Score 1-2)

| Risk ID | Category | Description | Prob | Impact | Score | Action |
|---------|----------|-------------|------|--------|-------|--------|
| R-05 | PERF | CLI depasse 2s (cold start Rust) | 1 | 2 | 2 | Monitor via benchmark CI |
| R-12 | TECH | Ollama indisponible | 2 | 1 | 2 | Message explicite + hint |

---

### Testability Concerns and Architectural Gaps

**ACTIONABLE CONCERNS - Architecture Team Must Address**

#### 1. Blockers to Fast Feedback

| Concern | Impact | What Architecture Must Provide | Owner | Timeline |
|---------|--------|-------------------------------|-------|----------|
| **No trait abstraction for connectors** | Cannot mock Jira/Squash/SharePoint/Ollama - tests require live services | Define `trait ConnectorClient` in tf-connectors + MockJiraClient, MockSquashClient impls | Dev | Sprint 0 |
| **No test data seeding** | Cannot inject specific SQLite states (tickets importes, config edge cases) | `tf-storage/test_helpers.rs` with `seed_tickets()`, `seed_config()` functions | Dev | Sprint 0 |
| **No PII verification mechanism** | Cannot automatically verify anonymization completeness | PII pattern scanner in tf-security callable from tests | Dev | Sprint 0 |

#### 2. Architectural Improvements Needed

1. **Fault injection for degraded mode**
   - **Current problem**: No way to simulate API failures in tests
   - **Required change**: Add `--simulate-failure jira|squash|ollama` flag or config override
   - **Impact if not fixed**: Degraded mode (NFR9) untestable without real outages
   - **Owner**: Dev
   - **Timeline**: Sprint 1

2. **Performance observability**
   - **Current problem**: No measurable pipeline duration metrics
   - **Required change**: Add `tracing` spans with duration for each pipeline stage
   - **Impact if not fixed**: NFR6 (CR <2min) and NFR8 (CLI <2s) not verifiable in CI
   - **Owner**: Dev
   - **Timeline**: Sprint 1

3. **Purge TTL specification**
   - **Current problem**: "TTL court" mentioned but no concrete value
   - **Required change**: Set TTL = 24h in config schema, implement purge function
   - **Impact if not fixed**: NFR5 (purge <24h) not testable
   - **Owner**: Dev
   - **Timeline**: Sprint 0

---

### Testability Assessment Summary

#### What Works Well

- Workspace multi-crates: each crate testable independently with clear boundaries
- CLI 100% headless: all business logic accessible sans UI, tests unitaires rapides
- Erreurs structurees (code + message + hint): assertions sur cas d'erreur simples
- Codes de sortie normalises (0/1/2/3): validation CLI deterministe
- Config externalisee (YAML + profils): permutation de configs de test triviale
- Rust type system: modeles types + validation stricte - bugs structurels elimines au compile-time
- Fallback CSV prevu architecturalement: tests offline possibles

#### Accepted Trade-offs (No Action Required)

- **Pas de metriques Prometheus/Datadog**: CLI single-user, logs JSON suffisants pour le MVP
- **Pas de tracing distribue (W3C)**: architecture monolithique locale, non necessaire

---

### Risk Mitigation Plans (High-Priority Risks >=6)

#### R-01: Fuite PII vers LLM cloud (Score: 6) - HIGH

**Mitigation Strategy:**

1. Implementer regex-based PII scanner dans `tf-security/anonymize.rs` (emails, noms, tokens, numeros)
2. Ajouter un hook pre-envoi dans `tf-llm/orchestrator.rs` qui bloque si PII detectee post-anonymisation
3. Integrer un scan CI qui verifie 0 pattern PII dans les sorties de test cloud

**Owner:** tf-security (Dev)
**Timeline:** Sprint 0
**Status:** Planned
**Verification:** Test canary injectant des PII connues et verifiant leur masquage en sortie

#### R-02: Secrets en clair (Score: 6) - HIGH

**Mitigation Strategy:**

1. Implementer `redact.rs` avec filtrage systematique des patterns secrets dans les logs
2. Ajouter un scan CI grep sur les fichiers de config et le repo (0 match autorise)
3. Tests unitaires verifiant que keyring.rs ne retourne jamais de secret dans les messages d'erreur

**Owner:** tf-security (Dev)
**Timeline:** Sprint 0
**Status:** Planned
**Verification:** Tests unitaires + scan CI automatique

#### R-03: Connecteurs non mockables (Score: 6) - HIGH

**Mitigation Strategy:**

1. Definir `trait ConnectorClient` avec methodes async pour chaque operation (import, read, write)
2. Implementer `MockJiraClient`, `MockSquashClient`, `MockSharePointClient` via mockall
3. Injecter les mocks via dependency injection dans tf-core/pipeline

**Owner:** tf-connectors (Dev)
**Timeline:** Sprint 0
**Status:** Planned
**Verification:** Tests unitaires du pipeline avec 100% mocks, 0 appel reseau

#### R-06: Proxy IT bloque APIs (Score: 6) - HIGH

**Mitigation Strategy:**

1. Implementer detection automatique d'echec de connexion (timeout + HTTP errors)
2. Basculement automatique vers CSV avec message explicite (cause + action)
3. Valider que le mode CSV produit des resultats fonctionnellement equivalents

**Owner:** tf-connectors (Dev)
**Timeline:** Sprint 0-1
**Status:** Planned
**Verification:** Test E2E avec mock serveur down -> fallback CSV -> meme sortie

---

### Assumptions and Dependencies

#### Assumptions

1. Le workspace Rust et la structure multi-crates sont en place avant le Sprint 0 test
2. Les crates de base (tf-config, tf-logging) sont fonctionnelles et testees unitairement
3. Les APIs Jira et Squash restent stables (pas de breaking changes) pendant le MVP
4. Le keyring OS est disponible sur le poste de dev; en CI, un mock est acceptable

#### Dependencies

1. **Trait abstractions (R-03)** - Required by Sprint 0
2. **Test data seeding module (C1)** - Required by Sprint 0
3. **PII scanner (R-01)** - Required by Sprint 0
4. **Purge TTL implementation (C6)** - Required by Sprint 0

#### Risks to Plan

- **Risk**: Sprint 0 blockers non termines a temps
  - **Impact**: QA ne peut pas ecrire de tests d'integration
  - **Contingency**: Commencer par les tests unitaires de tf-config et tf-security (0 dependance externe)

---

**End of Architecture Document**

**Next Steps for Architecture Team:**

1. Review Quick Guide and prioritize Sprint 0 blockers (trait abstractions, seeding, PII scanner)
2. Assign owners and timelines for high-priority risks R-01, R-02, R-03, R-06
3. Validate assumptions (keyring mock en CI, APIs stables)
4. Provide feedback to QA on testability gaps

**Next Steps for QA Team:**

1. Wait for Sprint 0 blockers to be resolved
2. Refer to companion QA doc (test-design-qa.md) for test scenarios
3. Begin test infrastructure setup (test helpers, mock implementations)

# Test Design: Story 0-5 - Journalisation baseline sans donnees sensibles

**Date:** 2026-02-06
**Author:** Edouard Zemb (via TEA Test Architect)
**Status:** Draft
**Epic:** 0 - Foundation & Access
**Story:** 0.5 - Journalisation baseline sans donnees sensibles
**Branch:** `feature/0-5-journalisation-baseline`

**Related:** See system-level docs (test-design-architecture.md, test-design-qa.md) for architectural context.

---

## Executive Summary

**Scope:** Full test design for tf-logging crate — structured JSON logging with automatic sensitive field redaction, file-based output, and non-blocking writer with LogGuard lifecycle.

**Risk Summary:**

- Total risks identified: 5
- High-priority risks (>=6): 2 (SEC: redaction incomplete, TECH: immutable tracing events)
- Critical categories: SEC (sensitive data in logs), TECH (tracing-subscriber architecture)

**Coverage Summary:**

- P0 scenarios: 4 (~3-5 hours)
- P1 scenarios: 8 (~4-7 hours)
- P2 scenarios: 2 (~1-2 hours)
- P3 scenarios: 0
- **Total**: 14 tests (~8-14 hours)

---

## Risk Assessment

### High-Priority Risks (Score >=6)

| Risk ID | Category | Description | Prob | Impact | Score | Mitigation | Owner | Timeline |
|---------|----------|-------------|------|--------|-------|------------|-------|----------|
| **R-05-01** | **SEC** | RedactingLayer incomplet : un pattern PII echappe au masquage (ex: champ non listé, variante de casse) | 2 | 3 | **6** | Tests exhaustifs sur tous les noms de champs sensibles + test negatif confirmant que des champs normaux ne sont PAS masques | Dev + QA | Sprint 0 |
| **R-05-02** | **TECH** | tracing-subscriber events immutables : RedactingLayer ne peut pas modifier les champs avant ecriture JSON — risque d'approche technique incorrecte | 3 | 2 | **6** | Privilegier un custom FormatEvent ou un Layer::on_event() qui intercepte et re-emet. Valider l'approche avec un spike test avant impl complete | Dev | Sprint 0 |

### Medium-Priority Risks (Score 3-5)

| Risk ID | Category | Description | Prob | Impact | Score | Mitigation | Owner |
|---------|----------|-------------|------|--------|-------|------------|-------|
| R-05-03 | TECH | Double init du subscriber tracing → panic au runtime | 2 | 2 | 4 | Documenter que init_logging ne doit etre appele qu'une fois. Test verifiant le comportement | Dev |
| R-05-05 | DATA | WorkerGuard droppe trop tot → logs perdus en fin d'execution | 2 | 2 | 4 | Test verifiant que les logs sont flushed quand le guard est droppe. Documentation claire du pattern let _guard | Dev |

### Low-Priority Risks (Score 1-2)

| Risk ID | Category | Description | Prob | Impact | Score | Action |
|---------|----------|-------------|------|--------|-------|--------|
| R-05-04 | OPS | Creation repertoire logs echoue (permissions insuffisantes) | 1 | 2 | 2 | Test d'erreur avec message actionnable |

---

## Acceptance Criteria → Test Mapping

### AC #1: Logs JSON structures (timestamp, commande, statut, perimetre)

| Test ID | Scenario | Niveau | Priorite | Subtask |
|---------|----------|--------|----------|---------|
| **0.5-UNIT-001** | `init_logging` cree le repertoire de logs et retourne un LogGuard valide | Unit | P1 | 7.1 |
| **0.5-UNIT-002** | Logs JSON generes contiennent les champs requis : `timestamp` (ISO 8601), `level` (MAJUSCULES), `message`, `target` | Unit | **P0** | 7.2 |
| **0.5-UNIT-006** | Niveau de log par defaut est `info` | Unit | P1 | 7.6 |
| **0.5-UNIT-007** | RUST_LOG override le niveau configure | Unit | P1 | 7.7 |

### AC #2: Champs sensibles masques automatiquement

| Test ID | Scenario | Niveau | Priorite | Subtask | Risk Link |
|---------|----------|--------|----------|---------|-----------|
| **0.5-UNIT-003** | Champs sensibles (`token`, `password`, `api_key`, `secret`, `auth`, `authorization`, `credential`, `credentials`, `passwd`, `pwd`, `apikey`, `key`) masques par `[REDACTED]` | Unit | **P0** | 7.3 | R-05-01 |
| **0.5-UNIT-004** | URLs avec parametres sensibles redactees (ex: `?token=abc123` → `?token=[REDACTED]`) | Unit | **P0** | 7.4 | R-05-01 |
| **0.5-UNIT-009** | Debug impl de LogGuard ne contient aucune donnee sensible | Unit | P1 | 7.9 | - |

### AC #3: Logs stockes dans le dossier de sortie configure

| Test ID | Scenario | Niveau | Priorite | Subtask |
|---------|----------|--------|----------|---------|
| **0.5-UNIT-005** | Logs ecrits dans le repertoire configure (`{output_folder}/logs/`) | Unit | **P0** | 7.5 |
| **0.5-UNIT-008** | LoggingError contient des hints actionnables (InitFailed, DirectoryCreationFailed, InvalidLogLevel) | Unit | P1 | 7.8 |

### Cross-AC (integration + non-regression)

| Test ID | Scenario | Niveau | Priorite | Subtask | Notes |
|---------|----------|--------|----------|---------|-------|
| **0.5-INT-001** | Simuler une commande CLI complete et verifier le contenu du fichier log JSON (champs requis + pas de PII) | Integration | P1 | 7.10 | End-to-end du crate |
| **0.5-INT-002** | `cargo test --workspace` passe toujours (non-regression tf-config + tf-security) | Integration | P1 | 7.11 | Ne pas casser les 327 tests existants |
| **0.5-UNIT-010** | LoggingConfig::from_project_config derive correctement log_dir depuis output_folder avec fallback "./logs" | Unit | P2 | Task 4 | Config derivation |
| **0.5-UNIT-011** | ANSI colors desactivees dans les logs fichier (with_ansi(false)) | Unit | P2 | Task 2.6 | Format |

---

## Test Coverage Plan

**IMPORTANT:** P0/P1/P2/P3 = priorite et risque, PAS timing d'execution.

### P0 (Critical)

**Criteres:** Bloque la fonctionnalite core + Risque eleve + Impact securite/compliance

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| **0.5-UNIT-002** | AC #1: Logs JSON avec champs requis | Unit | - | Fondation de toute la journalisation |
| **0.5-UNIT-003** | AC #2: Tous les champs sensibles masques | Unit | R-05-01 | NFR4 compliance — EXHAUSTIF sur les 12 noms de champs |
| **0.5-UNIT-004** | AC #2: URLs avec params sensibles redactees | Unit | R-05-01 | Reutilise tf_config::redact_url_sensitive_params |
| **0.5-UNIT-005** | AC #3: Logs dans le repertoire configure | Unit | - | Verification I/O fichier |

**Total P0:** 4 tests

### P1 (High)

**Criteres:** Fonctionnalite importante + Workflows frequents

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| **0.5-UNIT-001** | init_logging cree dir + retourne LogGuard | Unit | - | Setup lifecycle |
| **0.5-UNIT-006** | Niveau par defaut = info | Unit | - | Config par defaut |
| **0.5-UNIT-007** | RUST_LOG override | Unit | - | EnvFilter |
| **0.5-UNIT-008** | LoggingError avec hints actionnables | Unit | - | Pattern erreurs structure |
| **0.5-UNIT-009** | Debug de LogGuard sans secrets | Unit | - | Security pattern |
| **0.5-INT-001** | Commande CLI simulee → log JSON complet sans PII | Integration | R-05-01 | Bout-en-bout du crate |
| **0.5-INT-002** | Non-regression workspace (cargo test --workspace) | Integration | - | 327 tests existants |

**Total P1:** 7 tests

### P2 (Medium)

**Criteres:** Secondaire + Edge cases

| Test ID | Requirement | Test Level | Notes |
|---------|-------------|------------|-------|
| **0.5-UNIT-010** | LoggingConfig derivation + fallback | Unit | Config edge case |
| **0.5-UNIT-011** | ANSI desactive pour fichier | Unit | Format |

**Total P2:** 2 tests

---

## Execution Strategy

**Philosophy:** Tous les tests dans `cargo test` sur chaque PR. Le crate tf-logging est petit (~14 tests), execution < 2 min.

### Every PR: cargo test (~1-2 min)

- Tous les tests unitaires (0.5-UNIT-001 a 011) via `cargo test -p tf-logging`
- Test d'integration (0.5-INT-001) dans `crates/tf-logging/tests/`
- Non-regression (0.5-INT-002) via `cargo test --workspace`
- Clippy + format : `cargo clippy -p tf-logging && cargo fmt -- --check`

**Aucun test nightly/weekly necessaire** — pas de benchmark, pas de chaos, pas d'I/O lourd.

---

## QA Effort Estimate

| Priorite | Count | Effort Range | Notes |
|----------|-------|-------------|-------|
| P0 | 4 | ~3-5 heures | Redaction exhaustive, validation JSON, I/O fichier |
| P1 | 7 | ~4-7 heures | Lifecycle, config, errors, integration |
| P2 | 2 | ~1-2 heures | Edge cases config, format |
| **Total** | **14** | **~8-14 heures** | **~1-2 jours** |

**Hypotheses :**
- tf-config expose `redact_url_sensitive_params` comme `pub` (Subtask 3.0 prerequis)
- `tempfile` et `assert_matches` deja en workspace dev-dependencies
- L'approche RedactingLayer est validee techniquement (spike test R-05-02)

---

## Risk Mitigation Plans

### R-05-01: RedactingLayer incomplet (Score: 6) - HIGH

**Strategie de mitigation :**

1. Definir la liste exhaustive des 12 noms de champs sensibles dans `SENSITIVE_FIELDS` (constante)
2. Test parametrise iterant sur CHAQUE nom de champ → verifier `[REDACTED]` dans la sortie
3. Test negatif : champs normaux (ex: `command`, `status`, `scope`) ne sont PAS masques
4. Test URLs : reutiliser `tf_config::redact_url_sensitive_params` pour les valeurs URL-like

**Owner:** Dev
**Timeline:** Sprint 0
**Verification:** 0.5-UNIT-003, 0.5-UNIT-004, 0.5-INT-001

### R-05-02: Events tracing immutables (Score: 6) - HIGH

**Strategie de mitigation :**

1. Spike test : implementer un prototype RedactingLayer minimal
2. Si `Layer::on_event()` ne peut pas modifier les champs → utiliser un custom `FormatEvent` qui redacte avant serialisation JSON
3. Documenter l'approche choisie dans le code (commentaire technique)

**Owner:** Dev
**Timeline:** Sprint 0 (debut de l'implementation)
**Verification:** 0.5-UNIT-003 passe avec l'approche choisie

---

## Test Implementation Patterns

### Pattern 1: Test de champs JSON requis (AC #1)

```rust
#[test]
fn log_output_contains_required_json_fields() {
    let dir = tempfile::tempdir().unwrap();
    let config = LoggingConfig {
        log_level: "info".to_string(),
        log_dir: dir.path().join("logs").to_string_lossy().to_string(),
        log_to_stdout: false,
    };
    let _guard = init_logging(&config).unwrap();

    tracing::info!(command = "triage", status = "success", scope = "lot-42", "Command executed");
    drop(_guard); // flush

    let log_file = find_log_file(dir.path().join("logs"));
    let content = std::fs::read_to_string(&log_file).unwrap();
    let json: serde_json::Value = serde_json::from_str(content.lines().last().unwrap()).unwrap();

    assert!(json.get("timestamp").is_some());
    assert!(json.get("level").is_some());
    assert_eq!(json["level"], "INFO");
    assert!(json.get("message").is_some() || json.get("fields").is_some());
}
```

### Pattern 2: Test exhaustif de redaction (AC #2)

```rust
const SENSITIVE_FIELDS: &[&str] = &[
    "token", "api_key", "apikey", "key", "secret",
    "password", "passwd", "pwd", "auth", "authorization",
    "credential", "credentials",
];

#[test]
fn all_sensitive_fields_are_redacted() {
    // Pour chaque champ sensible, emettre un event et verifier [REDACTED]
    for field_name in SENSITIVE_FIELDS {
        let dir = tempfile::tempdir().unwrap();
        // ... init logging ...
        // Emettre: tracing::info!({ *field_name } = "secret_value_123", "test")
        // Lire le fichier log
        // Assert: contient "[REDACTED]", ne contient PAS "secret_value_123"
    }
}

#[test]
fn normal_fields_are_not_redacted() {
    // Emettre: tracing::info!(command = "triage", status = "ok", "test")
    // Assert: contient "triage", contient "ok" (PAS masque)
}
```

### Pattern 3: Test de non-regression (workspace)

```rust
// crates/tf-logging/tests/integration_test.rs
#[test]
fn full_logging_lifecycle() {
    let dir = tempfile::tempdir().unwrap();
    let config = LoggingConfig { /* ... */ };

    // Init
    let guard = init_logging(&config).unwrap();

    // Log with sensitive + normal fields
    tracing::info!(command = "report", token = "secret123", status = "success", "Pipeline complete");

    // Flush
    drop(guard);

    // Verify file exists and content
    let log_content = read_log_file(&dir);
    assert!(log_content.contains("Pipeline complete"));
    assert!(log_content.contains("[REDACTED]"));
    assert!(!log_content.contains("secret123"));
    assert!(log_content.contains("report")); // command not redacted
}
```

---

## Assumptions and Dependencies

### Assumptions

1. tf-config est stable (297 tests passent) et expose `ProjectConfig.output_folder`
2. `redact_url_sensitive_params` sera expose comme `pub` (Subtask 3.0)
3. tracing-subscriber 0.3.x supporte un mecanisme de redaction (custom FormatEvent ou Layer intercepteur)
4. `tempfile` et `assert_matches` sont deja disponibles comme workspace dev-dependencies

### Dependencies

1. **Subtask 3.0** : `redact_url_sensitive_params` expose pub dans tf-config — Required before 0.5-UNIT-004
2. **Spike R-05-02** : Valider l'approche RedactingLayer — Required before implementation Task 3

### Risks to Plan

- **Risk**: L'approche RedactingLayer s'avere impossible avec tracing-subscriber 0.3.x
  - **Impact**: Implementation de la redaction bloquee
  - **Contingency**: Utiliser un wrapper autour de `tracing_subscriber::fmt::format::JsonFields` avec un custom Formatter qui filtre en sortie

---

## Quality Gate Criteria

| Gate | Critere | Seuil |
|------|---------|-------|
| PR Gate | P0 pass rate | **100%** |
| PR Gate | P1 pass rate | **>= 95%** |
| PR Gate | cargo clippy -p tf-logging | 0 warning |
| PR Gate | cargo test --workspace | Tous les 327+ tests passent |
| Story Done | Tous les 14 tests passent | **100%** |
| Story Done | 0 secret en clair dans les logs | Verifie par 0.5-UNIT-003 + 0.5-INT-001 |

---

## Appendix: Knowledge Base References

- **Risk Governance**: `risk-governance.md` — Scoring (P x I), gate decisions
- **Probability-Impact**: `probability-impact.md` — Echelle 1-3, seuils action
- **Test Levels**: `test-levels-framework.md` — Unit vs Integration selection
- **Test Priorities**: `test-priorities-matrix.md` — P0-P3 criteria
- **Test Quality**: `test-quality.md` — Deterministic, isolated, <300 lines

## Related Documents

- PRD: `_bmad-output/planning-artifacts/prd.md` (FR30, NFR4)
- Epic: `_bmad-output/planning-artifacts/epics.md` (Epic 0, Story 0.5)
- Architecture: `_bmad-output/planning-artifacts/architecture.md` (tf-logging, tracing stack)
- Story: `_bmad-output/implementation-artifacts/0-5-journalisation-baseline-sans-donnees-sensibles.md`
- System-Level Test Design: `_bmad-output/test-design-architecture.md` + `_bmad-output/test-design-qa.md`

---

**Generated by:** BMad TEA Agent
**Workflow:** `_bmad/tea/testarch/test-design` (Epic-Level Mode)
**Version:** 5.0 (BMad v6)

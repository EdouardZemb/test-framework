# Story 0.5b: Refactor tf-config test suite for maintainability

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a QA maintainer,
I want refactorer la suite de tests tf-config pour la maintenabilite,
so that les futures stories peuvent ajouter des tests sans aggraver la dette technique.

## Acceptance Criteria

1. **Given** le module de test monolithique dans `config.rs` (~3231 lignes, 211 tests)
   **When** le refactoring est termine
   **Then** les tests sont repartis en sous-modules fonctionnels de < 500 lignes chacun

2. **Given** les 80+ tests de validation dupliques (pattern YAML-load-assert-error)
   **When** le refactoring est termine
   **Then** un macro `test_config_rejects!` couvre >= 80% des tests de validation

3. **Given** les headers de test organises par round de review AI ("REVIEW 5", "REVIEW 12")
   **When** le refactoring est termine
   **Then** les tests sont organises par fonctionnalite (URL validation, path traversal, serde errors, etc.)

4. **Given** les 6 tests whitespace endpoint utilisant `std::env::temp_dir()` manuellement
   **When** le refactoring est termine
   **Then** ils utilisent `create_temp_config()` avec cleanup automatique via tempfile

5. **Given** un baseline pre-refactor capture
   **When** `cargo test --workspace` est execute apres refactoring
   **Then** 0 regression est observee et aucun test tf-config n'est supprime involontairement

6. **Given** le refactoring complete
   **When** `cargo clippy --workspace --all-targets -- -D warnings` est execute
   **Then** 0 warnings

## Tasks / Subtasks

- [x] Task 1: Creer la structure de sous-modules de test (AC: #1, #3)
  - [x] Subtask 1.1: Garder un seul bloc `#[cfg(test)] mod tests { ... }` dans `crates/tf-config/src/config.rs` (approche authoritative)
  - [x] Subtask 1.2: Creer des sous-modules inline dans ce bloc: `helpers`, `url_validation`, `path_validation`, `serde_errors`, `llm_config`, `redact_url`, `config_loading`, `profile_summary`
  - [x] Subtask 1.3: Extraire `create_temp_config()` et assertions communes dans `mod helpers`
  - [x] Subtask 1.4: Deplacer les tests URL (scheme, IPv4, IPv6, whitespace, hostname) dans `mod url_validation` (avec sous-module `ip_address`)
  - [x] Subtask 1.5: Deplacer les tests traversal/null bytes/formats chemins dans `mod path_validation`
  - [x] Subtask 1.6: Deplacer les tests erreurs serde/type/champs manquants dans `mod serde_errors`
  - [x] Subtask 1.7: Deplacer les tests cloud/local/defaults/edge cases dans `mod llm_config`
  - [x] Subtask 1.8: Deplacer les tests de redaction URL dans `mod redact_url` (avec sous-modules `debug_and_trait`, `url_redaction`, `param_encoding`, `separators_and_edge_cases`)
  - [x] Subtask 1.9: Deplacer les tests load_config/check_output_folder dans `mod config_loading`, et `active_profile_summary` dans `mod profile_summary`
  - [x] Subtask 1.10: Verifier que chaque sous-module logique reste < 500 lignes — helpers: 38, config_loading: 299, url_validation: 415 (+ ip_address: 272), path_validation: 364, serde_errors: 367, llm_config: 455, redact_url submodules: 139/395/212/263, profile_summary: 119

- [x] Task 2: Extraire le macro `test_config_rejects!` (AC: #2)
  - [x] Subtask 2.1: Definir les macros dans `mod helpers` — `test_config_rejects!` (AND pattern) et `test_config_rejects_any!` (OR pattern)
  - [x] Subtask 2.2: Convertir les tests de validation URL dupliques en invocations du macro (14 invocations)
  - [x] Subtask 2.3: Convertir les tests de validation path dupliques en invocations du macro (11 invocations)
  - [x] Subtask 2.4: Convertir les tests de validation serde/type dupliques en invocations du macro (10 invocations: 2 rejects + 4 rejects_any + 4 rejects)
  - [x] Subtask 2.5: Convertir les tests LLM config rejects en invocations du macro (17 invocations)
  - [x] Subtask 2.6: Garder en fonctions explicites les 12 tests avec match/complex assertions/Ok checks

- [x] Task 3: Normaliser la gestion des fichiers temporaires (AC: #4)
  - [x] Subtask 3.1: Identifier les 6 tests whitespace endpoint utilisant `std::env::temp_dir()` manuellement
  - [x] Subtask 3.2: Les convertir en invocations `test_config_rejects!` avec `create_temp_config()` automatique

- [x] Task 4: Validation finale (AC: #5, #6)
  - [x] Subtask 4.1: Baseline pre-refactor capture (211 inline tests) vs post-refactor (211 inline tests) — 0 regressions, 0 suppressions
  - [x] Subtask 4.2: `cargo clippy --workspace --all-targets -- -D warnings` — 0 warnings
  - [x] Subtask 4.3: `cargo fmt --check` — formatage correct
  - [x] Subtask 4.4: Tous les sous-modules < 500 lignes (max: llm_config 455 lines)
  - [x] Subtask 4.5: Macro coverage = 53/65 = 81.5% (>= 80% threshold)

## Dev Notes

### Scope: Tests Only — Zero Production Code Changes

Ce refactoring touche EXCLUSIVEMENT les modules `#[cfg(test)]` de `crates/tf-config/src/config.rs`. Aucune modification du code de production. Les tests d'integration dans `crates/tf-config/tests/` ne sont PAS concernes (deja bien structures).

### Architecture Compliance

**Structure actuelle (a refactorer) :**
```
crates/tf-config/src/
├── config.rs       # 5935 lignes dont ~3231 lignes de tests (211 tests dans un seul mod tests)
├── error.rs
├── lib.rs
├── profiles.rs
└── template.rs
```

**Decision implementation (authoritative):**
- Garder `#[cfg(test)] mod tests { ... }` inline dans `crates/tf-config/src/config.rs`
- Ne PAS basculer vers `mod tests;` ni creer `crates/tf-config/src/tests/`
- Organiser le contenu en sous-modules logiques inline (`mod url_validation { ... }`, etc.)
- Appliquer la contrainte `< 500 lignes` par bloc de sous-module inline

**Structure cible (authoritative) :**
```
crates/tf-config/src/
├── config.rs
│   └── #[cfg(test)] mod tests {
│       ├── mod helpers { ... }
│       ├── mod url_validation { ... }
│       ├── mod path_validation { ... }
│       ├── mod serde_errors { ... }
│       ├── mod llm_config { ... }
│       ├── mod redact_url { ... }
│       ├── mod config_loading { ... }
│       └── mod profile_summary { ... }
│   }
├── error.rs
├── lib.rs
├── profiles.rs
└── template.rs
```

### Execution Plan (authoritative)
1. Isoler d'abord `helpers` + macro `test_config_rejects!`.
2. Migrer les tests par domaine vers les sous-modules inline.
3. Remplacer les 6 tests endpoint whitespace manuels par `create_temp_config()`.
4. Valider baseline vs post-refactor (`test`, `clippy`, `fmt`, couverture macro, tailles modules).

### Macro Pattern Prouve (tf-logging reference)

Le crate tf-logging a deja prouve le pattern macro dans `redact.rs` :

```rust
// Pattern existant dans tf-logging/src/redact.rs (fonctionne)
macro_rules! test_sensitive_field_redacted {
    ($test_name:ident, $field:ident) => {
        #[test]
        fn $test_name() {
            let temp = tempdir().unwrap();
            let log_dir = temp.path().join("logs");
            let config = LoggingConfig { /* ... */ };
            // ... setup, emit, assert
        }
    };
}

test_sensitive_field_redacted!(test_token_redacted, token);
test_sensitive_field_redacted!(test_api_key_redacted, api_key);
// ... 12 invocations au lieu de 12 fonctions copy-paste
```

**Macro a creer pour tf-config :**

```rust
/// Macro for testing that a YAML config is rejected with expected error messages.
/// Eliminates copy-paste for 80+ validation tests.
macro_rules! test_config_rejects {
    ($name:ident, $yaml:expr, $($expected:expr),+ $(,)?) => {
        #[test]
        fn $name() {
            let result = load_config(&create_temp_config($yaml));
            assert!(result.is_err(), "Expected rejection for {}", stringify!($name));
            let err = result.unwrap_err().to_string();
            $(
                assert!(
                    err.contains($expected),
                    "Error for {} should contain '{}', got: '{}'",
                    stringify!($name), $expected, err
                );
            )+
        }
    };
}

// Usage:
test_config_rejects!(
    test_url_scheme_only_rejected,
    "project_name: \"test\"\noutput_folder: \"./out\"\njira:\n  endpoint: \"http://\"",
    "URL"
);
```

**Tests qui ne doivent PAS utiliser le macro :**
- Tests qui verifient le variant exact de l'erreur (`ConfigError::MissingField { .. }`)
- Tests qui verifient le contenu du hint avec `assert_matches!`
- Tests qui verifient des valeurs specifiques apres chargement reussi
- Tests d'integration avec setup complexe

### Organisation Fonctionnelle (remplacement des headers par review round)

**Mapping actuel -> cible :**

| Headers actuels (par review AI) | Sous-module cible |
|---|---|
| REVIEW 5: URL scheme tests | `url_validation` |
| REVIEW 6: IPv6 URL tests | `url_validation` |
| REVIEW 9: URL edge cases | `url_validation` |
| REVIEW 12: URL sensitive params | `redact_url` |
| REVIEW 13: URL authority parsing | `url_validation` |
| REVIEW 14: URL whitespace | `url_validation` |
| REVIEW 18: IPv4 mapped IPv6 | `url_validation` |
| REVIEW 22: URL encoding | `url_validation` |
| REVIEW 23: Hostname validation | `url_validation` |
| REVIEW 5: Path traversal | `path_validation` |
| REVIEW 7: Null bytes | `path_validation` |
| REVIEW 8: Serde type errors | `serde_errors` |
| REVIEW 10: Missing fields | `serde_errors` |
| REVIEW 11: LLM mode validation | `llm_config` |
| REVIEW 15-17: LLM edge cases | `llm_config` |
| Tests basiques load_config | `config_loading` |
| Tests profile_summary | `profile_summary` |
| Tests check_output_folder | `config_loading` |
| Tests redact_url_sensitive_params | `redact_url` |

### Normalisation des 6 Tests Whitespace Endpoint

**Pattern actuel (6 tests, lignes ~5237-5398) :**
```rust
#[test]
fn test_whitespace_endpoint_leading_space() {
    let yaml = "...";
    let path = std::env::temp_dir().join("test_ws_endpoint.yaml");
    std::fs::write(&path, yaml).unwrap();
    let result = load_config(&path);
    std::fs::remove_file(&path).ok(); // Pas garanti en cas de panic!
    assert!(result.is_err());
}
```

**Pattern cible :**
```rust
test_config_rejects!(
    test_whitespace_endpoint_leading_space,
    "project_name: \"test\"\noutput_folder: \"./out\"\njira:\n  endpoint: \"  http://jira.example.com\"",
    "URL"  // ou le message d'erreur attendu
);
```

### Anti-Patterns a Eviter

- **NE PAS** changer le code de production (lignes 1-~2700 de config.rs) — refactoring tests UNIQUEMENT
- **NE PAS** modifier les tests dans `crates/tf-config/tests/` (integration tests deja bien structures)
- **NE PAS** modifier d'autres crates (tf-logging, tf-security)
- **NE PAS** changer le comportement des tests — memes assertions, memes cas testes
- **NE PAS** ajouter de nouvelles dependances
- **NE PAS** supprimer de tests — tous les 211 tests config.rs inline doivent etre preserves
- **NE PAS** renommer les fonctions de test — les noms existants sont stables et potentiellement references dans les docs

### Previous Story Intelligence (Story 0-5)

**Patterns etablis a respecter :**
- `thiserror` pour enum d'erreurs avec variants specifiques et hints explicites
- Custom `Debug` impl masquant les donnees sensibles
- Tests dans le meme fichier (`#[cfg(test)] mod tests`) — cette story reorganise les sous-modules mais reste inline
- Workspace `members = ["crates/*"]` glob pattern — pas de changement necessaire
- `assert_matches!` pour verifier les variants d'erreur

**Apprentissages de la story 0-5 (8 rounds de review, 50+ findings) :**
- Les line counts dans le File List doivent etre exacts
- Le macro-driven parameterized testing elimine efficacement la duplication (prouve dans redact.rs)
- `create_temp_config()` est le helper standard pour les tests tf-config — 200+ tests l'utilisent deja
- Les tests paralleles fonctionnent correctement avec le subscriber thread-local

**Metriques actuelles de la suite de tests :**
- tf-config: 263 unit tests (211 inline dans config.rs + 52 dans tests/)
- tf-config: 10 doc-tests
- tf-logging: 62 unit tests + 2 doc-tests
- tf-security: 30 unit tests (16 ignored — keyring)
- **Total workspace: 417+ tests, 0 regressions attendues**

### Git Intelligence

**Commit recents pertinents :**
- `19bd527` chore: cargo fmt + sprint 0-5b planning (#20)
- `ef6d14c` chore(ci): add fmt check, cargo-audit, and init_logging timing test (#19)
- `d2e1f6d` feat(tf-logging): structured logging with sensitive field redaction (Story 0-5) (#18)

**Branche attendue :** `feature/0-5b-refactoring-test-suite-tf-config`
**Commit message pattern :** `refactor(tf-config): split monolithic test suite into functional modules (Story 0-5b)`

### Success Criteria (from Sprint Change Proposal)

| Criterion | Target | Measurement |
|-----------|--------|-------------|
| Test regressions | 0 | Baseline pre-refactor vs post-refactor: `cargo test --workspace` vert + pas de suppression involontaire dans `cargo test -p tf-config -- --list` |
| Max sub-module size | < 500 lines | Line count per test sub-module |
| Maintainability score | >= 70/100 (target 80) | TEA `testarch-test-review` re-run |
| Macro coverage | >= 80% of validation tests | Count of `test_config_rejects!` invocations vs total validation tests |
| clippy clean | 0 warnings | `cargo clippy --workspace -- -D warnings` |

### File Structure Requirements

**Fichiers a creer :** Aucun. Reorganisation interne du bloc `#[cfg(test)] mod tests { ... }` dans `config.rs`.

**Fichiers a modifier :**
- `crates/tf-config/src/config.rs` — restructuration du bloc `#[cfg(test)] mod tests { ... }`

### Testing Requirements

Pas de nouveaux tests a ecrire. Le Definition of Done est que TOUS les tests existants passent sans regression apres le refactoring.

**Commandes de validation :**
```bash
cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --check
```

```bash
cargo test -p tf-config -- --list
```

### References

- [Source: _bmad-output/planning-artifacts/sprint-change-proposal-2026-02-08.md] — 4 propositions detaillees (P0: split module, P0: macro, P1: reorganize, P1: normalize temp)
- [Source: 0-5-journalisation-baseline-sans-donnees-sensibles.md] — Previous story learnings, macro pattern in redact.rs
- [Source: architecture.md#Implementation Patterns] — Naming conventions, test framework: cargo test built-in
- [Source: _bmad-output/test-review.md] — Maintainability score 45/100, critical issues identified
- [Source: _bmad-output/traceability-matrix.md] — Evidence gaps and gate decision
- [Source: _bmad-output/nfr-assessment.md] — NFR assessment 78%, concerns raised

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (claude-opus-4-6)

### Debug Log References

- Baseline captured at `/tmp/tf-config-baseline-inline.txt` (211 test names)
- Post-refactor comparison verified 0 diffs in test function names

### Completion Notes List

1. Monolithic test module (3231 lines, 211 tests) split into 8 functional submodules + 5 nested submodules
2. Two macros created: `test_config_rejects!` (AND pattern, 48 invocations) and `test_config_rejects_any!` (OR pattern, 5 invocations)
3. 6 whitespace endpoint tests converted from manual `std::env::temp_dir()` to macro-based `create_temp_config()`
4. `url_validation` split with nested `mod ip_address` (IPv4/IPv6 tests)
5. `redact_url` split into 4 nested submodules: `debug_and_trait`, `url_redaction`, `param_encoding`, `separators_and_edge_cases`
6. Zero production code changes — only `#[cfg(test)] mod tests` block modified
7. All 211 inline test functions preserved with identical names
8. Full workspace: 263 tf-config + 62 tf-logging + 30 tf-security tests passing

### Change Log

| File | Lines Changed | Description |
|------|--------------|-------------|
| `crates/tf-config/src/config.rs` | ~3400 lines (test block only) | Restructured `#[cfg(test)] mod tests` into functional submodules with macros |

### File List

| File | Status | Lines |
|------|--------|-------|
| `crates/tf-config/src/config.rs` | Modified | 5554 (was 5935) |

### Metrics

| Criterion | Target | Actual | Status |
|-----------|--------|--------|--------|
| Test regressions | 0 | 0 | PASS |
| Max sub-module size | < 500 lines | 455 (llm_config) | PASS |
| Macro coverage | >= 80% | 81.5% (53/65) | PASS |
| clippy clean | 0 warnings | 0 warnings | PASS |
| fmt clean | 0 diffs | 0 diffs | PASS |

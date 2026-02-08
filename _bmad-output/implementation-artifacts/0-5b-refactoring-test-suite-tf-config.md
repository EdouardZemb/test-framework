# Story 0.5b: Refactor tf-config test suite for maintainability

Status: ready-for-dev

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

5. **Given** le refactoring complete
   **When** `cargo test --workspace` est execute
   **Then** tous les 417+ tests passent avec 0 regressions

6. **Given** le refactoring complete
   **When** `cargo clippy --workspace --all-targets -- -D warnings` est execute
   **Then** 0 warnings

## Tasks / Subtasks

- [ ] Task 1: Creer la structure de sous-modules de test (AC: #1, #3)
  - [ ] Subtask 1.1: Creer `crates/tf-config/src/tests/mod.rs` avec declarations de sous-modules et re-export du helper `create_temp_config`
  - [ ] Subtask 1.2: Creer `crates/tf-config/src/tests/helpers.rs` — extraire `create_temp_config()` et assertions communes
  - [ ] Subtask 1.3: Creer `crates/tf-config/src/tests/url_validation.rs` — regrouper ~50 tests de validation URL (scheme, IPv4, IPv6, whitespace, hostname)
  - [ ] Subtask 1.4: Creer `crates/tf-config/src/tests/path_validation.rs` — regrouper ~30 tests traversal, null bytes, formats chemins
  - [ ] Subtask 1.5: Creer `crates/tf-config/src/tests/serde_errors.rs` — regrouper ~40 tests erreurs de type, champs manquants, parsing YAML
  - [ ] Subtask 1.6: Creer `crates/tf-config/src/tests/llm_config.rs` — regrouper ~25 tests cloud mode, local mode, defaults, edge cases
  - [ ] Subtask 1.7: Creer `crates/tf-config/src/tests/redact_url.rs` — regrouper ~30 tests redaction parametres URL sensibles
  - [ ] Subtask 1.8: Creer `crates/tf-config/src/tests/config_loading.rs` — regrouper ~15 tests load_config, fixtures, config basique
  - [ ] Subtask 1.9: Creer `crates/tf-config/src/tests/profile_summary.rs` — regrouper ~10 tests active_profile_summary, check_output_folder
  - [ ] Subtask 1.10: Remplacer le `#[cfg(test)] mod tests { ... }` monolithique dans config.rs par `#[cfg(test)] mod tests;` pointant vers le dossier `tests/`

- [ ] Task 2: Extraire le macro `test_config_rejects!` (AC: #2)
  - [ ] Subtask 2.1: Definir le macro dans `helpers.rs` — pattern: `test_config_rejects!($name:ident, $yaml:expr, $($expected:expr),+)` qui cree un `#[test]` executant `load_config(&create_temp_config($yaml))` et verifiant `is_err()` + `err.to_string().contains($expected)`
  - [ ] Subtask 2.2: Convertir les tests de validation URL dupliques en invocations du macro
  - [ ] Subtask 2.3: Convertir les tests de validation path dupliques en invocations du macro
  - [ ] Subtask 2.4: Convertir les tests de validation serde/type dupliques en invocations du macro
  - [ ] Subtask 2.5: Convertir les tests LLM config rejects en invocations du macro (quand applicable)
  - [ ] Subtask 2.6: Garder en fonctions explicites les tests qui verifient des details specifiques (enum variant, hint texte exact, etc.) qui ne rentrent pas dans le pattern du macro

- [ ] Task 3: Normaliser la gestion des fichiers temporaires (AC: #4)
  - [ ] Subtask 3.1: Identifier les 6 tests whitespace endpoint utilisant `std::env::temp_dir()` manuellement (lignes ~4778-4905)
  - [ ] Subtask 3.2: Les convertir pour utiliser `create_temp_config(yaml)` au lieu de `std::env::temp_dir().join(...)` + `std::fs::write(...)` + `std::fs::remove_file(...)`

- [ ] Task 4: Validation finale (AC: #5, #6)
  - [ ] Subtask 4.1: Executer `cargo test --workspace` — verifier 0 regressions (417+ tests passent)
  - [ ] Subtask 4.2: Executer `cargo clippy --workspace --all-targets -- -D warnings` — verifier 0 warnings
  - [ ] Subtask 4.3: Executer `cargo fmt --check` — verifier formatage correct
  - [ ] Subtask 4.4: Verifier que chaque sous-module fait < 500 lignes
  - [ ] Subtask 4.5: Verifier que >= 80% des tests de validation utilisent le macro `test_config_rejects!`

## Dev Notes

### Scope: Tests Only — Zero Production Code Changes

Ce refactoring touche EXCLUSIVEMENT les modules `#[cfg(test)]` de `crates/tf-config/src/config.rs`. Aucune modification du code de production. Les tests d'integration dans `crates/tf-config/tests/` ne sont PAS concernes (deja bien structures).

### Architecture Compliance

**Structure actuelle (a refactorer) :**
```
crates/tf-config/src/
├── config.rs       # 5935 lignes dont ~3231 lignes de tests (211 tests dans un seul mod tests)
├── error.rs        # 98 lignes
├── lib.rs          # 74 lignes
├── profiles.rs     # 146 lignes
└── template.rs     # 1613 lignes
```

**Structure cible :**
```
crates/tf-config/src/
├── config.rs       # ~2700 lignes (production code seulement + `#[cfg(test)] mod tests;`)
├── tests/          # Nouveau dossier de sous-modules de test
│   ├── mod.rs           # Declarations des sous-modules + imports communs
│   ├── helpers.rs       # create_temp_config(), test_config_rejects! macro, assertions communes
│   ├── url_validation.rs     # ~50 tests (< 500 lignes)
│   ├── path_validation.rs    # ~30 tests (< 500 lignes)
│   ├── serde_errors.rs       # ~40 tests (< 500 lignes)
│   ├── llm_config.rs         # ~25 tests (< 500 lignes)
│   ├── redact_url.rs         # ~30 tests (< 500 lignes)
│   ├── config_loading.rs     # ~15 tests (< 500 lignes)
│   └── profile_summary.rs    # ~10 tests (< 500 lignes)
├── error.rs
├── lib.rs
├── profiles.rs
└── template.rs
```

**Approche pour les sous-modules inline :** En Rust, un `#[cfg(test)] mod tests;` dans `config.rs` pointe vers `tests/mod.rs` (ou `tests.rs`). Les sous-modules sont declares dans `mod.rs` et peuvent acceder aux items prives de `config.rs` via `use super::super::*;` (puisqu'ils sont des sous-modules du module `tests` qui est lui-meme un sous-module de `config`).

**ALTERNATIVE si la structure `tests/` en dossier pose probleme :** Utiliser un seul fichier `config/tests.rs` contenant des `mod url_validation { ... }` inline. Cela evite de creer un dossier mais le fichier resterait gros. **Preferer l'approche dossier** pour respecter le seuil de 500 lignes par fichier.

**NOTE IMPORTANTE sur la structure des modules Rust :**
Quand `config.rs` contient `#[cfg(test)] mod tests;`, Rust cherche le fichier `config/tests.rs` OU `config/tests/mod.rs`. Mais `config.rs` existe DEJA comme fichier — il faudrait le renommer en `config/mod.rs` ou utiliser l'edition 2021 path resolution. **L'approche la plus propre** est de garder le module tests inline dans `config.rs` mais avec des sous-modules dans des fichiers separes via `#[path = "..."]` :

```rust
// Dans config.rs, a la fin:
#[cfg(test)]
mod tests {
    #[path = "tests_helpers.rs"]
    mod helpers;
    #[path = "tests_url_validation.rs"]
    mod url_validation;
    // ... etc
}
```

**OU** (plus idiomatique) : garder tous les sous-modules inline dans le `mod tests { }` block existant, mais les organiser en `mod url_validation { ... }`, `mod path_validation { ... }`, etc. directement dans config.rs. Chaque sous-module fait < 500 lignes, le total reste ~3000 lignes mais est ORGANISE par fonctionnalite.

**DECISION RECOMMANDEE : sous-modules inline dans config.rs.** C'est la solution la plus simple et la plus robuste qui evite les complications de chemin de modules. Le fichier reste gros mais chaque sous-module est navigable et < 500 lignes. Les fichiers separes via `#[path]` sont une alternative si l'equipe prefere.

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

**Pattern actuel (6 tests, lignes ~4778-4905) :**
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
| Test regressions | 0 | `cargo test --workspace` — all 417+ tests pass |
| Max sub-module size | < 500 lines | Line count per test sub-module |
| Maintainability score | >= 70/100 (target 80) | TEA `testarch-test-review` re-run |
| Macro coverage | >= 80% of validation tests | Count of `test_config_rejects!` invocations vs total validation tests |
| clippy clean | 0 warnings | `cargo clippy --workspace -- -D warnings` |

### File Structure Requirements

**Fichiers a creer :** AUCUN fichier de production. Uniquement reorganisation du code `#[cfg(test)]` existant.

Si approche sous-modules inline (recommandee) :
- Aucun nouveau fichier — restructuration interne de `config.rs` `mod tests { ... }`

Si approche fichiers separes via `#[path]` :
- `crates/tf-config/src/tests_helpers.rs`
- `crates/tf-config/src/tests_url_validation.rs`
- `crates/tf-config/src/tests_path_validation.rs`
- `crates/tf-config/src/tests_serde_errors.rs`
- `crates/tf-config/src/tests_llm_config.rs`
- `crates/tf-config/src/tests_redact_url.rs`
- `crates/tf-config/src/tests_config_loading.rs`
- `crates/tf-config/src/tests_profile_summary.rs`

**Fichiers a modifier :**
- `crates/tf-config/src/config.rs` — restructuration du bloc `#[cfg(test)] mod tests { ... }`

### Testing Requirements

Pas de nouveaux tests a ecrire. Le Definition of Done est que TOUS les tests existants passent sans regression apres le refactoring.

**Commande de validation :**
```bash
cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --check
```

### References

- [Source: sprint-change-proposal-2026-02-08.md] — 4 propositions detaillees (P0: split module, P0: macro, P1: reorganize, P1: normalize temp)
- [Source: 0-5-journalisation-baseline-sans-donnees-sensibles.md] — Previous story learnings, macro pattern in redact.rs
- [Source: architecture.md#Implementation Patterns] — Naming conventions, test framework: cargo test built-in
- [Source: _bmad-output/test-review.md] — Maintainability score 45/100, critical issues identified
- [Source: _bmad-output/traceability-matrix.md] — Evidence gaps and gate decision
- [Source: _bmad-output/nfr-assessment.md] — NFR assessment 78%, concerns raised

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List

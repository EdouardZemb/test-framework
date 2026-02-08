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

5. **Given** un baseline pre-refactor capture
   **When** `cargo test --workspace` est execute apres refactoring
   **Then** 0 regression est observee et aucun test tf-config n'est supprime involontairement

6. **Given** le refactoring complete
   **When** `cargo clippy --workspace --all-targets -- -D warnings` est execute
   **Then** 0 warnings

## Tasks / Subtasks

- [ ] Task 1: Creer la structure de sous-modules de test (AC: #1, #3)
  - [ ] Subtask 1.1: Garder un seul bloc `#[cfg(test)] mod tests { ... }` dans `crates/tf-config/src/config.rs` (approche authoritative)
  - [ ] Subtask 1.2: Creer des sous-modules inline dans ce bloc: `helpers`, `url_validation`, `path_validation`, `serde_errors`, `llm_config`, `redact_url`, `config_loading`, `profile_summary`
  - [ ] Subtask 1.3: Extraire `create_temp_config()` et assertions communes dans `mod helpers`
  - [ ] Subtask 1.4: Deplacer les tests URL (scheme, IPv4, IPv6, whitespace, hostname) dans `mod url_validation`
  - [ ] Subtask 1.5: Deplacer les tests traversal/null bytes/formats chemins dans `mod path_validation`
  - [ ] Subtask 1.6: Deplacer les tests erreurs serde/type/champs manquants dans `mod serde_errors`
  - [ ] Subtask 1.7: Deplacer les tests cloud/local/defaults/edge cases dans `mod llm_config`
  - [ ] Subtask 1.8: Deplacer les tests de redaction URL dans `mod redact_url`
  - [ ] Subtask 1.9: Deplacer les tests load_config/check_output_folder dans `mod config_loading`, et `active_profile_summary` dans `mod profile_summary`
  - [ ] Subtask 1.10: Verifier que chaque sous-module logique reste < 500 lignes dans `config.rs` (mesure par bloc `mod <name> { ... }`)

- [ ] Task 2: Extraire le macro `test_config_rejects!` (AC: #2)
  - [ ] Subtask 2.1: Definir le macro dans `helpers.rs` — pattern: `test_config_rejects!($name:ident, $yaml:expr, $($expected:expr),+)` qui cree un `#[test]` executant `load_config(&create_temp_config($yaml))` et verifiant `is_err()` + `err.to_string().contains($expected)`
  - [ ] Subtask 2.2: Convertir les tests de validation URL dupliques en invocations du macro
  - [ ] Subtask 2.3: Convertir les tests de validation path dupliques en invocations du macro
  - [ ] Subtask 2.4: Convertir les tests de validation serde/type dupliques en invocations du macro
  - [ ] Subtask 2.5: Convertir les tests LLM config rejects en invocations du macro (quand applicable)
  - [ ] Subtask 2.6: Garder en fonctions explicites les tests qui verifient des details specifiques (enum variant, hint texte exact, etc.) qui ne rentrent pas dans le pattern du macro

- [ ] Task 3: Normaliser la gestion des fichiers temporaires (AC: #4)
  - [ ] Subtask 3.1: Identifier les 6 tests whitespace endpoint utilisant `std::env::temp_dir()` manuellement (lignes ~5237-5398)
  - [ ] Subtask 3.2: Les convertir pour utiliser `create_temp_config(yaml)` au lieu de `std::env::temp_dir().join(...)` + `std::fs::write(...)` + `std::fs::remove_file(...)`

- [ ] Task 4: Validation finale (AC: #5, #6)
  - [ ] Subtask 4.1: Capturer un baseline pre-refactor (`cargo test --workspace`, puis `cargo test -p tf-config -- --list`) et comparer post-refactor pour verifier 0 regressions et aucune suppression involontaire de tests
  - [ ] Subtask 4.2: Executer `cargo clippy --workspace --all-targets -- -D warnings` — verifier 0 warnings
  - [ ] Subtask 4.3: Executer `cargo fmt --check` — verifier formatage correct
  - [ ] Subtask 4.4: Verifier que chaque bloc `mod <name> { ... }` dans `#[cfg(test)] mod tests` fait < 500 lignes
  - [ ] Subtask 4.5: Verifier que >= 80% des tests de validation utilisent le macro `test_config_rejects!`

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

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List

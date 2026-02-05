# Story 0.2: Définir et sélectionner des profils de configuration

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a QA tester (TRA),
I want définir et sélectionner des profils de configuration,
so that basculer rapidement de contexte.

## Acceptance Criteria

1. **Given** une config.yaml avec des profils
   **When** je sélectionne un profil
   **Then** la configuration du profil est appliquée et affichée dans le résumé

2. **Given** un profil inconnu
   **When** je tente de le sélectionner
   **Then** un message indique l'erreur et liste les profils disponibles

3. **Given** un profil appliqué
   **When** les logs sont écrits
   **Then** ils ne contiennent aucune donnée sensible

## Tasks / Subtasks

- [x] Task 1: Définir le schéma YAML des profils (AC: #1)
  - [x] Subtask 1.1: Étendre `config.rs` avec section `profiles: HashMap<String, ProfileOverride>` dans ProjectConfig
  - [x] Subtask 1.2: Créer struct `ProfileOverride` avec les champs surchargables (jira, squash, llm, templates, output_folder)
  - [x] Subtask 1.3: Ajouter `#[serde(deny_unknown_fields)]` sur ProfileOverride
  - [x] Subtask 1.4: Documenter le schéma avec doc-comments

- [x] Task 2: Implémenter la sélection de profil (AC: #1, #2)
  - [x] Subtask 2.1: Créer méthode `ProjectConfig::with_profile(profile: &str) -> Result<ProjectConfig, ConfigError>` (alternative à fonction standalone)
  - [x] Subtask 2.2: Implémenter la fusion (merge) des overrides du profil sur la config de base via `apply_profile`
  - [x] Subtask 2.3: Ajouter variant `ConfigError::ProfileNotFound { requested: String, available: Vec<String> }` avec hint
  - [x] Subtask 2.4: Valider que le profil sélectionné existe et lister les disponibles si non trouvé

- [x] Task 3: Implémenter l'affichage du résumé de profil (AC: #1)
  - [x] Subtask 3.1: Créer méthode `ProjectConfig::active_profile_summary(&self) -> String` pour résumer le profil actif
  - [x] Subtask 3.2: Afficher les valeurs effectives après merge (endpoints, mode LLM, output_folder)
  - [x] Subtask 3.3: Summary shows effective values (profile source tracking not needed for MVP)

- [x] Task 4: Garantir la sécurité des logs avec profils (AC: #3)
  - [x] Subtask 4.1: Implémenter `Debug` custom pour `ProfileOverride` masquant les champs sensibles
  - [x] Subtask 4.2: Implémenter trait `Redact` pour `ProfileOverride`
  - [x] Subtask 4.3: Vérifier que le merge préserve la redaction des secrets

- [x] Task 5: Compléter profiles.rs avec l'implémentation réelle (AC: #1, #2, #3)
  - [x] Subtask 5.1: Remplacer le stub actuel par les types réels (ProfileOverride, ProfileId)
  - [x] Subtask 5.2: Exporter les nouveaux types dans lib.rs
  - [x] Subtask 5.3: Retirer l'attribut `#[doc(hidden)]` des exports profiles

- [x] Task 6: Tests unitaires et intégration (AC: #1, #2, #3)
  - [x] Subtask 6.1: Tests pour chargement de config avec profil valide
  - [x] Subtask 6.2: Tests pour sélection profil inexistant avec message d'erreur et liste
  - [x] Subtask 6.3: Tests pour merge correct des overrides (jira, llm, output_folder)
  - [x] Subtask 6.4: Tests pour absence de données sensibles dans Debug et logs
  - [x] Subtask 6.5: Tests pour profil vide (aucun override)
  - [x] Subtask 6.6: Créer fixtures YAML: `config_with_profiles.yaml`, `config_profile_override_jira.yaml`

### Review Follow-ups (AI)

- [x] [AI-Review][HIGH] Ajouter validation après merge dans `with_profile()` - appeler `validate_config(&merged)?` avant de retourner [crates/tf-config/src/config.rs:595-617]
- [x] [AI-Review][HIGH] Ajouter test pour `active_profile_summary()` vérifiant l'affichage du profil actif et valeurs effectives (AC #1 partiellement non couvert) [crates/tf-config/tests/profile_tests.rs]
- [x] [AI-Review][MEDIUM] Documenter `sprint-status.yaml` dans la File List de la story
- [x] [AI-Review][MEDIUM] Appliquer `is_safe_path()` validation au `output_folder` des profils après merge [crates/tf-config/src/config.rs]
- [x] [AI-Review][MEDIUM] Supprimer trait `Redact` dupliqué dans profiles.rs et réutiliser celui de config.rs [crates/tf-config/src/profiles.rs:121-124]
- [x] [AI-Review][MEDIUM] Convertir les doctests `ignore` en `no_run` ou les rendre compilables [crates/tf-config/src/config.rs, profiles.rs]
- [x] [AI-Review][LOW] Ajouter test pour profil avec `output_folder: ""` vide
- [x] [AI-Review][LOW] Évaluer si `ProfileConfig` backward-compat stub peut être supprimé [crates/tf-config/src/profiles.rs:132-138]

### Review Follow-ups Round 2 (AI - 2026-02-05)

- [x] [AI-Review][HIGH] Ajouter test pour `with_profile()` sur config sans section profiles (profiles: None) - AC #2 edge case [crates/tf-config/tests/profile_tests.rs]
- [x] [AI-Review][HIGH] Ajouter test pour profil avec URL Jira invalide - valider que `validate_config()` rejette après merge [crates/tf-config/tests/profile_tests.rs]
- [x] [AI-Review][MEDIUM] Supprimer tests dupliqués entre `profiles.rs` et `profile_unit_tests.rs` (6 tests en double) [crates/tf-config/src/profiles.rs:127-258]
- [x] [AI-Review][MEDIUM] Améliorer message `ProfileNotFound` quand `profiles: None` - éviter "Available profiles: " vide [crates/tf-config/src/error.rs:39]
- [x] [AI-Review][LOW] Mettre à jour doc-comments TDD obsolètes dans `profile_tests.rs` (parle de "types that DON'T EXIST YET") [crates/tf-config/tests/profile_tests.rs:1-14]
- [x] [AI-Review][LOW] Mettre à jour doc-comments TDD obsolètes dans `profile_unit_tests.rs` [crates/tf-config/tests/profile_unit_tests.rs:1-18]
- [x] [AI-Review][LOW] Renommer `test_profile_override_can_remove_section` en `test_profile_with_none_preserves_base_value` (nom trompeur) [crates/tf-config/tests/profile_unit_tests.rs:401]

### Review Follow-ups Round 3 (AI - 2026-02-05)

- [x] [AI-Review][MEDIUM] Supprimer commentaire TDD obsolète "These imports will fail until ProfileOverride is implemented" [crates/tf-config/tests/profile_unit_tests.rs:18]
- [x] [AI-Review][MEDIUM] Ajouter test pour LlmConfig invalide après merge de profil (mode: cloud + cloud_enabled: false) [crates/tf-config/tests/profile_tests.rs]
- [x] [AI-Review][MEDIUM] Ajouter test direct pour `ProfileOverride::redacted()` au lieu de seulement `format!("{:?}")` [crates/tf-config/tests/profile_unit_tests.rs]
- [x] [AI-Review][LOW] Supprimer test redondant `test_with_profile_validates_merged_config_path_traversal` (doublon de profile_tests.rs) [crates/tf-config/tests/profile_unit_tests.rs:443-477]
- [x] [AI-Review][LOW] Ajouter exemple d'usage de `.redacted()` dans la documentation du module profiles [crates/tf-config/src/profiles.rs:36]
- [x] [AI-Review][LOW] Harmoniser format du summary: "LLM mode:" vs "Jira endpoint:" [crates/tf-config/src/config.rs:725-728]
- [x] [AI-Review][LOW] Considérer ajouter `local_endpoint` au LLM config de la fixture pour éviter fragilité [crates/tf-config/tests/fixtures/config_with_profiles.yaml:17]

### Review Follow-ups Round 4 (AI - 2026-02-05)

- [x] [AI-Review][MEDIUM] Ajouter test pour profile override de `templates` - seul champ ProfileOverride sans test de merge [crates/tf-config/tests/profile_tests.rs]
- [x] [AI-Review][MEDIUM] Ajouter `#[derive(PartialEq)]` à ProfileOverride pour améliorer testabilité [crates/tf-config/src/profiles.rs:88]
- [x] [AI-Review][MEDIUM] Considérer ajouter chemin du fichier config au contexte de ProfileNotFound pour UX [crates/tf-config/src/error.rs:51] - **DEFERRED**: Nécessiterait changement d'API; AC #2 déjà satisfait; call site peut ajouter contexte si nécessaire
- [x] [AI-Review][LOW] Ajouter profil qui override `llm` dans fixture principale config_with_profiles.yaml [crates/tf-config/tests/fixtures/config_with_profiles.yaml]
- [x] [AI-Review][LOW] Renforcer assertion dans test_active_profile_summary_shows_profile_and_values (accepte "local" OU "Local") [crates/tf-config/tests/profile_unit_tests.rs:558]
- [x] [AI-Review][LOW] Mettre à jour doc-comment active_profile_summary - retire mention "profile vs base tracking" non implémenté [crates/tf-config/src/config.rs:704]

### Review Follow-ups Round 5 (AI - 2026-02-05)

- [x] [AI-Review][MEDIUM] Mettre à jour README.md pour documenter la fonctionnalité des profils (syntaxe YAML, API with_profile(), active_profile_summary()) [crates/tf-config/README.md]
- [x] [AI-Review][MEDIUM] Ajouter test pour profil avec templates path traversal - valider que validate_config() rejette après merge [crates/tf-config/tests/profile_tests.rs]
- [x] [AI-Review][MEDIUM] Ajouter test pour profil avec Squash URL invalide - compléter couverture des validations post-merge [crates/tf-config/tests/profile_tests.rs]
- [x] [AI-Review][LOW] Considérer ajouter templates au résumé dans active_profile_summary() pour visibilité complète [crates/tf-config/src/config.rs:708]
- [x] [AI-Review][LOW] Ajouter test utilisant PartialEq directement (assert_eq!(profile, profile.clone())) pour valider le derive [crates/tf-config/tests/profile_unit_tests.rs]
- [x] [AI-Review][LOW] Considérer consolidation des fixtures de profils invalides dans un fichier unique [crates/tf-config/tests/fixtures/] - **DEFERRED**: Garder les fixtures séparées pour lisibilité et maintenance; chaque fixture teste un cas spécifique avec sa documentation

## Dev Notes

### Technical Stack Requirements

**Versions exactes à utiliser (identiques à Story 0.1):**
- Rust edition: 2021 (MSRV 1.75+)
- `serde = "1.0"` avec derive feature
- `serde_yaml = "0.9"` (migration future planifiée)
- `thiserror = "2.0"` pour les erreurs structurées

**Patterns de merge obligatoires:**
```rust
impl ProjectConfig {
    /// Merge profile overrides onto base config
    pub fn with_profile(&self, profile_id: &str) -> Result<ProjectConfig, ConfigError> {
        let profile = self.profiles.get(profile_id)
            .ok_or_else(|| ConfigError::ProfileNotFound {
                profile: profile_id.to_string(),
                available: self.profiles.keys().cloned().collect(),
            })?;

        let mut merged = self.clone();
        // Apply overrides...
        merged.active_profile = Some(profile_id.to_string());
        Ok(merged)
    }
}
```

### Architecture Compliance

**Crate tf-config - Extension du module existant**

La Story 0.1 a créé la base. Cette story étend avec:
- Section `profiles` dans ProjectConfig
- Struct ProfileOverride pour les surcharges
- Logique de merge
- Nouveaux variants ConfigError

**Structure après implémentation:**
```
crates/
└── tf-config/
    ├── Cargo.toml
    └── src/
        ├── lib.rs         # Exports mis à jour
        ├── config.rs      # ProjectConfig + ProfileOverride + merge
        ├── profiles.rs    # Types de profils (remplace le stub)
        └── error.rs       # +ProfileNotFound variant
```

### Schema config.yaml avec profils

```yaml
project_name: "my-project"
output_folder: "./output"

jira:
  endpoint: "https://jira.prod.example.com"
  token: "${JIRA_TOKEN}"

squash:
  endpoint: "https://squash.prod.example.com"

llm:
  mode: "auto"
  local_endpoint: "http://localhost:11434"

# Profils de configuration
profiles:
  dev:
    output_folder: "./output-dev"
    jira:
      endpoint: "https://jira.dev.example.com"
    llm:
      mode: "local"

  staging:
    jira:
      endpoint: "https://jira.staging.example.com"
    squash:
      endpoint: "https://squash.staging.example.com"

  prod:
    # Profil minimal, utilise les valeurs de base
```

### Library/Framework Requirements

**Dépendances Cargo.toml inchangées:**
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
thiserror = "2.0"
```

### Previous Story Intelligence (Story 0.1)

**Patterns établis à réutiliser:**
- ConfigError avec variants spécifiques et hints explicites
- Validation stricte avec `#[serde(deny_unknown_fields)]` sur toutes les structs
- Trait `Redact` pour masquer les secrets dans les logs
- Custom `Debug` impl masquant token/password/api_key
- `parse_serde_error()` pour convertir les erreurs YAML en messages explicites
- `is_valid_url()` avec support hostnames internes et IPv6
- `is_safe_path()` pour validation anti-traversal

**Apprentissages des 14 reviews:**
- TOUJOURS ajouter `#[serde(deny_unknown_fields)]` aux nouvelles structs
- TOUJOURS implémenter Debug custom et Redact pour les structs avec secrets
- Tester les cas limites: profil vide, profil avec override partiel, profil inconnu
- Validation stricte des URLs et chemins dans les overrides
- Messages d'erreur: toujours inclure `champ + raison + hint`

**Fichiers modifiés dans Story 0.1 (à préserver):**
- 134 tests passent (123 unit + 8 integration + 3 doc-tests)
- Ne pas casser les tests existants lors de l'extension

### Testing Requirements

**Framework:** `cargo test` built-in

**Patterns de test obligatoires:**
```rust
#[test]
fn test_load_config_with_valid_profile() {
    let config = load_config_with_profile(
        Path::new("tests/fixtures/config_with_profiles.yaml"),
        Some("dev")
    ).unwrap();
    assert_eq!(config.active_profile, Some("dev".to_string()));
    assert_eq!(config.output_folder, "./output-dev");
}

#[test]
fn test_unknown_profile_lists_available() {
    let result = load_config_with_profile(
        Path::new("tests/fixtures/config_with_profiles.yaml"),
        Some("nonexistent")
    );
    let err = result.unwrap_err();
    assert!(matches!(err, ConfigError::ProfileNotFound { .. }));
    assert!(err.to_string().contains("dev"));
    assert!(err.to_string().contains("staging"));
    assert!(err.to_string().contains("prod"));
}

#[test]
fn test_profile_override_redacts_secrets() {
    let override_cfg = ProfileOverride {
        jira: Some(JiraConfig {
            endpoint: "https://jira.dev".to_string(),
            token: Some("secret-token".to_string()),
        }),
        ..Default::default()
    };
    let debug_str = format!("{:?}", override_cfg);
    assert!(!debug_str.contains("secret-token"));
    assert!(debug_str.contains("[REDACTED]"));
}
```

### File Structure Requirements

**Naming conventions (identiques à Story 0.1):**
- Fichiers: `snake_case.rs`
- Modules: `snake_case`
- Structs/Enums: `PascalCase`
- Functions/variables: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`

### Project Structure Notes

- Ce crate reste le premier dans l'ordre des dépendances
- Aucune dépendance vers d'autres crates tf-*
- Les tests d'intégration utilisent `CARGO_MANIFEST_DIR` pour les fixtures

### Anti-Patterns to Avoid (from Story 0.1 reviews)

- NE PAS oublier `#[serde(deny_unknown_fields)]` sur ProfileOverride
- NE PAS logger les tokens/secrets même dans les overrides
- NE PAS utiliser `skip_serializing` sans implémenter Serialize
- NE PAS accepter les URLs avec port vide ou IPv4/IPv6 invalides
- NE PAS retourner ParseError générique - toujours fournir field + hint

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#Technology Stack]
- [Source: _bmad-output/planning-artifacts/architecture.md#Project Structure & Boundaries]
- [Source: _bmad-output/planning-artifacts/architecture.md#Implementation Patterns]
- [Source: _bmad-output/planning-artifacts/prd.md#FR24]
- [Source: _bmad-output/planning-artifacts/epics.md#Story 0.2]
- [Source: _bmad-output/implementation-artifacts/0-1-configurer-un-projet-via-config-yaml.md#Completion Notes]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.5 (claude-opus-4-5-20251101)

### Debug Log References

N/A - No debug issues encountered during implementation.

### Completion Notes List

**Implementation Summary:**

1. **ProfileOverride struct** - Created in `profiles.rs` with all overridable fields (output_folder, jira, squash, llm, templates). Uses `#[serde(deny_unknown_fields)]` for strict validation.

2. **ProjectConfig extensions** - Added `profiles: Option<HashMap<String, ProfileOverride>>` and `active_profile: Option<String>` fields to support profile storage and tracking.

3. **Profile selection API** - Implemented `with_profile(&str)` method that:
   - Looks up profile by name in the profiles map
   - Returns `ConfigError::ProfileNotFound` with available profiles list if not found
   - Applies profile overrides and sets `active_profile`

4. **Merge logic** - Implemented `apply_profile(&ProfileOverride)` method that:
   - Overrides fields when profile specifies them
   - Preserves base config values when profile field is None
   - Allows profile chaining (multiple apply_profile calls)

5. **Security** - Custom `Debug` impl for `ProfileOverride` delegates to child structs (JiraConfig, LlmConfig, SquashConfig) which already redact secrets. Implemented `Redact` trait for consistency.

6. **Summary API** - `active_profile_summary()` method provides human-readable configuration state.

**Test Results:**
- 241 tests pass (200 unit + 8 integration + 15 profile integration + 14 profile unit + 7 doc-tests)
- No regressions from Story 0.1
- All Acceptance Criteria covered by tests

**Design Decisions:**
- Used method-based API (`config.with_profile("dev")`) instead of standalone function for better ergonomics
- `ProfileNotFound` uses `requested`/`available` field names for clarity
- Profile does NOT modify `project_name` (not part of ProfileOverride) per design

**Code Review Follow-ups Resolved (2026-02-05):**

1. ✅ **[HIGH] Validation after merge** - Added `validate_config(&merged)?` call in `with_profile()` to ensure profile overrides don't break configuration invariants (path traversal, empty values, invalid LLM configs).

2. ✅ **[HIGH] active_profile_summary() tests** - Added 2 tests in `profile_unit_tests.rs`:
   - `test_active_profile_summary_shows_profile_and_values` - verifies profile name and effective values displayed
   - `test_active_profile_summary_no_profile` - verifies graceful handling when no profile active

3. ✅ **[MEDIUM] sprint-status.yaml documented** - Added to File List as modified file.

4. ✅ **[MEDIUM] is_safe_path() for profiles** - Now enforced via `validate_config` call after merge. Added tests:
   - `test_with_profile_rejects_path_traversal_in_output_folder`
   - `test_with_profile_rejects_empty_output_folder`

5. ✅ **[MEDIUM] Removed duplicate Redact trait** - Deleted Redact trait definition from profiles.rs, now imports from config.rs.

6. ✅ **[MEDIUM] Doctests converted** - Changed from `ignore` to `no_run` or fully compilable examples in config.rs and profiles.rs.

7. ✅ **[LOW] Empty output_folder test** - Added test fixture `config_profile_invalid_path.yaml` with empty_path profile and corresponding test.

8. ✅ **[LOW] ProfileConfig stub removed** - Deleted deprecated backward-compat stub from profiles.rs and lib.rs export.

**Code Review Follow-ups Round 2 Resolved (2026-02-05):**

1. ✅ **[HIGH] Test for profiles: None edge case** - Added `test_with_profile_on_config_without_profiles_section` in profile_tests.rs to verify proper error handling when config has no profiles section.

2. ✅ **[HIGH] Test for invalid Jira URL in profile** - Added `test_with_profile_rejects_invalid_jira_url_after_merge` with new fixture `config_profile_invalid_jira.yaml` to verify validate_config() rejects invalid URLs after merge.

3. ✅ **[MEDIUM] Removed duplicate tests** - Deleted 6 duplicate tests from profiles.rs (now only in profile_unit_tests.rs): test_profile_override_default_has_all_none, test_profile_override_is_clone, test_profile_override_redacts_jira_token_in_debug, test_profile_override_redacts_squash_password_in_debug, test_profile_override_redacts_llm_api_key_in_debug.

4. ✅ **[MEDIUM] Improved ProfileNotFound message** - Added `format_available_profiles()` helper in error.rs to show "No profiles defined in configuration. Add a 'profiles' section to config.yaml." when available list is empty.

5. ✅ **[LOW] Updated profile_tests.rs doc-comments** - Removed TDD "types that DON'T EXIST YET" language, updated to reflect implemented status.

6. ✅ **[LOW] Updated profile_unit_tests.rs doc-comments** - Removed TDD "will FAIL TO COMPILE" language, updated module documentation.

7. ✅ **[LOW] Renamed misleading test** - Changed `test_profile_override_can_remove_section` to `test_profile_with_none_preserves_base_value` with updated documentation.

**Code Review Follow-ups Round 3 Resolved (2026-02-05):**

1. ✅ **[MEDIUM] Removed obsolete TDD comment** - Deleted "These imports will fail until ProfileOverride is implemented" comment from profile_unit_tests.rs.

2. ✅ **[MEDIUM] Test for LlmConfig invalid after merge** - Added `test_with_profile_rejects_invalid_llm_config_after_merge` and `test_with_profile_accepts_valid_llm_config` tests with new fixture `config_profile_invalid_llm.yaml` to verify validate_config() rejects mode=cloud + cloud_enabled=false after merge.

3. ✅ **[MEDIUM] Test for ProfileOverride::redacted()** - Added `test_profile_override_redacted_trait_method` that directly tests the `.redacted()` method instead of only `format!("{:?}")`, verifying secrets are redacted and output matches Debug formatting.

4. ✅ **[LOW] Removed redundant test** - Deleted `test_with_profile_validates_merged_config_path_traversal` from profile_unit_tests.rs (duplicate of test in profile_tests.rs).

5. ✅ **[LOW] Added .redacted() example in docs** - Added comprehensive usage example in profiles.rs module documentation showing how to safely log ProfileOverride with secrets redacted.

6. ✅ **[LOW] Harmonized summary format** - Changed `active_profile_summary()` to use consistent format: "Jira: {url}", "Squash: {url}", "LLM: {mode}" instead of mixed "Jira endpoint:" vs "LLM mode:".

7. ✅ **[LOW] Added local_endpoint to fixture** - Added `local_endpoint: "http://localhost:11434"` to config_with_profiles.yaml LLM section for robustness.

**Code Review Follow-ups Round 4 Resolved (2026-02-05):**

1. ✅ **[MEDIUM] Test for templates override** - Added `test_profile_merge_overrides_templates` in profile_tests.rs with new `with_templates` profile in fixture.

2. ✅ **[MEDIUM] PartialEq derive** - Added `#[derive(PartialEq)]` to ProfileOverride, JiraConfig, SquashConfig, TemplatesConfig, and LlmConfig for improved testability.

3. ✅ **[MEDIUM] Config path in ProfileNotFound** - DEFERRED: Would require API changes to add source_path to ProjectConfig. Current behavior already satisfies AC #2 (error with available profiles). Call site can add file context if needed.

4. ✅ **[LOW] LLM profile in main fixture** - Added `with_llm` profile to config_with_profiles.yaml with test `test_profile_merge_overrides_llm_in_main_fixture`.

5. ✅ **[LOW] Strengthened assertion** - Updated test to verify exact format "LLM: local" instead of accepting "local" OR "Local".

6. ✅ **[LOW] Updated doc-comment** - Removed mention of "profile vs base tracking" from active_profile_summary() doc-comment as feature not implemented.

### File List

**Modified:**
- crates/tf-config/src/lib.rs - Updated exports for ProfileOverride, removed deprecated ProfileConfig backward-compat export
- crates/tf-config/src/config.rs - Added profiles/active_profile fields, with_profile (now with validate_config), apply_profile, active_profile_summary methods; fixed doctests; harmonized summary format; added PartialEq derive to JiraConfig/SquashConfig/TemplatesConfig/LlmConfig; updated active_profile_summary doc-comment; added templates to active_profile_summary() (Round 5)
- crates/tf-config/src/error.rs - Added ProfileNotFound variant
- crates/tf-config/src/profiles.rs - Replaced stub with full ProfileOverride implementation, removed duplicate Redact trait definition, removed deprecated ProfileConfig; fixed doctests; added .redacted() usage example in module doc; added PartialEq derive
- crates/tf-config/tests/fixtures/config_with_profiles.yaml - Fixed LLM validation; added local_endpoint; added with_templates and with_llm profiles (Round 4)
- crates/tf-config/tests/profile_tests.rs - 19 tests total including templates/Squash URL validation tests (Round 5)
- crates/tf-config/tests/profile_unit_tests.rs - 14 tests total; added PartialEq test (Round 5)
- crates/tf-config/README.md - Added comprehensive Configuration Profiles section (Round 5)
- _bmad-output/implementation-artifacts/sprint-status.yaml - Updated story status tracking

**Created:**
- crates/tf-config/tests/profile_tests.rs - Integration tests (19 tests including path traversal, templates, Squash URL validation)
- crates/tf-config/tests/profile_unit_tests.rs - Unit tests (14 tests including active_profile_summary and PartialEq tests)
- crates/tf-config/tests/fixtures/config_with_profiles.yaml - Multi-profile fixture
- crates/tf-config/tests/fixtures/config_profile_override_jira.yaml - Secret redaction fixture
- crates/tf-config/tests/fixtures/config_profile_invalid_path.yaml - Invalid path validation fixture
- crates/tf-config/tests/fixtures/config_profile_invalid_jira.yaml - Invalid Jira URL validation fixture (Round 2)
- crates/tf-config/tests/fixtures/config_profile_invalid_llm.yaml - Invalid LLM config validation fixture (Round 3)
- crates/tf-config/tests/fixtures/config_profile_invalid_squash.yaml - Invalid Squash URL validation fixture (Round 5)
- crates/tf-config/tests/fixtures/config_profile_invalid_templates.yaml - Invalid templates path traversal fixture (Round 5)

### Change Log

- 2026-02-05: Story 0-2 implementation complete. Added configuration profiles support with ProfileOverride, with_profile method, ProfileNotFound error, and comprehensive tests.
- 2026-02-05: Addressed code review findings - 8 items resolved. Added validate_config after merge, tests for active_profile_summary, tests for path traversal validation, removed duplicate Redact trait, converted doctests from ignore to no_run/compilable, removed deprecated ProfileConfig stub.
- 2026-02-05: Code review round 2 - 7 new action items created (2 HIGH, 2 MEDIUM, 3 LOW). Missing tests for edge cases (profiles: None, invalid URL in profile), duplicate tests, and outdated TDD comments.
- 2026-02-05: **Code review round 2 resolved - 7 items fixed.** Added tests for profiles: None and invalid Jira URL edge cases, removed 6 duplicate tests from profiles.rs, improved ProfileNotFound message for empty profiles, updated outdated TDD doc-comments, renamed misleading test. 238 tests pass.
- 2026-02-05: Code review round 3 - 7 new action items created (0 HIGH, 3 MEDIUM, 4 LOW). Minor cleanup: obsolete TDD comments, missing test for LlmConfig validation after merge, redundant test, documentation improvements.
- 2026-02-05: **Code review round 3 resolved - 7 items fixed.** Removed obsolete TDD comment, added test for LlmConfig invalid after merge, added test for ProfileOverride::redacted() trait method, removed redundant path traversal test, added .redacted() usage example in module doc, harmonized summary format (Jira/Squash/LLM all use "X: value"), added local_endpoint to fixture. 241 tests pass.
- 2026-02-05: Code review round 4 - 6 new action items created (0 HIGH, 3 MEDIUM, 3 LOW). Missing test for templates override, PartialEq derive, doc-comment accuracy. All ACs verified implemented. 241 tests pass.
- 2026-02-05: **Code review round 4 resolved - 6 items addressed.** Added test for templates override, added PartialEq derive to ProfileOverride and dependent structs, deferred config path in error (AC #2 satisfied), added with_llm profile and test, strengthened assertion, updated doc-comment. 243 tests pass. Story complete and ready for review.
- 2026-02-05: Code review round 5 - 6 new action items created (0 HIGH, 3 MEDIUM, 3 LOW). Documentation gaps (README missing profiles section), test coverage gaps (templates path traversal, squash invalid URL), minor improvements. All ACs verified implemented. 243 tests pass.
- 2026-02-05: **Code review round 5 resolved - 6 items addressed.** Added comprehensive Configuration Profiles section to README.md, added tests for templates path traversal and Squash invalid URL, added templates to active_profile_summary(), added PartialEq test, deferred fixtures consolidation for maintainability. 248 tests pass. Story complete and ready for review.
- 2026-02-05: **Code review round 6 (final) - APPROVED.** All 3 ACs verified implemented with full test coverage. 248 tests pass. 3 LOW findings (all previously deferred). Story marked as done.


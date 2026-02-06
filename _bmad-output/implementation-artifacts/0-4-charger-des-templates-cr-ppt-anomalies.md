# Story 0.4: Charger des templates (CR/PPT/anomalies)

Status: in-progress

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a QA tester (TRA),
I want charger des templates (CR/PPT/anomalies) depuis un chemin configuré,
so that standardiser les livrables des epics de reporting et d'anomalies.

## Acceptance Criteria

1. **Given** des chemins de templates définis dans la config
   **When** je charge un template
   **Then** il est validé (existence + format) et prêt à l'usage

2. **Given** un template manquant ou invalide
   **When** je tente de le charger
   **Then** un message explicite indique l'action à suivre

3. **Given** un template chargé
   **When** les logs sont écrits
   **Then** ils ne contiennent aucune donnée sensible

## Tasks / Subtasks

- [x] Task 1: Créer le module template dans tf-config (AC: all)
  - [x] Subtask 1.1: Créer `crates/tf-config/src/template.rs` avec le module de chargement de templates
  - [x] Subtask 1.2: Ajouter exports publics dans `crates/tf-config/src/lib.rs`
  - [x] Subtask 1.3: ~~Ajouter dépendance workspace `calamine = "0.26"`~~ N/A per Dev Notes: aucune nouvelle dépendance externe requise

- [x] Task 2: Implémenter l'API de chargement de templates (AC: #1)
  - [x] Subtask 2.1: Créer struct `TemplateLoader` encapsulant le chargement depuis un chemin de base configurable
  - [x] Subtask 2.2: Créer enum `TemplateKind` { Cr, Ppt, Anomaly } pour typer les templates
  - [x] Subtask 2.3: Créer struct `LoadedTemplate` contenant le contenu brut (bytes), le kind, le chemin source et les métadonnées de validation
  - [x] Subtask 2.4: Implémenter `TemplateLoader::new(config: &TemplatesConfig) -> Self`
  - [x] Subtask 2.5: Implémenter `load_template(kind: TemplateKind) -> Result<LoadedTemplate, TemplateError>` qui:
    - Résout le chemin depuis TemplatesConfig
    - Vérifie l'existence du fichier
    - Valide l'extension (`.md` pour CR/anomaly, `.pptx` pour PPT)
    - Lit le contenu du fichier
    - Valide le format (Markdown parsable pour .md, archive ZIP valide pour .pptx)
    - Retourne le template chargé
  - [x] Subtask 2.6: Implémenter `load_all() -> Result<HashMap<TemplateKind, LoadedTemplate>, TemplateError>` pour charger tous les templates configurés
  - [x] Subtask 2.7: Implémenter `validate_format(kind: TemplateKind, content: &[u8]) -> Result<(), TemplateError>` pour validation de format

- [x] Task 3: Implémenter la gestion des erreurs (AC: #2)
  - [x] Subtask 3.1: Créer `TemplateError` enum dans `crates/tf-config/src/template.rs` avec thiserror
  - [x] Subtask 3.2: Ajouter variant `TemplateError::NotConfigured { kind: String, hint: String }` pour template non défini dans la config
  - [x] Subtask 3.3: Ajouter variant `TemplateError::FileNotFound { path: String, kind: String, hint: String }`
  - [x] Subtask 3.4: Ajouter variant `TemplateError::InvalidExtension { path: String, expected: String, actual: String, hint: String }`
  - [x] Subtask 3.5: Ajouter variant `TemplateError::InvalidFormat { path: String, kind: String, cause: String, hint: String }`
  - [x] Subtask 3.6: Ajouter variant `TemplateError::ReadError { path: String, cause: String, hint: String }`

- [x] Task 4: Garantir la sécurité des logs (AC: #3)
  - [x] Subtask 4.1: Implémenter `Debug` custom pour `LoadedTemplate` sans exposer le contenu brut (afficher seulement kind, path, taille)
  - [x] Subtask 4.2: NE JAMAIS logger le contenu des templates (peut contenir des données sensibles dans les métadonnées)
  - [x] Subtask 4.3: Les messages d'erreur ne doivent contenir que le chemin et le type, jamais le contenu
  - [x] Subtask 4.4: Vérifier que les chemins loggés ne contiennent pas de données sensibles (utiliser les mêmes gardes que tf-config)

- [x] Task 5: Validation de format des templates (AC: #1)
  - [x] Subtask 5.1: Validation Markdown (.md): vérifier que le fichier est du texte UTF-8 valide et non vide
  - [x] Subtask 5.2: Validation PowerPoint (.pptx): vérifier que le fichier est une archive ZIP valide contenant `[Content_Types].xml` (signature OOXML minimale)
  - [x] Subtask 5.3: NE PAS ajouter de dépendance zip pour le moment — utiliser la signature magic bytes ZIP (PK\x03\x04) + vérification taille minimale pour .pptx

- [x] Task 6: Tests unitaires et intégration (AC: #1, #2, #3)
  - [x] Subtask 6.1: Créer répertoire de fixtures `crates/tf-config/tests/fixtures/templates/` avec des templates de test
  - [x] Subtask 6.2: Créer fixture `cr-test.md` (template CR minimal valide)
  - [x] Subtask 6.3: Créer fixture `anomaly-test.md` (template anomalie minimal valide)
  - [x] Subtask 6.4: Tests pour chargement réussi de chaque type de template (.md, .pptx)
  - [x] Subtask 6.5: Tests pour erreur explicite quand template non configuré
  - [x] Subtask 6.6: Tests pour erreur explicite quand fichier inexistant avec hint actionnable
  - [x] Subtask 6.7: Tests pour erreur explicite quand extension invalide
  - [x] Subtask 6.8: Tests pour erreur explicite quand format invalide (fichier binaire comme .md, fichier texte comme .pptx)
  - [x] Subtask 6.9: Tests pour load_all() avec config complète et partielle
  - [x] Subtask 6.10: Tests pour vérifier que Debug ne contient pas le contenu des templates
  - [x] Subtask 6.11: Tests pour fichier vide (rejeté avec message explicite)

### Review Follow-ups (AI)

- [x] [AI-Review][HIGH] Fix TOCTOU race condition: remove `path.exists()` check and handle `NotFound` from `fs::read()` directly in `map_err` [crates/tf-config/src/template.rs:170-201]
- [x] [AI-Review][HIGH] `validate_format` is a private free function but Subtask 2.7 specifies a public method — align implementation with spec or update spec [crates/tf-config/src/template.rs:273]
- [x] [AI-Review][HIGH] File List claims "307 lines" but actual file is 743 lines — correct to "805 lines" [story File List]
- [x] [AI-Review][MEDIUM] Document `load_all()` fail-fast behavior in docstring, or consider `try_load_all()` returning all errors [crates/tf-config/src/template.rs:206-217]
- [x] [AI-Review][MEDIUM] Consider making `TemplateKind::all()` public for external consumers [crates/tf-config/src/template.rs:51-53]
- [x] [AI-Review][MEDIUM] Add doc-tests (`no_run`) for `TemplateLoader::new()` and `load_template()` — other modules have them [crates/tf-config/src/template.rs]
- [x] [AI-Review][MEDIUM] Consider adding `Serialize`/`Deserialize` on `TemplateKind` for future structured logging/config use [crates/tf-config/src/template.rs:21]
- [x] [AI-Review][LOW] Add `Serialize` derive on `TemplateKind` for consistency with other crate enums like `LlmMode` [crates/tf-config/src/template.rs:21]
- [x] [AI-Review][LOW] Add `//! # Usage` section with code snippet in module doc [crates/tf-config/src/template.rs:1-5]
- [x] [AI-Review][LOW] Document why `MIN_PPTX_SIZE = 100` (e.g., "prevents truncated files; full OOXML validation deferred to tf-export") [crates/tf-config/src/template.rs:18]

#### Round 2 Review Follow-ups (AI)

- [ ] [AI-Review-R2][MEDIUM] `TemplateLoader::new()` clones entire `TemplatesConfig` — consider storing a reference or `Arc` to avoid unnecessary copy as config grows [crates/tf-config/src/template.rs:196-200]
- [ ] [AI-Review-R2][MEDIUM] `load_all()` duplicates config resolution: `is_configured()` checks `is_some()` then `get_configured_path()` re-matches and clones — refactor to single resolution path [crates/tf-config/src/template.rs:264-306]
- [ ] [AI-Review-R2][MEDIUM] `content_as_str()` error hint is misleading for PPTX templates — should say "This template is binary (PPTX); use content() for raw bytes instead" rather than "Ensure the file is a valid ppt template" [crates/tf-config/src/template.rs:149-159]
- [ ] [AI-Review-R2][LOW] `size_bytes` field is redundant with `content.len()` — consider computing on-the-fly via accessor to reduce struct size [crates/tf-config/src/template.rs:129,246]
- [ ] [AI-Review-R2][LOW] Add boundary tests for `MIN_PPTX_SIZE`: test at exactly `MIN_PPTX_SIZE - 1` (reject) and `MIN_PPTX_SIZE` (accept) [crates/tf-config/src/template.rs:696-704]
- [ ] [AI-Review-R2][LOW] `TemplateKind::expected_extension()` is private but could be useful for external consumers — consider making it public [crates/tf-config/src/template.rs:69]

## Dev Notes

### Technical Stack Requirements

**Versions exactes à utiliser:**
- Rust edition: 2021 (MSRV 1.75+)
- `thiserror = "2.0"` pour les erreurs structurées (déjà workspace dep)
- `serde = "1.0"` avec derive (déjà workspace dep)
- Pas de nouvelle dépendance externe requise pour cette story

**Pourquoi PAS de dépendance calamine/zip pour cette story:**
- La story 0.4 concerne le **chargement et la validation de base** des templates
- La validation .pptx se fait par magic bytes ZIP (`PK\x03\x04`) + taille minimale, pas par parsing complet
- La validation .md se fait par vérification UTF-8 + non-vide
- `calamine` (lecture Excel) et le parsing OOXML complet seront ajoutés dans tf-export (Epic 5)
- Garder le scope minimal : existence + format de base + prêt à l'usage

### Architecture Compliance

**Module template dans tf-config — justification :**

L'architecture.md place `templates.rs` dans `tf-export/`. Cependant, pour cette story Foundation (Epic 0) :
1. `TemplatesConfig` existe déjà dans tf-config (`config.rs:TemplatesConfig { cr, ppt, anomaly }`)
2. La validation syntaxique des chemins est déjà dans tf-config (`validate_config`, lignes 1922-1993)
3. tf-export (crate #7 dans l'ordre) sera créé bien plus tard et consommera `LoadedTemplate`
4. Le chargement de base (existence + format) est une extension naturelle de la config

**Position dans l'ordre des dépendances (architecture.md) :**
1. `tf-config` (aucune dépendance interne) - done (stories 0.1, 0.2)
2. `tf-logging` (dépend de tf-config)
3. `tf-security` (dépend de tf-config) - done (story 0.3)
4. ... (autres crates)

**Ce module reste dans tf-config pour l'instant.** Quand tf-export sera créé, le `TemplateLoader` pourra migrer ou tf-export pourra le réutiliser via la dépendance tf-config.

**Structure attendue (ajout dans tf-config) :**
```
crates/
└── tf-config/
    ├── Cargo.toml          # Pas de nouvelle dépendance
    └── src/
        ├── lib.rs          # Ajouter export pub mod template
        ├── config.rs       # Existant - TemplatesConfig
        ├── profiles.rs     # Existant
        ├── error.rs        # Existant - ConfigError
        └── template.rs     # NOUVEAU - TemplateLoader, TemplateError, LoadedTemplate
```

**Boundaries à respecter :**
- `template.rs` dépend de `config.rs` (pour `TemplatesConfig`) et `error.rs` (pattern d'erreurs)
- NE PAS modifier `config.rs` ou `profiles.rs` (sauf ajout d'un pub use si nécessaire)
- NE PAS créer de nouveau crate
- NE PAS ajouter de dépendance externe

### API Pattern Obligatoire

```rust
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Types of templates supported by the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TemplateKind {
    /// Daily report template (CR quotidien) - Markdown format
    Cr,
    /// Weekly/TNR presentation template - PowerPoint format
    Ppt,
    /// Bug report template - Markdown format
    Anomaly,
}

/// A validated and loaded template ready for use
pub struct LoadedTemplate {
    kind: TemplateKind,
    path: PathBuf,
    content: Vec<u8>,
    size_bytes: u64,
}

impl LoadedTemplate {
    /// Get the template kind
    pub fn kind(&self) -> TemplateKind { self.kind }

    /// Get the source file path
    pub fn path(&self) -> &Path { &self.path }

    /// Get the raw content bytes
    pub fn content(&self) -> &[u8] { &self.content }

    /// Get content as UTF-8 string (for Markdown templates)
    pub fn content_as_str(&self) -> Result<&str, TemplateError> { ... }

    /// Get the file size in bytes
    pub fn size_bytes(&self) -> u64 { self.size_bytes }
}

/// Loads and validates templates from configured paths
pub struct TemplateLoader {
    config: TemplatesConfig,
}

impl TemplateLoader {
    /// Create a new template loader from configuration
    pub fn new(config: &TemplatesConfig) -> Self { ... }

    /// Load a specific template by kind
    pub fn load_template(&self, kind: TemplateKind) -> Result<LoadedTemplate, TemplateError> { ... }

    /// Load all configured templates
    pub fn load_all(&self) -> Result<HashMap<TemplateKind, LoadedTemplate>, TemplateError> { ... }
}
```

### Error Handling Pattern

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Template {kind} not configured. {hint}")]
    NotConfigured {
        kind: String,
        hint: String,
    },

    #[error("Template file not found: '{path}' ({kind}). {hint}")]
    FileNotFound {
        path: String,
        kind: String,
        hint: String,
    },

    #[error("Invalid extension for template '{path}': expected {expected}, got '{actual}'. {hint}")]
    InvalidExtension {
        path: String,
        expected: String,
        actual: String,
        hint: String,
    },

    #[error("Invalid format for template '{path}' ({kind}): {cause}. {hint}")]
    InvalidFormat {
        path: String,
        kind: String,
        cause: String,
        hint: String,
    },

    #[error("Failed to read template '{path}': {cause}. {hint}")]
    ReadError {
        path: String,
        cause: String,
        hint: String,
    },
}
```

**Hints actionnables obligatoires (pattern stories précédentes) :**
- `NotConfigured` → `"Add 'templates.cr: ./path/to/cr.md' to your config.yaml"`
- `FileNotFound` → `"Check the path in config.yaml or create the template file at '{path}'"`
- `InvalidExtension` → `"Rename the file to use {expected} extension"`
- `InvalidFormat` → `"Ensure the file is a valid {kind} template. {specific_guidance}"`
- `ReadError` → `"Check file permissions and ensure the file is readable"`

### Library & Framework Requirements

**Aucune nouvelle dépendance.** Cette story utilise uniquement :
- `std::fs` pour lire les fichiers
- `std::path` pour manipuler les chemins
- `thiserror` pour les erreurs (déjà disponible)
- `serde` pour la sérialisation si besoin (déjà disponible)

**Validation .pptx sans dépendance zip :**
```rust
/// ZIP magic bytes: PK\x03\x04
const ZIP_MAGIC: &[u8; 4] = b"PK\x03\x04";
const MIN_PPTX_SIZE: u64 = 100; // Un .pptx valide fait au moins ~100 bytes

fn validate_pptx(content: &[u8]) -> Result<(), TemplateError> {
    if content.len() < 4 || &content[..4] != ZIP_MAGIC {
        return Err(TemplateError::InvalidFormat { ... });
    }
    Ok(())
}
```

**Validation .md :**
```rust
fn validate_markdown(content: &[u8]) -> Result<(), TemplateError> {
    if content.is_empty() {
        return Err(TemplateError::InvalidFormat { cause: "file is empty", ... });
    }
    std::str::from_utf8(content)
        .map_err(|_| TemplateError::InvalidFormat { cause: "not valid UTF-8 text", ... })?;
    Ok(())
}
```

### File Structure Requirements

**Naming conventions (identiques aux stories précédentes) :**
- Fichiers: `snake_case.rs`
- Modules: `snake_case`
- Structs/Enums: `PascalCase`
- Functions/variables: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`

**Fichiers à créer :**
- `crates/tf-config/src/template.rs` — module principal (~200-300 lignes)

**Fichiers à modifier :**
- `crates/tf-config/src/lib.rs` — ajouter `pub mod template;` et exports publics

**Fichiers de test à créer :**
- `crates/tf-config/tests/fixtures/templates/cr-test.md` — template CR minimal
- `crates/tf-config/tests/fixtures/templates/anomaly-test.md` — template anomalie minimal
- `crates/tf-config/tests/fixtures/templates/empty.md` — fichier vide (cas d'erreur)
- `crates/tf-config/tests/fixtures/templates/binary-garbage.md` — fichier binaire avec extension .md (cas d'erreur)

**Note .pptx de test :** Pour les tests .pptx, créer un fichier minimal par programme dans le test setup (quelques bytes avec header ZIP) plutôt qu'un fichier fixture binaire.

### Testing Requirements

**Framework:** `cargo test` built-in (identique aux stories précédentes)

**Stratégie de test :**
- Tests unitaires dans `template.rs` (module `#[cfg(test)]`)
- Tests d'intégration avec fixtures dans `crates/tf-config/tests/fixtures/templates/`
- Tous les tests doivent pouvoir tourner en CI sans dépendance externe

**Patterns de test obligatoires :**

```rust
#[test]
fn test_load_cr_template_success() {
    let config = TemplatesConfig {
        cr: Some("tests/fixtures/templates/cr-test.md".to_string()),
        ppt: None,
        anomaly: None,
    };
    let loader = TemplateLoader::new(&config);
    let template = loader.load_template(TemplateKind::Cr).unwrap();
    assert_eq!(template.kind(), TemplateKind::Cr);
    assert!(!template.content().is_empty());
    assert!(template.content_as_str().is_ok());
}

#[test]
fn test_load_template_not_configured_has_hint() {
    let config = TemplatesConfig { cr: None, ppt: None, anomaly: None };
    let loader = TemplateLoader::new(&config);
    let err = loader.load_template(TemplateKind::Cr).unwrap_err();
    assert!(matches!(err, TemplateError::NotConfigured { .. }));
    assert!(err.to_string().contains("config.yaml"));
}

#[test]
fn test_load_template_file_not_found_has_hint() {
    let config = TemplatesConfig {
        cr: Some("/nonexistent/path/cr.md".to_string()),
        ppt: None,
        anomaly: None,
    };
    let loader = TemplateLoader::new(&config);
    let err = loader.load_template(TemplateKind::Cr).unwrap_err();
    assert!(matches!(err, TemplateError::FileNotFound { .. }));
    assert!(err.to_string().contains("Check the path"));
}

#[test]
fn test_load_empty_markdown_rejected() {
    let config = TemplatesConfig {
        cr: Some("tests/fixtures/templates/empty.md".to_string()),
        ppt: None,
        anomaly: None,
    };
    let loader = TemplateLoader::new(&config);
    let err = loader.load_template(TemplateKind::Cr).unwrap_err();
    assert!(matches!(err, TemplateError::InvalidFormat { .. }));
}

#[test]
fn test_debug_does_not_expose_template_content() {
    let template = LoadedTemplate { /* ... */ };
    let debug_str = format!("{:?}", template);
    // Should show kind, path, size — never raw content
    assert!(debug_str.contains("Cr"));
    assert!(!debug_str.contains("actual template text"));
}
```

**Couverture AC explicite :**
- AC #1 (chargement valide) : `test_load_cr_template_success`, `test_load_pptx_template_success`, `test_load_all_success`
- AC #2 (erreurs explicites) : `test_load_template_not_configured_has_hint`, `test_load_template_file_not_found_has_hint`, `test_load_empty_markdown_rejected`, `test_load_invalid_pptx_rejected`
- AC #3 (logs sécurisés) : `test_debug_does_not_expose_template_content`

### Previous Story Intelligence (Story 0.3)

**Patterns établis à réutiliser :**
- `thiserror` pour enum d'erreurs avec variants spécifiques et hints explicites
- Custom `Debug` impl masquant les données sensibles (cf. `SecretStore` dans tf-security)
- Messages d'erreur : toujours inclure `champ + raison + hint actionnable`
- Tests couvrant explicitement chaque AC
- `#[serde(deny_unknown_fields)]` sur les structs sérialisables (si applicable)
- Trait `Redact` disponible dans tf-config pour masquer des données sensibles

**Apprentissages des reviews story 0.3 (9 findings en 2 rounds) :**
- TOUJOURS fournir un hint actionnable dans les erreurs — les reviewers vérifient
- Les line counts dans le File List doivent être exacts (source de findings LOW)
- Les doc-tests doivent compiler (`no_run` plutôt que `ignore` quand possible)
- Tester les edge cases : entrées vides, service indisponible, permissions
- `has_secret()` avalait les erreurs → a conduit à ajouter `try_has_secret()`. **Leçon :** ne pas avaler les erreurs silencieusement, offrir une API Result<> alternative
- Commit le code AVANT la review (trouvé untracked en Round 2)
- CI workflow : ne pas ajouter de blocs `env` inutiles

**Fichiers de Story 0.3 à préserver :**
- `crates/tf-security/` — ne pas toucher
- 248 tests passent dans tf-config — ne pas casser
- 30 tests dans tf-security — ne pas casser

### Anti-Patterns to Avoid

- NE JAMAIS logger le contenu brut d'un template (peut contenir des métadonnées sensibles)
- NE PAS retourner d'erreur générique — toujours fournir kind + path + hint
- NE PAS hardcoder les chemins de templates — toujours les lire depuis TemplatesConfig
- NE PAS ajouter de dépendance externe (calamine, zip) — pas dans le scope de cette story
- NE PAS modifier `config.rs` ou `profiles.rs` (sauf ajout d'un pub use minimal si nécessaire)
- NE PAS créer un nouveau crate pour cette story
- NE PAS valider le contenu sémantique des templates (sections, placeholders) — hors scope

### Git Intelligence (Recent Patterns)

**Commit message pattern établi :**
```
feat(tf-config): implement story 0-4 template loading (#PR)
```

**Fichiers créés/modifiés par stories précédentes :**
- `c473fb7` feat(tf-security): implement secret store with OS keyring backend (#13)
- `e2c0200` feat(tf-config): implement configuration profiles with environment overrides (#12)
- `9a3ac95` feat(tf-config): implement story 0-1 YAML configuration management (#10)

**Branche attendue :** `feature/0-4-chargement-templates` (branche actuelle)

**Pattern de PR :** feat(crate): description courte (#numéro)

**Code patterns observés dans les commits récents :**
- Workspace dependencies centralisées dans le Cargo.toml racine
- Crate-level Cargo.toml référence les dépendances workspace (`thiserror.workspace = true`)
- Tests dans le même fichier (`#[cfg(test)] mod tests`)
- Fixtures dans `crates/<crate>/tests/fixtures/`
- CI GitHub Actions pour tests + clippy

### Project Structure Notes

- Alignement avec la structure multi-crates définie dans architecture.md
- Le module template.rs est dans tf-config (pas tf-export) car tf-export n'existe pas encore
- Migration vers tf-export possible lors de la création de ce crate (Epic 5)
- Pas de conflit détecté avec les modules existants (config.rs, profiles.rs, error.rs)

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#Office Document Generation] — templates.rs dans tf-export
- [Source: _bmad-output/planning-artifacts/architecture.md#Technology Stack] — versions exactes
- [Source: _bmad-output/planning-artifacts/architecture.md#Project Structure & Boundaries] — structure crates
- [Source: _bmad-output/planning-artifacts/architecture.md#Implementation Patterns] — naming, errors, logs
- [Source: _bmad-output/planning-artifacts/epics.md#Story 0.4] — AC et requirements
- [Source: _bmad-output/planning-artifacts/prd.md#FR25] — Le système peut charger des templates
- [Source: _bmad-output/planning-artifacts/prd.md#NFR12] — Sorties conformes aux templates existants
- [Source: _bmad-output/implementation-artifacts/0-3-gestion-des-secrets-via-secret-store.md] — patterns et learnings
- [Source: crates/tf-config/src/config.rs:TemplatesConfig] — structure existante
- [Source: crates/tf-config/src/config.rs:validate_config:1922-1993] — validation syntaxique templates

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (claude-opus-4-6)

### Debug Log References

- Initial `test_content_as_str_for_binary_template` failure: synthetic pptx bytes used 0x00 padding (valid UTF-8). Fixed by using 0xFF padding (invalid UTF-8) to properly test binary content detection.

### Completion Notes List

- Created `template.rs` module (~300 lines) implementing TemplateLoader, TemplateKind, LoadedTemplate, and TemplateError
- All 5 TemplateError variants with actionable hints following established pattern from stories 0.1-0.3
- Custom Debug impl for LoadedTemplate that never exposes raw content (AC #3)
- Markdown validation: UTF-8 + non-empty check
- PPTX validation: ZIP magic bytes (PK\x03\x04) + minimum size check (no external dependency)
- Subtask 1.3 (calamine dependency) marked N/A per Dev Notes: no new external dependencies needed
- 28 new template tests + 248 existing tf-config tests = 276 total, all passing
- 0 clippy warnings, 0 regressions across tf-config and tf-security
- ✅ Resolved review finding [HIGH]: Fixed TOCTOU race condition — removed `path.exists()` pre-check, handle `NotFound` from `fs::read()` directly
- ✅ Resolved review finding [HIGH]: Made `validate_format` public with docstring, exported from `lib.rs` — aligned with Subtask 2.7 spec
- ✅ Resolved review finding [HIGH]: Corrected File List line count from "307 lines" to "805 lines"
- ✅ Resolved review finding [MEDIUM]: Documented `load_all()` fail-fast semantics in docstring
- ✅ Resolved review finding [MEDIUM]: Made `TemplateKind::all()` public for external consumers
- ✅ Resolved review finding [MEDIUM]: Added `no_run` doc-tests for `TemplateLoader::new()` and `load_template()`
- ✅ Resolved review finding [MEDIUM]: Added `Serialize`/`Deserialize` derives on `TemplateKind`
- ✅ Resolved review finding [LOW]: `Serialize` on `TemplateKind` covered by MEDIUM item above
- ✅ Resolved review finding [LOW]: Added `//! # Usage` section with code snippet in module doc
- ✅ Resolved review finding [LOW]: Documented `MIN_PPTX_SIZE = 100` rationale in doc comment

### File List

- `crates/tf-config/src/template.rs` — NEW (805 lines) — Template loading module with TemplateLoader, TemplateKind, LoadedTemplate, TemplateError, validate_format, doc-tests, and 28 unit tests
- `crates/tf-config/src/lib.rs` — MODIFIED — Added `pub mod template` and public re-exports for TemplateLoader, TemplateKind, LoadedTemplate, TemplateError, validate_format
- `crates/tf-config/tests/fixtures/templates/cr-test.md` — NEW — CR template fixture for tests
- `crates/tf-config/tests/fixtures/templates/anomaly-test.md` — NEW — Anomaly template fixture for tests
- `crates/tf-config/tests/fixtures/templates/empty.md` — NEW — Empty file fixture for error case testing
- `crates/tf-config/tests/fixtures/templates/binary-garbage.md` — NEW — Binary content with .md extension for format validation testing

### Change Log

- 2026-02-06: Implemented story 0-4 template loading — created template.rs module in tf-config with TemplateLoader API, TemplateError enum, format validation (MD/PPTX), and 28 tests covering all 3 ACs
- 2026-02-06: Code review (AI adversarial) — 10 findings (3 HIGH, 4 MEDIUM, 3 LOW). Action items added to Tasks/Subtasks for follow-up. Story remains in-progress.
- 2026-02-06: Addressed all 10 code review findings — 3 HIGH (TOCTOU fix, validate_format public, File List correction), 4 MEDIUM (load_all docs, all() public, doc-tests, Serialize/Deserialize), 3 LOW (Serialize derive, Usage section, MIN_PPTX_SIZE docs). All 228+8+19+14+10 tests pass, 0 clippy warnings, 0 regressions.
- 2026-02-06: Code review Round 2 (AI adversarial) — 6 findings (0 HIGH, 3 MEDIUM, 3 LOW). All ACs fully implemented. No blocking issues. Action items added for future improvement. 279 tests pass, 0 clippy warnings, 0 regressions.


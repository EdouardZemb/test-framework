# Story 0.5: Journalisation baseline sans donnees sensibles

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a QA maintainer,
I want une journalisation baseline sans donnees sensibles,
so that garantir l'auditabilite minimale des executions des le debut.

## Acceptance Criteria

1. **Given** la journalisation activee
   **When** une commande CLI s'execute
   **Then** des logs JSON structures sont generes (timestamp, commande, statut, perimetre)

2. **Given** des champs sensibles sont presents dans le contexte
   **When** ils seraient journalises
   **Then** ils sont masques automatiquement

3. **Given** une execution terminee
   **When** les logs sont ecrits
   **Then** ils sont stockes dans le dossier de sortie configure

## Tasks / Subtasks

- [ ] Task 1: Creer le crate tf-logging dans le workspace (AC: all)
  - [ ] Subtask 1.0: Ajouter `"crates/tf-logging"` dans la liste `members` de `[workspace]` du `Cargo.toml` racine
  - [ ] Subtask 1.1: Creer `crates/tf-logging/Cargo.toml` avec dependances workspace (`tracing`, `tracing-subscriber`, `tracing-appender`, `serde`, `serde_json`, `thiserror`) + dependance interne `tf-config`
  - [ ] Subtask 1.2: Creer `crates/tf-logging/src/lib.rs` avec exports publics
  - [ ] Subtask 1.3: Ajouter les nouvelles dependances workspace dans `Cargo.toml` racine : `tracing = "0.1"`, `tracing-subscriber = { version = "0.3", features = ["json", "env-filter", "fmt"] }`, `tracing-appender = "0.2"`

- [ ] Task 2: Implementer le module d'initialisation du logging (AC: #1, #3)
  - [ ] Subtask 2.1: Creer `crates/tf-logging/src/init.rs` avec la fonction publique `init_logging(config: &LoggingConfig) -> Result<LogGuard, LoggingError>`
  - [ ] Subtask 2.2: Configurer `tracing-subscriber` avec format JSON structure (timestamp RFC 3339 UTC, level, message, target, spans)
  - [ ] Subtask 2.3: Configurer `tracing-appender::rolling::RollingFileAppender` avec rotation DAILY et ecriture dans `{output_folder}/logs/`
  - [ ] Subtask 2.4: Utiliser `tracing_appender::non_blocking()` pour performance non-bloquante ; retourner un `LogGuard` wrappant le `WorkerGuard` pour garantir le flush
  - [ ] Subtask 2.5: Supporter la configuration du niveau de log via `EnvFilter` (RUST_LOG en priorite, sinon `info` par defaut). Tant que `ProjectConfig` n'expose pas de champ logging dedie, ne pas introduire de dependance a `config.log_level`.
  - [ ] Subtask 2.6: Desactiver ANSI colors pour les logs fichier (`with_ansi(false)`)

- [ ] Task 3: Implementer le layer de redaction des champs sensibles (AC: #2)
  - [ ] Subtask 3.0: Exposer `redact_url_sensitive_params` comme `pub` dans `crates/tf-config/src/config.rs` (actuellement `pub(crate)`) et ajouter le re-export dans `crates/tf-config/src/lib.rs` pour que tf-logging puisse l'utiliser
  - [ ] Subtask 3.1: Creer `crates/tf-logging/src/redact.rs` avec un `RedactingLayer` implementant `tracing_subscriber::Layer`
  - [ ] Subtask 3.2: Definir la liste des noms de champs sensibles a masquer : `token`, `api_key`, `apikey`, `key`, `secret`, `password`, `passwd`, `pwd`, `auth`, `authorization`, `credential`, `credentials`
  - [ ] Subtask 3.3: Implementer un `RedactingVisitor` implementant `tracing::field::Visit` qui remplace les valeurs des champs sensibles par `[REDACTED]`
  - [ ] Subtask 3.4: Integrer le `RedactingLayer` dans la stack du subscriber (avant le layer JSON). Note technique : les events tracing sont immutables — l'approche recommandee est soit (a) implementer un custom `FormatEvent` qui redacte les champs avant ecriture JSON, soit (b) utiliser `Layer::on_event()` pour intercepter et re-emettre avec champs redactes. Privilegier l'approche la plus simple qui fonctionne avec `tracing-subscriber` 0.3.x
  - [ ] Subtask 3.5: Reutiliser `tf_config::redact_url_sensitive_params()` pour les champs contenant des URLs (detecter les valeurs qui ressemblent a des URLs et les redacter)

- [ ] Task 4: Implementer la configuration du logging (AC: #1, #3)
  - [ ] Subtask 4.1: Creer `crates/tf-logging/src/config.rs` avec struct `LoggingConfig { log_level: String, log_dir: String, log_to_stdout: bool }` (pas Option — le fallback est applique dans `from_project_config()`)
  - [ ] Subtask 4.2: Implementer la derivation de `LoggingConfig` depuis `ProjectConfig` : `log_dir = format!("{}/logs", config.output_folder)`, avec fallback sur `"./logs"` si `output_folder` est vide
  - [ ] Subtask 4.3: Creer le repertoire de logs s'il n'existe pas (`fs::create_dir_all`)
  - [ ] Subtask 4.4: Definir explicitement la source de `log_to_stdout` pour eviter toute ambiguite: valeur par defaut `false` dans `from_project_config()`, puis override explicite possible uniquement depuis tf-cli (mode interactif) avant appel a `init_logging`.

- [ ] Task 5: Implementer la gestion des erreurs (AC: all)
  - [ ] Subtask 5.1: Creer `crates/tf-logging/src/error.rs` avec `LoggingError` enum (thiserror)
  - [ ] Subtask 5.2: Ajouter variant `LoggingError::InitFailed { cause: String, hint: String }` pour echec d'initialisation
  - [ ] Subtask 5.3: Ajouter variant `LoggingError::DirectoryCreationFailed { path: String, cause: String, hint: String }` pour echec creation repertoire logs
  - [ ] Subtask 5.4: Ajouter variant `LoggingError::InvalidLogLevel { level: String, hint: String }` pour niveau de log invalide

- [ ] Task 6: Implementer le LogGuard et le lifecycle (AC: #3)
  - [ ] Subtask 6.1: Creer struct `LogGuard` wrappant `tracing_appender::non_blocking::WorkerGuard`
  - [ ] Subtask 6.2: `LogGuard` doit implementer `Drop` pour flusher les logs restants a la fermeture
  - [ ] Subtask 6.3: Documenter que le `LogGuard` doit etre garde vivant (`let _guard = init_logging(...)`) pendant toute la duree de l'application

- [ ] Task 7: Tests unitaires et integration (AC: #1, #2, #3)
  - [ ] Subtask 7.1: Test que `init_logging` cree le repertoire de logs et retourne un LogGuard valide
  - [ ] Subtask 7.2: Test que les logs JSON generes contiennent les champs requis : `timestamp`, `level`, `message`, `target`
  - [ ] Subtask 7.3: Test que les champs sensibles (`token`, `password`, `api_key`, etc.) sont masques par `[REDACTED]` dans la sortie
  - [ ] Subtask 7.4: Test que les URLs contenant des parametres sensibles sont redactees
  - [ ] Subtask 7.5: Test que les logs sont bien ecrits dans le repertoire configure (`{output_folder}/logs/`)
  - [ ] Subtask 7.6: Test que le niveau de log par defaut est `info`
  - [ ] Subtask 7.7: Test que RUST_LOG override le niveau configure
  - [ ] Subtask 7.8: Test que LoggingError contient des hints actionnables
  - [ ] Subtask 7.9: Test que Debug impl de LogGuard ne contient aucune donnee sensible
  - [ ] Subtask 7.10: Test d'integration : simuler une commande CLI complete et verifier le contenu du fichier log JSON
  - [ ] Subtask 7.11: Test de non-regression : executer `cargo test --workspace` et verifier que l'ensemble de la suite de tests passe toujours apres ajout de tf-logging (sans se baser sur un nombre fixe de tests).

## Dev Notes

### Technical Stack Requirements

**Versions exactes a utiliser (depuis architecture.md) :**
- Rust edition: 2021 (MSRV 1.75+)
- `tracing = "0.1"` (derniere stable: 0.1.44)
- `tracing-subscriber = "0.3"` avec features `["json", "env-filter", "fmt"]` (derniere stable: 0.3.22)
- `tracing-appender = "0.2"` (derniere stable: 0.2.4)
- `thiserror = "2.0"` pour les erreurs structurees (deja workspace dep)
- `serde = "1.0"` avec derive (deja workspace dep)
- `serde_json = "1.0"` (deja workspace dep)

**Dependance interne :**
- `tf-config` pour acceder a `ProjectConfig.output_folder`, au trait `Redact` et a `redact_url_sensitive_params()`

**Points critiques tracing-subscriber 0.3.x :**
- Feature `json` DOIT etre activee explicitement (retirée des defaults depuis 0.3.0)
- Utilise `time` crate (pas `chrono`) pour les timestamps — pas d'action necessaire, c'est interne
- `with_ansi(false)` n'est plus gate derriere la feature "ansi" depuis 0.3.19
- `EnvFilter` supporte des filtres complexes : `RUST_LOG=warn,tf_logging=debug`

### Architecture Compliance

**Position dans l'ordre des dependances (architecture.md) :**
1. `tf-config` (aucune dependance interne) - done (stories 0.1, 0.2, 0.4)
2. **`tf-logging` (depend de tf-config)** ← CETTE STORY
3. `tf-security` (depend de tf-config) - done (story 0.3)
4. `tf-storage` (depend de tf-config, tf-security)
5. ... (autres crates)

**Crate tf-logging — structure attendue :**

> **Note architecture:** architecture.md montre `mod.rs` + `logging.rs` comme structure simplifiee. L'implementation detaillee utilise `lib.rs` + modules separes (init.rs, redact.rs, config.rs, error.rs), ce qui est plus idiomatique en Rust et suit le pattern etabli par tf-config et tf-security. **Suivre la structure ci-dessous, pas celle de architecture.md.**

```
crates/
└── tf-logging/
    ├── Cargo.toml
    └── src/
        ├── lib.rs          # Public API: init_logging, LogGuard, LoggingError, LoggingConfig
        ├── init.rs         # Subscriber setup, file appender, non-blocking writer
        ├── redact.rs       # RedactingLayer, RedactingVisitor, SENSITIVE_FIELDS
        ├── config.rs       # LoggingConfig struct, derivation depuis ProjectConfig
        └── error.rs        # LoggingError enum
```

**Boundaries a respecter :**
- `tf-logging` depend de `tf-config` (pour `ProjectConfig`, `Redact`, `redact_url_sensitive_params`)
- `tf-logging` NE depend PAS de `tf-security` (pas besoin du keyring pour le logging)
- NE PAS modifier `tf-config` ou `tf-security` (sauf ajout d'un `pub` si une fonction de redaction n'est pas encore publique)
- NE PAS creer d'autre crate

**Format de sortie JSON (architecture.md) :**
```json
{
  "timestamp": "2026-02-06T10:30:45.123Z",
  "level": "INFO",
  "message": "Command executed",
  "target": "tf_cli::commands::run",
  "fields": {
    "command": "triage",
    "status": "success",
    "scope": "lot-42"
  }
}
```

**Convention exit codes (architecture.md) :**
- 0 OK, 1 General error, 2 Validation error, 3 Integration error
- Les logs doivent tracer le code de sortie final

### Existing Redaction Infrastructure to Reuse

**Trait `Redact`** (public, dans `tf-config::Redact`) : `fn redacted(&self) -> String` — implementations sur `JiraConfig`, `SquashConfig`, `LlmConfig`, `ProjectConfig`.

**`redact_url_sensitive_params(url: &str) -> String`** (dans `crates/tf-config/src/config.rs:214`) : redacte les params sensibles dans les URLs (token, api_key, password, etc. en snake_case/camelCase/kebab-case). **Actuellement `pub(crate)` — DOIT etre change en `pub` et re-exporte dans `tf-config/src/lib.rs` avant utilisation par tf-logging** (cf. Subtask 3.0).

### API Pattern Obligatoire

```rust
use tf_config::ProjectConfig;

/// Configuration for the logging subsystem
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error). Default: "info"
    pub log_level: String,
    /// Directory for log files. Default: "{output_folder}/logs"
    pub log_dir: String,
    /// Also output logs to stdout (for interactive mode)
    pub log_to_stdout: bool,
}

impl LoggingConfig {
    /// Derive logging config from project configuration
    pub fn from_project_config(config: &ProjectConfig) -> Self { ... }
}

/// Guard that must be kept alive to ensure logs are flushed
pub struct LogGuard {
    _guard: tracing_appender::non_blocking::WorkerGuard,
}

/// Initialize the logging subsystem
/// Returns a LogGuard that MUST be kept alive for the application lifetime
pub fn init_logging(config: &LoggingConfig) -> Result<LogGuard, LoggingError> { ... }
```

### Error Handling Pattern

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoggingError {
    #[error("Failed to initialize logging: {cause}. {hint}")]
    InitFailed {
        cause: String,
        hint: String,
    },

    #[error("Failed to create log directory '{path}': {cause}. {hint}")]
    DirectoryCreationFailed {
        path: String,
        cause: String,
        hint: String,
    },

    #[error("Invalid log level '{level}'. {hint}")]
    InvalidLogLevel {
        level: String,
        hint: String,
    },
}
```

**Hints actionnables obligatoires (pattern stories precedentes) :**
- `InitFailed` → `"Check that the log directory is writable and tracing is not already initialized"`
- `DirectoryCreationFailed` → `"Verify permissions on the parent directory or set a different output_folder in config.yaml"`
- `InvalidLogLevel` → `"Valid levels are: trace, debug, info, warn, error. Set via RUST_LOG env var (or future dedicated logging config when available)."`

### Library & Framework Requirements

**Nouvelles dependances workspace a ajouter :**
```toml
# Dans Cargo.toml racine [workspace.dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter", "fmt"] }
tracing-appender = "0.2"
```

**Crate Cargo.toml :**
```toml
[package]
name = "tf-logging"
version.workspace = true
edition.workspace = true
rust-version.workspace = true

[dependencies]
tf-config = { path = "../tf-config" }
tracing.workspace = true
tracing-subscriber.workspace = true
tracing-appender.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true

[dev-dependencies]
tempfile.workspace = true
assert_matches.workspace = true
```

### File Structure Requirements

**Naming conventions (identiques aux stories precedentes) :**
- Fichiers: `snake_case.rs`
- Modules: `snake_case`
- Structs/Enums: `PascalCase`
- Functions/variables: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`

**Fichiers a creer :**
- `crates/tf-logging/Cargo.toml`
- `crates/tf-logging/src/lib.rs` (~30-50 lignes)
- `crates/tf-logging/src/init.rs` (~100-150 lignes)
- `crates/tf-logging/src/redact.rs` (~150-200 lignes)
- `crates/tf-logging/src/config.rs` (~50-80 lignes)
- `crates/tf-logging/src/error.rs` (~40-60 lignes)

**Fichiers a modifier :**
- `Cargo.toml` (racine) — ajouter `"crates/tf-logging"` dans `[workspace] members` ET ajouter dependances workspace `tracing`, `tracing-subscriber`, `tracing-appender`
- `crates/tf-config/src/config.rs` — changer `pub(crate) fn redact_url_sensitive_params` en `pub fn redact_url_sensitive_params`
- `crates/tf-config/src/lib.rs` — ajouter re-export `pub use config::redact_url_sensitive_params;`
- `Cargo.lock` — mis a jour automatiquement

### Testing Requirements

**Framework:** `cargo test` built-in (identique aux stories precedentes)

**Strategie de test :**
- Tests unitaires dans chaque module (`#[cfg(test)] mod tests`)
- Tests d'integration dans `crates/tf-logging/tests/`
- Utiliser `tempdir` pour les tests d'ecriture de fichiers logs
- Utiliser `assert_matches!` (crate `assert_matches` en dev-dep) pour verifier les variants d'erreur — meilleurs messages d'erreur que `assert!(matches!(...))`
- Tous les tests doivent pouvoir tourner en CI sans dependance externe

**Patterns de test a implementer :**

```rust
// Test AC #1: logs JSON structures avec champs requis
#[test]
fn test_log_output_contains_required_json_fields() {
    // Setup: init logging vers un buffer ou tempdir
    // Action: emettre un event tracing::info!(command = "triage", status = "success", "Command executed")
    // Assert: chaque ligne du fichier est parseable par serde_json::from_str::<serde_json::Value>()
    // Assert: le JSON contient "timestamp" (format ISO 8601), "level" (en MAJUSCULES: "INFO"), "message", "target"
    // Note: tracing-subscriber JSON met les span fields dans "fields" et le level en MAJUSCULES
}

// Test AC #2: champs sensibles masques
#[test]
fn test_sensitive_fields_are_redacted() {
    // Setup: init logging avec RedactingLayer
    // Action: emettre tracing::info!(token = "secret123", "test")
    // Assert: le fichier contient [REDACTED] et PAS "secret123"
}

// Test AC #2: URLs avec params sensibles redactees
#[test]
fn test_urls_with_sensitive_params_redacted() {
    // Action: emettre tracing::info!(endpoint = "https://api.example.com?token=abc123")
    // Assert: le fichier contient "token=[REDACTED]" et PAS "abc123"
}

// Test AC #3: logs dans le bon repertoire
#[test]
fn test_logs_written_to_configured_directory() {
    // Setup: tempdir comme output_folder
    // Action: init_logging + emettre un event
    // Assert: fichier existe dans {tempdir}/logs/
}
```

**Couverture AC explicite :**
- AC #1 (logs JSON structures) : tests champs JSON, format timestamp, level, target
- AC #2 (champs sensibles masques) : tests redaction par nom de champ, redaction URLs, non-exposition secrets
- AC #3 (stockage configure) : tests creation repertoire, ecriture fichier, rotation journaliere

### Previous Story Intelligence (Story 0.4)

**Patterns etablis a reutiliser :**
- `thiserror` pour enum d'erreurs avec variants specifiques et hints explicites
- Custom `Debug` impl masquant les donnees sensibles
- Messages d'erreur : toujours inclure `champ + raison + hint actionnable`
- Tests couvrant explicitement chaque AC
- Workspace dependencies centralisees dans le Cargo.toml racine
- Crate-level Cargo.toml reference les dependances workspace (`tracing.workspace = true`)
- Tests dans le meme fichier (`#[cfg(test)] mod tests`)

**Apprentissages des reviews story 0.4 (52 findings en 10 rounds) :**
- TOCTOU : ne pas verifier existence puis lire, utiliser le resultat de l'operation directement
- Toujours fournir un hint actionnable dans les erreurs
- Utiliser `#[serde(rename_all = "lowercase")]` si serde est derive sur des enums
- Les line counts dans le File List doivent etre exacts
- Documenter les limitations connues (chemins relatifs, ordres d'iteration, etc.)
- Les tests d'erreur doivent verifier le TYPE d'erreur (`assert!(matches!(...))`) et pas juste `is_err()`
- Ne pas dupliquer la logique de validation — creer une seule source de verite
- `pub(crate)` vs `pub` : exposer ce qui sera reutilise par d'autres crates
- Ajouter `Clone`, `Debug`, `PartialEq` la ou c'est trivial et utile pour les tests

**Fichiers de Story 0.4 a preserver :**
- `crates/tf-config/` — 297 tests passent, ne pas casser
- `crates/tf-security/` — 30 tests passent, ne pas casser

### Anti-Patterns to Avoid

- NE JAMAIS logger des secrets (tokens, passwords, api_keys) en clair — utiliser le RedactingLayer
- NE PAS utiliser `println!` ou `eprintln!` pour le logging — utiliser exclusivement `tracing::*` macros
- NE PAS initialiser le subscriber tracing plus d'une fois (sinon panic) — garder `init_logging` idempotent ou documenter
- NE PAS utiliser `std::mem::forget(_guard)` — retourner le LogGuard a l'appelant pour qu'il le garde vivant
- NE PAS hardcoder les chemins de repertoire de logs — lire depuis LoggingConfig
- NE PAS ajouter de dependance a `chrono` — tracing-subscriber utilise `time` en interne
- NE PAS modifier `tf-config` ou `tf-security` sauf pour exposer `redact_url_sensitive_params` comme `pub` (cf. Subtask 3.0)
- NE PAS ajouter de flag pour desactiver la redaction — la redaction est une exigence de securite (NFR4) et doit etre toujours active

### Git Intelligence (Recent Patterns)

**Commit message pattern etabli :**
```
feat(tf-logging): implement baseline structured logging (Story 0-5) (#PR)
```

**Fichiers crees/modifies par stories precedentes :**
- `5db9664` feat(tf-config): implement template loading with format validation (Story 0-4) (#15)
- `c473fb7` feat(tf-security): implement secret store with OS keyring backend (#13)
- `e2c0200` feat(tf-config): implement configuration profiles with environment overrides (#12)
- `9a3ac95` feat(tf-config): implement story 0-1 YAML configuration management (#10)

**Branche attendue :** `feature/0-5-journalisation-baseline` (branche actuelle)

**Code patterns observes dans les commits recents :**
- Workspace dependencies centralisees dans le Cargo.toml racine
- Crate-level Cargo.toml reference les dependances workspace (`thiserror.workspace = true`)
- Tests dans le meme fichier (`#[cfg(test)] mod tests`)
- Fixtures dans `crates/<crate>/tests/fixtures/`
- CI GitHub Actions pour tests + clippy

### Project Structure Notes

- Alignement avec la structure multi-crates definie dans architecture.md
- tf-logging est le crate #2 dans l'ordre d'implementation (apres tf-config)
- tf-logging depend de tf-config pour la configuration et les fonctions de redaction
- Aucun conflit detecte avec les modules existants
- Le crate sera consomme par tf-cli (main.rs) pour initialiser le logging au demarrage

### References

- [Source: architecture.md] — Logging & Diagnostics (tracing stack), Technology Stack (versions), Format Patterns (JSON logs), Implementation Patterns (naming/errors), Project Structure (crate boundaries), Crate Dependencies (tf-logging #2)
- [Source: epics.md#Story 0.5] — AC et requirements
- [Source: prd.md#FR30] — Journalisation sans donnees sensibles ; [#NFR4] — Audit logs minimaux, conservation 90 jours ; [#NFR8] — CLI < 2s (non-blocking logging)
- [Source: 0-4-charger-des-templates-cr-ppt-anomalies.md] — patterns et learnings (thiserror, TOCTOU, hints, tests)
- [Source: crates/tf-config/src/config.rs:214] — `redact_url_sensitive_params` (pub(crate) → a exposer pub) + trait `Redact`

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List

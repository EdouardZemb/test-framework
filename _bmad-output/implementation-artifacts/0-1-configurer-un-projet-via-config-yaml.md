# Story 0.1: Configurer un projet via config.yaml

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a QA tester (TRA),
I want configurer un projet via un fichier config.yaml,
so that centraliser la configuration et eviter la re-saisie.

## Acceptance Criteria

1. **Given** un fichier config.yaml present
   **When** je lance l'outil
   **Then** la configuration est lue et validee selon le schema attendu

2. **Given** une configuration invalide
   **When** je lance l'outil
   **Then** un message explicite indique le champ en defaut et la correction attendue

3. **Given** une configuration chargee
   **When** les logs sont ecrits
   **Then** ils ne contiennent aucune donnee sensible

## Tasks / Subtasks

- [ ] Task 1: Initialiser le workspace Rust si non existant (AC: prerequis)
  - [ ] Subtask 1.1: Verifier si Cargo.toml workspace existe, sinon executer `cargo generate --git https://github.com/rust-starter/rust-starter-generate`
  - [ ] Subtask 1.2: Configurer le workspace multi-crates dans Cargo.toml racine

- [ ] Task 2: Creer le crate tf-config (AC: #1)
  - [ ] Subtask 2.1: Creer la structure `crates/tf-config/Cargo.toml` avec dependances serde, serde_yaml, thiserror
  - [ ] Subtask 2.2: Creer `crates/tf-config/src/lib.rs` avec module exports
  - [ ] Subtask 2.3: Creer `crates/tf-config/src/config.rs` avec struct Config et derive Deserialize

- [ ] Task 3: Implementer le schema de configuration (AC: #1)
  - [ ] Subtask 3.1: Definir la struct ProjectConfig avec champs requis (project_name, jira_endpoint, squash_endpoint, output_folder, etc.)
  - [ ] Subtask 3.2: Implementer la fonction `load_config(path: &Path) -> Result<ProjectConfig, ConfigError>`
  - [ ] Subtask 3.3: Ajouter validation des champs obligatoires

- [ ] Task 4: Implementer la validation stricte avec messages explicites (AC: #2)
  - [ ] Subtask 4.1: Creer l'enum ConfigError avec variants specifiques (MissingField, InvalidValue, FileNotFound, ParseError)
  - [ ] Subtask 4.2: Implementer Display pour ConfigError avec format "champ X invalide: raison + correction attendue"
  - [ ] Subtask 4.3: Ajouter validation de types et contraintes (URLs valides, chemins existants si requis)

- [ ] Task 5: Garantir l'absence de donnees sensibles dans les logs (AC: #3)
  - [ ] Subtask 5.1: Implementer trait Redact pour masquer les champs sensibles (tokens, passwords)
  - [ ] Subtask 5.2: Utiliser #[serde(skip_serializing)] ou equivalent pour champs sensibles
  - [ ] Subtask 5.3: Ajouter tests unitaires verifiant que les logs ne contiennent pas de secrets

- [ ] Task 6: Tests unitaires et integration (AC: #1, #2, #3)
  - [ ] Subtask 6.1: Tests pour config valide chargee correctement
  - [ ] Subtask 6.2: Tests pour config invalide avec messages d'erreur clairs
  - [ ] Subtask 6.3: Tests pour absence de donnees sensibles dans les sorties

## Dev Notes

### Technical Stack Requirements

**Versions exactes a utiliser:**
- Rust edition: 2021 (MSRV 1.75+)
- `serde = "1.0"` avec derive feature
- `serde_yaml = "0.9"` (ou equivalent moderne)
- `thiserror = "2.0"` pour les erreurs structurees
- `anyhow = "1.0"` pour la propagation d'erreurs

**Patterns d'erreur obligatoires:**
```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: String },

    #[error("Invalid configuration: field '{field}' {reason}. Expected: {hint}")]
    ValidationError { field: String, reason: String, hint: String },

    #[error("Failed to parse configuration: {0}")]
    ParseError(#[from] serde_yaml::Error),
}
```

### Architecture Compliance

**Crate tf-config - aucune dependance interne**

Ce crate est le premier dans l'ordre d'implementation et ne doit dependre d'aucun autre crate du projet.

**Structure obligatoire:**
```
crates/
└── tf-config/
    ├── Cargo.toml
    └── src/
        ├── lib.rs       # pub mod config; pub mod profiles;
        ├── config.rs    # ProjectConfig struct + load_config()
        └── profiles.rs  # (stub pour Story 0.2)
```

### Library/Framework Requirements

**Dependances Cargo.toml pour tf-config:**
```toml
[package]
name = "tf-config"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
thiserror = "2.0"
```

### File Structure Requirements

**Naming conventions:**
- Fichiers: `snake_case.rs`
- Modules: `snake_case`
- Structs/Enums: `PascalCase`
- Functions/variables: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`

**Schema config.yaml attendu:**
```yaml
project_name: "my-project"
jira:
  endpoint: "https://jira.example.com"
  # token stocke via secret store (Story 0.3)
squash:
  endpoint: "https://squash.example.com"
  # auth via secret store
output_folder: "./output"
templates:
  cr: "./templates/cr.md"
  ppt: "./templates/ppt.pptx"
  anomaly: "./templates/anomaly.md"
llm:
  mode: "auto"  # auto | local | cloud
  local_endpoint: "http://localhost:11434"
```

### Testing Requirements

**Framework:** `cargo test` built-in

**Patterns de test obligatoires:**
1. Test config valide: charger un fichier YAML valide et verifier tous les champs
2. Test config invalide: verifier que les erreurs sont explicites avec champ + raison + hint
3. Test redaction: verifier que `Debug` et logs ne revelent pas de secrets

**Exemple de test:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_valid_config() {
        let config = load_config(Path::new("tests/fixtures/valid_config.yaml")).unwrap();
        assert_eq!(config.project_name, "test-project");
    }

    #[test]
    fn test_invalid_config_shows_helpful_error() {
        let result = load_config(Path::new("tests/fixtures/missing_field.yaml"));
        let err = result.unwrap_err();
        assert!(err.to_string().contains("field"));
        assert!(err.to_string().contains("Expected"));
    }
}
```

### Project Structure Notes

- Ce crate est le premier a implementer dans l'ordre des dependances
- Il sera utilise par tf-logging, tf-security, et tous les autres crates
- Ne pas ajouter de dependances vers d'autres crates tf-*

### Previous Story Intelligence

Premiere story de l'Epic 0 - pas de story precedente.

### Git Intelligence

Projet nouveau - pas de commits precedents dans le code source.

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#Technology Stack]
- [Source: _bmad-output/planning-artifacts/architecture.md#Project Structure & Boundaries]
- [Source: _bmad-output/planning-artifacts/architecture.md#Implementation Patterns]
- [Source: _bmad-output/planning-artifacts/prd.md#FR23]
- [Source: _bmad-output/planning-artifacts/epics.md#Story 0.1]

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List

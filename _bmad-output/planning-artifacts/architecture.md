---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8]
inputDocuments:
  - '_bmad-output/planning-artifacts/prd.md'
  - '_bmad-output/planning-artifacts/ux-design-specification.md'
workflowType: 'architecture'
project_name: 'test-framework'
user_name: 'Edouard'
date: '2026-01-30T14:20:47+01:00'
lastStep: 8
status: 'complete'
completedAt: '2026-01-30T15:13:39+01:00'
validationReview:
  date: '2026-01-30'
  reviewer: 'Architecture Validation Workflow'
  additions:
    - 'Technology Stack (versions exactes)'
    - 'LLM Integration Architecture (tf-llm crate)'
    - 'Office Document Generation (tf-export crate)'
    - 'Crate Dependencies (ordre implémentation)'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._


## Project Context Analysis

### Requirements Overview

**Functional Requirements:**
Le produit couvre un flux QA complet via CLI : ingestion/triage (Jira/Squash), checklist testabilité et scoring Go/Clarify/No‑Go, génération assistée de stratégies/cas/anomalies, gestion des preuves et campagnes (TNR), reporting quotidien/hebdo, exports multi‑formats, configuration par projet/profil et modes interactif/batch. L’architecture devra soutenir des workflows guidés, des modules réutilisables, et une traçabilité forte (preuves ↔ cas ↔ anomalies ↔ tickets).

**Non-Functional Requirements:**
- Sécurité & privacy strictes : LLM local prioritaire, anonymisation obligatoire avant cloud, secrets via secret store, least privilege.
- Conformité & audit : logs minimaux sans données sensibles, rétention 90 jours, purge données locales < 24h.
- Performance : CLI réactive, extraction & génération rapides.
- Fiabilité : reprise/replay, mode dégradé si intégrations indisponibles.
- Compatibilité : intégrations Jira/Squash/SharePoint/Office, sorties conformes aux templates.
- Contraintes IT : proxy, droits d’installation, scripts signés possibles, exécution sur poste M365.

**Scale & Complexity:**
Projet d’automatisation QA multi‑outils avec forte contrainte de conformité et plusieurs workflows critiques.

- Primary domain: CLI tool / QA process automation / integrations
- Complexity level: high
- Estimated architectural components: ~10–12 (connecteurs, orchestration, templates, stockage, anonymisation, logs, config/profiles, export, diagnostics, UX CLI)

### Technical Constraints & Dependencies

- Jira (token) et Squash (Basic auth initialement), SharePoint comme source de preuves/livrables.
- LLM local supporte et cloud disponible des le MVP, avec anonymisation automatique obligatoire.
- Données sensibles minimisées, chiffrement au repos, purge locale.
- Environnement poste M365, restrictions IT (proxy, scripts signés, droits).

### Cross-Cutting Concerns Identified

- Privacy & anonymisation by design
- Gestion des secrets et least‑privilege
- Observabilité minimale (logs sans données sensibles)
- Traçabilité preuves ↔ tickets ↔ anomalies ↔ campagnes
- Mode dégradé + replay en cas d’échec d’intégration
- Standardisation des templates & formats d’export
- Configuration multi‑projets/profils

### Architectural Decision Areas (ADR‑style, à trancher ensuite)

1. **Orchestration CLI (interactif vs batch)**
   - Options : moteur de workflow unique vs pipelines spécialisés par commande
   - Trade‑offs : simplicité d’usage vs flexibilité multi‑projets

2. **Intégrations & mode dégradé**
   - Options : connecteurs API natifs vs import/export CSV en fallback systématique
   - Trade‑offs : robustesse offline vs complexité de mapping

3. **Sécurité & anonymisation**
   - Options : anonymisation inline (pipeline) vs service dédié
   - Trade‑offs : performance locale vs séparation des responsabilités

4. **Stockage & traçabilité**
   - Options : stockage local chiffré minimal vs cache plus riche
   - Trade‑offs : performance/replay vs exigences de purge

5. **Templates & standardisation**
   - Options : templates versionnés dans repo vs templates externes (SharePoint)
   - Trade‑offs : contrôle/versionning vs adoption par le client

6. **Observabilité minimale**
   - Options : logs structurés locaux vs journalisation centralisée (si autorisée)
   - Trade‑offs : conformité IT vs facilité de diagnostic

### Pre‑mortem (risques majeurs à prévenir)

- **Échec d’adoption** : flux trop verbeux, surcharge de flags, pas assez “run unique”.
- **Blocage IT** : proxy/sécurité empêche API → besoin d’un mode dégradé CSV réellement utilisable.
- **Fuite de données** : anonymisation incomplète avant cloud → obligation d’un pipeline de masquage vérifiable.
- **Qualité des sorties insuffisante** : templates non alignés client → versionning/validation des templates requis.
- **Données incohérentes** : mappings Jira/Squash/Excel divergents → besoin d’un mapping central et d’un audit minimal.
- **Temps d’exécution trop long** : extraction lente → stratégie de cache minimale et mode “scope réduit”.
- **Diagnostic impossible** : logs trop pauvres → logs structurés sans données sensibles + replay.

### Comparative Decision Matrix (critères pour trancher)

**1) Orchestration**
- Options : workflow unique vs pipelines par commande
- Critères : simplicité d’usage, extensibilité, testabilité, cohérence UX, effort maintenance

**2) Mode dégradé**
- Options : fallback CSV obligatoire vs optionnel
- Critères : robustesse IT, coût mapping, adoption user, risques d’erreur

**3) Anonymisation**
- Options : inline vs service dédié
- Critères : performance locale, traçabilité, isolation des risques, facilité d’audit

**4) Stockage local**
- Options : cache minimal vs cache enrichi
- Critères : vitesse, conformité purge, facilité replay, surface de risque

**5) Templates**
- Options : repo‑versionnés vs externes
- Critères : gouvernance, acceptation client, facilité de mise à jour, auditabilité

**6) Observabilité**
- Options : logs locaux structurés vs centralisation
- Critères : conformité IT, capacité de debug, effort d’implémentation

### Cross‑Functional War Room (PM / Ingé / Design)

- **PM (désirabilité & adoption)** : “Le run unique doit rester simple. Les options avancées doivent être cachées derrière `--advanced` pour éviter la surcharge. Le mode dégradé CSV doit être first‑class, sinon adoption faible en environnement restreint.”
- **Ingénieur (faisabilité & robustesse)** : “Le cœur doit être un orchestrateur pipeline modulaire avec connecteurs isolés. Les intégrations doivent être remplaçables et testables. Les logs structurés sans données sensibles sont indispensables pour diagnostiquer sans risque.”
- **Designer (expérience & clarté)** : “Le CLI doit maintenir la promesse ‘run → review → send’. Toute complexité doit apparaître seulement en cas d’erreur. Les templates doivent être gouvernés pour éviter la re‑saisie et les écarts de format.”


## Starter Template Evaluation

### Primary Technology Domain

CLI tool (Rust) based on project requirements analysis

### Starter Options Considered

- rust-starter (cargo-generate): Rust CLI starter with clap, configuration, logging, and CI/release workflows.
- Baseline: cargo new (minimal, manual setup).

### Selected Starter: rust-starter

**Rationale for Selection:**
Provides a CLI-focused Rust base with common tooling and CI/release scaffolding, aligned with Windows/WSL support and future distribution needs.

**Initialization Command:**

```bash
cargo generate --git https://github.com/rust-starter/rust-starter-generate
```

**Architectural Decisions Provided by Starter:**

**Language & Runtime:**
Rust CLI binary

**Styling Solution:**
N/A (CLI)

**Build Tooling:**
Cargo with starter CI/release workflows

**Testing Framework:**
Cargo test defaults (can be extended later)

**Code Organization:**
CLI-oriented structure with clap and configuration/logging utilities

**Development Experience:**
Template-driven scaffolding with CI/release ready setup


## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions (Block Implementation):**
- Stockage local principal en SQLite (version actuelle 3.51.1).
- Cache enrichi sécurisé (chiffrement applicatif, TTL court, purge auto, replay explicite).
- Validation stricte des données (reject + rapport d’erreurs).
- Gestion des secrets via OS keyring (keyring crate 3.6.2).
- Anonymisation inline obligatoire avant tout envoi cloud.
- Logs structurés JSON sans données sensibles.

**Important Decisions (Shape Architecture):**
- Intégrations REST + mode dégradé CSV first-class.
- Erreurs structurées (code + message + hint).
- Retry exponentiel avec backoff.
- Modèles internes typés + validation stricte.

**Deferred Decisions (Post-MVP):**
- Aucun choix différé critique à ce stade.

### Data Architecture

- **Storage**: SQLite local (3.51.1)
- **Cache**: enrichi, chiffré applicativement, TTL court, purge auto, replay explicite
- **Validation**: stricte (reject + diagnostics)

### Authentication & Security

- **Secrets**: OS keyring (keyring 3.6.2)
- **Chiffrement local**: chiffrement applicatif (pas de SQLCipher)
- **Anonymisation**: pipeline inline obligatoire
- **Logs**: JSON structurés sans PII

### API & Communication Patterns

- **Intégrations**: REST + fallback CSV obligatoire
- **Erreurs**: structurées (code + message + hint)
- **Retry**: exponentiel + backoff
- **Contrats internes**: modèles typés + validation stricte

### Frontend Architecture

- N/A (CLI uniquement)

### Infrastructure & Deployment

- **Distribution**: npm wrapper (téléchargement binaire par plateforme)
- **CI/CD**: GitHub Actions
- **Versioning**: SemVer + changelog
- **Observabilité**: logs JSON locaux + commande `tf diagnostics`

### Technology Stack (Versions Exactes)

**Runtime & Core:**
- Rust edition: 2021 (MSRV 1.75+)
- Async runtime: `tokio = "1.41"` (full features)
- HTTP client: `reqwest = "0.12"` (rustls-tls, json)
- CLI framework: `clap = "4.5"` (derive, env)
- Serialization: `serde = "1.0"`, `serde_json = "1.0"`

**Storage & Security:**
- SQLite: `rusqlite = "0.32"` (bundled)
- Secrets: `keyring = "3.6"`
- Encryption: `aes-gcm = "0.10"` (chiffrement applicatif)
- Hashing: `argon2 = "0.5"` (key derivation)

**Logging & Diagnostics:**
- Tracing: `tracing = "0.1"`, `tracing-subscriber = "0.3"` (json)
- Error handling: `thiserror = "2.0"`, `anyhow = "1.0"`

**Office & Export:**
- Excel read: `calamine = "0.26"`
- Excel write: `rust_xlsxwriter = "0.79"`
- CSV: `csv = "1.3"`
- Markdown: `pulldown-cmark = "0.12"`

**Testing:**
- Unit/Integration: `cargo test` (built-in)
- Mocking: `mockall = "0.13"`
- Assertions: `assert_cmd = "2.0"`, `predicates = "3.1"`

### LLM Integration Architecture

**Crate:** `tf-llm/`

**Strategy:** LLM local prioritaire, cloud disponible des le MVP (mode auto ou cloud), anonymisation obligatoire avant tout envoi.

**Local Runtime:**
- Primary: Ollama via HTTP API (`http://localhost:11434`)
- Fallback: llama.cpp binding si Ollama indisponible
- Models recommandés: `mistral:7b-instruct`, `codellama:13b`

**Cloud Mode (MVP):**
- OpenAI-compatible API (configurable endpoint)
- Activation: si `config.llm.cloud_enabled = true` et `config.llm.mode = "cloud"` ou `mode = "auto"`
- Pré-requis: passage obligatoire par `tf-security/anonymize.rs`

**Module Structure:**
```
tf-llm/
├── Cargo.toml
└── src/
    ├── mod.rs
    ├── local.rs       # Ollama/llama.cpp client
    ├── cloud.rs       # OpenAI-compatible client
    ├── orchestrator.rs # Routing local/cloud
    ├── prompts.rs     # Prompt templates
    └── types.rs       # Request/Response types
```

**Orchestration Flow:**
1. `tf-core/pipeline` appelle `tf-llm/orchestrator`
2. Orchestrator lit `config.llm.mode` (auto|local|cloud)
3. Mode local → appel local, sinon erreur si indisponible
4. Mode cloud → anonymisation → cloud (si enabled), sinon erreur explicite
5. Mode auto → local si OK, sinon anonymisation → cloud si enabled, sinon erreur avec hint "Démarrer Ollama"

**Configuration:**
```yaml
llm:
  mode: "auto"
  local_endpoint: "http://localhost:11434"
  local_model: "mistral:7b-instruct"
  cloud_enabled: false
  cloud_endpoint: "https://api.openai.com/v1"
  cloud_model: "gpt-4o-mini"
  timeout_seconds: 120
  max_tokens: 4096
```

### Office Document Generation

**Crate:** `tf-export/`

**Strategy:** Génération native Excel, templates pour PowerPoint.

**Module Structure:**
```
tf-export/
├── Cargo.toml
└── src/
    ├── mod.rs
    ├── excel.rs       # rust_xlsxwriter pour .xlsx
    ├── ppt.rs         # Template OOXML pour .pptx
    ├── markdown.rs    # Export .md
    ├── html.rs        # Export .html
    └── templates.rs   # Gestion des templates
```

**Excel Generation:**
- Crate: `rust_xlsxwriter` (natif Rust, pas de dépendance Python)
- Lecture existants: `calamine`
- Formats: CR quotidien, exports données, matrices traçabilité

**PowerPoint Generation:**
- Approche: Template OOXML (fichier .pptx = archive ZIP avec XML)
- Le système charge un template .pptx et remplace les placeholders
- Templates stockés dans `assets/templates/*.pptx`
- Limitation: pas de génération de slides complexes from scratch

**Templates Location:**
```
assets/
└── templates/
    ├── cr-quotidien.xlsx
    ├── ppt-hebdo.pptx
    ├── ppt-tnr.pptx
    └── anomalie.md
```

**Data Flow:**
1. `tf-core/domain/report` prépare les données structurées
2. `tf-export` charge le template approprié
3. Injection des données dans le template
4. Export vers le format cible

### Decision Impact Analysis

**Implementation Sequence:**
1. Stockage local + chiffrement applicatif + purge/TTL
2. Gestion des secrets (keyring) + anonymisation inline
3. Connecteurs REST + fallback CSV + erreurs structurées
4. **LLM local (Ollama) + orchestrator**
5. **Export Office (Excel natif, PPT templates)**
6. Observabilité + diagnostics
7. Pipeline CI/CD + distribution npm wrapper

**Cross-Component Dependencies:**
- Cache enrichi ↔ chiffrement ↔ purge/TTL ↔ diagnostics
- REST/CSV fallback ↔ erreurs structurées ↔ validation stricte
- **LLM orchestrator ↔ tf-security/anonymize ↔ cloud (mode auto/cloud)**
- **tf-core/domain/report ↔ tf-export ↔ templates**


## Implementation Patterns & Consistency Rules

### Pattern Categories Defined

**Critical Conflict Points Identified:**
Naming, structure, formats, error handling, retry, logging, and diagnostics.

### Naming Patterns

**Code Naming Conventions:**
- Files: `snake_case.rs`
- Modules: `snake_case`
- Struct/Enum: `PascalCase`
- Functions/variables: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`

**Data/JSON Naming:**
- JSON fields: `snake_case`
- Dates: ISO-8601 UTC (`2026-01-30T12:34:56Z`)

### Structure Patterns

**Project Organization:**
- `src/main.rs` entrypoint
- `src/cli/` for CLI parsing & prompts
- `src/commands/` for command handlers
- `src/pipeline/` for orchestration flows
- `src/connectors/` for Jira/Squash/SharePoint/CSV integrations
- `src/domain/` for domain models
- `src/storage/` for SQLite/cache
- `src/security/` for anonymisation/secrets/redaction
- `src/diagnostics/` for replay & debug
- `src/errors/` for structured errors
- `src/config/` for config/profiles
- `src/logging/` for tracing setup
- `tests/` for integration tests
- `docs/` for CLI usage docs

### Format Patterns

**API/CLI Output:**
- JSON envelope: `{ "data": ..., "error": null, "meta": {...} }`
- Error object: `{ "error": { "code": "...", "message": "...", "hint": "...", "details": {...} } }`

**Logging Format:**
- JSON structured logs with fields: `timestamp`, `level`, `message`, `context`

### Process Patterns

**Exit Codes:**
- 0 OK
- 1 General error
- 2 Validation error
- 3 Integration error

**Retry & Recovery:**
- Exponential backoff, max 3 retries
- Auto-fallback to CSV when API unavailable

**Validation:**
- Strict validation on load, reject + error report

**Diagnostics:**
- `tf diagnostics` produces a report with no PII

### Enforcement Guidelines

**All AI Agents MUST:**
- Follow naming conventions for Rust & JSON
- Place code in the agreed directories
- Use structured errors + JSON logs
- Respect exit codes and retry policies

**Pattern Enforcement:**
- Lint + CI checks for structure and naming
- Review checklist includes pattern compliance
- Any exceptions documented in `docs/pattern-exceptions.md`

### Pattern Examples

**Good Examples:**
- `src/connectors/jira_client.rs`
- JSON error: `{ "error": { "code": "JIRA_AUTH", "message": "Invalid token", "hint": "Re-login", "details": {"status": 401} } }`

**Anti-Patterns:**
- CamelCase file names (`JiraClient.rs`)
- Logs with raw PII
- Ad-hoc folder creation (`src/utils2/`)


## Project Structure & Boundaries

### Complete Project Directory Structure
```
test-framework/
├── Cargo.toml                  # workspace
├── README.md
├── .gitignore
├── .env.example
├── .github/
│   └── workflows/
│       └── ci.yml
├── docs/
│   ├── cli-usage.md
│   ├── workflows.md
│   └── pattern-exceptions.md
├── crates/
│   ├── tf-cli/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       └── cli/
│   │           ├── mod.rs
│   │           └── commands/
│   │               ├── run.rs
│   │               ├── triage.rs
│   │               ├── anomalies.rs
│   │               ├── report.rs
│   │               ├── tnr.rs
│   │               └── diagnostics.rs
│   ├── tf-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── pipeline/
│   │       │   ├── mod.rs
│   │       │   └── run_pipeline.rs
│   │       ├── domain/
│   │       │   ├── mod.rs
│   │       │   ├── ticket.rs
│   │       │   ├── testcase.rs
│   │       │   ├── anomaly.rs
│   │       │   └── report.rs
│   │       ├── errors/
│   │       │   ├── mod.rs
│   │       │   └── error.rs
│   │       └── validation/
│   │           ├── mod.rs
│   │           └── validate.rs
│   ├── tf-connectors/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── jira.rs
│   │       ├── squash.rs
│   │       ├── sharepoint.rs
│   │       └── csv.rs
│   ├── tf-storage/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── sqlite.rs
│   │       ├── cache.rs
│   │       └── purge.rs
│   ├── tf-security/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── keyring.rs
│   │       ├── error.rs
│   │       ├── anonymize.rs
│   │       └── redact.rs
│   ├── tf-config/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── config.rs
│   │       └── profiles.rs
│   ├── tf-logging/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── logging.rs
│   ├── tf-diagnostics/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── diagnostics.rs
│   ├── tf-llm/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── mod.rs
│   │       ├── local.rs
│   │       ├── cloud.rs
│   │       ├── orchestrator.rs
│   │       ├── prompts.rs
│   │       └── types.rs
│   └── tf-export/
│       ├── Cargo.toml
│       └── src/
│           ├── mod.rs
│           ├── excel.rs
│           ├── ppt.rs
│           ├── markdown.rs
│           ├── html.rs
│           └── templates.rs
├── assets/
│   └── templates/
│       ├── cr-quotidien.xlsx
│       ├── ppt-hebdo.pptx
│       ├── ppt-tnr.pptx
│       └── anomalie.md
├── tests/
│   └── integration/
│       ├── run_pipeline.rs
│       └── connectors_csv.rs
```

### Architectural Boundaries

**API Boundaries:**
- CLI boundary in `tf-cli` only
- External APIs via `tf-connectors`
- Data access in `tf-storage`

**Component Boundaries:**
- Pipeline orchestration in `tf-core`
- Security/anonymisation in `tf-security`
- Diagnostics isolated in `tf-diagnostics`
- LLM orchestration in `tf-llm`
- Document generation in `tf-export`

**Service Boundaries:**
- Connectors are the only modules that call external services (Jira, Squash, SharePoint)
- `tf-llm` calls LLM services (local Ollama or cloud API)
- Core pipeline is pure business logic

**Data Boundaries:**
- SQLite cache access only in `tf-storage`
- Domain models live in `tf-core`

### Requirements to Structure Mapping

**FR Categories → Modules:**
- Ingestion/Triage → `tf-connectors`, `tf-core/pipeline`, `tf-cli/commands/triage.rs`
- Test Design → `tf-core/domain` + `tf-llm` (génération assistée) + CLI commands
- Execution/Evidence → `tf-connectors` + `tf-core/pipeline`
- Anomaly Mgmt → `tf-core/domain` + `tf-llm` (génération brouillons) + `tf-cli/commands/anomalies.rs`
- Reporting → `tf-core/domain/report` + `tf-export` (Excel/PPT) + `tf-cli/commands/report.rs`
- Config/Reuse → `tf-config`
- Compliance/Safety → `tf-security`, `tf-logging`, `tf-diagnostics`
- LLM Integration → `tf-llm` (local prioritaire, cloud disponible en MVP via mode auto/cloud et anonymisation `tf-security`)

### Integration Points

**Internal Communication:**
- `tf-cli` → `tf-core` orchestrates
- `tf-core` → `tf-connectors` for external data
- `tf-core` → `tf-storage` for cache
- `tf-core` → `tf-security` for anonymisation
- `tf-core` → `tf-llm` for AI-assisted generation
- `tf-core` → `tf-export` for document generation
- `tf-llm` → `tf-security` for anonymisation before cloud calls

**External Integrations:**
- Jira/Squash/SharePoint via `tf-connectors`
- CSV import/export via `tf-connectors/csv.rs`
- LLM local (Ollama) via `tf-llm/local.rs`
- LLM cloud (OpenAI-compatible) via `tf-llm/cloud.rs`

**Data Flow:**
CLI → Core Pipeline → Connectors/Storage → LLM (optional) → Export → Domain Outputs → CLI Output

### File Organization Patterns

**Configuration Files:**
- `.env.example` at root
- `config.yaml` loaded via `tf-config`

**Source Organization:**
- crate boundaries define ownership

**Test Organization:**
- integration tests in `tests/integration`

**Asset Organization:**
- `assets/templates/` for Office templates (xlsx, pptx, md)
- Templates are versioned with the codebase
- User-customizable templates can override defaults via config

### Development Workflow Integration

**Development Server Structure:**
- N/A (CLI)

**Build Process Structure:**
- Workspace builds all crates; `tf-cli` produces final binary

**Deployment Structure:**
- CI builds `tf-cli` per OS; npm wrapper downloads binaries


## Architecture Validation Results

### Coherence Validation ✅

**Decision Compatibility:**
Les choix sont compatibles : Rust CLI + workspace, SQLite local chiffré applicativement, secrets via keyring, logs JSON, intégrations REST + fallback CSV, CI GitHub Actions, distribution npm wrapper.

**Pattern Consistency:**
Les conventions de nommage/structure/format/logs sont alignées avec la stack Rust et les exigences de conformité.

**Structure Alignment:**
La structure multi-crates isole bien connecteurs, sécurité, stockage, diagnostics et CLI, réduisant les conflits entre agents.

### Requirements Coverage Validation ✅

**Epic/Feature Coverage:**
N/A (pas d'epics, mapping par catégories FR).

**Functional Requirements Coverage:**
Toutes les catégories FR sont couvertes : ingestion/triage, génération, exécution, anomalies, reporting, config et réutilisation.

**Non-Functional Requirements Coverage:**
Privacy/anonymisation, secret store, least privilege, audit logs, purge locale, performance CLI, mode dégradé, fiabilité/replay sont tous adressés.

### Implementation Readiness Validation ✅

**Decision Completeness:**
Décisions critiques documentées avec versions (SQLite 3.51.1, keyring 3.6.2).

**Structure Completeness:**
Arborescence complète multi-crates + tests + docs.

**Pattern Completeness:**
Nommage, structure, formats, erreurs, retry, logs, diagnostics sont définis.

### Gap Analysis Results

- **Critical gaps:** none (LLM architecture et Office generation ajoutés)
- **Important gaps:** none (Technology stack avec versions exactes documenté)
- **Nice-to-have:** documenter ultérieurement la politique exacte de TTL/purge et le format du rapport diagnostics.

### Validation Issues Addressed

Aucun point bloquant détecté.

### Architecture Completeness Checklist

**✅ Requirements Analysis**

- [x] Project context thoroughly analyzed
- [x] Scale and complexity assessed
- [x] Technical constraints identified
- [x] Cross-cutting concerns mapped

**✅ Architectural Decisions**

- [x] Critical decisions documented with versions
- [x] Technology stack fully specified
- [x] Integration patterns defined
- [x] Performance considerations addressed

**✅ Implementation Patterns**

- [x] Naming conventions established
- [x] Structure patterns defined
- [x] Communication patterns specified
- [x] Process patterns documented

**✅ Project Structure**

- [x] Complete directory structure defined
- [x] Component boundaries established
- [x] Integration points mapped
- [x] Requirements to structure mapping complete

### Architecture Readiness Assessment

**Overall Status:** READY FOR IMPLEMENTATION  
**Confidence Level:** high

**Key Strengths:**
- Séparation claire des responsabilités (core/connectors/security/storage)
- Conformité privacy/LLM local by design
- Mode dégradé robuste via CSV

**Areas for Future Enhancement:**
- Spécifier TTL/purge exacts
- Définir le format final du rapport diagnostics

### Implementation Handoff

**AI Agent Guidelines:**
- Follow all architectural decisions exactly as documented
- Use implementation patterns consistently across all components
- Respect project structure and boundaries
- Refer to this document for all architectural questions

**First Implementation Priority:**
Initialiser le workspace via le starter `rust-starter` puis poser les crates (`tf-cli`, `tf-core`, `tf-connectors`, `tf-storage`, `tf-security`, `tf-config`, `tf-logging`, `tf-diagnostics`, `tf-llm`, `tf-export`).

**Crate Dependencies (ordre d'implémentation recommandé):**
1. `tf-config` (aucune dépendance interne)
2. `tf-logging` (dépend de tf-config)
3. `tf-security` (dépend de tf-config)
4. `tf-storage` (dépend de tf-config, tf-security)
5. `tf-connectors` (dépend de tf-config, tf-storage)
6. `tf-llm` (dépend de tf-config, tf-security)
7. `tf-export` (dépend de tf-config)
8. `tf-core` (dépend de tous les crates ci-dessus)
9. `tf-diagnostics` (dépend de tf-config, tf-logging, tf-storage)
10. `tf-cli` (dépend de tf-core, tf-diagnostics)

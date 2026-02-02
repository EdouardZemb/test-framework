---
stepsCompleted: [1, 2, 3, 4, 5, 6]
project_name: test-framework
user_name: Edouard
date: 2026-01-30
filesIncluded:
  prd: _bmad-output/planning-artifacts/prd.md
  architecture: _bmad-output/planning-artifacts/architecture.md
  epics: _bmad-output/planning-artifacts/epics.md
  ux: _bmad-output/planning-artifacts/ux-design-specification.md
---

# Implementation Readiness Assessment Report

**Date:** 2026-01-30
**Project:** test-framework

## Document Discovery

### PRD Files Selected
- `_bmad-output/planning-artifacts/prd.md` (17610 bytes, 2026-01-30 23:29:25 +0100)

### PRD Files Excluded (per user)
- `_bmad-output/planning-artifacts/prd-validation.md` (14525 bytes, 2026-01-30 10:33:29 +0100)
- `_bmad-output/planning-artifacts/prd-validation-report.md` (20627 bytes, 2026-01-30 22:53:10 +0100)

### Architecture Files
- `_bmad-output/planning-artifacts/architecture.md` (27240 bytes, 2026-01-30 22:26:42 +0100)

### Epics & Stories Files
- `_bmad-output/planning-artifacts/epics.md` (32586 bytes, 2026-01-30 23:29:43 +0100)

### UX Design Files
- `_bmad-output/planning-artifacts/ux-design-specification.md` (23447 bytes, 2026-01-30 23:38:47 +0100)

### Issues
- None

## PRD Analysis

### Functional Requirements

## Functional Requirements Extracted

FR1: L'utilisateur peut importer des tickets depuis Jira pour une periode ou un perimetre donne.
FR2: Le systeme peut appliquer une checklist de testabilite a un ticket.
FR3: Le systeme peut produire un score Go/Clarify/No-Go par ticket.
FR4: L'utilisateur peut marquer un ticket comme "clarification requise" et tracer la cause.
FR5: L'utilisateur peut regrouper les tickets par lot/perimetre/cadence.
FR6: Le systeme peut generer un brouillon de strategie de test a partir d'un ticket.
FR7: Le systeme peut generer des propositions de cas de test (nominaux, variantes, limites).
FR8: L'utilisateur peut editer et valider les brouillons avant publication.
FR9: Le systeme peut integrer une checklist ESN dans la preparation des cas.
FR10: L'utilisateur peut lier des preuves a des cas de test (references vers SharePoint).
FR11: Le systeme peut centraliser les references de preuves par campagne.
FR12: Le systeme peut associer les preuves aux tickets Jira concernes.
FR13: L'utilisateur peut preparer une campagne TNR a partir d'un modele.
FR14: L'utilisateur peut generer un brouillon d'anomalie depuis un cas de test.
FR15: Le systeme applique un template standardise d'anomalie (champs obligatoires).
FR16: L'utilisateur peut lier l'anomalie a la campagne Squash et au ticket Jira.
FR17: Le systeme peut taguer les anomalies TNR avec un label specifique.
FR18: Le systeme peut generer un CR quotidien pre-rempli (blocs standard).
FR19: L'utilisateur peut editer le CR et produire une version finale.
FR20: Le systeme peut pre-generer le PPT hebdo avec les sections pertinentes.
FR21: Le systeme peut generer un PPT TNR (GO/NO-GO, reserves, liens).
FR22: Le systeme peut produire des exports multi-formats (txt/json/csv/md/html/yaml).
FR23: L'utilisateur peut configurer un projet via config.yaml.
FR24: L'utilisateur peut definir des profils de configuration par contexte.
FR25: Le systeme peut charger des templates (CR, PPT, anomalies).
FR26: Le systeme peut fonctionner en mode interactif ou batch.
FR27: Le systeme peut fournir une auto-completion shell.
FR28: Le systeme peut anonymiser automatiquement les donnees avant envoi cloud.
FR29: Le systeme peut fonctionner avec LLM local uniquement.
FR30: Le systeme peut journaliser les executions sans donnees sensibles.
FR31: Le systeme peut gerer les secrets via un secret store.

Total FRs: 31

### Non-Functional Requirements

## Non-Functional Requirements Extracted

NFR1: Donnees sensibles jamais envoyees vers un LLM cloud sans anonymisation automatique (controle sur 100% des envois sortants).
NFR2: Secrets stockes via secret store, jamais en clair (0 secret en clair dans le repo ou la configuration).
NFR3: Least privilege : read-only par defaut, ecriture validee explicitement (aucune ecriture sans activation explicite).
NFR4: Audit logs minimaux sans donnees sensibles (horodatage, perimetre, statut) et conservation 90 jours.
NFR5: Donnees temporaires locales purgees sous 24h (automatique).
NFR6: Generation d'un CR quotidien pre-rempli < 2 minutes pour un perimetre standard.
NFR7: CR quotidien pret a envoyer < 5 minutes apres le run (incluant relecture/ajustements humains).
NFR8: Extraction Jira/Squash pour un lot < 5 minutes (a confirmer).
NFR9: CLI reactive : reponse interactive < 2 secondes pour commandes simples.
NFR10: En cas d'echec d'une integration, l'outil doit continuer en mode degrade (taux d'echec < 2% des executions).
NFR11: Reprise possible sur derniere execution (mode replay) sans perte de contexte, reprise < 5 minutes.
NFR12: Support Jira/Squash via API avec auth token (Jira) et Basic (Squash), taux de succes >= 95% sur appels en volume standard.
NFR13: Sorties multi-formats strictement conformes aux templates existants (CR, PPT, anomalies), sans personnalisation visuelle.
NFR14: Fonctionnement hors VM, VM reservee aux applis testees.
NFR15: Fraicheur des donnees integrees < 15 minutes sur pipeline standard.
NFR16: Execution possible sur poste M365, sans dependances cloud obligatoires.
NFR17: Respect des restrictions IT (scripts signes, proxy, droits d'installation).

Total NFRs: 17

### Additional Requirements

**Compliance & Regulatory**
- Usage IA sensible : LLM local prioritaire, cloud possible mais jamais sans anonymisation automatique.
- Mecanisme d'anonymisation by design avant tout envoi vers un LLM cloud.
- Aucune donnee sensible (clients finaux) ne doit sortir du perimetre sans masquage.

**Regulatory Requirements (Assurance)**
- Conformite aux exigences assurance par juridiction (a confirmer avec le client).
- Respect des politiques internes de conservation des preuves et livrables (duree, lieu, acces).
- Journalisation minimale pour audit interne (qui, quand, quoi) sans donnees sensibles.

**Risk Modeling (Assurance)**
- Hors scope du MVP : aucune decision actuarielle ni modele de risque automatique.
- Si des indicateurs de risque sont ajoutes plus tard, ils doivent etre explicitement valides par le client.

**Fraud Detection**
- Hors scope du MVP : aucune detection de fraude automatique.
- Possibilite future de signaler des anomalies suspectes via tags standardises (sans decision automatique).

**Reporting Compliance**
- Conformite aux formats de reporting attendus par le client (PPT, CR, TNR).
- Traçabilite des rapports via stockage SharePoint et references Jira/Squash.

**Technical Constraints**
- Jira : API token OK (hors VM).
- Squash : Basic auth au depart (token a demander).
- Least privilege : demarrage read-only, ecriture apres validation.
- Secrets dans un coffre/secret store, jamais en clair.
- HTTPS/TLS pour API ; Squash en HTTP probable (a faire evoluer).
- Stockage au repos sur SharePoint ; donnees locales minimisees, chiffrees, purges.
- Audit logs minimaux : horodatage, perimetre, version, statut, sans donnees sensibles.
- Restrictions IT a confirmer : scripts signes/macros, droits install, proxy, appels externes.
- Execution idealement sur environnement approuve (poste + M365), VM reservee aux applis testees.

**Integration Requirements**
- Critiques : Jira + Squash (sources de verite), SharePoint (preuves/livrables), Excel & PowerPoint (CR + reporting).
- Souhaitees : Outlook + Teams pour diffusion et recuperation d'infos reunions.
- Jira : API OK, tokens OK.
- Squash : API OK en Basic, token non dispo immediatement.

### PRD Completeness Assessment

The PRD provides explicit, numbered FRs and detailed NFRs with measurable targets. Several constraints and integration requirements are specified. However, multiple items are marked “a confirmer” (e.g., Jira/Squash timing, regulatory details), and open questions remain around API quotas, templates approval cadence, and KPI thresholds, which may affect implementation scope and validation criteria.

## Epic Coverage Validation

### Coverage Matrix

| FR Number | PRD Requirement | Epic Coverage | Status |
| --------- | --------------- | ------------ | ------ |
| FR1 | L'utilisateur peut importer des tickets depuis Jira pour une periode ou un perimetre donne. | Epic 1 | ✓ Covered |
| FR2 | Le systeme peut appliquer une checklist de testabilite a un ticket. | Epic 1 | ✓ Covered |
| FR3 | Le systeme peut produire un score Go/Clarify/No-Go par ticket. | Epic 1 | ✓ Covered |
| FR4 | L'utilisateur peut marquer un ticket comme "clarification requise" et tracer la cause. | Epic 1 | ✓ Covered |
| FR5 | L'utilisateur peut regrouper les tickets par lot/perimetre/cadence. | Epic 1 | ✓ Covered |
| FR6 | Le systeme peut generer un brouillon de strategie de test a partir d'un ticket. | Epic 2 | ✓ Covered |
| FR7 | Le systeme peut generer des propositions de cas de test (nominaux, variantes, limites). | Epic 2 | ✓ Covered |
| FR8 | L'utilisateur peut editer et valider les brouillons avant publication. | Epic 2 | ✓ Covered |
| FR9 | Le systeme peut integrer une checklist ESN dans la preparation des cas. | Epic 2 | ✓ Covered |
| FR10 | L'utilisateur peut lier des preuves a des cas de test (references vers SharePoint). | Epic 3 | ✓ Covered |
| FR11 | Le systeme peut centraliser les references de preuves par campagne. | Epic 3 | ✓ Covered |
| FR12 | Le systeme peut associer les preuves aux tickets Jira concernes. | Epic 3 | ✓ Covered |
| FR13 | L'utilisateur peut preparer une campagne TNR a partir d'un modele. | Epic 3 | ✓ Covered |
| FR14 | L'utilisateur peut generer un brouillon d'anomalie depuis un cas de test. | Epic 4 | ✓ Covered |
| FR15 | Le systeme applique un template standardise d'anomalie (champs obligatoires). | Epic 4 | ✓ Covered |
| FR16 | L'utilisateur peut lier l'anomalie a la campagne Squash et au ticket Jira. | Epic 4 | ✓ Covered |
| FR17 | Le systeme peut taguer les anomalies TNR avec un label specifique. | Epic 4 | ✓ Covered |
| FR18 | Le systeme peut generer un CR quotidien pre-rempli (blocs standard). | Epic 5 | ✓ Covered |
| FR19 | L'utilisateur peut editer le CR et produire une version finale. | Epic 5 | ✓ Covered |
| FR20 | Le systeme peut pre-generer le PPT hebdo avec les sections pertinentes. | Epic 5 | ✓ Covered |
| FR21 | Le systeme peut generer un PPT TNR (GO/NO-GO, reserves, liens). | Epic 5 | ✓ Covered |
| FR22 | Le systeme peut produire des exports multi-formats (txt/json/csv/md/html/yaml). | Epic 5 | ✓ Covered |
| FR23 | L'utilisateur peut configurer un projet via config.yaml. | Epic 0 | ✓ Covered |
| FR24 | L'utilisateur peut definir des profils de configuration par contexte. | Epic 0 | ✓ Covered |
| FR25 | Le systeme peut charger des templates (CR, PPT, anomalies). | Epic 6 | ✓ Covered |
| FR26 | Le systeme peut fonctionner en mode interactif ou batch. | Epic 6 | ✓ Covered |
| FR27 | Le systeme peut fournir une auto-completion shell. | Epic 6 | ✓ Covered |
| FR28 | Le systeme peut anonymiser automatiquement les donnees avant envoi cloud. | Epic 7 | ✓ Covered |
| FR29 | Le systeme peut fonctionner avec LLM local uniquement. | Epic 7 | ✓ Covered |
| FR30 | Le systeme peut journaliser les executions sans donnees sensibles. | Epic 7 | ✓ Covered |
| FR31 | Le systeme peut gerer les secrets via un secret store. | Epic 0 | ✓ Covered |

### Missing Requirements

No missing FRs identified. All PRD FRs (FR1–FR31) are covered in the epics.

### Coverage Statistics

- Total PRD FRs: 31
- FRs covered in epics: 31
- Coverage percentage: 100%

## UX Alignment Assessment

### UX Document Status

Found: `_bmad-output/planning-artifacts/ux-design-specification.md`

### Alignment Issues

- UX specifies visual foundation details (color palette “Pro-Calme”, IBM Plex typography) for outputs/docs; PRD does not specify visual standards. Confirm whether these are required constraints for report templates.
- UX defines CLI interaction components (progress indicator, status line, next-action suggestions, fzf-like picker, recovery cards) that are not explicitly called out in the Architecture decisions. Ensure `tf-cli` scope and template assets include these UX components.

### Warnings

- None (UX is present and broadly aligned with PRD and Architecture). Key UX performance targets and CLI-only scope match PRD and Architecture.

## Epic Quality Review

### 🔴 Critical Violations

1. **Epic 5 depends on Epic 6 (template loading) to function**
   - Evidence: Story 5.1/5.3/5.4 require templates to be available; template loading is only covered in Epic 6 (Story 6.1).
   - Impact: Epic 5 cannot be completed independently; violates epic independence rule.
   - Recommendation: Move “Charger des templates (CR/PPT/anomalies)” into Epic 0 or create a prerequisite story in Epic 5; or reorder epics so template loading precedes reporting.

2. **Epic 2 depends on Epic 7 for anonymization compliance**
   - Evidence: Stories 2.1/2.2 include cloud mode with anonymization, but anonymization capability is in Epic 7 (Story 7.1).
   - Impact: Epic 2 cannot meet its acceptance criteria without Epic 7, creating a forward dependency.
   - Recommendation: Either move anonymization capability earlier (Epic 0/1) or scope Epic 2 to local-only in MVP and add a follow‑up story for cloud anonymization.

### 🟠 Major Issues

1. **Logging/security cross‑cutting dependency is implemented last**
   - Evidence: Nearly every story asserts “logs sans données sensibles,” but logging/security implementation is in Epic 7.
   - Impact: Story acceptance criteria depend on later epic; risks non‑compliance during early implementation.
   - Recommendation: Introduce a cross‑cutting “logging/privacy baseline” story in Epic 0 or Epic 1, or split Epic 7 into a baseline (early) and advanced (later) set.

2. **Checklist/scoring configuration is assumed but not explicitly covered**
   - Evidence: Story 1.3/1.4 require checklist and scoring rules to be configured; there is no explicit story for checklist/scoring configuration (separate from templates).
   - Impact: Hidden dependency; may block Epic 1 completion.
   - Recommendation: Add a story for “Configurer la checklist testabilité + règles de scoring” (likely Epic 0 or Epic 1).

### 🟡 Minor Concerns

- **Story 5.1 scope is large** (multi‑source integration + template + SLA). Consider splitting into “source ingestion” vs “CR assembly” for easier delivery/testing.
- **Epic 0 sits before Epic 1, but Story 1.1 is also foundational**. Not a blocker, but consider moving the starter‑template setup into Epic 0 for clearer sequencing (unless you want to keep it as Epic 1 by convention).

### Recommendations Summary

- Reorder or refactor epics to eliminate forward dependencies (Templates & Logging/Privacy baselines before Reporting/LLM usage).
- Add explicit configuration stories for checklist/scoring to remove hidden prerequisites.
- Split large reporting stories for incremental delivery and clearer acceptance testing.

## Summary and Recommendations

### Overall Readiness Status

NOT READY

### Critical Issues Requiring Immediate Action

- Epic 5 depends on Epic 6 for template loading (violates epic independence).
- Epic 2 depends on Epic 7 for anonymization requirements (forward dependency).

### Recommended Next Steps

1. Reorder or refactor epics so template loading and privacy/logging baselines are delivered before reporting and LLM‑assisted features.
2. Add explicit stories for checklist/scoring configuration and baseline logging/privacy to remove hidden prerequisites.
3. Split oversized reporting stories (e.g., Story 5.1) into smaller, independently testable units.

### Final Note

This assessment identified 6 issues across 2 categories (UX alignment, epic quality). Address the critical issues before proceeding to implementation, or proceed with explicit risk acceptance.

**Assessment Date:** 2026-01-30
**Assessor:** Implementation Readiness Workflow

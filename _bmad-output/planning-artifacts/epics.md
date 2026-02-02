---
stepsCompleted:
  - 'step-01-validate-prerequisites'
  - 'step-02-design-epics'
  - 'step-03-create-stories'
  - 'step-04-final-validation'
inputDocuments:
  - '_bmad-output/planning-artifacts/prd.md'
  - '_bmad-output/planning-artifacts/architecture.md'
  - '_bmad-output/planning-artifacts/ux-design-specification.md'
---

# test-framework - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for test-framework, decomposing the requirements from the PRD, UX Design if it exists, and Architecture requirements into implementable stories.

## Requirements Inventory

### Functional Requirements

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

### NonFunctional Requirements

NFR1: Donnees sensibles jamais envoyees vers un LLM cloud sans anonymisation automatique (controle sur 100% des envois sortants).
NFR2: Secrets stockes via secret store, jamais en clair (0 secret en clair dans le repo ou la configuration).
NFR3: Least privilege : read-only par defaut, ecriture validee explicitement (aucune ecriture sans activation explicite).
NFR4: Audit logs minimaux sans donnees sensibles (horodatage, perimetre, statut) et conservation 90 jours.
NFR5: Donnees temporaires locales purgees sous 24h (automatique).
NFR6: Generation d'un CR quotidien pre-rempli < 2 minutes pour un perimetre standard; CR pret a envoyer < 5 minutes apres le run (incluant relecture/ajustements humains).
NFR7: Extraction Jira/Squash pour un lot < 5 minutes (a confirmer).
NFR8: CLI reactive : reponse interactive < 2 secondes pour commandes simples.
NFR9: En cas d'echec d'une integration, l'outil doit continuer en mode degrade (taux d'echec < 2% des executions).
NFR10: Reprise possible sur derniere execution (mode replay) sans perte de contexte, reprise < 5 minutes.
NFR11: Support Jira/Squash via API avec auth token (Jira) et Basic (Squash), taux de succes >= 95% sur appels en volume standard.
NFR12: Sorties multi-formats strictement conformes aux templates existants (CR, PPT, anomalies), sans personnalisation visuelle.
NFR13: Fonctionnement hors VM, VM reservee aux applis testees.
NFR14: Fraicheur des donnees integrees < 15 minutes sur pipeline standard.
NFR15: Execution possible sur poste M365, sans dependances cloud obligatoires.
NFR16: Respect des restrictions IT (scripts signes, proxy, droits d'installation).

### Additional Requirements

- Starter template requis: rust-starter (cargo-generate). Init: "cargo generate --git https://github.com/rust-starter/rust-starter-generate".
- Stockage local SQLite (3.51.1) avec cache enrichi chiffre, TTL court, purge auto, replay explicite.
- Validation stricte des donnees (reject + rapport d'erreurs) et modeles internes types.
- Gestion des secrets via OS keyring (keyring crate 3.6.2).
- Anonymisation inline obligatoire avant tout envoi cloud.
- Logs JSON structures sans donnees sensibles + commande diagnostics ("tf diagnostics").
- Integrations REST + fallback CSV first-class, retry exponentiel avec backoff, erreurs structurees (code + message + hint).
- Distribution via npm wrapper (binaire par plateforme), CI/CD GitHub Actions, versioning SemVer + changelog.
- Conventions strictes: nommage Rust/JSON, JSON envelope standard, codes de sortie (0/1/2/3).
- Structure projet multi-crates et boundaries telles que definies (tf-cli, tf-core, tf-connectors, tf-storage, tf-security, tf-config, tf-logging, tf-diagnostics).
- UX CLI: run unique guide, mode interactif + batch, fallback CSV si APIs indisponibles.
- UX exigences: messages courts et actionnables, "next action" en fin de flow, zero double saisie, etats visibles.
- Accessibilite outputs/docs: contraste eleve, etats non relies uniquement aux couleurs, lisibilite terminal.
- Requirements UX performance: pre-remplissage CR < 2 minutes, CR pret a envoyer < 5 minutes apres run (incluant relecture/ajustements), checklist explicite Go/Clarify/No-Go.

### UX Acceptance Criteria (Global)

- Indicateur de progression visible pour les actions longues (import, run, generation CR/PPT).
- Status line claire (OK/WARN/ERROR) avec indication explicite du mode degrade (fallback CSV).
- "Next action" proposee en fin de flow (run/triage/report/anomalies).
- Erreurs actionnables avec cause + action conseillee (retry, reconfig, fallback CSV).
- Selection rapide type recherche/picker pour listes volumineuses (tickets, cas).

### FR Coverage Map

FR1: Epic 1 - Import tickets Jira pour un perimetre/period
FR2: Epic 1 - Checklist testabilite
FR3: Epic 1 - Score Go/Clarify/No-Go
FR4: Epic 1 - Marquer clarification + cause
FR5: Epic 1 - Regrouper tickets par lot/perimetre/cadence
FR6: Epic 2 - Brouillon strategie de test
FR7: Epic 2 - Propositions de cas de test
FR8: Epic 2 - Edition/validation des brouillons
FR9: Epic 2 - Checklist ESN integree
FR10: Epic 3 - Lier preuves aux executions de cas
FR11: Epic 3 - Centraliser references de preuves
FR12: Epic 3 - Associer preuves aux tickets Jira
FR13: Epic 3 - Preparer campagne TNR
FR14: Epic 4 - Brouillon d'anomalie depuis cas
FR15: Epic 4 - Template standardise d'anomalie
FR16: Epic 4 - Lier anomalie Squash + Jira
FR17: Epic 4 - Taguer anomalies TNR
FR18: Epic 5 - CR quotidien pre-rempli
FR19: Epic 5 - Edition + version finale CR
FR20: Epic 5 - PPT hebdo pre-genere
FR21: Epic 5 - PPT TNR GO/NO-GO
FR22: Epic 5 - Exports multi-formats
FR23: Epic 0 - config.yaml par projet (baseline)
FR24: Epic 0 - Profils de configuration (baseline)
FR25: Epic 0 - Chargement templates (baseline)
FR26: Epic 6 - Modes interactif/batch
FR27: Epic 6 - Auto-completion shell
FR28: Epic 0 - Anonymisation avant cloud (baseline)
FR29: Epic 7 - LLM local uniquement
FR30: Epic 0 - Journaliser sans donnees sensibles (baseline)
FR31: Epic 0 - Secrets via secret store (baseline)

## Epic List

### Epic 0: Foundation & Access
Mettre en place le socle minimal (config, profils, templates, logs, secrets, anonymisation) avant les epics orientés utilisateur.
**FRs covered:** FR23, FR24, FR25, FR28, FR30, FR31

### Epic 1: Triage & readiness
Permettre d'importer les tickets et decider rapidement Go/Clarify/No-Go par perimetre.
**FRs covered:** FR1, FR2, FR3, FR4, FR5

### Epic 2: Assistance a la conception des tests
Generer et valider des brouillons de strategie et de cas de test avec checklist ESN.
**FRs covered:** FR6, FR7, FR8, FR9

### Epic 3: Execution & preuves
Lier, centraliser et reutiliser les preuves; preparer une campagne TNR.
**FRs covered:** FR10, FR11, FR12, FR13

### Epic 4: Gestion des anomalies
Generer des anomalies standardisees et les relier a Jira/Squash, y compris TNR.
**FRs covered:** FR14, FR15, FR16, FR17

### Epic 5: Reporting & exports
Produire CR quotidien, PPT hebdo/TNR et exports multi-formats.
**FRs covered:** FR18, FR19, FR20, FR21, FR22

### Epic 6: Configuration & automatisation CLI
Modes interactif/batch et auto-completion pour accelerer l'usage CLI.
**FRs covered:** FR26, FR27

### Epic 7: Conformite & securite operationnelle
Garantir le mode LLM local uniquement et les exigences d'audit/retention.
**FRs covered:** FR29

## Epic 0: Foundation & Access

Mettre en place le socle minimal (config, profils, templates, logs, secrets, anonymisation) avant les epics orientés utilisateur.

### Story 0.1: Configurer un projet via config.yaml

As a QA tester (TRA),
I want configurer un projet via un fichier config.yaml,
So that centraliser la configuration et eviter la re-saisie.

**Acceptance Criteria:**

**Given** un fichier config.yaml present
**When** je lance l'outil
**Then** la configuration est lue et validee selon le schema attendu

**Given** une configuration invalide
**When** je lance l'outil
**Then** un message explicite indique le champ en defaut et la correction attendue

**Given** une configuration chargee
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

### Story 0.2: Definir et selectionner des profils de configuration

As a QA tester (TRA),
I want definir et selectionner des profils de configuration,
So that basculer rapidement de contexte.

**Acceptance Criteria:**

**Given** une config.yaml avec des profils
**When** je selectionne un profil
**Then** la configuration du profil est appliquee et affichee dans le resume

**Given** un profil inconnu
**When** je tente de le selectionner
**Then** un message indique l'erreur et liste les profils disponibles

**Given** un profil applique
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

### Story 0.3: Gestion des secrets via secret store

As a QA tester (TRA),
I want stocker et recuperer les secrets via un secret store OS,
So that eviter tout secret en clair dans le repo ou la config.

**Acceptance Criteria:**

**Given** un keyring disponible
**When** j'enregistre un secret
**Then** il est stocke dans le keyring et recuperable par l'outil

**Given** un keyring indisponible
**When** je tente d'enregistrer un secret
**Then** un message explicite indique l'action a suivre

**Given** un secret utilise
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

### Story 0.4: Charger des templates (CR/PPT/anomalies)

As a QA tester (TRA),
I want charger des templates (CR/PPT/anomalies) depuis un chemin configure,
So that standardiser les livrables des epics de reporting et d'anomalies.

**Acceptance Criteria:**

**Given** des chemins de templates definis dans la config
**When** je charge un template
**Then** il est valide (existence + format) et pret a l'usage

**Given** un template manquant ou invalide
**When** je tente de le charger
**Then** un message explicite indique l'action a suivre

**Given** un template charge
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

### Story 0.5: Journalisation baseline sans donnees sensibles

As a QA maintainer,
I want une journalisation baseline sans donnees sensibles,
So that garantir l'auditabilite minimale des executions des le debut.

**Acceptance Criteria:**

**Given** la journalisation activee
**When** une commande CLI s'execute
**Then** des logs JSON structures sont generes (timestamp, commande, statut, perimetre)

**Given** des champs sensibles sont presents dans le contexte
**When** ils seraient journalises
**Then** ils sont masques automatiquement

**Given** une execution terminee
**When** les logs sont ecrits
**Then** ils sont stockes dans le dossier de sortie configure

### Story 0.6: Configurer checklist de testabilite et regles de scoring

As a QA tester (TRA),
I want configurer la checklist de testabilite et les regles de scoring par projet/profil,
So that garantir un triage reproductible et conforme.

**Acceptance Criteria:**

**Given** une config.yaml valide avec checklist et regles de scoring
**When** je charge un projet ou un profil
**Then** la checklist et les regles sont valides et disponibles pour le triage

**Given** une checklist ou des regles manquantes
**When** je tente de lancer le triage
**Then** un message explicite indique les champs manquants et l'action a suivre

**Given** une configuration appliquee
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

### Story 0.7: Anonymisation automatique avant envoi cloud

As a QA tester (TRA),
I want anonymiser automatiquement les donnees sensibles avant tout envoi cloud,
So that respecter la conformite et reduire les risques des le MVP.

**Acceptance Criteria:**

**Given** l'option cloud activee et des regles d'anonymisation configurees
**When** un envoi vers le cloud est declenche
**Then** les donnees sont anonymisees avant l'envoi

**Given** une anonymisation qui echoue
**When** l'envoi est tente
**Then** l'envoi est bloque avec un message explicite

**Given** une anonymisation effectuee
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

## Epic 1: Triage & readiness

Permettre d'importer les tickets et decider rapidement Go/Clarify/No-Go par perimetre.

### Story 1.1: Initialiser la base du projet via rust-starter

As a QA maintainer,
I want initialiser le workspace CLI avec le starter rust-starter,
So that disposer d'une base de code standard, prete pour les premieres features.

**Acceptance Criteria:**

**Given** l'acces au repo du starter
**When** j'execute `cargo generate --git https://github.com/rust-starter/rust-starter-generate`
**Then** un workspace Rust est cree avec la structure CLI attendue

**Given** le workspace genere
**When** je lance `cargo build`
**Then** le build passe sans erreur

**Given** le workspace genere
**When** je verifie la structure
**Then** les elements cles attendus sont presents (CLI, config/logging, CI)

### Story 1.2: Import Jira par periode et perimetre

As a QA tester (TRA),
I want importer des tickets Jira par periode et perimetre (projet/board/JQL, avec filtres de statut),
So that disposer d'une liste fiable de tickets a trier sans export manuel.

**Acceptance Criteria:**

**Given** un config.yaml valide, un profil actif et un token Jira read-only
**When** je lance l'import avec une periode et un perimetre
**Then** les tickets sont importes et un resume est affiche (compte, perimetre, periode)
**And** les champs requis sont valides (id, titre, statut, priorite)

**Given** un token Jira invalide
**When** je lance l'import
**Then** un message d'erreur clair est affiche avec un hint d'action (ex: reconfigurer le token)

**Given** l'API Jira est indisponible
**When** je lance l'import
**Then** le mode degrade CSV est propose comme alternative

**Given** un perimetre standard
**When** l'import s'execute
**Then** la duree reste < 5 minutes

**Given** l'import se termine
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

### Story 1.3: Checklist de testabilite

As a QA tester (TRA),
I want appliquer une checklist de testabilite a un ticket,
So that decider rapidement si le ticket est testable ou doit etre clarifie.

**Acceptance Criteria:**

**Given** un ticket importe et une checklist definie pour le projet/profil
**When** j'applique la checklist au ticket
**Then** les resultats par critere sont visibles pour ce ticket
**And** les criteres critiques exigent une reponse obligatoire

**Given** aucune checklist n'est disponible
**When** je tente d'appliquer la checklist
**Then** un message clair propose l'action a suivre (charger/parametrer un template)

**Given** une checklist appliquee
**When** le resultat est enregistre
**Then** il est journalise sans donnees sensibles

### Story 1.4: Scoring Go/Clarify/No-Go

As a QA tester (TRA),
I want generer un score Go/Clarify/No-Go a partir des reponses de checklist,
So that decider vite et rendre le triage explicite.

**Acceptance Criteria:**

**Given** une checklist appliquee au ticket
**When** je calcule le score
**Then** le score Go/Clarify/No-Go et ses raisons sont visibles pour ce ticket
**And** les regles de scoring suivent la configuration du projet/profil

**Given** des regles de scoring manquantes
**When** je tente de calculer le score
**Then** un message clair propose l'action a suivre (configurer les regles)

**Given** un score calcule
**When** le resultat est enregistre
**Then** il est journalise sans donnees sensibles

### Story 1.5: Marquer \"clarification requise\" + cause

As a QA tester (TRA),
I want marquer un ticket en \"Clarification requise\" et saisir la cause,
So that rendre la decision explicite et tracable.

**Acceptance Criteria:**

**Given** un score Clarify ou No-Go calcule
**When** je marque le ticket en \"Clarification requise\"
**Then** la cause est obligatoire et enregistree pour ce ticket
**And** une liste de causes standard est disponible avec un commentaire optionnel

**Given** aucune cause n'est fournie
**When** je tente d'enregistrer
**Then** la validation bloque et indique la cause manquante

**Given** une clarification enregistree
**When** les resultats sont exportes
**Then** la cause apparait dans le resume (CR/rapport)

**Given** la clarification est enregistree
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

### Story 1.6: Regroupement par lot/perimetre/cadence

As a QA tester (TRA),
I want regrouper les tickets importes par lot/perimetre/cadence,
So that organiser le triage et prioriser plus vite.

**Acceptance Criteria:**

**Given** des tickets importes avec leurs metadonnees
**When** j'applique un regroupement par lot/perimetre/cadence
**Then** des groupes sont crees avec un resume par groupe (compte, statuts cles)
**And** les criteres de regroupement suivent la configuration du profil

**Given** des tickets avec metadonnees manquantes
**When** je regroupe
**Then** ils sont listes en \"non groupes\" avec un message explicite

**Given** un regroupement effectue
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

## Epic 2: Assistance a la conception des tests

Generer et valider des brouillons de strategie et de cas de test avec checklist ESN.

### Story 2.1: Brouillon de strategie de test

As a QA tester (TRA),
I want generer un brouillon de strategie de test a partir d'un ticket,
So that accelerer la preparation et standardiser l'approche.

**Acceptance Criteria:**

**Given** un ticket importe et un LLM local ou cloud configure
**When** je genere la strategie de test
**Then** un brouillon est cree avec les sections attendues (contexte, perimetre, objectifs, risques)

**Given** un LLM indisponible
**When** je demande la generation
**Then** un modele vide est propose avec un message d'explication

**Given** une option cloud activee
**When** la generation est lancee
**Then** l'anonymisation est appliquee avant tout envoi

**Given** un brouillon genere
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

### Story 2.2: Propositions de cas de test

As a QA tester (TRA),
I want generer des propositions de cas de test (nominaux, variantes, limites) a partir d'un ticket,
So that gagner du temps sur la conception et elargir la couverture.

**Acceptance Criteria:**

**Given** un ticket importe, une strategie de test disponible et un LLM local ou cloud configure
**When** je genere des propositions de cas
**Then** une liste de cas structures est produite (titre, preconditions, etapes, resultats attendus)

**Given** un LLM indisponible
**When** je demande la generation
**Then** un modele vide est propose avec un message d'explication

**Given** une option cloud activee
**When** la generation est lancee
**Then** l'anonymisation est appliquee avant tout envoi

**Given** des propositions generees
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

### Story 2.3: Editer et valider les brouillons

As a QA tester (TRA),
I want editer et valider les brouillons de strategie et de cas de test,
So that assurer la qualite avant publication.

**Acceptance Criteria:**

**Given** des brouillons generes
**When** je les edite
**Then** les modifications sont enregistrees en mode brouillon

**Given** des champs requis manquants
**When** je tente de valider
**Then** la validation est bloquee avec un message explicite

**Given** tous les champs requis complets
**When** je valide un brouillon
**Then** son statut passe a \"valide\"
**And** un historique minimal est conserve (date/validateur)

**Given** un brouillon valide
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

### Story 2.4: Integrer la checklist ESN

As a QA tester (TRA),
I want integrer la checklist ESN dans la preparation des cas de test,
So that garantir la conformite aux standards ESN et reduire les retours.

**Acceptance Criteria:**

**Given** une checklist ESN configuree pour le projet/profil
**When** je prepare des cas de test
**Then** les criteres ESN sont appliques et visibles

**Given** des criteres ESN manquants
**When** je tente de valider
**Then** la validation est bloquee avec un message explicite

**Given** aucune checklist ESN n'est disponible
**When** je tente de l'appliquer
**Then** un message clair propose l'action a suivre (charger/parametrer la checklist ESN)

**Given** une checklist ESN appliquee
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

## Epic 3: Execution & preuves

Lier, centraliser et reutiliser les preuves; preparer une campagne TNR.

### Story 3.1: Lier des preuves a une execution de cas de test

As a QA tester (TRA),
I want lier une preuve (lien SharePoint) a une execution de cas de test,
So that assurer la tracabilite fiable des preuves par execution.

**Acceptance Criteria:**

**Given** une execution de cas de test existante et un acces SharePoint configure
**When** j'ajoute un lien de preuve
**Then** la preuve est liee a l'execution et visible dans le detail

**Given** un lien invalide
**When** je tente de l'enregistrer
**Then** un message explicite demande la correction

**Given** une preuve liee
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

### Story 3.2: Centraliser les references de preuves par campagne

As a QA tester (TRA),
I want centraliser toutes les references de preuves d'une campagne,
So that retrouver rapidement toutes les preuves liees.

**Acceptance Criteria:**

**Given** des executions avec preuves liees
**When** je consulte une campagne
**Then** une liste consolidee des preuves est affichee avec leurs liens

**Given** une campagne sans preuves
**When** je consulte la liste
**Then** un message indique qu'aucune preuve n'est liee

**Given** une liste consolidee
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

### Story 3.3: Associer preuves ↔ tickets Jira

As a QA tester (TRA),
I want associer les preuves d'une execution aux tickets Jira concernes,
So that assurer une tracabilite bout-en-bout preuves ↔ tickets.

**Acceptance Criteria:**

**Given** des preuves liees a des executions et des tickets Jira importes
**When** j'associe une preuve a un ticket
**Then** l'association est visible dans le resume du ticket et de l'execution

**Given** un ticket introuvable
**When** je tente l'association
**Then** un message explicite indique l'echec

**Given** une association creee
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

### Story 3.4: Preparer une campagne TNR a partir d'un modele

As a QA tester (TRA),
I want preparer une campagne TNR a partir d'un modele,
So that standardiser la campagne et gagner du temps.

**Acceptance Criteria:**

**Given** un modele TNR disponible pour le projet
**When** je lance la preparation d'une campagne
**Then** une campagne TNR est creee avec la structure et les champs attendus

**Given** un modele TNR manquant
**When** je tente la preparation
**Then** un message indique l'action a suivre (charger/parametrer le modele)

**Given** une campagne TNR creee
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

## Epic 4: Gestion des anomalies

Generer des anomalies standardisees et les relier a Jira/Squash, y compris TNR.

### Story 4.1: Brouillon d'anomalie depuis une execution de cas de test

As a QA tester (TRA),
I want generer un brouillon d'anomalie depuis une execution de cas de test,
So that accelerer la creation d'anomalies coherentes.

**Acceptance Criteria:**

**Given** une execution de cas de test avec preuve liee
**When** je genere une anomalie
**Then** un brouillon est cree avec les champs essentiels (contexte, repro, attendu, observe)

**Given** une execution introuvable
**When** je tente la generation
**Then** un message explicite indique l'echec

**Given** un brouillon genere
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

### Story 4.2: Template standardise d'anomalie

As a QA tester (TRA),
I want appliquer un template standardise d'anomalie,
So that uniformiser la qualite des anomalies.

**Acceptance Criteria:**

**Given** un template d'anomalie configure pour le projet
**When** je prepare une anomalie
**Then** le template est applique avec l'ordre et les champs attendus
**And** les champs obligatoires sont valides

**Given** aucun template n'est disponible
**When** je tente d'appliquer le template
**Then** un message clair propose l'action a suivre (charger/parametrer le template)

**Given** un template applique
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

### Story 4.3: Lier l'anomalie a la campagne Squash et au ticket Jira

As a QA tester (TRA),
I want lier une anomalie a la campagne Squash et au ticket Jira correspondant,
So that assurer une tracabilite complete.

**Acceptance Criteria:**

**Given** une anomalie creee, une campagne Squash connue et un ticket Jira importe
**When** j'associe l'anomalie
**Then** les liens Squash et Jira sont visibles dans l'anomalie

**Given** un identifiant Squash ou Jira invalide
**When** je tente l'association
**Then** un message explicite indique l'echec

**Given** une association creee
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

### Story 4.4: Taguer les anomalies TNR par numero de lot

As a QA tester (TRA),
I want taguer une anomalie TNR avec un label global suffixe par numero de lot,
So that distinguer clairement les anomalies TNR par lot.

**Acceptance Criteria:**

**Given** une anomalie creee et un numero de lot disponible
**When** je tague l'anomalie en TNR
**Then** le label TNR est applique au format global + numero de lot (ex: `TNR-LOT-123`)

**Given** un numero de lot manquant
**When** je tente de taguer en TNR
**Then** un message explicite indique l'action a suivre

**Given** un tag TNR applique
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

## Epic 5: Reporting & exports

Produire CR quotidien, PPT hebdo/TNR et exports multi-formats.

### Story 5.1: Collecter les donnees de reporting

As a QA tester (TRA),
I want collecter les donnees de reporting depuis Jira/Squash/Outlook/Excel,
So that disposer d'un dataset normalise pour alimenter CR et PPT.

**Acceptance Criteria:**

**Given** des sources connectees et un perimetre defini
**When** je lance la collecte
**Then** un dataset normalise est produit avec la trace des sources et horodatage

**Given** une source indisponible
**When** je lance la collecte
**Then** un message explicite propose un mode degrade (CSV/import) et la collecte continue avec les sources restantes

**Given** un perimetre standard
**When** je collecte les donnees
**Then** la duree reste < 5 minutes

**Given** une collecte terminee
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

### Story 5.2: Generer un CR quotidien pre-rempli

As a QA tester (TRA),
I want generer un CR quotidien pre-rempli a partir du dataset et du template CR,
So that reduire le temps de reporting et standardiser le CR.

**Acceptance Criteria:**

**Given** un dataset collecte et un template CR disponible
**When** je lance la generation du CR
**Then** un CR est cree avec les sections attendues (resume, avancement, anomalies, risques)

**Given** un template CR manquant
**When** je tente la generation
**Then** un message explicite indique l'action a suivre (charger/parametrer le template)

**Given** un perimetre standard
**When** je genere le CR
**Then** la duree reste < 2 minutes

**Given** un CR genere
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

### Story 5.3: Editer et finaliser le CR

As a QA tester (TRA),
I want editer le CR genere et produire une version finale,
So that assurer la qualite avant diffusion.

**Acceptance Criteria:**

**Given** un CR genere
**When** je l'edite
**Then** les modifications sont enregistrees en brouillon

**Given** des champs requis manquants
**When** je tente de finaliser
**Then** la finalisation est bloquee avec un message explicite

**Given** tous les champs requis complets
**When** je finalise le CR
**Then** son statut passe a \"final\"

**Given** un CR finalise
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

### Story 5.4: Pre-generer le PPT hebdo

As a QA tester (TRA),
I want pre-generer un PPT hebdo a partir d'un template et des sources,
So that gagner du temps et garantir un format homogene.

**Acceptance Criteria:**

**Given** un template PPT disponible et des sources connectees
**When** je lance la generation hebdo
**Then** un PPT est cree avec les sections attendues (KPIs, anomalies cles, priorites)

**Given** un template PPT manquant
**When** je tente la generation
**Then** un message indique l'action a suivre (charger/parametrer le template)

**Given** un PPT genere
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

### Story 5.5: Generer le PPT TNR

As a QA tester (TRA),
I want generer un PPT TNR (GO/NO-GO, reserves, liens preuves/anomalies),
So that soutenir une decision GO/NO-GO structuree et rapide.

**Acceptance Criteria:**

**Given** une campagne TNR preparee et executee, et un template PPT TNR disponible
**When** je lance la generation
**Then** un PPT TNR est cree avec les sections attendues (GO/NO-GO, reserves, liens)

**Given** un template PPT TNR manquant
**When** je tente la generation
**Then** un message indique l'action a suivre (charger/parametrer le template)

**Given** un PPT TNR genere
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

### Story 5.6: Exports multi-formats

As a QA tester (TRA),
I want exporter les livrables en formats txt/json/csv/md/html/yaml,
So that assurer la compatibilite et la reutilisation.

**Acceptance Criteria:**

**Given** un livrable genere (CR/anomalies/rapports)
**When** je demande un export avec un format supporte
**Then** un fichier est genere dans le format choisi

**Given** un format non supporte
**When** je tente l'export
**Then** un message explicite indique les formats autorises

**Given** un export effectue
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

## Epic 6: Configuration & automatisation CLI

Modes interactif/batch et auto-completion pour accelerer l'usage CLI.

### Story 6.1: Modes interactif et batch

As a QA tester (TRA),
I want executer l'outil en mode interactif guide ou en mode batch,
So that adapter l'usage selon le contexte.

**Acceptance Criteria:**

**Given** une configuration valide
**When** j'execute une commande en mode interactif ou en batch
**Then** le resultat fonctionnel est equivalent

**Given** des flags incompatibles
**When** je lance la commande
**Then** un message explicite indique l'erreur et la correction

**Given** un mode batch
**When** je lance une commande
**Then** la sortie machine-readable est disponible si demandee

**Given** une execution terminee
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

### Story 6.2: Auto-completion shell

As a QA tester (TRA),
I want activer l'auto-completion shell pour les commandes, flags et profils,
So that reduire les erreurs et accelerer l'usage.

**Acceptance Criteria:**

**Given** la CLI installee et un shell compatible
**When** j'active la completion
**Then** les commandes, flags et profils sont proposes en auto-completion

**Given** un shell non supporte
**When** je tente d'activer la completion
**Then** un message explicite indique les shells supportes

**Given** la completion activee
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

## Epic 7: Conformite & securite operationnelle

Garantir le mode LLM local uniquement et les exigences d'audit/retention.

### Story 7.1: Mode LLM local uniquement

As a QA tester (TRA),
I want executer toutes les generations en mode LLM local uniquement,
So that garantir la confidentialite.

**Acceptance Criteria:**

**Given** un LLM local configure
**When** j'execute une generation
**Then** aucune requete cloud n'est effectuee

**Given** un LLM local indisponible
**When** je lance une generation en mode local uniquement
**Then** un message explicite indique l'echec et l'action a suivre

**Given** une generation terminee
**When** les logs sont ecrits
**Then** ils ne contiennent aucune donnee sensible

### Story 7.2: Retention et audit des logs

As a QA maintainer,
I want appliquer une retention et produire un audit minimal des logs,
So that respecter les exigences de conformite sans exposer de donnees sensibles.

**Acceptance Criteria:**

**Given** des logs baseline generes
**When** la politique de retention s'applique
**Then** les logs de plus de 90 jours sont purges automatiquement

**Given** un audit interne demande
**When** j'execute `tf diagnostics`
**Then** un rapport minimal est produit (timestamp, perimetre, version, statut) sans donnees sensibles

**Given** un export de logs demande
**When** il est genere
**Then** les champs sensibles sont masques et les metadonnees minimales conservees

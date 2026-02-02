---
stepsCompleted:
  - 'step-01-init'
  - 'step-02-discovery'
  - 'step-03-success'
  - 'step-04-journeys'
  - 'step-05-domain'
  - 'step-06-innovation'
  - 'step-07-project-type'
  - 'step-08-scoping'
  - 'step-09-functional'
  - 'step-10-nonfunctional'
  - 'step-11-polish'
inputDocuments:
  - '_bmad-output/prd/qa-tester-process-optimization-brief-client.md'
  - '_bmad-output/prd/qa-tester-process-optimization-prd.md'
  - '_bmad-output/brainstorming/brainstorming-session-2026-01-29T09:10:17Z.md'
workflowType: 'prd'
classification:
  projectType: 'cli_tool'
  domain: 'insuretech'
  complexity: 'high'
  projectContext: 'brownfield'
documentCounts:
  briefCount: 1
  researchCount: 0
  brainstormingCount: 1
  projectDocsCount: 1
---

# Product Requirements Document - QA Tester Process Optimization (TRA)

**Author:** Edouard Zemb
**Date:** 2026-01-30

## Executive Summary

- Optimiser le workflow du testeur TRA (assurance) sur un projet brownfield, sans changer les outils existants (Jira, Squash, Excel, PPT, Outlook, Teams, SharePoint).
- Priorites : fiabilite et qualite des processus, standardisation des livrables, reduction de la charge mentale, gain de 2 a 3 heures par semaine.
- Approche : outillage CLI reutilisable et scriptable, avec automatisations invisibles et integration LLM local + cloud des le MVP (cloud soumis a anonymisation automatique).
- Differenciateur : pipeline modulaire multi-projets et usage futur des preuves video pour ameliorer la qualite des cas de test.

## Success Criteria

### User Success
- Gain de temps de 2 a 3 heures par semaine sur le reporting et la structuration.
- Baisse nette de la charge mentale grace a des formats standardises (checklist + anomalies).
- Comptes rendus finalises sans rework majeur.

### Business Success
- Perception client d'une montee en professionnalisme : reporting coherent, anomalies claires, decisions plus sures.
- Hausse du taux de tickets "ready" apres checklist (seuil a definir avec le client).
- Conformite aux templates d'anomalies et de reporting superieure a 90% (a confirmer).

### Technical Success
- Logique reutilisable et ameliorable pour d'autres projets (modulaire, documentee).
- Fonctionnement possible 100% en local, avec option cloud disponible des le MVP (toujours anonymisee).
- Donnees tracees et coherentes entre Jira, Squash, Excel et PPT.

### Measurable Outcomes
- Gain de temps: 2 a 3 heures par semaine.
- Reduction des erreurs de reporting: -50% vs baseline sur 4 semaines (seuil minimal -30%).
- Feedback client positif: note >= 4/5 sur 2 cycles hebdo consecutifs et 0 retour negatif majeur sur 1 mois.

## Product Scope

### MVP Strategy & Philosophy
**MVP Approach:** Problem MVP + Platform MVP
**Resource Requirements:** 1 personne (toi) + acces API + secret store + config multi-profils

### MVP - Minimum Viable Product

**Core User Journeys Supported:**
- Parcours "jour maitrise" (triage -> devis -> conception -> execution -> CR)
- Parcours "edge case" avec garde-fous + CR securise

**Must-Have Capabilities:**
- Checklist testabilite + score Go / Clarify / No-Go
- Templates anomalies + preuves standardisees
- Extraction Jira/Squash -> pre-remplissage CR quotidien
- Generation de brouillons (cas de tests / anomalies) via LLM local ou cloud (avec anonymisation)
- config.yaml + profils + secret store (base reutilisable)

### Growth Features (Post-MVP)
- Auto-remplissage PPT hebdo (structure fixe)
- Source unique et standardisation des metriques
- Suggestions Squash (reutilisation assets)
- Notifications Teams/Outlook si APIs OK
- Dashboards simples (KPI)

### Vision (Future)
- IA sur preuves video pour ameliorer les cas de test
- Score qualite / fiabilite continu des tickets
- Planification intelligente (cadence QA/HO)
- Analyse historique multi-lots

### Risk Mitigation Strategy
**Technical Risks:** integration preuves video + qualite IA -> valider sur echantillon avant generalisation
**Market Risks:** resistance aux templates/process -> mode opt-in + comparaison avant/apres
**Resource Risks:** acces API / droits IT / temps -> prioriser MVP auto-contenu + fallback manuel

## User Journeys

### Journey 1 - Primary User (toi) - Succes "jour maitrise"
Tu demarres a 9h. Les emails sont tries rapidement, puis le weekly TRA aligne les priorites. Le planning mensuel te donne une trajectoire claire.
En phase devis, les tickets Jira passent par une checklist testabilite : ceux qui ne sont pas "ready" sont immediatement identifies. Un pipeline d'aide propose une extraction d'exigences et une strategie de tests structuree, que tu ajustes.
En conception, Squash est alimente par des propositions de cas de test pretes a relire, avec la checklist ESN integree.
En execution, tu lances la campagne ; les preuves video sont centralisees automatiquement et liees aux cas. Les anomalies se generent avec un template structure, preuves liees, et l'association Squash <-> Jira est fluide.
En fin de journee, le CR quotidien est pre-rempli (extraction Jira/Squash/Outlook), tu corriges et envoies en 5 minutes.
Le mercredi, le PPT hebdo est pre-genere a partir des sources, tu ajustes les priorites et l'envoies.
Resultat : un rythme maitrise, une qualite percue superieure, et un effort mental reduit.

### Journey 2 - Primary User - Edge Case "jour qui deborde"
Le weekly deborde, les tickets sont volumineux et mal cadres. Tu risques la dispersion.
Le systeme te ramene a un cadre industriel : tri automatique des tickets, alertes sur manque d'infos, actions proposees (clarification, demande MOA, report).
Il suit la consommation de charge vs prevision, t'alerte avant la derive, et propose un plan de rattrapage (priorite, suspension, decalage).
En fin de journee, meme si tout part en vrille, le CR pre-rempli securise le rituel, evite le "non-envoi".
Resultat : tu gardes le controle, meme dans le chaos.

### Journey 3 - Admin/Ops (toi) - Configuration et gouvernance
Tu ajustes les templates (anomalies, CR, PPT), les regles de checklist, les seuils KPI, et les sources de verite.
Tu ajoutes un nouveau perimetre, tu declares ses regles de nommage, et tu reutilises le pipeline existant.
Resultat : l'outillage est modulaire, maintenable, reutilisable.

### Journey 4 - Support/Qualite - Investigation d'erreurs
Une extraction ne colle pas. Tu consultes les logs de pipeline (quelle source, quelle transformation, quel mapping) et tu corriges.
Tu peux rejouer une extraction en mode diagnostic.
Resultat : transparence, fiabilite, et reduction des erreurs silencieuses.

### Journey 5 - Reutilisation/Integration - Un autre projet branche la logique
Un futur testeur clone le workflow, charge ses propres sources (Jira/Squash) et ses templates.
Les modules sont interchangeables (reporting, checklist, anomalies, generation).
Resultat : transfert rapide, industrialisation multi-projets.

### Journey 6 - TNR - Phase critique GO/NO-GO
En fin de lot, la campagne TNR est copiee et structuree automatiquement.
Les anomalies TNR sont taguees, les preuves sont centralisees, et le PPT TNR est pre-rempli (GO/NO-GO, reserves, liens).
Resultat : decision finale plus fiable, moins de stress.

### Journey Requirements Summary
- Pipeline d'extraction Jira/Squash/Outlook/Excel/PPT -> pre-remplissage CR et PPT
- Checklist testabilite + scoring Go / Clarify / No-Go
- Generation de brouillons (cas de tests, anomalies) avec LLM local
- Tracabilite complete preuves <-> cas <-> anomalies <-> tickets
- Modules reutilisables + configuration par projet
- Logs/diagnostics et mode replay
- Gestion charge vs prevision + alertes
- Support TNR (campagne, tags, reporting GO/NO-GO)

## Domain-Specific Requirements

### Compliance & Regulatory
- Usage IA sensible : LLM local prioritaire, cloud possible mais jamais sans anonymisation automatique.
- Mecanisme d'anonymisation by design avant tout envoi vers un LLM cloud.
- Aucune donnee sensible (clients finaux) ne doit sortir du perimetre sans masquage.

### Regulatory Requirements (Assurance)
- Conformite aux exigences assurance par juridiction (a confirmer avec le client).
- Respect des politiques internes de conservation des preuves et livrables (duree, lieu, acces).
- Journalisation minimale pour audit interne (qui, quand, quoi) sans donnees sensibles.

### Risk Modeling (Assurance)
- Hors scope du MVP : aucune decision actuarielle ni modele de risque automatique.
- Si des indicateurs de risque sont ajoutes plus tard, ils doivent etre explicitement valides par le client.

### Fraud Detection
- Hors scope du MVP : aucune detection de fraude automatique.
- Possibilite future de signaler des anomalies suspectes via tags standardises (sans decision automatique).

### Reporting Compliance
- Conformite aux formats de reporting attendus par le client (PPT, CR, TNR).
- Traçabilite des rapports via stockage SharePoint et references Jira/Squash.

### Technical Constraints
- Jira : API token OK (hors VM).
- Squash : Basic auth au depart (token a demander).
- Least privilege : demarrage read-only, ecriture apres validation.
- Secrets dans un coffre/secret store, jamais en clair.
- HTTPS/TLS pour API ; Squash en HTTP probable (a faire evoluer).
- Stockage au repos sur SharePoint ; donnees locales minimisees, chiffrees, purges.
- Audit logs minimaux : horodatage, perimetre, version, statut, sans donnees sensibles.
- Restrictions IT a confirmer : scripts signes/macros, droits install, proxy, appels externes.
- Execution idealement sur environnement approuve (poste + M365), VM reservee aux applis testees.

### Integration Requirements
- Critiques : Jira + Squash (sources de verite), SharePoint (preuves/livrables), Excel & PowerPoint (CR + reporting).
- Souhaitees : Outlook + Teams pour diffusion et recuperation d'infos reunions.
- Jira : API OK, tokens OK.
- Squash : API OK en Basic, token non dispo immediatement.

### Risk Mitigations
- Squash token manquant : demarrage en Basic, plan d'evolution.
- Outlook/Teams API possiblement restreints : risque sur automatisation diffusion.
- Cloud LLM : risque privacy -> anonymisation automatique + option LLM local.

## Innovation & Novel Patterns

### Detected Innovation Areas
- Exploitation des preuves video pour detecter des ecarts avec les cas de test et proposer des ameliorations de redaction (priorite d'innovation).
- LLM local integre au workflow : CLI conversationnel + etapes automatiques dans le pipeline selon le contexte.
- Pipeline d'industrialisation reutilisable multi-projets (modules interchangeables).

### Market Context & Competitive Landscape
- Innovation interne : peu d'outillage TRA combine Jira/Squash/SharePoint et IA locale de maniere integree.
- Differenciation via l'usage des preuves video pour ameliorer la qualite des cas de test.

### Validation Approach
- Prouver la valeur sur un lot pilote : reduction des erreurs de reporting et gain de temps sans perte de qualite.
- Valider l'assistant LLM local sur un sous-ensemble de tickets et cas de test, avec relecture humaine obligatoire.
- Valider la detection d'ecarts via preuves video sur un echantillon limite avant generalisation.

### Risk Mitigation
- Si l'IA video est trop complexe : fallback sur pipeline d'amelioration manuelle assistee.
- Si l'IA locale est insuffisante : option cloud avec anonymisation automatique.
- Si resistance au changement : activer en mode opt-in et comparer performances avant/apres.

## CLI Tool Specific Requirements

### Project-Type Overview
- Outil CLI interactif et scriptable, destine a industrialiser les flux QA (reporting, checklist, generation de brouillons).
- Usage multi-projets avec profils et configuration reutilisable.

### Technical Architecture Considerations
- Modes d'execution : assistant interactif + batch/script.
- Sorties multi-formats : texte, JSON, CSV, Markdown, HTML, YAML.
- Separation stricte des secrets via secret store / env vars.

### Command Structure
- Commandes principales orientees workflow (triage, design, execute, report, tnr).
- Sous-commandes pour extraction, generation, publication, diagnostics.

### Output Formats
- Selection via flag (--format json|csv|md|html|yaml|text).
- Mode par defaut humain lisible + option machine-readable.

### Config Schema
- config.yaml par projet comme source principale.
- Overrides via env vars + flags CLI.
- Profiles pour basculer de contexte (--profile tra-dev).

### Scripting Support
- Codes de sortie normalises (succes/echec).
- Mode non-interactif (--yes, --dry-run, --no-llm).

### Shell Completion
- Auto-completion Bash/Zsh (commandes + flags + profils).

### Implementation Considerations
- Logging minimal (sans donnees sensibles).
- Support LLM local + fallback cloud (avec anonymisation).

## Functional Requirements

### Ingestion & Triage
- FR1: L'utilisateur peut importer des tickets depuis Jira pour une periode ou un perimetre donne.
- FR2: Le systeme peut appliquer une checklist de testabilite a un ticket.
- FR3: Le systeme peut produire un score Go/Clarify/No-Go par ticket.
- FR4: L'utilisateur peut marquer un ticket comme "clarification requise" et tracer la cause.
- FR5: L'utilisateur peut regrouper les tickets par lot/perimetre/cadence.

### Test Design Assistance
- FR6: Le systeme peut generer un brouillon de strategie de test a partir d'un ticket.
- FR7: Le systeme peut generer des propositions de cas de test (nominaux, variantes, limites).
- FR8: L'utilisateur peut editer et valider les brouillons avant publication.
- FR9: Le systeme peut integrer une checklist ESN dans la preparation des cas.

### Execution & Evidence
- FR10: L'utilisateur peut lier des preuves a des cas de test (references vers SharePoint).
- FR11: Le systeme peut centraliser les references de preuves par campagne.
- FR12: Le systeme peut associer les preuves aux tickets Jira concernes.
- FR13: L'utilisateur peut preparer une campagne TNR a partir d'un modele.

### Anomaly Management
- FR14: L'utilisateur peut generer un brouillon d'anomalie depuis un cas de test.
- FR15: Le systeme applique un template standardise d'anomalie (champs obligatoires).
- FR16: L'utilisateur peut lier l'anomalie a la campagne Squash et au ticket Jira.
- FR17: Le systeme peut taguer les anomalies TNR avec un label specifique.

### Reporting & Communication
- FR18: Le systeme peut generer un CR quotidien pre-rempli (blocs standard).
- FR19: L'utilisateur peut editer le CR et produire une version finale.
- FR20: Le systeme peut pre-generer le PPT hebdo avec les sections pertinentes.
- FR21: Le systeme peut generer un PPT TNR (GO/NO-GO, reserves, liens).
- FR22: Le systeme peut produire des exports multi-formats (txt/json/csv/md/html/yaml).

### Configuration & Reuse
- FR23: L'utilisateur peut configurer un projet via config.yaml.
- FR24: L'utilisateur peut definir des profils de configuration par contexte.
- FR25: Le systeme peut charger des templates (CR, PPT, anomalies).
- FR26: Le systeme peut fonctionner en mode interactif ou batch.
- FR27: Le systeme peut fournir une auto-completion shell.

### Compliance & Safety
- FR28: Le systeme peut anonymiser automatiquement les donnees avant envoi cloud.
- FR29: Le systeme peut fonctionner avec LLM local uniquement.
- FR30: Le systeme peut journaliser les executions sans donnees sensibles.
- FR31: Le systeme peut gerer les secrets via un secret store.

## Non-Functional Requirements

### Security & Privacy
- Donnees sensibles jamais envoyees vers un LLM cloud sans anonymisation automatique (controle sur 100% des envois sortants).
- Secrets stockes via secret store, jamais en clair (0 secret en clair dans le repo ou la configuration).
- Least privilege : read-only par defaut, ecriture validee explicitement (aucune ecriture sans activation explicite).
- Audit logs minimaux sans donnees sensibles (horodatage, perimetre, statut) et conservation 90 jours.
- Donnees temporaires locales purgees sous 24h (automatique).

### Performance
- Generation d'un CR quotidien pre-rempli < 2 minutes pour un perimetre standard.
- CR quotidien pret a envoyer < 5 minutes apres le run (incluant relecture/ajustements humains).
- Extraction Jira/Squash pour un lot < 5 minutes (a confirmer).
- CLI reactive : reponse interactive < 2 secondes pour commandes simples.

### Reliability & Recoverability
- En cas d'echec d'une integration, l'outil doit continuer en mode degrade (taux d'echec < 2% des executions).
- Reprise possible sur derniere execution (mode replay) sans perte de contexte, reprise < 5 minutes.

### Integration & Compatibility
- Support Jira/Squash via API avec auth token (Jira) et Basic (Squash), taux de succes >= 95% sur appels en volume standard.
- Sorties multi-formats strictement conformes aux templates existants (CR, PPT, anomalies), sans personnalisation visuelle.
- Fonctionnement hors VM, VM reservee aux applis testees.
- Fraicheur des donnees integrees < 15 minutes sur pipeline standard.

### Operational Constraints
- Execution possible sur poste M365, sans dependances cloud obligatoires.
- Respect des restrictions IT (scripts signes, proxy, droits d'installation).

## Open Questions

- Acces API Jira/Squash : quotas et limites formelles a confirmer.
- Authentification Squash via token : delai et conditions d'obtention.
- Outlook/Teams : acces API reel et cout/licence.
- Validation des templates (checklist, anomalies, reporting) : qui approuve et a quel rythme ?
- Seuils cibles pour les KPIs (tickets ready, erreurs de reporting, temps de reporting).

## Next Steps

- Confirmer contraintes et acces (APIs, exports, macros/scripts autorises, proxy).
- Valider checklist de testabilite et templates (anomalies, CR, PPT).
- Lancer un pilote sur 1-2 perimetres pour mesurer gain de temps et qualite.

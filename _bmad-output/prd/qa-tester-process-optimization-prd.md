PRD: QA Tester Process Optimization (TRA)
Version: 1.1
Owner: Business Analyst (Mary)
Status: Draft
Date: January 29, 2026

1) Overview
This PRD defines improvements to QA tester workflows in a TRA context, with a focus on quality and reliability. The current process suffers from ambiguous ticket inputs, manual data transposition across tools, and inconsistent reporting standards. The target outcome is a reliable, standardized, traceable process that reduces cognitive load and improves output consistency, without changing the toolset.

2) Background & Problem
The tester must manage multiple perimeters, perform estimation, create detailed test designs, execute tests, document evidence, and produce daily/weekly reports. Inputs (Jira tickets) often lack explicit acceptance criteria and require significant clarification. Reporting involves heavy manual extraction and re-entry into Excel and PPT, which causes errors and inconsistent outputs.

3) Goals & Success Criteria
Primary Goals
- Increase quality and reliability of test preparation and reporting.
- Standardize inputs and outputs to reduce variability.

Secondary Goals
- Reduce cognitive load and manual effort.
- Improve traceability across tests, evidence, anomalies, and reporting.

Success Criteria (KPIs)
- % tickets passing testability checklist before estimation
- Reduction in manual data entry points per report
- Template compliance rate for anomalies
- Reporting error rate (mismatch/outdated metrics)
- Reporting time per week (secondary KPI)

4) Scope
In Scope (Priority)
A) Preparation / Scoping
- Testability qualification
- Estimation standardization
- Strategy validation
- Reuse of existing test assets
- Planning aligned with QA/HO cadence

C) Reporting / CR
- Weekly PPT reporting (fixed slide structure)
- Daily CR reporting per perimeter
- Final test/recipe reports + Jira comments

Out of Scope (Phase 1)
- Test execution automation
- Tool replacement or migration
- Contractual changes to client processes

5) Users & Stakeholders
- Primary User: QA Tester (TRA)
- Stakeholders: CP (Project Manager), MOA, QA Lead / Pilot

6) Current Workflow Summary
Preparation / Scoping
- Receive Jira tickets; estimate complexity and effort
- Create estimates in Excel; align strategy with CP/MOA
- Review Squash for reusable test cases
- Align with QA and HO release cadence

Reporting
- Weekly PPT: fixed slides with metrics, anomalies, priorities
- Daily CR: Excel tables pasted into email
- Recipe reports + Jira comment summarizing campaigns and evidence

7) Pain Points
- Ambiguous ticket inputs -> unreliable test design
- Manual transposition across tools -> errors, inconsistencies
- Lack of standardized anomaly template
- Reporting burden -> cognitive overload
- Environment availability disrupts flow

8) Requirements
Functional Requirements
FR1 - Testability Checklist & Score
Provide a short checklist and numeric score for ticket readiness.
Output: Go / Clarify / No-Go.

FR2 - Formal Strategy Validation
Introduce a visible approval step (CP/MOA) before design.

FR3 - Single Source of Truth
Each metric used in reports must derive from one authoritative source.

FR4 - Automated Report Population
Daily/weekly CR blocks are auto-filled from Jira/Squash/Excel.

FR5 - Standardized Anomaly Template
Mandatory fields for clarity (context, repro, evidence, impact).

FR6 - Evidence Traceability
Consistent evidence naming + links in reports and tickets.

FR7 - Calendar-Aware Planning
Highlight risk against QA and HO cadence.

FR8 - Test Asset Reuse Discovery
Automatically surface relevant test cases in Squash.

Non-Functional Requirements
- Must work with existing tools (Jira, Squash, Excel, PPT, Outlook, Teams, SharePoint)
- Must respect fixed weekly PPT slide structure
- Daily CR must hide empty sections
- Minimal change management load for stakeholders

9) User Stories (High-Level)
1. As a tester, I want a testability checklist so I can decide whether a ticket is ready.
2. As a tester, I want formal strategy validation to avoid testing unclear scope.
3. As a tester, I want reports to auto-fill so I avoid manual retyping.
4. As a CP, I want consistent weekly reports so I can trust the data.
5. As a MOA, I want standardized anomaly reports for faster analysis.

10) Constraints & Dependencies
- Weekly report slide structure is fixed; only hidden if irrelevant.
- Daily CR must exclude empty blocks.
- API/export access to Jira/Squash may be required.
- Stakeholder validation workflows must be accepted.

11) Risks
- Limited API access could reduce automation potential.
- Change resistance to new templates or validation gates.
- Over-engineering might increase workload if not streamlined.

12) Open Questions
- Are Jira/Squash APIs accessible, or only exports?
- What KPI thresholds matter most to CP/MOA?
- Are macros/scripts acceptable in the client environment?
- Who approves new templates/standards?

13) Milestones (Suggested)
Phase 1 - Standardization (Quick Wins)
- Testability checklist + anomaly template
- Reporting source of truth definition

Phase 2 - Automation
- Auto-fill daily CR
- PPT report data injection

Phase 3 - Optimization
- Asset reuse discovery
- KPI dashboards and quality gates

14) Requirements Prioritization
P0 - Must Have (critical for quality and reliability)
- FR1 - Testability Checklist & Score
- FR2 - Formal Strategy Validation
- FR3 - Single Source of Truth
- FR5 - Standardized Anomaly Template
- FR6 - Evidence Traceability

P1 - Should Have (strong impact, but not blocking)
- FR4 - Automated Report Population
- FR7 - Calendar-Aware Planning
- FR8 - Test Asset Reuse Discovery

P2 - Nice to Have (optimization / future)
- KPI dashboards beyond core KPIs
- Advanced automation or AI-assisted ticket analysis
- Cross-perimeter predictive planning

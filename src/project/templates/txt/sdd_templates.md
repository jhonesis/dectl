# SDD Document Templates

Full content templates for each document in the Spec-Driven Development suite.
Documents marked **[CRITICAL ONLY]** are required for regulated, financial, health, or otherwise
high-stakes ("bank-grade") systems — see SKILL.md Step 0 for tier classification.

---

## constitution.md

```markdown
# Project Constitution
> *Governing principles for [Project Name]. This document is the supreme authority — all other documents must comply with it.*
> *Tier: [STANDARD | CRITICAL] | Version: 1.0 | Last updated: YYYY-MM-DD | Approved by: [name/role]*

## 1. Project Identity
- **Name**: 
- **Purpose**: One sentence describing what this project does and for whom.
- **Owners**: 
- **Regulatory scope** (CRITICAL only): List named frameworks in scope, e.g. PCI-DSS, SOX, GDPR, GLBA,
  PSD2, DORA, HIPAA, SOC 2 Type II, ISO 27001. If unsure, write "TBD — requires legal/compliance review"
  rather than guessing.

## 2. Core Principles
List 3–7 non-negotiable principles guiding all decisions.
Example:
- Simplicity over cleverness — prefer boring, readable code
- Security by default — never expose sensitive data without explicit intent
- Test everything — no feature ships without tests
- (CRITICAL) Least privilege by default — every credential, role, and service account gets the minimum
  access needed and nothing more
- (CRITICAL) No silent failures on money movement — every state-changing financial operation is
  idempotent, logged, and reconcilable

## 3. Technology Constraints
### Mandatory Stack
- Language: 
- Framework: 
- Database: 
- Hosting/Cloud: 

### Forbidden Technologies
- (List anything explicitly prohibited and why)

### Required Integrations
- (External services this project must connect to)

## 4. Coding Standards
- **Style guide**: (link or description)
- **Naming conventions**: 
- **Folder structure**: 
- **Comment policy**: 

## 5. Testing Strategy
- **Unit tests**: required for all business logic
- **Integration tests**: required for all API endpoints
- **E2E tests**: required for critical user flows
- **Coverage target**: _%
- (CRITICAL) **Security testing**: SAST/DAST in CI, dependency vulnerability scanning, mandatory
  penetration test before first production release and annually thereafter

## 6. Security Non-Negotiables
- All inputs validated and sanitized
- Authentication required on all private routes
- No secrets in source code (use env vars / secrets manager)
- (Add project-specific rules)
- (CRITICAL) All data classified as Confidential or Restricted is encrypted at rest (AES-256 or
  equivalent) and in transit (TLS 1.2+)
- (CRITICAL) All privileged actions (admin, financial approval, data export) require step-up
  authentication and are logged with actor, timestamp, and reason
- (CRITICAL) Segregation of duties: the person/service that initiates a financial transaction cannot
  be the sole approver of it
- (CRITICAL) Secrets and cryptographic keys are rotated on a defined schedule and never logged

## 7. Data Classification Policy [CRITICAL ONLY]
| Class | Definition | Examples | Handling rules |
|---|---|---|---|
| Public | No harm if disclosed | Marketing copy | No restrictions |
| Internal | Minor harm if disclosed | Internal docs | Access limited to employees |
| Confidential | Real harm to business or individual | Account balances, contact info | Encrypted, access-logged, need-to-know |
| Restricted | Severe harm; regulatory exposure | Card numbers, SSN, health records, credentials | Encrypted, tokenized/masked where possible, strict RBAC, audit-logged, retention-limited |

## 8. Change Management & Approval [CRITICAL ONLY]
- Who can approve production deployments:
- Who can approve changes to `constitution.md` or `compliance-matrix.md`:
- Required reviewers for changes touching money movement or Restricted data:
- Emergency change ("break-glass") process and post-hoc review requirement:

## 9. Incident Response Ownership [CRITICAL ONLY]
- On-call/escalation owner:
- Breach notification process pointer (link to runbook — do not draft legal notification language here):
- Maximum time to detect / time to contain targets:

## 10. Definition of Done
A task is complete when:
- [ ] **Code compiles** without errors (Build passes)
- [ ] **Verify step passes** — the feature runs and produces expected output
- [ ] **Tests pass** (unit + integration)
- [ ] **Constitution compliance review** — does the implementation respect all Core Principles?
- [ ] **No new linting errors**
- [ ] **PR reviewed and approved**
- [ ] **Spec/plan updated** if implementation deviated
- [ ] (CRITICAL) **Audit logging verified** for the change, where applicable
- [ ] (CRITICAL) **Compliance-matrix row updated** with evidence, where applicable

> **Note**: DoD is non-negotiable. No task is complete until ALL checklist items pass. The "Constitution compliance review" prevents gradual erosion of project principles.
```

---

## spec.md

```markdown
# Feature Specification: [Feature Name]
> *Technology-agnostic. Describes WHAT to build, not HOW.*
> *Tier: [STANDARD | CRITICAL] | Version: 1.0 | Status: Draft | Last updated: YYYY-MM-DD*

## Overview
Brief description of the feature and the problem it solves.

## Context & Motivation
Why is this feature needed? What user pain does it address?

## Users & Personas
- **[Persona 1]**: Description of this type of user and their goals
- **[Persona 2]**: ...
- (CRITICAL) Note any privileged personas explicitly (e.g., "Compliance Officer", "Fraud Analyst",
  "System Admin") since they typically carry different acceptance criteria and audit requirements.

## Data Sensitivity Overview [CRITICAL ONLY]
State up front what classes of data this feature touches (per constitution.md's classification):
- Data touched: [e.g., account balance (Confidential), card PAN (Restricted)]
- Regulatory tags potentially applicable: [e.g., PCI-DSS, GDPR] — flag for compliance-matrix.md, do not
  assert compliance here.

## Functional Requirements

### REQ-001: [Requirement Name]
**User Story**:
> As a [persona], I want [goal] so that [benefit/reason].

**Acceptance Criteria**:
- WHEN [condition] THEN the system SHALL [expected behavior]
- WHEN [condition] THEN the system SHALL [expected behavior]

**Data sensitivity** (CRITICAL only): [Public/Internal/Confidential/Restricted]
**Regulatory tag** (CRITICAL only): [e.g., PCI-DSS 3.4, or "none identified"]

**Notes**: Any clarifications or edge cases.

---

### REQ-002: [Requirement Name]
**User Story**:
> As a [persona], I want [goal] so that [benefit/reason].

**Acceptance Criteria**:
- WHEN [condition] THEN the system SHALL [expected behavior]

---

## Non-Functional Requirements
- **Performance**: (e.g., processes input in < 2s for typical workload)
- **Availability** (CRITICAL: state target, e.g., "99.95% uptime, RTO 15 min, RPO 1 min")
- **Accessibility**: (e.g., WCAG 2.1 AA compliance)
- **Security**: (e.g., all data encrypted at rest and in transit; see constitution.md §6)
- **Auditability** (CRITICAL ONLY): every state-changing action produces an immutable, timestamped,
  attributable log entry retained for [X years per regulatory requirement]
- **Localization**: (e.g., supports English and Spanish)

## Out of Scope
Explicitly list what this feature does NOT include:
- 
- 

## Open Questions
List unresolved questions that need answers before development starts:
- [ ] Question 1
- [ ] Question 2
- (CRITICAL) [ ] Any question about applicable regulation must be resolved with legal/compliance before
  `plan.md` is finalized — do not assume an answer.
```

---

## requirements.md (validation checklist)

```markdown
# Requirements Validation Checklist
> *Validates that spec.md is complete, unambiguous, and technology-agnostic before planning begins.*
> *Tier: [STANDARD | CRITICAL]*

## Completeness
- [ ] Every user story has at least one acceptance criterion
- [ ] All personas are defined
- [ ] Non-functional requirements are specified
- [ ] Out-of-scope is explicitly stated
- [ ] All open questions are resolved

## Clarity
- [ ] Each acceptance criterion uses SHALL (not "should" or "may")
- [ ] No ambiguous terms ("fast", "easy", "simple") without measurable definitions
- [ ] No implementation details or technology names in spec.md

## Consistency
- [ ] No contradictory requirements
- [ ] Requirements are numbered sequentially (REQ-001, REQ-002…)
- [ ] No duplicate requirements

## Task Readiness
- [ ] Each requirement maps to 2–3 atomic, individually verifiable tasks in plan.md
- [ ] Every task has Build: + Verify: + Gate: defined
- [ ] Each phase has Build Gate + Verify Gate

## Traceability
- [ ] Each requirement maps to a clear user need
- [ ] All personas mentioned in user stories are defined

## Security & Compliance [CRITICAL ONLY]
- [ ] Every requirement touching Confidential/Restricted data has a stated data sensitivity class
- [ ] Every requirement with a plausible regulatory tag is flagged for compliance-matrix.md
- [ ] Privileged personas and their distinct acceptance criteria are identified
- [ ] Auditability requirement is present for every state-changing operation
- [ ] No requirement assumes a compliance status without a "requires legal/compliance review" flag where relevant

## Verdict
- [ ] ✅ READY TO PLAN — All checks passed
- [ ] ⚠️ NEEDS REVISION — Items marked above must be resolved
```

---

## threat-model.md [CRITICAL ONLY]

```markdown
# Threat Model: [Feature/System Name]
> *STRIDE-based threat analysis. Produced after spec.md, before plan.md is finalized.*
> *Version: 1.0 | Status: Draft | Last updated: YYYY-MM-DD | Reviewed by: [security owner]*

## System Context
Brief description of what's being threat-modeled and its boundaries.

## Trust Boundary Diagram
```
[Diagram — Mermaid or ASCII — showing actors, systems, and where data crosses from a less-trusted
zone (e.g., public internet, third-party API) to a more-trusted zone (e.g., internal service, database).]

Example Mermaid:
graph LR
    User[External User] -->|HTTPS, untrusted| WAF
    WAF -->|trusted zone boundary| API[API Gateway]
    API -->|internal network| Core[Core Banking Service]
    Core -->|encrypted| DB[(Ledger DB)]
    Core -->|trust boundary: 3rd party| PaymentRail[External Payment Rail]
```

## Assets in Scope
List what needs protecting: account balances, credentials, PII, transaction integrity, availability, etc.

## Threats (STRIDE)

### T-001: [Threat name]
- **Category**: Spoofing | Tampering | Repudiation | Information Disclosure | Denial of Service | Elevation of Privilege
- **Actor**: [e.g., external attacker, malicious insider, compromised third-party vendor]
- **Entry point**: [e.g., public API endpoint, admin panel, third-party webhook]
- **Description**: What could go wrong and how.
- **Impact**: [Low | Medium | High | Critical] — justify in terms of financial loss, data exposure, or regulatory exposure
- **Mitigating control**: [Specific control — reference plan.md section or a tasks.md task ID once assigned]
- **Residual risk**: [Accepted / Mitigated / Requires further work] — if accepted, name the accepting owner

---

### T-002: [Threat name]
...

## Abuse Cases (insider / legitimate-access misuse)
- **AC-001**: [e.g., "An employee with legitimate account access modifies a customer's balance without
  an approval workflow."] — **Control**: [e.g., dual control / maker-checker required for balance adjustments]

## Unmitigated / Accepted Risks Summary
| Threat ID | Description | Why accepted | Accepted by | Review date |
|---|---|---|---|---|
| | | | | |
```

---

## research.md

```markdown
# Technical Research
> *Documents unknowns investigated during planning. Captures decisions and their rationale.*
> *Written alongside plan.md.*

## Research Questions

### RQ-001: [Question]
**Context**: Why this question matters.
**Options Evaluated**:
| Option | Pros | Cons |
|--------|------|------|
| A | ... | ... |
| B | ... | ... |

**Decision**: Option [X]
**Rationale**: Why this option was chosen.

---

### RQ-002: [Question]
...

## External Dependencies Investigated
| Dependency | Version | License | Risk Level | Notes |
|-----------|---------|---------|-----------|-------|
| | | | | |

(CRITICAL) For any dependency handling money movement, cryptography, or Restricted data, note whether it
is independently audited/certified (e.g., a payment processor's PCI-DSS attestation) and link the evidence.

## Proof of Concepts
List any spikes or PoCs conducted:
- **PoC-001**: [What was tested, what was learned]
```

---

## plan.md

```markdown
# Technical Implementation Plan: [Feature/Project Name]
> *Technology-specific. Describes HOW to build what spec.md defines.*
> *Tier: [STANDARD | CRITICAL] | Version: 1.0 | Status: Draft | Last updated: YYYY-MM-DD*

## References
- Implements: [link to spec.md]
- Constitution: [link to constitution.md]
- Threat model (CRITICAL): [link to threat-model.md]
- Research: [link to research.md]

## Tech Stack
| Layer | Technology | Version | Justification |
|-------|-----------|---------|---------------|
| Language | | | |
| Framework | | | |
| Database | | | |
| Auth | | | |
| Hosting | | | |

## Architecture Overview

### API Projects
```
[ASCII or Mermaid diagram showing client → API → DB flow]

Example Mermaid:
graph TD
    Client --> API
    API --> DB
    API --> Cache
```

### CLI / Library Projects (no API)
```
[ASCII or Mermaid diagram showing module/function call flow]

Example Mermaid for CLI:
graph LR
    CLI[CLI Parser] --> Core[Core Logic]
    Core --> Output[Output Formatter]
    Core --> FileIO[File I/O]
```

## Data Flow
Describe how data moves through the system for the main use cases.

## Security Architecture [CRITICAL ONLY]
- **Authentication**: [e.g., OAuth2/OIDC + MFA for all human users; mTLS for service-to-service]
- **Authorization model**: [e.g., RBAC/ABAC — define roles and what each can do; note maker-checker
  workflows for money movement]
- **Encryption**: at rest [algorithm/method], in transit [TLS version], key management [e.g., KMS/HSM,
  rotation schedule]
- **Secrets management**: [e.g., vault service, no secrets in code/CI logs]
- **Audit logging**: what gets logged, where it's stored, retention period, tamper-evidence approach
  (e.g., append-only, hash-chained, or shipped to a separate log store the app can't modify)
- **Network segmentation**: how the trust zones from threat-model.md map to actual network/VPC boundaries

## Availability & Resilience [CRITICAL ONLY]
- **RTO** (Recovery Time Objective): [e.g., 15 minutes]
- **RPO** (Recovery Point Objective): [e.g., 1 minute / zero data loss for committed transactions]
- **Failover strategy**: [e.g., active-active across availability zones]
- **Idempotency strategy for financial operations**: [e.g., idempotency keys on all POST/PATCH state-
  changing endpoints, reconciliation job to detect drift]

## Implementation Phases

### Phase 1: [Name] (estimated: X days)

**Goal**:

**Build Gate**: `[build command]` — must pass with 0 errors
  - Examples: `cargo build` (Rust), `go build` (Go), `npm run build` (JS/TS), `python -m build` (Python)
**Verify Gate**: `[test command]` or manual smoke test confirms phase goal is met
**Rule**: Each task in this phase MUST compile and verify BEFORE the next task begins

**Deliverables**:
- 

**Tasks**: T001–T00N
**Requirements covered**: REQ-001, REQ-002

### Phase 2: [Name] (estimated: X days)

**Goal**:

**Build Gate**: `[build command]` — must pass with 0 errors
**Verify Gate**: `[test command]` or manual smoke test confirms phase goal is met
**Rule**: Each task in this phase MUST compile and verify BEFORE the next task begins

**Deliverables**:
- 

**Tasks**: T00N–T00M
**Requirements covered**: REQ-003, REQ-004

## Risks & Mitigations
| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| | | | |

(CRITICAL) For risks with Medium+ impact touching money or Restricted data, also add a row to `risk-register.md`.

## Dependencies & Prerequisites
- [ ] Dependency 1 (reason needed)
- [ ] Dependency 2

## Testing Approach
- Unit tests: [strategy]
- Integration tests: [strategy]
- E2E tests: [strategy]
- **Per-task verification**: after each task, compile + verify before proceeding to next
- (CRITICAL) Security tests: [SAST/DAST tools, pen-test scope and timing, dependency scanning]
- (CRITICAL) Reconciliation/consistency tests: [how you'll verify the ledger/system of record never
  drifts silently — e.g., nightly balance reconciliation job]
```

---

## data-model.md

```markdown
# Data Model
> *Defines all entities, their attributes, and relationships.*
> *Tier: [STANDARD | CRITICAL]*

## Entities

### [EntityName]
| Field | Type | Required | Classification (CRITICAL) | Description |
|-------|------|----------|---------------------------|-------------|
| id | UUID | ✅ | Internal | Primary key |
| created_at | timestamp | ✅ | Internal | Auto-set on creation |
| ... | | | | |

(CRITICAL) Classification values come from constitution.md §7 (Public/Internal/Confidential/Restricted).
Any Restricted field must state: masking/tokenization approach, and retention period.

### [EntityName2]
...

## Relationships
Describe entity relationships:
- [Entity1] has many [Entity2] (via foreign key `entity1_id`)
- [Entity2] belongs to [Entity1]

## Entity Relationship Diagram
```
[ERD in ASCII or Mermaid format]
```

## Data Retention & Deletion Policy [CRITICAL ONLY]
| Entity/Field | Retention period | Deletion/anonymization method | Regulatory driver |
|---|---|---|---|
| | | | |

## Audit Trail Requirements [CRITICAL ONLY]
For each entity that represents money, balances, or Restricted data, specify:
- What changes must be logged (every field change vs. specific fields)
- Whether history is kept via an append-only ledger/event table vs. mutate-in-place with a separate audit log
- Who/what can be identified as the actor for every change (no anonymous writes to financial state)

## Migration Notes
If modifying an existing schema, describe migrations needed. (CRITICAL) Include a rollback plan for any
migration touching a table with financial or Restricted data.
```

---

## interface-contracts/{api,cli,lib}.md

```markdown
# Interface Contracts

## Interface Type
Select the type that matches this project:

- **`type: api`** — REST/GraphQL endpoints. Document with OpenAPI-style endpoint definitions.
- **`type: cli`** — CLI commands, flags, and arguments. Document with usage + flag tables.
- **`type: library`** — Public API functions/methods. Document with function signatures + examples.

---

### API Template

## Base URL
`/api/v1`

## Authentication
All endpoints require `Authorization: Bearer <token>` unless marked 🔓.
(CRITICAL) State token lifetime, refresh strategy, and step-up auth requirement for privileged endpoints.

## Standard Error Taxonomy [CRITICAL ONLY]
Define a consistent error envelope so partial failures are never ambiguous:
```json
{
  "error_code": "INSUFFICIENT_FUNDS",
  "message": "Human-readable, non-sensitive message",
  "trace_id": "uuid-for-support-and-audit-correlation"
}
```

---

## Endpoints

### POST /[resource]
**Description**: [What this does]
**Requirements**: REQ-001
**Idempotency** (CRITICAL, required on all state-changing endpoints): Requires `Idempotency-Key` header;
duplicate keys within [window] return the original result without reprocessing.
**Audit logging** (CRITICAL): Logs actor, timestamp, request parameters (redacting Restricted fields), and outcome.
**Rate limiting** (CRITICAL): [e.g., 10 req/min per account, 429 on breach]

**Request Body**:
```json
{
  "field1": "string",
  "field2": "number"
}
```

**Response 201**:
```json
{
  "id": "uuid",
  "field1": "string"
}
```

**Error Responses**:
- `400 Bad Request`: Invalid input
- `401 Unauthorized`: Missing/invalid token
- `403 Forbidden`: Authenticated but not authorized for this action (CRITICAL: distinguish from 401 explicitly)
- `409 Conflict`: Resource already exists / idempotency key conflict with different payload
- `422 Unprocessable Entity`: [e.g., INSUFFICIENT_FUNDS, LIMIT_EXCEEDED — CRITICAL: enumerate every
  domain-specific failure a financial operation can have; never let money-movement failures fall through
  to a generic 500]
```

---

## compliance-matrix.md [CRITICAL ONLY]

```markdown
# Compliance Traceability Matrix
> *Maps regulatory/control requirements to spec requirements, implementation, and evidence.*
> *This document structures traceability — it does not itself certify compliance. Formal certification
> requires review by qualified legal/compliance/audit personnel.*
> *Version: 1.0 | Last updated: YYYY-MM-DD | Reviewed by: [compliance owner]*

## Frameworks in Scope
List frameworks named in constitution.md §1 (e.g., PCI-DSS v4.0, GDPR, SOC 2 Type II). If a framework's
applicability is uncertain, mark it "Under legal review" rather than including or excluding it by assumption.

## Traceability Table
| Control ID | Framework | Control description | Spec requirement | Implementation task | Evidence/test | Status |
|---|---|---|---|---|---|---|
| | | | REQ-00X | T0XX | [e.g., automated test name, audit log sample] | Not started / In progress / Implemented / Verified |

## Gaps & Remediation Plan
| Gap | Risk if unaddressed | Remediation | Target date | Owner |
|---|---|---|---|---|
| | | | | |

## Sign-off
| Role | Name | Date | Notes |
|---|---|---|---|
| Compliance owner | | | |
| Security owner | | | |
| Engineering owner | | | |
```

---

## risk-register.md [CRITICAL ONLY]

```markdown
# Risk Register
> *Living document — revisited at every phase gate, not written once and forgotten.*
> *Version: 1.0 | Last updated: YYYY-MM-DD*

## Risk Categories
Security | Operational | Financial | Third-Party/Vendor | Regulatory

## Register

### RISK-001: [Short name]
- **Category**: [one of the above]
- **Description**: What could happen and why.
- **Likelihood**: Low / Medium / High
- **Impact**: Low / Medium / High / Critical
- **Current mitigation**: What's already in place or planned (reference plan.md / tasks.md)
- **Residual risk after mitigation**: Low / Medium / High
- **Owner**: [accountable person/role]
- **Review date**: [next scheduled review]
- **Status**: Open / Mitigated / Accepted / Closed

---

### RISK-002: ...

## Summary by Category
| Category | Open | Mitigated | Accepted | Highest residual severity |
|---|---|---|---|---|
| Security | | | | |
| Operational | | | | |
| Financial | | | | |
| Third-Party/Vendor | | | | |
| Regulatory | | | | |
```

---

## access-control-matrix.md [CRITICAL ONLY]

```markdown
# Access Control Matrix
> *Every role, what it can access, and how privileged actions are approved.*
> *Revision History: | Version | Date | Author | Change summary | ... |*
> *Version: 1.0 | Last updated: YYYY-MM-DD | Reviewed by: [security/compliance owner]*

## Roles Overview
| Role | Description | Privileged? |
|---|---|---|
| Customer | End user, own-account access only | No |
| Support Agent | Read-only access to customer accounts for support tickets | Yes (read) |
| Fraud Analyst | Can approve/reject held transactions | Yes |
| Financial Systems Admin | Can adjust balances, reverse transactions | Yes |
| Engineer (prod) | Deploy access, no direct data access | Yes |

## Permission Matrix
| Role | Data class accessed | Actions allowed | Approval workflow required | Justification |
|---|---|---|---|---|
| Customer | Own Confidential data | Read, initiate own transfers | N/A (self-service) | Core product function |
| Support Agent | Customer Confidential data (masked Restricted fields) | Read only | N/A, but access is logged | Support resolution |
| Fraud Analyst | Confidential + flagged transaction details | Approve/reject held transfers | Cannot approve own transactions | Fraud control |
| Financial Systems Admin | Confidential + Restricted | Manual balance adjustment | Maker-checker: second admin must approve | High-risk irreversible action |
| Engineer (prod) | None directly; deploy pipeline only | Deploy code | Peer review + on-call approval | Segregation of duties |

## Access Review & Recertification
- **Cadence**: [e.g., quarterly]
- **Owner**: [role responsible for running recertification]
- **Process**: [e.g., automated report of all privileged-role holders sent to their manager for confirm/revoke]
- **Deprovisioning SLA**: access revoked within [X hours] of role change or termination

## Break-Glass / Emergency Access
- **When permitted**: [e.g., production incident requiring direct DB access]
- **Who can invoke**: [named roles]
- **Post-hoc requirement**: [e.g., logged automatically, reviewed within 24h by security owner]
```

---

## disaster-recovery-plan.md [CRITICAL ONLY]

```markdown
# Disaster Recovery & Business Continuity Plan
> *Revision History: | Version | Date | Author | Change summary | ... |*
> *Version: 1.0 | Last updated: YYYY-MM-DD | Reviewed by: [engineering + compliance owners]*

## Scope
Systems/components covered by this plan: [list].

## Recovery Objectives (per component)
| Component | RTO | RPO | Justification |
|---|---|---|---|
| Ledger / core balances | 15 min | 0 (zero data loss for committed transactions) | Financial integrity non-negotiable |
| Reporting/analytics dashboard | 4 hours | 1 hour | Non-critical for real-time operation |

## Backup Strategy
- **What's backed up**: [databases, config, secrets metadata (not secret values)]
- **Frequency**: [e.g., continuous WAL streaming + daily full snapshot]
- **Storage location**: [e.g., separate region/account from production]
- **Encryption**: backups encrypted at rest with the same or stronger standard as production

## Failover Procedure
1. Detection: [how failure is detected — monitoring alert, health check]
2. Decision: [who declares a disaster and authorizes failover]
3. Execution: [step-by-step, or link to runbook — named roles execute, not "the team"]
4. Validation: [how you confirm the failover system is serving correct, consistent data]
5. Failback: [process to return to primary once resolved]

## Communication Plan
| Audience | Who notifies | Within what timeframe | Channel |
|---|---|---|---|
| Internal leadership | [role] | [e.g., 30 min] | [e.g., incident Slack channel] |
| Customers | [role] | [e.g., 2 hours if service-impacting] | [e.g., status page] |
| Regulators (if applicable) | [role — typically Compliance/Legal, not engineering] | [per regulatory requirement — confirm with Legal] | [formal notification channel] |

## Testing & Drills
- **Last tested**: [date]
- **Test type**: [tabletop exercise / partial failover / full live failover]
- **Result & follow-ups**: [what worked, what didn't, remediation tasks opened]
- **Next scheduled test**: [date] — an untested plan should appear as an open item in risk-register.md
```

---

## vendor-risk-assessment.md [CRITICAL ONLY]

```markdown
# Vendor / Third-Party Risk Assessment
> *Revision History: | Version | Date | Author | Change summary | ... |*
> *Version: 1.0 | Last updated: YYYY-MM-DD | Reviewed by: [security/compliance owner]*

## Vendor Inventory
List every third party in the architecture diagram (plan.md) that touches Confidential/Restricted data,
money movement, or is a single point of failure.

### Vendor: [Name, e.g., "Fraud Scoring API Provider"]
- **What they access**: [data/scope]
- **Why needed**: [business justification]
- **Compliance attestations on file**: [e.g., SOC 2 Type II report dated X, or "requested, not yet received"]
- **Contractual data-protection terms**: [e.g., DPA signed, breach-notification clause, data residency commitment]
- **Criticality**: [Low/Medium/High/Critical — if this vendor fails, what breaks]
- **Fallback if unavailable**: [e.g., fail-closed per threat-model.md T-003]
- **Exit strategy / data portability**: [how you'd migrate off this vendor and what happens to data on exit]
- **Last reviewed**: [date] | **Next review**: [date]

---

### Vendor: [Name 2]
...

## Vendor Risk Summary
| Vendor | Criticality | Attestation on file? | Open concerns |
|---|---|---|---|
| | | | |
```

---

## tasks.md

```markdown
# Implementation Tasks
> *Atomic, ordered, trackable tasks derived from plan.md.*
> *Each task = independently implementable + testable + reviewable as single PR.*
> *CRITICAL: After EACH task, compile + verify before moving to the next task.*
> *Tier: [STANDARD | CRITICAL]*

## Legend
- `[Txxx]` = Task ID
- `[P]` = Can run in parallel with other [P] tasks in same phase
- `[SEC]` / `[COMPLIANCE]` = Security or compliance task (CRITICAL tier) — release-blocking, not optional
- `S/M/L` = Estimated complexity (Small/Medium/Large)
- `(REQ-xxx)` = Traceability to spec requirement
- **Build**: command to compile the project after this task
- **Verify**: command or check to confirm the task works
- **Gate**: task must pass Build + Verify before the next task begins

---

> **Note**: Tasks in this template are generic placeholders. Replace with your project's actual first phase tasks.

## Phase 1: Foundation

**Build Gate**: `[build command]` — must pass with 0 errors before phase is complete

- [ ] [T001] [Setup] Initialize project structure and install dependencies — S
  **Build**: `[build command]` passes without errors
  **Verify**: project runs without errors (e.g., `--help`, `/health`, or import works)
  **Gate**: must pass before T002

- [ ] [T002] [Setup] Configure build system and linting — S
  **Build**: `[build command]` passes without errors
  **Verify**: linter passes with 0 errors
  **Gate**: must pass before T003

- [ ] [T003] [Core] Implement core data structures/types — S (REQ-001)
  **Build**: `[build command]` passes without errors
  **Verify**: unit tests for core types pass
  **Gate**: must pass before T004

- [ ] [T004][P] [Core] Implement primary business logic function/module — M (REQ-001)
  **Build**: `[build command]` passes without errors
  **Verify**: `[test command]` passes for the module
  **Gate**: must pass before T005

- [ ] [T005][P] [Core] Implement secondary business logic — M (REQ-002)
  **Build**: `[build command]` passes without errors
  **Verify**: `[test command]` passes for the module
  **Gate**: must pass before T006

- [ ] [T006] [Core] Write unit tests for core modules — M (REQ-001, REQ-002)
  **Build**: `[build command]` passes without errors
  **Verify**: `[test command]` passes all tests with >= 80% coverage
  **Gate**: must pass before T007

## Phase 2: [Name]

**Build Gate**: `[build command]` — must pass with 0 errors before phase is complete

- [ ] [T007] [Feature] ... — M (REQ-003)
  **Build**: `[build command]` passes without errors
  **Verify**: [specific verify step for this task]
  **Gate**: must pass before T008

## Phase N: Security, Compliance & Release Readiness [CRITICAL ONLY]

- [ ] [T0XX][SEC] Implement audit logging for all state-changing endpoints per interface-contracts/api.md — M
- [ ] [T0XX][SEC] Configure encryption at rest for Restricted-classified fields — M
- [ ] [T0XX][SEC] Run SAST/DAST scans and remediate findings above [severity threshold] — L
- [ ] [T0XX][SEC] Conduct penetration test and remediate critical/high findings — L
- [ ] [T0XX][COMPLIANCE] Complete compliance-matrix.md traceability and obtain sign-off — M
- [ ] [T0XX][COMPLIANCE] Verify reconciliation job catches injected test discrepancies — M
- [ ] [T0XX][SEC] Implement roles/permissions per access-control-matrix.md, including maker-checker on privileged actions — L
- [ ] [T0XX][SEC] Run first disaster-recovery drill per disaster-recovery-plan.md and log results — M
- [ ] [T0XX][COMPLIANCE] Confirm attestations on file for every vendor in vendor-risk-assessment.md — M
- [ ] [T0XX] Release go/no-go review with security, compliance, and engineering owners — S

---

## Progress Tracking
- Total tasks: X
- Completed: 0
- In progress: 0
- Blocked: 0
```

---

## CLAUDE.md (Agent Context File)

```markdown
# Agent Context: [Project Name]

> *Read this file first in every session. It orients you to the project and its SDD artifacts.*
> *Tier: [STANDARD | CRITICAL]*

## Project Summary
One paragraph describing what this project does.

## Current Status
- Phase: [e.g., Planning / Phase 1 Implementation / Phase 2]
- Last updated: YYYY-MM-DD
- Active branch:

## SDD Artifact Index
| Document | Path | Status |
|----------|------|--------|
| Constitution | `specs/constitution.md` | ✅ Final |
| Spec | `specs/spec.md` | ✅ Approved |
| Requirements checklist | `specs/requirements.md` | ✅ Passed |
| Threat model (CRITICAL) | `specs/threat-model.md` | ✅ Approved |
| Technical plan | `specs/plan.md` | ✅ Approved |
| Data model | `specs/data-model.md` | ✅ Approved |
| API contracts | `specs/interface-contracts/api.md` | ✅ Approved |
| Compliance matrix (CRITICAL) | `specs/compliance-matrix.md` | 🔄 In progress |
| Risk register (CRITICAL) | `specs/risk-register.md` | 🔄 In progress |
| Access control matrix (CRITICAL) | `specs/access-control-matrix.md` | 🔄 In progress |
| Disaster recovery plan (CRITICAL) | `specs/disaster-recovery-plan.md` | 🔄 In progress |
| Vendor risk assessment (CRITICAL) | `specs/vendor-risk-assessment.md` | 🔄 In progress |
| Tasks | `specs/tasks.md` | 🔄 In progress |

## Key Rules (from constitution.md)
- [Extract the 3–5 most critical rules the AI agent must follow]
- (CRITICAL) Never mark a `[SEC]` or `[COMPLIANCE]` task complete without the evidence required in
  compliance-matrix.md

## Tech Stack (quick reference)
- Backend:
- Frontend:
- Database:
- Auth:

## How to Work with This Project
1. Always check `specs/tasks.md` for the next pending task
2. When implementing a task, reference its requirement in `spec.md`
3. Mark tasks `[x]` when complete
4. If implementation deviates from spec, update spec.md FIRST, then continue
5. Never write code for tasks not in `tasks.md` without user approval
6. (CRITICAL) Never implement a task touching money movement or Restricted data without checking
   `threat-model.md` and `compliance-matrix.md` for applicable controls first
7. (CRITICAL) Flag — don't silently resolve — any conflict discovered between `plan.md` and
   `compliance-matrix.md`; these require a human decision
```

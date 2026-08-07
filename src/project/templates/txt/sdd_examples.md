# SDD Worked Examples

Two complete worked examples:
1. **TaskFlow** (STANDARD tier) — a simple team task management web app, showing the base 9-document flow.
2. **LedgerPay Funds Transfer** (CRITICAL tier) — a bank-grade money-movement feature, showing every
   additional document (threat-model, compliance-matrix, risk-register) and how CRITICAL-only sections
   of the standard documents get filled in.
Use Example 1 as the template for ordinary product/feature work. Use Example 2 as the template whenever
the project touches money, regulated data, or was explicitly described as needing to be "bank-grade,"
"audit-ready," or "production-critical."

### Adaptation Guidelines
- Adapt **structure**, not content. Every project needs a constitution with Core Principles, a spec with REQ-00X format, a plan with architecture, etc.
- Customize **Build/Verify/Gate** commands in tasks.md to match your project's toolchain.
- Each task is atomic + individually verifiable; each phase keeps its Build Gate + Verify Gate.

---

# EXAMPLE 1 (STANDARD): "TaskFlow" — A team task management web app

---

## constitution.md (example)

```markdown
# Project Constitution — TaskFlow

## 1. Project Identity
- **Name**: TaskFlow
- **Purpose**: A web app that lets small teams create, assign, and track tasks collaboratively.
- **Owners**: Product team

## 2. Core Principles
- Simplicity first — no feature ships if it can't be explained in one sentence
- Offline-tolerant — critical read operations work without network
- Mobile-first responsive design

## 3. Technology Constraints
### Mandatory Stack
- Language: TypeScript (strict mode)
- Framework: Next.js 14 (App Router)
- Database: PostgreSQL via Supabase
- Auth: Supabase Auth (OAuth + magic link)

### Forbidden Technologies
- No class components in React (hooks only)
- No `any` type in TypeScript
- No direct database queries from client components

## 4. Testing Strategy
- Unit tests: Vitest for all utility functions and hooks
- Integration tests: Playwright for critical user flows
- Coverage target: 80%

## 5. Definition of Done
- [ ] Feature matches spec acceptance criteria
- [ ] Build passes without errors
- [ ] Verify step passes — the feature runs and produces expected output
- [ ] Unit tests written and passing
- [ ] No TypeScript errors
- [ ] Reviewed by one team member
```

---

## spec.md (example)

```markdown
# Feature Specification: Task Management
> Technology-agnostic | Version: 1.0 | Status: Approved

## Overview
Allow team members to create, assign, update, and complete tasks within a shared workspace.

## Users & Personas
- **Team Member**: Creates and works on tasks
- **Team Manager**: Assigns tasks and monitors progress

## Functional Requirements

### REQ-001: Task Creation
**User Story**:
> As a Team Member, I want to create a task with a title and description so that I can document work that needs to be done.

**Acceptance Criteria**:
- WHEN a user submits a task form with a valid title THEN the system SHALL create the task and display it in the task list
- WHEN a user submits a task form without a title THEN the system SHALL display an error and not create the task
- WHEN a task is created THEN the system SHALL automatically set its status to "pending" and record the creator

### REQ-002: Task Assignment
**User Story**:
> As a Team Manager, I want to assign tasks to team members so that responsibility is clear.

**Acceptance Criteria**:
- WHEN a manager selects a team member from the assignment dropdown THEN the system SHALL update the task assignee
- WHEN a task is assigned THEN the system SHALL notify the assigned member

### REQ-003: Task Status Updates
**User Story**:
> As a Team Member, I want to update the status of my tasks so that the team can track progress.

**Acceptance Criteria**:
- WHEN a team member changes a task status THEN the system SHALL update it to one of: pending, in-progress, done
- WHEN a task is marked "done" THEN the system SHALL record the completion timestamp

## Non-Functional Requirements
- **Performance**: Task list loads in < 1.5s for up to 200 tasks
- **Accessibility**: WCAG 2.1 AA

## Out of Scope
- Comments on tasks (v2)
- File attachments (v2)
- Time tracking (v2)
```

---

## plan.md (example)

```markdown
# Technical Plan — Task Management Feature
> Implements: spec.md | Stack defined in constitution.md

## Tech Stack
| Layer | Technology | Justification |
|-------|-----------|---------------|
| Frontend | Next.js 14 + React | SSR + App Router, team standard |
| Styling | Tailwind CSS | Utility-first, fast iteration |
| Backend | Next.js API Routes | Co-located with frontend |
| Database | PostgreSQL (Supabase) | Relational, real-time subscriptions |
| Auth | Supabase Auth | Handles OAuth + session |
| ORM | Prisma | Type-safe queries |

## Architecture
```
Browser → Next.js App Router
              ├── Server Components (data fetching)
              ├── API Routes (mutations)
              └── Supabase Client (real-time)
                       └── PostgreSQL
```

## Implementation Phases

### Phase 1: Foundation (3 days)

**Goal**: Project scaffolded, database and auth wired.
**Build Gate**: `npm run build` — must pass with 0 errors
**Verify Gate**: `npm run dev` serves the app and `/docs` or home loads
**Rule**: Each task in this phase MUST compile and verify BEFORE the next task begins

- Project setup, database schema, auth
- Requirements: prerequisites for REQ-001, REQ-002, REQ-003

### Phase 2: Core Task Features (4 days)

**Goal**: Create, list, assign, update tasks end-to-end.
**Build Gate**: `npm run build` — must pass with 0 errors
**Verify Gate**: `npx playwright test` passes
**Rule**: Each task in this phase MUST compile and verify BEFORE the next task begins

- Requirements: REQ-001, REQ-002, REQ-003

### Phase 3: Polish & Testing (2 days)

**Goal**: Loading states, error handling, accessibility, E2E coverage.
**Build Gate**: `npm run build` — must pass with 0 errors
**Verify Gate**: `npx playwright test` passes; `npm run lint` clean
**Rule**: Each task in this phase MUST compile and verify BEFORE the next task begins

## Risks
| Risk | Mitigation |
|------|-----------|
| Supabase real-time latency | Optimistic UI updates |
| Prisma migration complexity | Use shadow DB in staging |
```

---

## tasks.md (example)

```markdown
# Implementation Tasks — TaskFlow

## Phase 1: Foundation

**Build Gate**: `npm run build` — must pass with 0 errors before phase is complete

- [ ] [T001] Initialize Next.js 14 project with TypeScript strict mode — S
  **Build**: `npm run build` passes without errors
  **Verify**: `npm run dev` starts and renders the home page
  **Gate**: must pass before T002

- [ ] [T002] Configure Supabase project and environment variables — S
  **Build**: `npm run build` passes without errors
  **Verify**: `.env.local` loads; Supabase client connects
  **Gate**: must pass before T003

- [ ] [T003] Set up Prisma with PostgreSQL connection — S
  **Build**: `npx prisma generate` succeeds
  **Verify**: `npx prisma db push` creates the schema
  **Gate**: must pass before T004

- [ ] [T004][P] Create database migration: users table — S (REQ-001)
  **Build**: `npm run build` passes without errors
  **Verify**: `npx prisma migrate dev` applies cleanly
  **Gate**: must pass before T005

- [ ] [T005][P] Create database migration: tasks table (id, title, description, status, assignee_id, creator_id, created_at, completed_at) — S (REQ-001, REQ-002, REQ-003)
  **Build**: `npm run build` passes without errors
  **Verify**: migration applies; table has all required columns
  **Gate**: must pass before T006

- [ ] [T006] Configure Supabase Auth with magic link — M
  **Build**: `npm run build` passes without errors
  **Verify**: sign-in flow completes in staging
  **Gate**: must pass before T007

- [ ] [T007] Create auth middleware for protected routes — M
  **Build**: `npm run build` passes without errors
  **Verify**: unauthenticated access redirects to login
  **Gate**: must pass before T008

## Phase 2: Core Features

**Build Gate**: `npm run build` — must pass with 0 errors before phase is complete

- [ ] [T008][P] Build POST /api/tasks endpoint (create task) — M (REQ-001)
  **Build**: `npm run build` passes without errors
  **Verify**: POST returns 201 and task persists
  **Gate**: must pass before T009

- [ ] [T009][P] Build GET /api/tasks endpoint (list tasks) — M (REQ-001)
  **Build**: `npm run build` passes without errors
  **Verify**: GET returns task list ordered by creation
  **Gate**: must pass before T010

- [ ] [T010] Build PATCH /api/tasks/[id] endpoint (update status + assignee) — M (REQ-002, REQ-003)
  **Build**: `npm run build` passes without errors
  **Verify**: PATCH updates status/assignee; validation rejects bad values
  **Gate**: must pass before T011

- [ ] [T011] Build TaskForm component with validation — M (REQ-001)
  **Build**: `npm run build` passes without errors
  **Verify**: empty-title submission shows error, no task created
  **Gate**: must pass before T012

- [ ] [T012] Build TaskList component with status badges — M (REQ-001)
  **Build**: `npm run build` passes without errors
  **Verify**: tasks render with correct status badges
  **Gate**: must pass before T013

- [ ] [T013] Build AssigneeDropdown component — S (REQ-002)
  **Build**: `npm run build` passes without errors
  **Verify**: assigning a member updates the task and notifies
  **Gate**: must pass before T014

- [ ] [T014] Implement status change handler — S (REQ-003)
  **Build**: `npm run build` passes without errors
  **Verify**: status transitions restricted to pending/in-progress/done; done records timestamp
  **Gate**: must pass before T015

- [ ] [T015] Write unit tests for task service functions — M (REQ-001, REQ-002, REQ-003)
  **Build**: `npm run build` passes without errors
  **Verify**: `npm test` passes for service functions
  **Gate**: must pass before T016

## Phase 3: Polish & Testing

**Build Gate**: `npm run build` — must pass with 0 errors before phase is complete

- [ ] [T016] Add loading skeletons for task list — S
  **Build**: `npm run build` passes without errors
  **Verify**: skeletons render during fetch
  **Gate**: must pass before T017

- [ ] [T017] Add error boundary and toast notifications — S
  **Build**: `npm run build` passes without errors
  **Verify**: API errors surface as toasts, app doesn't crash
  **Gate**: must pass before T018

- [ ] [T018] Write Playwright E2E test: create task flow — M
  **Build**: `npm run build` passes without errors
  **Verify**: `npx playwright test create-task` passes
  **Gate**: must pass before T019

- [ ] [T019] Write Playwright E2E test: assign and complete task flow — M
  **Build**: `npm run build` passes without errors
  **Verify**: `npx playwright test assign-complete` passes
  **Gate**: must pass before T020

- [ ] [T020] Accessibility audit and fixes — M
  **Build**: `npm run build` passes without errors
  **Verify**: axe scan reports 0 critical/high issues
  **Gate**: must pass before phase close

---

## Progress: 0/20 tasks complete
```

---

---

# EXAMPLE 2 (CRITICAL — bank-grade): "LedgerPay" Funds Transfer Feature

Context: a fictional digital bank's core service needs a feature letting customers transfer funds
between their own accounts and to other verified LedgerPay users, in real time, with full auditability.
This is the kind of feature where getting the spec suite right — not just the code — is the actual job.

---

## constitution.md (example, excerpt — CRITICAL sections only)

```markdown
# Project Constitution — LedgerPay Core Banking Platform

## 1. Project Identity
- **Name**: LedgerPay
- **Purpose**: Digital bank platform enabling account management and real-time funds transfers.
- **Owners**: Head of Engineering (eng), Chief Compliance Officer (compliance), CISO (security)
- **Regulatory scope**: PCI-DSS v4.0 (card data), GLBA (US financial privacy), SOC 2 Type II,
  applicable state money-transmitter regulations — final applicability confirmed by Legal, not by this document.

## 2. Core Principles
- Least privilege by default — every credential, role, and service account gets the minimum access needed
- No silent failures on money movement — every state-changing financial operation is idempotent, logged,
  and reconcilable
- Maker-checker for anything irreversible — no single human or service can unilaterally move money above
  a defined threshold without a second approver
- Fail closed — if a downstream dependency (fraud check, balance service) is unavailable, the transfer
  is rejected, never silently allowed

## 6. Security Non-Negotiables
- All Confidential/Restricted data encrypted at rest (AES-256) and in transit (TLS 1.2+)
- All privileged actions (manual balance adjustment, transfer reversal, limit override) require step-up
  auth (MFA) and are logged with actor, timestamp, and reason
- Segregation of duties: the engineer who can deploy code cannot also approve production financial
  reconciliation reports
- Secrets rotated every 90 days minimum; never logged, never committed

## 7. Data Classification Policy
| Class | Examples in this system | Handling rules |
|---|---|---|
| Confidential | Account balance, transaction history | Encrypted, access-logged, need-to-know |
| Restricted | Card PAN, SSN/tax ID, bank routing+account numbers, auth credentials | Encrypted, tokenized (PAN via a PCI-scoped tokenization vault — never stored raw), strict RBAC, audit-logged, retention per regulatory minimum then deleted |

## 8. Change Management & Approval
- Production deploys touching the ledger service require: 1 engineering reviewer + 1 approval from
  the on-call financial-systems lead
- Changes to `constitution.md` or `compliance-matrix.md` require CISO + Compliance Officer sign-off
- Break-glass emergency changes are permitted but require a same-day post-hoc review and ticket

## 9. Incident Response Ownership
- On-call: Financial Systems on-call rotation (PagerDuty)
- Breach notification: escalate to Compliance Officer within 1 hour of confirmed Restricted-data exposure;
  actual regulatory notification language is drafted by Legal, not engineering
- Target time to detect anomalous transfer patterns: < 5 minutes (via real-time fraud monitoring)
```

---

## spec.md (example, excerpt)

```markdown
# Feature Specification: Funds Transfer
> Technology-agnostic | Tier: CRITICAL | Version: 1.0 | Status: Approved

## Overview
Allow an authenticated customer to transfer funds from their own LedgerPay account to another account
they own, or to another verified LedgerPay customer, with the transfer reflected in both parties' balances
atomically and irreversibly once settled.

## Users & Personas
- **Customer (Sender)**: Initiates a transfer from their account
- **Customer (Recipient)**: Receives funds
- **Fraud Analyst** (privileged): Reviews flagged transfers before they settle
- **Compliance Officer** (privileged): Audits transfer history for regulatory reporting

## Data Sensitivity Overview
- Data touched: account balance (Confidential), account/routing numbers if external transfer (Restricted),
  transaction history (Confidential)
- Regulatory tags potentially applicable: GLBA (financial privacy), state money-transmitter rules,
  BSA/AML transaction monitoring — flagged for compliance-matrix.md, not asserted here.

## Functional Requirements

### REQ-001: Initiate Internal Transfer
**User Story**:
> As a Customer, I want to transfer funds between my own accounts so that I can manage my money across accounts.

**Acceptance Criteria**:
- WHEN a customer submits a transfer with a valid amount ≤ available balance THEN the system SHALL debit
  the source account and credit the destination account atomically
- WHEN a customer submits a transfer exceeding available balance THEN the system SHALL reject it with
  `INSUFFICIENT_FUNDS` and SHALL NOT partially apply the transfer
- WHEN a transfer is submitted twice with the same idempotency key THEN the system SHALL process it only once
- WHEN a transfer completes THEN the system SHALL create an immutable ledger entry recording amount,
  timestamp, source, destination, and initiating actor

**Data sensitivity**: Confidential (balances)
**Regulatory tag**: BSA/AML transaction monitoring (flag for compliance-matrix.md)

---

### REQ-002: Transfer to External LedgerPay Customer
**User Story**:
> As a Customer, I want to send funds to another verified LedgerPay user so that I can pay another person.

**Acceptance Criteria**:
- WHEN a customer submits a transfer to a verified recipient within their daily limit THEN the system
  SHALL process it per REQ-001's atomicity guarantee
- WHEN a transfer would exceed the customer's daily transfer limit THEN the system SHALL reject it with
  `LIMIT_EXCEEDED`
- WHEN a transfer is flagged by the fraud detection service THEN the system SHALL hold it in a
  `pending_review` state and SHALL NOT settle it until a Fraud Analyst approves or rejects it
- WHEN a Fraud Analyst approves a held transfer THEN the system SHALL log the analyst's identity and
  reasoning before settling

**Data sensitivity**: Confidential / Restricted (if external bank details involved)
**Regulatory tag**: BSA/AML, GLBA

## Non-Functional Requirements
- **Performance**: Transfer decision (accept/reject/hold) returned in < 500ms at p95
- **Availability**: 99.95% uptime; RTO 15 minutes; RPO zero for committed transactions
- **Auditability**: every transfer state change produces an immutable, actor-attributed, timestamped log
  entry retained 7 years (pending Legal confirmation of exact regulatory retention period)

## Out of Scope
- International wire transfers (separate spec)
- Recurring/scheduled transfers (v2)

## Open Questions
- [ ] Exact regulatory retention period for transaction logs — requires Legal confirmation
- [ ] Whether state money-transmitter licensing applies in all operating states — requires Legal review
```

---

## threat-model.md (example, excerpt)

```markdown
# Threat Model: Funds Transfer
> Tier: CRITICAL | Reviewed by: CISO

## Trust Boundary Diagram
```
graph LR
    Customer[Customer App] -->|HTTPS, untrusted| Gateway[API Gateway + WAF]
    Gateway -->|trust boundary| TransferSvc[Transfer Service]
    TransferSvc -->|internal, mTLS| LedgerDB[(Ledger DB - source of truth)]
    TransferSvc -->|trust boundary: 3rd party| FraudAPI[External Fraud Scoring API]
    TransferSvc -->|internal| AuditLog[(Append-only Audit Log)]
```

## Assets in Scope
Account balances, transfer integrity/atomicity, customer PII, audit log integrity, service availability.

## Threats (STRIDE)

### T-001: Replayed transfer request causes duplicate debit
- **Category**: Tampering
- **Actor**: External attacker intercepting/replaying a request, or a buggy client retry
- **Entry point**: POST /transfers endpoint
- **Impact**: Critical — direct financial loss to customer, reconciliation failure
- **Mitigating control**: Mandatory `Idempotency-Key` header; server rejects/dedupes repeated keys
  (see interface-contracts/api.md, task T0XX)
- **Residual risk**: Mitigated

### T-002: Insider manually adjusts a balance without approval
- **Category**: Elevation of Privilege / Repudiation
- **Actor**: Employee with legitimate database or admin-panel access
- **Entry point**: Internal admin tooling
- **Impact**: Critical — undetected fraud, regulatory violation
- **Mitigating control**: All balance adjustments go through a maker-checker workflow; direct DB writes
  to balance fields are disabled outside the transfer service; all adjustments audit-logged with two
  attributed identities
- **Residual risk**: Mitigated

### T-003: Fraud-scoring third-party API is unavailable
- **Category**: Denial of Service (of a control, not the system)
- **Actor**: N/A (dependency failure)
- **Entry point**: External Fraud Scoring API integration
- **Impact**: High — could allow fraudulent transfers through if the system fails open
- **Mitigating control**: Fail-closed policy — if fraud API times out or errors, transfer is held in
  `pending_review` for manual analyst review rather than auto-approved
- **Residual risk**: Mitigated

## Abuse Cases
- **AC-001**: A Fraud Analyst approves their own flagged transfer. **Control**: analysts are blocked from
  approving transfers where they are the sender or recipient; enforced at the application layer and
  reviewed in periodic access audits.

## Unmitigated / Accepted Risks Summary
| Threat ID | Description | Why accepted | Accepted by | Review date |
|---|---|---|---|---|
| T-004 | Sophisticated SIM-swap attack bypassing SMS-based MFA | Full mitigation (hardware key mandate) deferred to Q3 due to UX cost; interim: risk-based step-up + transaction limits | CISO | Next quarterly review |
```

---

## data-model.md (example, excerpt)

```markdown
# Data Model — LedgerPay Funds Transfer

## Entities

### Account
| Field | Type | Required | Classification | Description |
|-------|------|----------|-----------------|-------------|
| id | UUID | ✅ | Internal | Primary key |
| owner_id | UUID | ✅ | Internal | FK to Customer |
| balance_cents | bigint | ✅ | Confidential | Current balance, stored as integer cents to avoid float errors |
| currency | string(3) | ✅ | Internal | ISO 4217 code |
| status | enum | ✅ | Internal | active / frozen / closed |

### Transfer (append-only ledger entry)
| Field | Type | Required | Classification | Description |
|-------|------|----------|-----------------|-------------|
| id | UUID | ✅ | Internal | Primary key |
| idempotency_key | string | ✅ | Internal | Client-supplied, unique per transfer attempt |
| source_account_id | UUID | ✅ | Internal | FK to Account |
| destination_account_id | UUID | ✅ | Internal | FK to Account |
| amount_cents | bigint | ✅ | Confidential | Transfer amount |
| status | enum | ✅ | Internal | pending / pending_review / settled / rejected |
| initiated_by | UUID | ✅ | Internal | Actor who initiated (customer or system) |
| approved_by | UUID | ⛔ nullable | Internal | Fraud analyst, if held for review |
| created_at | timestamp | ✅ | Internal | |
| settled_at | timestamp | ⛔ nullable | Internal | |

Transfer rows are **never updated in place** for the amount/accounts fields — status transitions are
appended as new rows referencing the original transfer id, preserving a full audit trail.

## Data Retention & Deletion Policy
| Entity/Field | Retention period | Deletion/anonymization method | Regulatory driver |
|---|---|---|---|
| Transfer ledger | 7 years (pending Legal confirmation) | Cold-archived, never hard-deleted within window | BSA recordkeeping (to confirm) |
| Account PII | Duration of relationship + 7 years | Anonymized after retention window | GLBA (to confirm) |

## Audit Trail Requirements
Every Transfer status transition is logged to the append-only audit log with: actor id, actor role,
timestamp, previous status, new status, and (if applicable) reason text. No anonymous or system-only
writes are permitted to Transfer or Account balance fields — every write traces to an attributable actor,
including scheduled/automated jobs (which log a designated service-account identity).
```

---

## interface-contracts/api.md (example, excerpt)

```markdown
# API Interface Contracts — Funds Transfer

## Authentication
`Authorization: Bearer <token>` (OIDC access token, 15-min lifetime). Step-up MFA token required for
transfers above the customer's configured "no-step-up" threshold.

## Standard Error Taxonomy
```json
{
  "error_code": "INSUFFICIENT_FUNDS",
  "message": "The transfer amount exceeds the available balance.",
  "trace_id": "b3f1-...-uuid"
}
```

---

### POST /transfers
**Description**: Initiate a funds transfer between accounts.
**Requirements**: REQ-001, REQ-002
**Idempotency**: Requires `Idempotency-Key` header. Duplicate keys within 24h return the original result
without reprocessing; a duplicate key with a different payload returns `409 Conflict`.
**Audit logging**: Logs actor, timestamp, source/destination account ids, amount, and outcome. Amount and
account identifiers are logged in full internally (Confidential-tier access only); never returned in
client-facing error messages beyond what the customer already knows.
**Rate limiting**: 20 requests/minute per account; `429 Too Many Requests` on breach.

**Request Body**:
```json
{
  "source_account_id": "uuid",
  "destination_account_id": "uuid",
  "amount_cents": 5000,
  "currency": "USD"
}
```

**Response 201 (settled)**:
```json
{
  "transfer_id": "uuid",
  "status": "settled",
  "settled_at": "2026-07-07T12:00:00Z"
}
```

**Response 202 (held for review)**:
```json
{
  "transfer_id": "uuid",
  "status": "pending_review"
}
```

**Error Responses**:
- `400 Bad Request`: Invalid input (e.g., malformed amount)
- `401 Unauthorized`: Missing/invalid token
- `403 Forbidden`: Token valid but account not owned by caller
- `409 Conflict`: Idempotency key reused with a different payload
- `422 Unprocessable Entity`: `INSUFFICIENT_FUNDS`, `LIMIT_EXCEEDED`, `DESTINATION_ACCOUNT_FROZEN`
- `503 Service Unavailable`: Downstream fraud-check dependency unavailable — transfer is held, not silently approved
```

---

## compliance-matrix.md (example, excerpt)

```markdown
# Compliance Traceability Matrix — Funds Transfer

## Frameworks in Scope
GLBA (financial privacy) — confirmed applicable. BSA/AML transaction monitoring — confirmed applicable.
State money-transmitter licensing — under legal review, not yet confirmed for all operating states.

## Traceability Table
| Control ID | Framework | Control description | Spec requirement | Implementation task | Evidence/test | Status |
|---|---|---|---|---|---|---|
| GLBA-1 | GLBA | Customer financial data encrypted at rest and in transit | REQ-001 | T041 | Automated encryption-at-rest test in CI | Implemented |
| AML-1 | BSA/AML | Transfers above threshold or matching risk pattern are held for review | REQ-002 | T045 | Test suite: transfer_holds_on_fraud_flag.spec | Verified |
| AML-2 | BSA/AML | All settled transfers retained in immutable audit form for examiner access | REQ-001 | T048 | Manual audit-log query sample reviewed by Compliance | In progress |

## Gaps & Remediation Plan
| Gap | Risk if unaddressed | Remediation | Target date | Owner |
|---|---|---|---|---|
| State money-transmitter licensing status unconfirmed for 3 states | Potential unlicensed operation | Legal review of licensing requirement per state | Before GA in those states | Compliance Officer |

## Sign-off
| Role | Name | Date | Notes |
|---|---|---|---|
| Compliance owner | [pending] | | Awaiting licensing review completion |
| Security owner | [pending] | | |
| Engineering owner | [pending] | | |
```

---

## risk-register.md (example, excerpt)

```markdown
# Risk Register — Funds Transfer

### RISK-001: Fraud-scoring vendor outage causes transfer backlog
- **Category**: Third-Party/Vendor
- **Description**: If the external fraud API has extended downtime, transfers fail-closed into
  `pending_review`, potentially overwhelming the Fraud Analyst queue.
- **Likelihood**: Medium
- **Impact**: Medium (customer experience, not financial loss, since fail-closed prevents fraud exposure)
- **Current mitigation**: Fail-closed design (see threat-model T-003); on-call alert if queue depth exceeds threshold
- **Residual risk after mitigation**: Low
- **Owner**: Financial Systems on-call lead
- **Review date**: Quarterly
- **Status**: Mitigated

### RISK-002: Regulatory licensing gap in 3 states
- **Category**: Regulatory
- **Description**: Money-transmitter licensing status unconfirmed in 3 operating states (see
  compliance-matrix.md gap).
- **Likelihood**: Low (Legal review in progress)
- **Impact**: Critical (regulatory action, forced service suspension)
- **Current mitigation**: Legal review underway; feature gated by state until confirmed
- **Residual risk after mitigation**: Medium until review completes
- **Owner**: Chief Compliance Officer
- **Review date**: Before GA
- **Status**: Open

## Summary by Category
| Category | Open | Mitigated | Accepted | Highest residual severity |
|---|---|---|---|---|
| Security | 0 | 3 | 1 | Medium (T-004, accepted) |
| Operational | 0 | 1 | 0 | Low |
| Financial | 0 | 0 | 0 | — |
| Third-Party/Vendor | 0 | 1 | 0 | Low |
| Regulatory | 1 | 0 | 0 | Critical (open) |
```

---

## access-control-matrix.md (example, excerpt)

```markdown
# Access Control Matrix — LedgerPay Funds Transfer

## Roles Overview
| Role | Description | Privileged? |
|---|---|---|
| Customer | Owns and operates their own accounts | No |
| Support Agent | Views masked account info to help with tickets | Yes (read) |
| Fraud Analyst | Approves/rejects held transfers | Yes |
| Financial Systems Admin | Can manually adjust balances | Yes |

## Permission Matrix
| Role | Data class accessed | Actions allowed | Approval workflow | Justification |
|---|---|---|---|---|
| Customer | Own Confidential balance/history | Initiate own transfers | Self-service | Core function |
| Support Agent | Confidential, Restricted fields masked | Read-only | Access logged, no approval needed | Ticket resolution |
| Fraud Analyst | Confidential + flagged transfer detail | Approve/reject held transfers | Cannot approve own transfers (see threat-model AC-001) | Fraud control |
| Financial Systems Admin | Confidential + Restricted | Manual balance adjustment | Second admin must co-approve (maker-checker) | Irreversible financial action |

## Access Review & Recertification
- Cadence: Quarterly
- Owner: Security Operations Lead
- Deprovisioning SLA: within 4 hours of role change or termination, enforced via HR-system integration

## Break-Glass / Emergency Access
- Permitted for: Sev-1 production incident with no other path to restore service
- Who can invoke: On-call Financial Systems Lead + one witness approver
- Post-hoc requirement: automatically logged and reviewed by CISO within 24 hours
```

---

## disaster-recovery-plan.md (example, excerpt)

```markdown
# Disaster Recovery Plan — LedgerPay Core

## Recovery Objectives
| Component | RTO | RPO |
|---|---|---|
| Ledger / balances | 15 min | 0 (zero data loss on committed transactions) |
| Fraud scoring integration | 30 min (fails closed in the meantime) | N/A (no persisted state) |
| Customer-facing dashboard | 4 hours | 1 hour |

## Failover Procedure (summary)
1. Automated health checks detect ledger DB primary failure
2. On-call Financial Systems Lead confirms and authorizes failover to standby replica in secondary AZ
3. Standby promoted; idempotency-key store validated for consistency before traffic resumes
4. Reconciliation job runs immediately post-failover to confirm no double-processed transfers

## Testing & Drills
- Last tested: [date] — tabletop exercise simulating primary-region outage
- Result: Failover completed within target RTO; found gap in alerting for the fraud-scoring dependency —
  remediation task opened (see tasks.md)
- Next scheduled test: live failover drill, next quarter
```

---

## vendor-risk-assessment.md (example, excerpt)

```markdown
# Vendor Risk Assessment — LedgerPay Funds Transfer

### Vendor: External Fraud Scoring API
- What they access: transaction metadata (amount, accounts involved — tokenized, not raw account numbers)
- Compliance attestations on file: SOC 2 Type II report, dated within last 12 months
- Contractual terms: DPA signed; 24-hour breach notification clause
- Criticality: High — but system fails closed if unavailable (see threat-model T-003), so customer risk is bounded
- Exit strategy: fraud-scoring interface is abstracted behind an internal service so a replacement vendor
  can be swapped without changing the transfer service itself

### Vendor: Cloud Infrastructure Provider
- What they access: full infrastructure hosting, encrypted data at rest
- Compliance attestations on file: SOC 2 Type II, PCI-DSS attestation of compliance (relevant to hosted
  tokenization vault)
- Criticality: Critical (single hosting provider)
- Open concern: no secondary cloud provider for true multi-cloud failover — tracked as RISK-003 in risk-register.md

## Vendor Risk Summary
| Vendor | Criticality | Attestation on file? | Open concerns |
|---|---|---|---|
| Fraud Scoring API | High | Yes (SOC 2) | None — fails closed |
| Cloud Infrastructure Provider | Critical | Yes (SOC 2 + PCI) | No secondary provider (RISK-003) |
```

---

## tasks.md (example, excerpt — security/compliance phase)

```markdown
# Implementation Tasks — Funds Transfer (excerpt)

## Phase 3: Security, Compliance & Release Readiness

**Build Gate**: `go build ./...` — must pass with 0 errors before phase is complete

- [ ] [T041][SEC] Implement encryption-at-rest for balance and account-number fields — M (REQ-001)
  **Build**: `go build ./...` passes without errors
  **Verify**: automated encryption-at-rest test in CI passes
  **Gate**: must pass before T042

- [ ] [T042][SEC] Implement Idempotency-Key handling and dedup store for POST /transfers — M (REQ-001)
  **Build**: `go build ./...` passes without errors
  **Verify**: duplicate-key replay test returns original result; conflict on different payload
  **Gate**: must pass before T043

- [ ] [T043][SEC] Implement maker-checker workflow for manual balance adjustments — L
  **Build**: `go build ./...` passes without errors
  **Verify**: single-approver adjustment is rejected; second approval settles
  **Gate**: must pass before T044

- [ ] [T044][SEC] Implement fail-closed handling when fraud-scoring API is unavailable — M (REQ-002)
  **Build**: `go build ./...` passes without errors
  **Verify**: fraud API outage → transfer held in `pending_review`, never auto-approved
  **Gate**: must pass before T045

- [ ] [T045][SEC] Write test suite verifying transfers hold on fraud-flag and reject on limit breach — M (REQ-002)
  **Build**: `go build ./...` passes without errors
  **Verify**: `go test ./transfers/...` passes for hold-on-flag and limit-reject cases
  **Gate**: must pass before T046

- [ ] [T046][SEC] Run SAST/DAST scans; remediate all Critical/High findings — L
  **Build**: `go build ./...` passes without errors
  **Verify**: scan report shows 0 Critical/High open findings
  **Gate**: must pass before T047

- [ ] [T047][SEC] Conduct third-party penetration test on transfer flow; remediate Critical/High findings — L
  **Build**: `go build ./...` passes without errors
  **Verify**: pen-test report shows 0 Critical/High open findings
  **Gate**: must pass before T048

- [ ] [T048][COMPLIANCE] Verify audit log immutability and completeness for all transfer state transitions — M
  **Build**: `go build ./...` passes without errors
  **Verify**: audit-log completeness query reviewed by Compliance
  **Gate**: must pass before T049

- [ ] [T049][COMPLIANCE] Complete compliance-matrix.md traceability table with evidence links — M
  **Build**: `go build ./...` passes without errors
  **Verify**: every matrix row has an evidence/test link
  **Gate**: must pass before T050

- [ ] [T050][COMPLIANCE] Confirm state money-transmitter licensing status with Legal for all launch states — S
  **Build**: `go build ./...` passes without errors
  **Verify**: Legal confirmation recorded in compliance-matrix.md
  **Gate**: must pass before T051

- [ ] [T051] Release go/no-go review with CISO, Compliance Officer, and Engineering owner — S
  **Build**: `go build ./...` passes without errors
  **Verify**: all three owners sign off; blocking risks resolved or accepted
  **Gate**: must pass before release
```

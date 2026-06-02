# SDD Document Templates

Full content templates for each document in the Spec-Driven Development suite.

---

## constitution.md

```markdown
# Project Constitution
> *Governing principles for [Project Name]. This document is the supreme authority — all other documents must comply with it.*

## 1. Project Identity
- **Name**: 
- **Purpose**: One sentence describing what this project does and for whom.
- **Owners**: 

## 2. Core Principles
List 3–7 non-negotiable principles guiding all decisions.
Example:
- Simplicity over cleverness — prefer boring, readable code
- Security by default — never expose sensitive data without explicit intent
- Test everything — no feature ships without tests

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

## 6. Security Non-Negotiables
- All inputs validated and sanitized
- Authentication required on all private routes
- No secrets in source code (use env vars)
- (Add project-specific rules)

## 7. Definition of Done
A task is complete when:
- [ ] Code implements the spec requirement
- [ ] **Build passes without errors** (compilation/transpile completes)
- [ ] **Verify passes**: the feature runs and produces expected output
- [ ] Tests pass (unit + integration)
- [ ] PR reviewed and approved
- [ ] No new linting errors
- [ ] Spec/plan updated if implementation deviated
```

---

## spec.md

```markdown
# Feature Specification: [Feature Name]
> *Technology-agnostic. Describes WHAT to build, not HOW.*
> *Version: 1.0 | Status: Draft | Last updated: YYYY-MM-DD*

## Overview
Brief description of the feature and the problem it solves.

## Context & Motivation
Why is this feature needed? What user pain does it address?

## Users & Personas
- **[Persona 1]**: Description of this type of user and their goals
- **[Persona 2]**: ...

## Functional Requirements

### REQ-001: [Requirement Name]
**User Story**:
> As a [persona], I want [goal] so that [benefit/reason].

**Acceptance Criteria**:
- WHEN [condition] THEN the system SHALL [expected behavior]
- WHEN [condition] THEN the system SHALL [expected behavior]

**Implementation Notes**: This requirement SHALL be implemented as 2–3 atomic, individually verifiable tasks. Each task must compile and verify before the next begins.

**Notes**: Any clarifications or edge cases.

---

### REQ-002: [Requirement Name]
**User Story**:
> As a [persona], I want [goal] so that [benefit/reason].

**Acceptance Criteria**:
- WHEN [condition] THEN the system SHALL [expected behavior]

**Implementation Notes**: This requirement SHALL be implemented as 2–3 atomic, individually verifiable tasks. Each task must compile and verify before the next begins.

---

## Non-Functional Requirements
- **Performance**: (e.g., page loads in < 2s under normal load)
- **Accessibility**: (e.g., WCAG 2.1 AA compliance)
- **Security**: (e.g., all data encrypted at rest)
- **Localization**: (e.g., supports English and Spanish)

## Out of Scope
Explicitly list what this feature does NOT include:
- 
- 

## Open Questions
List unresolved questions that need answers before development starts:
- [ ] Question 1
- [ ] Question 2
```

---

## requirements.md (validation checklist)

```markdown
# Requirements Validation Checklist
> *Validates that spec.md is complete, unambiguous, and technology-agnostic before planning begins.*

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

## Verdict
- [ ] ✅ READY TO PLAN — All checks passed
- [ ] ⚠️ NEEDS REVISION — Items marked above must be resolved
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

## Proof of Concepts
List any spikes or PoCs conducted:
- **PoC-001**: [What was tested, what was learned]
```

---

## plan.md

```markdown
# Technical Implementation Plan: [Feature/Project Name]
> *Technology-specific. Describes HOW to build what spec.md defines.*
> *Version: 1.0 | Status: Draft | Last updated: YYYY-MM-DD*

## References
- Implements: [link to spec.md]
- Constitution: [link to constitution.md]
- Research: [link to research.md]

## Tech Stack
| Layer | Technology | Version | Justification |
|-------|-----------|---------|---------------|
| Frontend | | | |
| Backend | | | |
| Database | | | |
| Auth | | | |
| Hosting | | | |

## Architecture Overview
```
[ASCII or Mermaid diagram of system architecture]

Example Mermaid:
graph TD
    Client --> API
    API --> DB
    API --> Cache
```

## Data Flow
Describe how data moves through the system for the main use cases.

## Implementation Phases

### Phase 1: [Name] (estimated: X days)

**Goal**:

**Build Gate**: `cargo build` / `npm run build` / `go build` — must pass with 0 errors
**Verify Gate**: `cargo test` / `npm test` or manual smoke test confirms phase goal is met
**Rule**: Each task in this phase MUST compile and verify BEFORE the next task begins

**Deliverables**:
- 

**Tasks**: T001–T00N
**Requirements covered**: REQ-001, REQ-002

### Phase 2: [Name] (estimated: X days)

**Goal**:

**Build Gate**: `cargo build` / `npm run build` / `go build` — must pass with 0 errors
**Verify Gate**: `cargo test` / `npm test` or manual smoke test confirms phase goal is met
**Rule**: Each task in this phase MUST compile and verify BEFORE the next task begins

**Deliverables**:
- 

**Tasks**: T00N–T00M
**Requirements covered**: REQ-003, REQ-004

## Risks & Mitigations
| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| | | | |

## Dependencies & Prerequisites
- [ ] Dependency 1 (reason needed)
- [ ] Dependency 2

## Testing Approach
- Unit tests: [strategy]
- Integration tests: [strategy]
- E2E tests: [strategy]
- **Per-task verification**: after each task, compile + verify before proceeding to next
```

---

## data-model.md

```markdown
# Data Model
> *Defines all entities, their attributes, and relationships.*

## Entities

### [EntityName]
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | UUID | ✅ | Primary key |
| created_at | timestamp | ✅ | Auto-set on creation |
| ... | | | |

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

## Migration Notes
If modifying an existing schema, describe migrations needed.
```

---

## interface-contracts/api.md

```markdown
# API Interface Contracts

## Base URL
`/api/v1`

## Authentication
All endpoints require `Authorization: Bearer <token>` unless marked 🔓.

---

## Endpoints

### POST /[resource]
**Description**: [What this does]
**Requirements**: REQ-001

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
- `409 Conflict`: Resource already exists

---
```

---

## tasks.md

```markdown
# Implementation Tasks
> *Atomic, ordered, trackable tasks derived from plan.md.*
> *Each task = independently implementable + testable + reviewable as single PR.*
> *CRITICAL: After EACH task, compile + verify before moving to the next task.*

## Legend
- `[Txxx]` = Task ID
- `[P]` = Can run in parallel with other [P] tasks in same phase
- `S/M/L` = Estimated complexity (Small/Medium/Large)
- `(REQ-xxx)` = Traceability to spec requirement
- **Build**: command to compile the project after this task
- **Verify**: command or check to confirm the task works
- **Gate**: task must pass Build + Verify before the next task begins

---

## Phase 1: [Name]

**Build Gate**: `cargo build` / `npm run build` / `go build` — must pass with 0 errors before phase is complete

- [ ] [T001] [Setup] Initialize project structure and install dependencies — S
  **Build**: `cargo build` passes without errors
  **Verify**: `cargo run` starts without errors
  **Gate**: must pass before T002

- [ ] [T002] [Setup] Configure environment variables and secrets management — S
  **Build**: `cargo build` passes without errors
  **Verify**: environment variables are loaded correctly (check with print/debug)
  **Gate**: must pass before T003

- [ ] [T003][P] [Auth] Create database schema for users table — S (REQ-001)
  **Build**: `cargo build` passes without errors
  **Verify**: `cargo db:migrate` creates users table, verify with `\dt` or equivalent
  **Gate**: must pass before T004

- [ ] [T004][P] [Auth] Implement POST /auth/register endpoint — M (REQ-001)
  **Build**: `cargo build` passes without errors
  **Verify**: `curl -X POST localhost:3000/auth/register -d '{"email":"test@test.com","password":"123"}'` returns 201
  **Gate**: must pass before T005

- [ ] [T005] [Auth] Implement POST /auth/login endpoint with JWT — M (REQ-001)
  **Build**: `cargo build` passes without errors
  **Verify**: `curl -X POST localhost:3000/auth/login -d '{"email":"test@test.com","password":"123"}'` returns 200 with JWT token
  **Gate**: must pass before T006

- [ ] [T006] [Auth] Write unit tests for auth service — M (REQ-001)
  **Build**: `cargo build` passes without errors
  **Verify**: `cargo test auth` passes all auth-related tests
  **Gate**: must pass before T007

## Phase 2: [Name]

**Build Gate**: `cargo build` / `npm run build` / `go build` — must pass with 0 errors before phase is complete

- [ ] [T007] [Feature] ... — M (REQ-002)
  **Build**: `cargo build` passes without errors
  **Verify**: [specific verify step for this task]
  **Gate**: must pass before T008

---

## Progress Tracking
- Total tasks: X
- Completed: 0
- In progress: 0
- Blocked: 0
```

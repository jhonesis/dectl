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
- [ ] **Code compiles** without errors (Build passes)
- [ ] **Verify step passes** — the feature runs and produces expected output
- [ ] **Tests pass** (unit + integration)
- [ ] **Constitution compliance review** — does the implementation respect all Core Principles?
- [ ] **No new linting errors**
- [ ] **PR reviewed and approved**
- [ ] **Spec/plan updated** if implementation deviated

> **Note**: DoD is non-negotiable. No task is complete until ALL checklist items pass. The "Constitution compliance review" prevents gradual erosion of project principles.
```

---

## spec.md

```markdown
# Feature Specification: [Feature Name]
> *Technology-agnostic. Describes WHAT to build, not HOW.*
> *Version: 1.0 | Status: Draft | Last updated: YYYY-MM-DD*

## Project Type
Select the type that best matches this project. This persona choice affects how requirements are framed:

- **CLI project** — User interacts via terminal, not browser. Personas include "Terminal User", "CI Pipeline", "Sysadmin".
- **Library project** — User imports the library, not an API. Personas include "Developer integrating this library", "Downstream project".
- **API project** — User calls HTTP endpoints. Personas include "API Client", "Frontend developer", "Third-party integrator".
- **Web application** — User interacts via browser. Personas include "End user", "Admin", "Visitor".

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
- **Performance**: (e.g., processes input in < 2s for typical workload)
- **Reliability**: (e.g., zero data loss on crash)
- **Security**: (e.g., all data encrypted at rest)
- **Composability**: (e.g., works as a UNIX pipe for CLI projects)

## Out of Scope
Explicitly list what this feature does NOT include:
- 
- 

## Open Questions
List unresolved questions that need answers before development starts:
- [ ] Question 1
- [ ] Question 2

## Edge Case Catalog
The agent MUST enumerate at least 3 edge cases per REQ. If the user has not specified edge case behavior, the agent MUST ask before proceeding. Evaluate each category:

- [ ] **Null/missing input**: what happens when required data is absent?
- [ ] **Network failure**: what happens when an external dependency is unreachable?
- [ ] **Duplicate data**: what happens when a unique constraint is violated?
- [ ] **Concurrent access**: what happens when two users modify the same resource?
- [ ] **Empty state**: what does the system look like with zero data?
- [ ] **Maximum load**: what happens at or beyond the performance threshold?
- [ ] **Malicious input**: what happens with injection, overflow, or malformed data?
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
| Language | | | |
| Program Type | CLI / API / Library / Web | | |
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

## Dependencies & Prerequisites
- [ ] Dependency 1 (reason needed)
- [ ] Dependency 2

## Testing Approach
- Unit tests: [strategy]
- Integration tests: [strategy]
- E2E tests: [strategy]
- **Per-task verification**: after each task, compile + verify before proceeding to next

## Purity Boundaries
Define which components are pure (no side effects: same input → same output, no I/O) and which are impure (perform I/O, mutate state, or call external systems).

| Component | Type | Reason |
|-----------|------|--------|
| `[module name]` | Pure / Impure | [justification] |

> Marking purity boundaries helps the AI agent reason about testability and parallel safety. Pure functions need only unit tests; impure functions need integration tests.

## Drift Detection
Drift is when the implemented code no longer matches the spec. Detect drift by:

- **Automated**: run test suite (`cargo test`, `pytest`) — if a test that was passing now fails, drift is detected
- **Manual**: compare spec acceptance criteria against actual behavior during code review
- **Structural**: run `dectl session end` at the end of each session — the decision capture log should match the spec REQs

### Drift Detection Checklist
- [ ] All REQ-xxx acceptance criteria have corresponding tests
- [ ] Test suite passes before every commit
- [ ] `dectl session end` confirms no undocumented decisions
- [ ] If drift detected: pause → update spec.md → update tasks.md → resume
```

---

## data-model.md

```markdown
# Data Model
> *Defines all entities, their attributes, and relationships.*

> **Note**: For CLI/library projects without databases, this section documents internal data structures (structs, enums, traits) instead of database tables.

## Entities

### [EntityName]
| Field | Type | Required | Indexes | Constraints | Description |
|-------|------|----------|---------|-------------|-------------|
| id | UUID | ✅ | PK | | Primary key |
| created_at | timestamp | ✅ | INDEX | | Auto-set on creation |
| ... | | | | | |

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

## Code-Level Data Structures (for non-DB projects)
If the project has no database, document the core data structures instead:

```rust
// Example: Rust struct
pub struct Config {
    pub verbose: bool,
    pub output_format: OutputFormat,
    pub input_path: Option<PathBuf>,
}

pub enum OutputFormat {
    Text,
    Json,
    Csv,
}
```

```typescript
// Example: TypeScript interface
interface Task {
  id: string;
  title: string;
  status: 'pending' | 'in-progress' | 'done';
  createdAt: Date;
}
```

## Migration Notes
If modifying an existing schema, describe migrations needed.
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

### CLI Template

## Usage
```
[command] [ARGS] [OPTIONS]
```

## Arguments
| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| FILE | Path | No | Input file path. Reads from stdin if omitted. |

## Options
| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--output, -o` | String | text | Output format |
| `--verbose, -v` | Flag | false | Enable verbose logging |

## Exit Codes
| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error |

## Examples
```bash
# Basic usage
command input.txt

# With options
command input.txt --output json --verbose
```

---

### Library Template

## Public API

### `fn do_something(input: Type) -> Result<Output>`
**Description**: What this function does.
**Parameters**:
| Name | Type | Description |
|------|------|-------------|
| input | Type | Description of the input |
**Returns**: Description of the return value.
**Example**:
```rust
let result = do_something(my_input);
assert!(result.is_ok());
```

### `fn process_data(config: Config) -> Vec<Item>`
**Description**: Processes data based on configuration.
**Throws**: Panics if config is invalid.
**Example**:
```rust
let items = process_data(Config::default());
println!("{} items processed", items.len());
```
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

---

## Progress Tracking
- Total tasks: X
- Completed: 0
- In progress: 0
- Blocked: 0
```

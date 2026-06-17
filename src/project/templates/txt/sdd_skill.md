---
name: spec-driven-development
description: >
  Generate the complete documentation suite for Spec-Driven Development (SDD) before writing any code.
  SDD treats specifications — not code — as the primary artifact. The spec declares intent; code realizes it.
  dectl project init --standard has already created .dec/sdd/ with this skill and templates.
  dectl spec init signals the agent to interview the user and generate all documents in specs/.

  USE THIS SKILL when the user wants to: start a new project or feature with AI assistance; create structured
  docs before coding; generate requirements, technical plan, or task breakdown; follow spec-first workflows;
  create constitution.md, spec.md, requirements.md, plan.md, tasks.md, research.md, data-model.md, or CLAUDE.md;
  work with Kiro, spec-kit, OpenSpec, BMAD or similar SDD tools; or convert a vague idea into an AI-ready blueprint.

  Trigger on: "plan before coding", "document first", "spec first", "create a spec",
  "write requirements", "dectl spec init", or any description of a project/feature to build with AI agents.
---

# Spec-Driven Development Skill

## Philosophy

Spec-Driven Development (SDD) **inverts the traditional workflow**:
- ❌ Traditional: Code first → document later (or never)
- ✅ SDD: Specify intent → plan technically → break into tasks → generate code

The specification is the **single source of truth**. Code is a transient byproduct.
Debugging means fixing the spec, not just the code.

**Key principle**: The SPEC is technology-agnostic (WHAT to build). The PLAN is technology-specific (HOW to build it). Never mix them. See the dedicated section below for enforcement rules.

---

## Document Suite

SDD produces a structured set of documents, in this order:

| # | Document | Purpose | Technology |
|---|----------|---------|------------|
| 1 | `constitution.md` | Governing principles, constraints, non-negotiables | Agnostic |
| 2 | `spec.md` | Feature intent, user stories, acceptance criteria | **Agnostic** |
| 3 | `requirements.md` | Checklist validating the spec is complete & unambiguous | Agnostic |
| 4 | `research.md` | Technical unknowns, decisions, options evaluated | Specific |
| 5 | `plan.md` | Architecture, stack, data models, phases, risks | **Specific** |
| 6 | `data-model.md` | Entity definitions, relationships, schemas | Specific |
| 7 | `interface-contracts/` | API endpoints, contracts, data shapes | Specific |
| 8 | `tasks.md` | Atomic, ordered, trackable implementation tasks | Specific |
| 9 | `specs/SKILL.md` | Agent instructions: how to implement following SDD | Meta |

Documents 1–3 are always produced. Documents 4–9 are produced as needed based on project scope.

---

## Workflow

### Step 0 — dectl spec init (entry point)

The user has run `dectl spec init`. This means:
1. `.dec/sdd/` exists with this skill and templates
2. `.dec/config/project.toml` has `[specs] dir = "specs"`
3. The agent MUST now interview the user and create all SDD documents in `specs/`

### Step 1 — Capture Intent (always start here)

Before writing any document, interview the user to understand:
1. **What** is being built? (product, feature, or bug fix)
2. **Who** are the users? What problems do they have?
3. **What constraints** exist? (tech stack, integrations, team conventions, compliance)
4. **What does success look like?** (acceptance criteria, metrics)
5. **What is explicitly OUT of scope?
6. **What existing systems does this integrate with?** (APIs, databases, other tools)
7. **What is the deployment model?** (self-hosted, SaaS, CLI via package manager, library published to registry)
8. **What is the expected scale?** (users, data volume, throughput)

Ask clarifying questions. Don't guess. The quality of the spec depends on the quality of the intent capture.

### Step 1.5 — Clarification Phase

Before choosing documents, resolve all known unknowns:

1. **List every "known unknown"** — things the user hasn't specified that could affect architecture or design
2. **For each unknown, ask a targeted question**: "You said [X]. Does that mean [interpretation A] or [interpretation B]?"
3. Example: User says "fast search" → ask "What response time is acceptable? <100ms, <500ms, or <2s?"
4. **Do NOT proceed to Step 2** until all identified unknowns are resolved

The quality of the spec is bounded by the quality of these clarifications. Rushing past ambiguity is the #1 cause of spec rejection.

### Step 2 — Choose the right document set

| Project type | Documents to produce |
|---|---|
| New greenfield project | All 9 documents |
| New feature on existing codebase | spec.md, plan.md, tasks.md (+ update CLAUDE.md) |
| Bug fix | bugfix.md (instead of spec), tasks.md |
| Planning only (no implementation yet) | constitution.md, spec.md, requirements.md |
| Quick exploration | spec.md only |

### Step 3 — Produce documents in order

**Never skip ahead.** Each document depends on the previous one:
```
constitution → spec → requirements → research → plan → data-model → interface-contracts → tasks
```

Always show the user each document for review/approval before moving to the next.

### Step 4 — Validate before handing off

Before declaring the SDD suite complete:
- ✅ Spec is technology-agnostic
- ✅ **spec.md has zero technology names** (grep for: React, Node, PostgreSQL, AWS, Docker, etc.)
- ✅ Every acceptance criterion in spec.md has at least one corresponding task in tasks.md
- ✅ Every task has a unique ID (T001, T002…)
- ✅ Tasks are independently implementable and testable
- ✅ **Every task has Build: + Verify: + Gate: — compile and verify after each task before the next**
- ✅ **Each phase has Build Gate + Verify Gate**
- ✅ No code has been written yet

### Step 5 — Log decisions to memory

After each document is approved, run:
```bash
dectl memory add "Document approved: [filename] — [summary]" --type decision
```

After the full suite is complete, run:
```bash
dectl memory add "SDD suite complete: [project] — [documents produced]" --type task
```

This creates traceability between spec documents and the project memory. Future agents can `dectl memory search` to find relevant decisions.

---

## Iterating on Existing Specs

Specs evolve as the project grows. Follow these guidelines when updating:

- **Minor change** (typo, clarification, reworded acceptance criterion) → update the relevant document, increment its `Version:` field
- **Major change** (new feature, changed architecture, scope shift) → create a new version document (`spec-v2.md`), keep the old one for reference
- After any change → run:
  ```bash
  dectl memory add "Spec updated: [description]" --type decision
  ```

Each document has a `Version:` field in its header. Agents MUST increment it on every meaningful change.

---

## Role System: Adversarial Agents

The AI simulates three distinct roles sequentially for every document:

### Coordinator
Conducts the interview (Step 1), asks clarification questions (Step 1.5), and manages the workflow. Decides whether to fix issues or document them as known limitations.

### Implementer
Writes the actual document content based on the Coordinator's notes, following the templates in `references/templates.md`.

### Verifier (Adversary)
After the Implementer finishes each document, the Verifier switches mindset and actively tries to find:
- **Missing edge cases** — cross-reference the Edge Case Catalog in the templates
- **Contradictions** between documents (e.g., spec says X, plan implements Y)
- **Technology names leaked into spec.md** — WHAT vs HOW violation
- **Untestable acceptance criteria** ("it should be fast" without a metric)
- **Non-atomic tasks** — giant tasks that hide complexity

**Process**: Coordinator → Implementer writes draft → Verifier reviews → if Verifier finds issues, Coordinator decides: fix now or document as known limitation and proceed.

This adversarial loop replaces the single-pass writing model. It catches errors that a single perspective misses.

---

## Critical Rule: WHAT vs HOW Separation

The single most important rule in SDD. Violating it is the most common cause of spec rejection.

| Document | Role | Content |
|----------|------|---------|
| **spec.md** | WHAT | Technology-agnostic. Describes user-facing behavior, not implementation. |
| **plan.md** | HOW | Technology-specific. Makes concrete technology decisions for every requirement. |

### Violation examples

**spec.md (WRONG)** — contains technology names:
> ❌ "The React frontend will fetch data from the PostgreSQL database via a REST API"
>
> ✅ "The user submits a task and it appears in the shared list"

**plan.md (WRONG)** — too vague, no technology decisions:
> ❌ "Users can create tasks"
>
> ✅ "React frontend calls Next.js API routes, which use Prisma to write to PostgreSQL"

### Gate rule

Before marking any document as complete, the **Verifier** role MUST check that:
- `spec.md` contains **zero technology names** (grep for: React, Node, PostgreSQL, AWS, Docker, etc.)
- `plan.md` contains **at least one technology decision per REQ**

---

## Model Tiering: Matching Effort to Task

Different documents require different levels of reasoning depth:

| Phase | Documents | Reasoning |
|-------|-----------|-----------|
| **Foundation** | Constitution, Spec, Requirements | Deepest reasoning. These define the project's foundations — errors here compound across all later phases. |
| **Design** | Research, Plan, Data Model | Careful reasoning with cost-benefit awareness. Research should be thorough but proportionate to project risk. |
| **Execution** | Interface Contracts, Tasks | Faster/cheaper inference. Tasks are repetitive and follow templates. The Verifier catches quality issues. |

This tiering is a suggestion, not a constraint. If the model has a single mode, apply more thinking time to Phase 1 documents.

---

## Document Templates

Read `references/templates.md` for the full content template for each document.

---

## Writing Guidelines

### constitution.md rules
- Written once per project, rarely modified
- Covers: coding style, forbidden patterns, required patterns, testing strategy, security non-negotiables, deployment constraints
- Think of it as the "project constitution" — supreme law that all other documents must respect
- **Definition of Done MUST include**: Build passes + Verify passes + Tests pass + PR reviewed

### spec.md rules
- Write in **natural language**, not pseudocode
- Every feature = one **User Story**: `As a [user], I want [goal] so that [benefit]`
- Every User Story has **Acceptance Criteria**: `WHEN [condition] THEN the system SHALL [behavior]`
- **No technology mentions** (no "React", "PostgreSQL", "REST API" — those go in plan.md)
- Number requirements sequentially: REQ-001, REQ-002…
- **Implementation Notes**: each requirement MUST include a note stating it will be implemented as 2–3 atomic, individually verifiable tasks

### plan.md rules
- Explicitly reference the spec requirements it implements (REQ-001 → …)
- Define the complete tech stack with justification
- Include architecture diagram (text/ASCII or Mermaid)
- List external dependencies, risks, and mitigation strategies
- Organize implementation into **phases**
- **Each phase MUST include**: Build Gate (compile command), Verify Gate (test/run command), and the rule that each task must compile and verify before the next task begins

### tasks.md rules
- Every task: `- [ ] [T001] Description — S` (checkbox + ID + complexity)
- Group tasks by Phase
- **NO mega-tasks**. "Implement auth" is invalid. Divide into: T002 register endpoint, T003 login endpoint, T004 JWT middleware, T005 auth unit tests
- **Each task MUST be**:
  - **Atomic** — does one thing, one concern
  - **Individual** — independently implementable, no hidden dependencies
  - **Verifiable** — has a concrete Verify step
- **Each task MUST include**:
  - **Build**: command to compile (`cargo build`, `npm run build`, `go build`)
  - **Verify**: command or action to confirm it works (`curl`, smoke test, app runs)
  - **Gate**: task must pass Build + Verify BEFORE the next task begins
- **Phases have Build Gates and Verify Gates**: the phase is complete only when ALL tasks pass individually AND the phase-level gates pass
- Mark parallel-safe tasks with `[P]` — but each still has its own Verify
- Tasks reference their spec requirement: `(REQ-002)`
- Estimated complexity: S/M/L

### research.md rules
- Documents unknowns investigated during planning
- Each research question: options evaluated → decision → rationale
- List external dependencies with license and risk level

---

## Output Format

Create all documents in `specs/` folder relative to project root.

Suggested structure:
```
project-root/
├── specs/
│   ├── constitution.md
│   ├── spec.md
│   ├── requirements.md
│   ├── research.md
│   ├── plan.md
│   ├── data-model.md
│   ├── interface-contracts/
│   │   └── api.md
│   └── tasks.md
├── .dec/
│   ├── config/project.toml       ← [specs] dir = "specs"
│   ├── isa/project.isa.md         ← "See specs/ for SDD artifacts"
│   └── sdd/
│       ├── SKILL.md               ← este archivo
│       └── references/
│           ├── templates.md
│           └── examples.md
└── AGENTS.md
```

---

## Implementation Preview

After the SDD documents are approved, each task is implemented with **Build → Verify → Gate** (compile, test, then proceed). See the project's coding skill for the full implementation protocol.

---

## Common Mistakes to Avoid

| ❌ Wrong | ✅ Right |
|---|---|
| Mentioning "React" in spec.md | Mention "React" only in plan.md |
| Writing vague acceptance criteria ("it should be fast") | Write measurable criteria ("response time < 200ms") |
| Giant tasks ("Build authentication system") | Atomic tasks ("Create POST /auth/register endpoint") |
| Skipping constitution.md | Always write it for projects > 1 week |
| Tasks without IDs | Every task gets T001, T002… |
| **Task without Build + Verify steps** | **Every task has Build: + Verify: + Gate: inline** |
| **Skipping compile between tasks** | **Compile AFTER every single task before starting the next** |
| **Phase without Build Gate** | **Each phase defines Build Gate + Verify Gate** |
| Spec written in one shot without user review | Show spec draft, get approval, then plan |

---

## Reference Files

- `references/templates.md` — Full content templates for each document
- `references/examples.md` — Worked examples for a sample project

Read these when writing the first document of a type you haven't written yet in this session.

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

  Trigger on: "plan before coding", "document first", "spec first", "quiero planificar", "create a spec",
  "write requirements", "dectl spec init", or any description of a project/feature to build with AI agents.
---

# Spec-Driven Development Skill

## Philosophy

Spec-Driven Development (SDD) **inverts the traditional workflow**:
- ❌ Traditional: Code first → document later (or never)
- ✅ SDD: Specify intent → plan technically → break into tasks → generate code

The specification is the **single source of truth**. Code is a transient byproduct.
Debugging means fixing the spec, not just the code.

**Key principle**: The SPEC is technology-agnostic (WHAT to build). The PLAN is technology-specific (HOW to build it). Never mix them.

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

Ask clarifying questions. Don't guess. The quality of the spec depends on the quality of the intent capture.

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
- ✅ Every acceptance criterion in spec.md has at least one corresponding task in tasks.md
- ✅ Every task has a unique ID (T001, T002…)
- ✅ Tasks are independently implementable and testable
- ✅ **Every task has Build: + Verify: + Gate: — compile and verify after each task before the next**
- ✅ **Each phase has Build Gate + Verify Gate**
- ✅ No code has been written yet

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

## Implementation Protocol (CRITICAL)

After the SDD documents are complete, implementation follows this protocol:

### For each Phase:
1. Verify the phase's **Build Gate** passes (`cargo build`, `npm run build`, etc.)
2. Execute tasks in order (or parallel if marked `[P]`)

### For each task:
1. **Implement** the task according to spec.md
2. **Build**: run the compile command — must pass 0 errors
3. **Verify**: run the verify command or check that the feature works as expected
4. **Gate**: only if Build + Verify pass, mark task `[x]` and move to the next task
5. If Build or Verify fails → **stop**, fix the issue, rebuild, reverify, then continue

### Phase completion:
1. All tasks marked `[x]`
2. Phase **Verify Gate** passes
3. Move to next phase

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

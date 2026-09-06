---
name: spec-driven-development
description: >
  Generate the complete documentation suite for Spec-Driven Development (SDD) before writing any code.
  SDD treats specifications — not code — as the primary artifact. The spec declares intent; code realizes it.
  dectl project init --standard has already created .dec/sdd/ with this skill and templates.
  dectl spec init signals the agent to interview the user and generate all documents in specs/.
  Supports standard and high-stakes/regulatory workflows (CRITICAL tier: threat-model.md, compliance-matrix.md,
  risk-register.md, access-control-matrix.md, disaster-recovery-plan.md, vendor-risk-assessment.md).

  USE THIS SKILL when the user wants to: start a new project or feature with AI assistance; create structured
  docs before coding; generate requirements, technical plan, or task breakdown; follow spec-first workflows;
  create constitution.md, spec.md, requirements.md, plan.md, tasks.md, research.md, data-model.md,
  threat-model.md, compliance-matrix.md, risk-register.md, or CLAUDE.md;
  work with Kiro, spec-kit, OpenSpec, BMAD or similar SDD tools; or convert a vague idea into an AI-ready blueprint.

  Trigger on: "plan before coding", "document first", "spec first", "create a spec",
  "write requirements", "dectl spec init", or any description of a project/feature to build with AI agents.
---

# Spec-Driven Development Skill

## Philosophy

Spec-Driven Development (SDD) **inverts the traditional workflow**:
- ❌ Traditional: Code first → document later (or never)
- ✅ SDD: Specify intent → threat-model & validate → plan technically → break into tasks → generate code

The specification is the **single source of truth**. Code is a transient byproduct.
Debugging means fixing the spec, not just the code. When AI coding agents write the implementation,
an ambiguous or incomplete spec produces confident-looking code that solves the wrong problem — this
is the failure mode SDD exists to prevent ("vibe coding" without a contract).

**Key principle**: The SPEC is technology-agnostic (WHAT to build, and under what constraints/risk).
The PLAN is technology-specific (HOW to build it). Never mix them. See the dedicated section below for enforcement rules.

**Second key principle for high-stakes systems**: When failure has real-world cost — money lost, data
breached, regulations violated, patients harmed — the spec suite must also capture *why the system can
be trusted*: threat models, control mappings, audit trails, and rollback/recovery guarantees. This isn't
bureaucracy for its own sake; it's the part of "intent" that a purely functional spec omits and that,
if skipped, becomes technical and compliance debt no amount of later code review can fully undo.

---

## Step 0 — Determine the rigor tier

Before producing anything, classify the project. Ask directly if unclear — this decision changes which
documents get produced.

| Signal | Tier |
|---|---|
| Internal tool, prototype, low blast radius if wrong | **STANDARD** |
| Consumer app, typical SaaS feature, no regulated data | **STANDARD** |
| Moves money, holds financial account data, or is a bank/fintech/payments system | **CRITICAL** |
| Handles health data (PHI), government ID, biometric data, or minors' data | **CRITICAL** |
| Subject to a named regulatory framework (PCI-DSS, SOX, GDPR, HIPAA, GLBA, PSD2, DORA, ISO 27001, SOC 2) | **CRITICAL** |
| Failure could cause financial loss, safety harm, or reportable breach | **CRITICAL** |
| User explicitly says "bank-grade," "production-critical," "regulated," "audit-ready," "necesita cumplimiento" | **CRITICAL** |

If genuinely ambiguous, default to asking one question rather than guessing down (under-specifying a
critical system is far more costly than over-specifying a simple one).

---

## Document Suite

SDD produces a structured set of documents, in this order. The **Tier** column shows when each is required.

| # | Document | Purpose | Nature | Tier |
|---|----------|---------|--------|------|
| 1 | `constitution.md` | Governing principles, constraints, non-negotiables, security & compliance posture | Agnostic | Both |
| 2 | `spec.md` | Feature intent, user stories, acceptance criteria, data sensitivity, NFRs | **Agnostic** | Both |
| 3 | `requirements.md` | Checklist validating the spec is complete, unambiguous, and (for CRITICAL) control-mapped | Agnostic | Both |
| 4 | `threat-model.md` | STRIDE-based threat analysis, trust boundaries, abuse cases | Agnostic/Specific mix | **CRITICAL** |
| 5 | `research.md` | Technical unknowns, decisions, options evaluated, PoCs | Specific | Both |
| 6 | `plan.md` | Architecture, stack, data flow, phases, risks, security architecture | **Specific** | Both |
| 7 | `data-model.md` | Entity definitions, relationships, schemas, data classification, retention | Specific | Both |
| 8 | `interface-contracts/` | API endpoints, contracts, error taxonomy, idempotency, rate limits | Specific | Both |
| 9 | `compliance-matrix.md` | Maps regulatory/control requirements → spec requirements → implementation evidence | Specific | **CRITICAL** |
| 10 | `risk-register.md` | Enumerated risks (security, operational, financial, third-party) with owner & mitigation | Specific | **CRITICAL** |
| 11 | `access-control-matrix.md` | Every role, what it can do, approval workflows, access review cadence | Specific | **CRITICAL** |
| 12 | `disaster-recovery-plan.md` | Backup/restore, failover, BCP, tested recovery procedure, communication plan | Specific | **CRITICAL** |
| 13 | `vendor-risk-assessment.md` | Due diligence on every third party touching data or money, exit strategy | Specific | **CRITICAL** |
| 14 | `tasks.md` | Atomic, ordered, trackable implementation tasks, including security/compliance gates | Specific | Both |
| 15 | `CLAUDE.md` / `AI.md` | Agent context file: rules + pointers to all artifacts + non-negotiables reminder | Meta | Both |


Documents 1–3 are always produced. For STANDARD projects, documents 5–8 and 14–15 are produced as needed based on
scope (see the table in Step 2). For CRITICAL projects, **all 15 documents are mandatory** — none may be skipped,
and document 4 (`threat-model.md`) must exist before `plan.md` is finalized.

Every document, in both tiers but especially CRITICAL, opens with a **Revision History** block:
`| Version | Date | Author | Change summary |` — one row per meaningful revision. For CRITICAL projects
this is not cosmetic: examiners and auditors expect to see when a control changed and why.

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
6. **What data does this touch, and how sensitive is it?** (PII, financial, health, credentials) — this
   alone often determines the tier.
7. **What regulatory or contractual obligations apply?** (if the user doesn't know, say so explicitly in
   the spec as an open question — never assume compliance requirements you haven't been told or verified.)

Ask clarifying questions. Don't guess. The quality of the spec depends on the quality of the intent capture.
For CRITICAL projects, also ask who the accountable owner/approver is for security and compliance sign-off —
this becomes the reviewer of record in `compliance-matrix.md` and `risk-register.md`.

### Step 1.5 — Clarification Phase

Before choosing documents, resolve all known unknowns:

1. **List every "known unknown"** — things the user hasn't specified that could affect architecture or design
2. **For each unknown, ask a targeted question**: "You said [X]. Does that mean [interpretation A] or [interpretation B]?"
3. Example: User says "fast search" → ask "What response time is acceptable? <100ms, <500ms, or <2s?"
4. **Do NOT proceed to Step 2** until all identified unknowns are resolved

The quality of the spec is bounded by the quality of these clarifications. Rushing past ambiguity is the #1 cause of spec rejection.

### Step 2 — Choose the right document set

| Project type | Tier | Documents to produce |
|---|---|---|
| Quick exploration | STANDARD | `spec.md` only |
| Planning only (no implementation yet) | STANDARD | `constitution.md`, `spec.md`, `requirements.md` |
| New feature on existing codebase | STANDARD | `spec.md`, `plan.md`, `tasks.md` (+ update `CLAUDE.md`) |
| Bug fix | STANDARD | `bugfix.md` (instead of spec), `tasks.md` |
| New greenfield project | STANDARD | All non-CRITICAL documents (1, 2, 3, 5, 6, 7, 8, 14, 15) |
| New module on existing codebase | STANDARD | `specs/<name>/` full document set (via `dectl spec add --scope module`) |
| Any financial, health, regulated, or "bank-grade" system | **CRITICAL** | **All 15 documents, no exceptions** |
| Feature added to an existing CRITICAL system | **CRITICAL** | `spec.md`, `threat-model.md` (delta), `plan.md`, `data-model.md` (delta), `compliance-matrix.md` (delta), `risk-register.md` (delta), `access-control-matrix.md` (delta if new roles), `tasks.md` |

### Step 3 — Produce documents in order

**Never skip ahead.** Each document depends on the previous one:

```
STANDARD:
constitution → spec → requirements → research → plan → data-model → interface-contracts → tasks

CRITICAL:
constitution → spec → requirements → threat-model → research → plan → data-model →
interface-contracts → compliance-matrix → risk-register → access-control-matrix →
disaster-recovery-plan → vendor-risk-assessment → tasks
```

Always show the user each document for review/approval before moving to the next. For CRITICAL projects,
explicitly ask for sign-off on `constitution.md`, `threat-model.md`, `compliance-matrix.md`, and
`access-control-matrix.md` — do not proceed past these four on an assumed approval.

### Step 4 — Validate before handing off (Quality Gates)

Before declaring the SDD suite complete, run this gate. Treat it the way `analyze`/cross-artifact-consistency
steps work in mature SDD tooling: a check that runs *after* tasks exist and *before* implementation starts.

**All projects:**
- ✅ Spec is technology-agnostic
- ✅ **spec.md has zero technology names** (grep -iE for the global denylist in Gate rule below)
- ✅ Every acceptance criterion in spec.md has at least one corresponding task in tasks.md
- ✅ Every task has a unique ID (T001, T002…)
- ✅ Tasks are independently implementable and testable
- ✅ **Every task has Build: + Verify: + Gate: — compile and verify after each task before the next**
- ✅ **Each phase has Build Gate + Verify Gate**
- ✅ No code has been written yet
- ✅ No contradictions between `constitution.md`, `spec.md`, and `plan.md`

**CRITICAL projects, additionally:**
- ✅ Every entry in `threat-model.md` maps to at least one mitigating control in `plan.md` or a task in `tasks.md`
- ✅ Every applicable regulatory requirement in `compliance-matrix.md` maps to a spec requirement AND an
  implementation task — no orphaned regulatory line items
- ✅ Every risk in `risk-register.md` has an owner, a likelihood/impact rating, and a mitigation or explicit
  acceptance
- ✅ `data-model.md` classifies every field touching PII/financial/health data and states its retention policy
- ✅ `interface-contracts/` defines error handling, idempotency, and audit logging for every state-changing endpoint
- ✅ Security and compliance tasks in `tasks.md` are not "nice to have" — they block release the same as functional tasks
- ✅ There is an explicit rollback/incident plan referenced (in `plan.md` or a linked runbook) for state-changing operations
- ✅ Every role in `access-control-matrix.md` maps to the minimum permissions needed — no role has access
  "just in case," and every privileged role has a defined approval/dual-control workflow where applicable
- ✅ `disaster-recovery-plan.md` states RTO/RPO consistently with `plan.md` and names who executes recovery
  and how it will actually be tested (a DR plan that has never been drilled is a hypothesis, not a plan)
- ✅ `vendor-risk-assessment.md` covers every third party in the architecture diagram that touches
  Confidential/Restricted data or money — no vendor connection is exempt from at least a baseline entry
- ✅ If the system makes or informs automated decisions affecting a person (credit, fraud flags, benefits,
  tax assessments), `spec.md` or `plan.md` states whether a human can review/override the decision and how
  an affected person can contest it

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

## Adding a Module with dectl

For complex subsystems that deserve their own spec subdirectory:

```bash
dectl spec add "auth" --scope module --from requirements.md
```

This creates `specs/auth/` with constitution, spec, plan, and tasks.
The root `specs/spec.md` gets a reference REQ pointing to the module.

After creation, implement via:
```bash
dectl workflow run execute_task --var task_id=AUTH-T001 --auto
```

---

## Iterating on Existing Specs

Specs evolve as the project grows. Follow these guidelines when updating:

- **Minor change** (typo, clarification, reworded acceptance criterion) → update the relevant document, increment its `Version:` field
- **Major change** (new feature, changed architecture, scope shift) → create a new version document (`spec-v2.md`), keep the old one for reference
- **New module** → `dectl spec add <name> --scope module --from requirements.md` (creates `specs/<name>/`)
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
- **Missing edge cases** — cross-reference the templates and the spec's acceptance criteria
- **Contradictions** between documents (e.g., spec says X, plan implements Y)
- **Technology names leaked into spec.md** — WHAT vs HOW violation
- **Untestable acceptance criteria** ("it should be fast" without a metric)
- **Non-atomic tasks** — giant tasks that hide complexity
- **For CRITICAL tier** — unmitigated threats, orphaned compliance rows, unjustified privileged roles

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

Global tech-name denylist (WHAT-vs-HOW) — single source of truth used by the automated verifier:
`react|vue|angular|node|postgres|docker|aws|once_cell|bouncycastle|cipher|preauthorize` (case-insensitive).

Before marking any document as complete, the **Verifier** role MUST check that:
- `spec.md` contains **zero technology names** (grep -iE for the global denylist above)
- `plan.md` contains **at least one technology decision per REQ**

### Precedence rule

In case of conflict between this SKILL and local conventions (existing specs, codebase patterns, prior REQs), the SKILL prevails — SKILL prevalece sobre convención local. Do not propagate a local violation by imitation; the skill corrects it.

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

Read `references/templates.md` for the full content template for each document (STANDARD and CRITICAL tiers).

---

## Writing Guidelines

### constitution.md rules
- Written once per project, rarely modified
- Covers: coding style, forbidden patterns, required patterns, testing strategy, security non-negotiables, deployment constraints
- Think of it as the "project constitution" — supreme law that all other documents must respect
- **Definition of Done MUST include**: Build passes + Verify passes + Tests pass + PR reviewed
- For CRITICAL tier: also covers data classification policy, incident response ownership, change-management process (who can approve production changes), and the named regulatory frameworks in scope

### spec.md rules
- Write in **natural language**, not pseudocode
- Every feature = one **User Story**: `As a [user], I want [goal] so that [benefit]`
- Every User Story has **Acceptance Criteria**: `WHEN [condition] THEN the system SHALL [behavior]`
- **No technology mentions** (no "React", "PostgreSQL", "REST API" — those go in plan.md)
- Number requirements sequentially: REQ-001, REQ-002…
- **Implementation Notes**: each requirement MUST include a note stating it will be implemented as 2–3 atomic, individually verifiable tasks
- For CRITICAL tier: every requirement touching money or regulated data states its **data sensitivity class** (Public / Internal / Confidential / Restricted) and any explicit regulatory tag (e.g., `[PCI-DSS]`, `[GDPR]`) — tags are a pointer to verify in `compliance-matrix.md`, not a substitute for legal review

### plan.md rules
- Explicitly reference the spec requirements it implements (REQ-001 → …)
- Define the complete tech stack with justification
- Include architecture diagram (text/ASCII or Mermaid) and, for CRITICAL tier, a **trust-boundary diagram**
- List external dependencies, risks, and mitigation strategies
- Organize implementation into **phases**
- **Each phase MUST include**: Build Gate (compile command), Verify Gate (test/run command), and the rule that each task must compile and verify before the next task begins
- For CRITICAL tier: include a **Security Architecture** section (authN/authZ model, encryption in transit/at rest, key management, secrets handling, logging/audit strategy, segregation of duties) and explicitly state RTO/RPO (recovery time/point objectives) if the system must stay available

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
- For CRITICAL tier: security, logging, and compliance tasks are tagged `[SEC]` / `[COMPLIANCE]` and are treated as release-blocking, not optional polish; include an explicit "penetration test / security review" and "compliance sign-off" task before the release/launch task

### research.md rules
- Documents unknowns investigated during planning
- Each research question: options evaluated → decision → rationale
- List external dependencies with license and risk level
- (CRITICAL) For any dependency handling money movement, cryptography, or Restricted data, note whether it is independently audited/certified

### threat-model.md rules (CRITICAL tier)
- Use STRIDE (Spoofing, Tampering, Repudiation, Information disclosure, Denial of service, Elevation of privilege) or an equivalent structured method — don't freeform a list of "things that could go wrong"
- Every threat has: an attacker/actor, an entry point, an impact rating, and a mitigating control (or an explicit "accepted risk" with an owner)
- Diagram trust boundaries (where data crosses from less-trusted to more-trusted zones) before enumerating threats
- Include abuse cases from legitimate-but-malicious insiders, not just external attackers — this matters disproportionately for financial systems

### compliance-matrix.md rules (CRITICAL tier)
- One row per regulatory/control requirement; columns: control ID, framework, description, spec requirement it satisfies, implementation task, evidence/test that proves it, status
- Never assert compliance that cannot be verified — mark unverified items as "requires legal/compliance review" rather than asserting the system "is PCI-DSS compliant." Structure the traceability; a qualified human/auditor must certify actual compliance.

### risk-register.md rules (CRITICAL tier)
- Categorize risks: Security, Operational, Financial, Third-Party/Vendor, Regulatory
- Every risk: likelihood, impact, current mitigation, residual risk, owner, review date
- Risks are living — this document gets revisited at each phase gate, not written once and forgotten

### access-control-matrix.md rules (CRITICAL tier)
- One row per role (not per person); list what data classes and actions each role can access
- Every privileged role (admin, approver, override) states its approval workflow — solo access to irreversible or financial actions is a finding, not a design choice
- State the access review/recertification cadence (e.g., quarterly) and who owns it
- Map roles back to `data-model.md` classifications — a role touching Restricted data needs a stated justification, not just a permission flag

### disaster-recovery-plan.md rules (CRITICAL tier)
- State RTO/RPO per system component (they can differ — the ledger's RPO is stricter than a reporting dashboard's)
- Name the actual people/roles who execute failover, not just "the on-call team"
- Include a communication plan: who tells customers, regulators, and leadership, and within what timeframe
- State how and when the plan is tested (tabletop exercise vs. live failover drill) and require at least one test before go-live — an untested DR plan should be flagged as an open risk in `risk-register.md`

### vendor-risk-assessment.md rules (CRITICAL tier)
- Every third party that touches Confidential/Restricted data, money movement, or is a single point of failure gets an entry — payment processors, cloud providers, fraud-scoring APIs, email/SMS providers, etc.
- Capture: what data/access the vendor has, their own compliance attestations (e.g., SOC 2 report, PCI attestation) if available, contractual data-protection terms, and an exit/portability strategy if the vendor relationship ends
- Never assert a vendor's compliance status without a document (attestation, report) to point to — mark unverified vendors as "attestation requested, not yet received"

---

## Domain-Specific Addenda

The base CRITICAL tier covers general regulated-system practice. Some domains carry additional, well-known
obligations worth raising proactively — as questions to confirm with the user/their legal counsel, never
as an assumed-compliant checkbox.

### Banking / Fintech / Payments
- Card data: if PANs are ever touched, ask whether tokenization/PCI-DSS scope reduction is in place, or
  whether a PCI-scoped vendor should handle card data instead of the system directly touching it
- AML/BSA: transaction monitoring and suspicious-activity flagging is usually a named legal requirement,
  not optional — confirm reporting obligations and thresholds with compliance
- Consumer protection: adverse-action notices (denials, limit reductions) often carry legal disclosure
  requirements — flag in `spec.md` as an open question if unconfirmed

### Tax / Revenue Systems
- Data classification: taxpayer ID numbers and return data are typically Restricted-tier by default
- Jurisdictions with federal tax-data-sharing rules (e.g., IRS Publication 1075-style safeguarding regimes)
  often mandate specific encryption, access-logging, and background-check requirements for anyone with
  system access — flag this explicitly rather than assuming a general security posture covers it
- Filing deadlines create availability requirements that are legally, not just operationally, significant —
  note peak-load and uptime requirements around statutory deadlines in `spec.md`'s NFRs

### Government / Public Sector
- Accessibility is frequently a legal requirement, not a best practice (e.g., WCAG-based statutes) —
  treat accessibility acceptance criteria as REQ-level, not NFR-level, polish
- Public-records and retention laws may require *longer* retention than a private company would otherwise
  choose, and may restrict deletion even upon a citizen's request — flag any conflict between "right to
  erasure"-style requests and records-retention law as an open question for legal, don't resolve it in code
- Procurement/vendor rules may restrict which cloud regions or vendors are eligible — confirm before
  `plan.md`'s tech stack is finalized

### Healthcare
- PHI is Restricted-tier by default; minimum necessary access principle applies to every role in
  `access-control-matrix.md`
- Business Associate-style agreements are typically required with any vendor touching PHI — this is a
  `vendor-risk-assessment.md` line item, not an assumption

**In every domain case above: the agent raises the consideration and flags it as needing confirmation from
the user's legal/compliance/regulatory function. The agent does not assert that a specific law applies, is
satisfied, or does not apply — that determination belongs to qualified counsel.**

---

## Output Format

Create all documents in `specs/` folder relative to project root.

Suggested structure (STANDARD):
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

Suggested structure (CRITICAL) — add the 6 CRITICAL-only documents:
```
project-root/
├── specs/
│   ├── constitution.md
│   ├── spec.md
│   ├── requirements.md
│   ├── threat-model.md
│   ├── research.md
│   ├── plan.md
│   ├── data-model.md
│   ├── interface-contracts/
│   │   └── api.md
│   ├── compliance-matrix.md
│   ├── risk-register.md
│   ├── access-control-matrix.md
│   ├── disaster-recovery-plan.md
│   ├── vendor-risk-assessment.md
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
| Treating security/compliance docs as optional for a financial system | For CRITICAL tier, they are mandatory gate items, same as functional requirements |
| Asserting "this is compliant with X" | State what controls are implemented and flag what needs human/auditor certification |
| Writing a threat model as a vague bullet list | Use STRIDE (or equivalent), one control per threat, no threat left unmitigated or unaccepted |
| Bundling audit logging as an afterthought task | Specify audit/log requirements per-endpoint in `interface-contracts/`, not as a generic backlog item |
| Granting a role access "just in case" | Every role in `access-control-matrix.md` justifies its access against `data-model.md` classifications |
| Writing a DR plan that's never been tested | Require at least one drill/tabletop exercise before go-live; untested DR is an open risk, not a control |
| Assuming a vendor is compliant because they're well-known | Request and cite their actual attestation (SOC 2, PCI report); mark unverified vendors explicitly |
| Resolving a legal/regulatory conflict (e.g., erasure vs. retention law) in code without asking | Flag it as an open question for legal/compliance in `spec.md`; never silently pick a side |
| Treating domain-specific rules (IRS, HIPAA, FedRAMP, PCI) as generically covered by "good security" | Call them out by name in the relevant domain addendum and confirm applicability with the user's counsel |

---

## Reference Files

- `references/templates.md` — Full content templates for each document (STANDARD and CRITICAL tiers),
  including `threat-model.md`, `compliance-matrix.md`, `risk-register.md`, `access-control-matrix.md`,
  `disaster-recovery-plan.md`, `vendor-risk-assessment.md`
- `references/examples.md` — Two fully worked examples: a STANDARD web app (TaskFlow), and a CRITICAL
  bank-grade funds-transfer feature (LedgerPay) showing every additional document in practice

Read these when writing the first document of a type you haven't written yet in this session.

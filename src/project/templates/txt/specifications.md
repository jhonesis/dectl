# Project Specifications — [Project Name]

> *Fill in this template (in English) and pass it to the agent via:*
>
> ```bash
> dectl spec init --from specifications.md
> ```
>
> *The agent uses this file as the source of truth to generate the SDD document suite in `specs/`.*

---

## 1. Tier Signals Checklist

Classify the project as **STANDARD** or **CRITICAL** (see `.dec/sdd/SKILL.md` Step 0). Check every signal that applies:

- [ ] Internal tool, prototype, low blast radius if wrong — **STANDARD**
- [ ] Consumer app, typical SaaS feature, no regulated data — **STANDARD**
- [ ] Moves money, holds financial account data, or is a bank/fintech/payments system — **CRITICAL**
- [ ] Handles health data (PHI), government ID, biometric data, or minors' data — **CRITICAL**
- [ ] Subject to a named regulatory framework (PCI-DSS, SOX, GDPR, HIPAA, GLBA, PSD2, DORA, ISO 27001, SOC 2) — **CRITICAL**
- [ ] Failure could cause financial loss, safety harm, or reportable breach — **CRITICAL**
- [ ] User explicitly says "bank-grade," "production-critical," "regulated," "audit-ready" — **CRITICAL**

**Determined tier**: `[STANDARD / CRITICAL]`

---

## 2. Identity

**What is being built?**
> One paragraph: product, feature, or bug fix. What it does and why it exists.

**Key principles / non-negotiables:**
- [ ] List governing rules the system must never violate
- [ ] e.g., "no vendor lock-in", "offline-first", "no PII stored"

---

## 3. Users & Personas

**Who are the users?**

| Persona | Role | Problem they have | What they need |
|---------|------|-------------------|----------------|
| [Name]  | [e.g., Admin] | [pain point] | [capability] |
| [Name]  | [e.g., End user] | [pain point] | [capability] |

**Privileged personas** (CRITICAL only): [e.g., system admins, support agents — what elevated access they have and why]

---

## 4. Scope

**In scope:**
- [ ] Feature/module A
- [ ] Feature/module B
- [ ] Integrations: [list]

**Explicitly out of scope (non-goals):**
- [ ] Feature X
- [ ] Platform Y support
- [ ] Compliance certification (deferred to [phase])

---

## 5. Success & Acceptance Criteria

**Definition of done — how do we know it works?**
- [ ] Criterion 1 (measurable, e.g., "search returns results in <500ms")
- [ ] Criterion 2
- [ ] Success metric: [e.g., % of users who complete onboarding]

**Acceptance criteria (WHEN / SHALL):**
- WHEN [action/condition], the system SHALL [observable behavior]

---

## 6. Technical Constraints

**Stack & constraints:**
- Languages / frameworks: [e.g., Rust + clap]
- Databases / storage: [e.g., SQLite, Postgres]
- Existing systems to integrate with: [list]
- Team conventions: [e.g., rustfmt, anyhow for errors, --json flags]
- Known limitations: [e.g., must run offline, binary size limit]

---

## 7. Data

**What data does this touch, and how sensitive is it?**

| Data | Sensitivity class (Public/Internal/Confidential/Restricted) | Where stored | Retention |
|------|------------------------------------------------------------|--------------|-----------|
| [e.g., user email] | [Internal] | [e.g., users table] | [e.g., until account deletion] |

- PII / financial / health / credentials handled? [yes/no + notes]
- Encryption / masking requirements: [notes]

---

## 8. Regulation & Compliance

**Which regulatory or contractual obligations apply?**
- [ ] None known — state as an open question in the spec if uncertain (never assume compliance you haven't verified)
- [ ] [Named framework, e.g., GDPR, PCI-DSS, SOC 2] — relevant obligations: [notes]

---

## 9. Risks & Dependencies

| Risk | Likelihood | Impact | Mitigation | Owner |
|------|-----------|--------|------------|-------|
| [risk] | [High/Med/Low] | [High/Med/Low] | [mitigation] | [owner] |

**Third-party dependencies:** [list any vendors/libraries touching data or money]

---

## 10. CRITICAL-Only Sections

> *Fill these ONLY if the tier determined in Section 1 is **CRITICAL**.*
> *They feed `compliance-matrix.md`, `risk-register.md`, `access-control-matrix.md`, and `disaster-recovery-plan.md`.*

### 10.1 Accountable Owner & Approvers
- Engineering owner: [name/role]
- Security owner: [name/role]
- Compliance owner: [name/role]
- Who approves production changes: [name/role]

### 10.2 Incident Response
- Severity levels: [e.g., Sev1–Sev4 definitions]
- On-call / escalation owner: [name/role]
- Response SLAs per severity: [e.g., Sev1: acknowledge <15min, fix <1h]
- Communication plan: [who to notify, how, when]

### 10.3 Audit & Sign-off
- Audit trail requirements: [e.g., append-only log of all state-changing actions]
- Who must sign off before release: [names/roles]
- Regulatory evidence retention: [duration + format]

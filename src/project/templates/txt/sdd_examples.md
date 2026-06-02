# SDD Worked Examples

A complete worked example using a simple "Task Manager" web app to illustrate each document.

---

## Example Project: "TaskFlow" — A team task management web app

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
- Project setup, database schema, auth
- Requirements: prerequisites for REQ-001, REQ-002, REQ-003

### Phase 2: Core Task Features (4 days)
- Create, list, assign, update tasks
- Requirements: REQ-001, REQ-002, REQ-003

### Phase 3: Polish & Testing (2 days)
- Loading states, error handling, accessibility
- E2E tests with Playwright

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

- [ ] [T001] Initialize Next.js 14 project with TypeScript strict mode — S
- [ ] [T002] Configure Supabase project and environment variables — S
- [ ] [T003] Set up Prisma with PostgreSQL connection — S
- [ ] [T004][P] Create database migration: users table — S (REQ-001)
- [ ] [T005][P] Create database migration: tasks table (id, title, description, status, assignee_id, creator_id, created_at, completed_at) — S (REQ-001, REQ-002, REQ-003)
- [ ] [T006] Configure Supabase Auth with magic link — M
- [ ] [T007] Create auth middleware for protected routes — M

## Phase 2: Core Features

- [ ] [T008][P] Build POST /api/tasks endpoint (create task) — M (REQ-001)
- [ ] [T009][P] Build GET /api/tasks endpoint (list tasks) — M (REQ-001)
- [ ] [T010] Build PATCH /api/tasks/[id] endpoint (update status + assignee) — M (REQ-002, REQ-003)
- [ ] [T011] Build TaskForm component with validation — M (REQ-001)
- [ ] [T012] Build TaskList component with status badges — M (REQ-001)
- [ ] [T013] Build AssigneeDropdown component — S (REQ-002)
- [ ] [T014] Implement status change handler — S (REQ-003)
- [ ] [T015] Write unit tests for task service functions — M (REQ-001, REQ-002, REQ-003)

## Phase 3: Polish & Testing

- [ ] [T016] Add loading skeletons for task list — S
- [ ] [T017] Add error boundary and toast notifications — S
- [ ] [T018] Write Playwright E2E test: create task flow — M
- [ ] [T019] Write Playwright E2E test: assign and complete task flow — M
- [ ] [T020] Accessibility audit and fixes — M

---

## Progress: 0/20 tasks complete
```
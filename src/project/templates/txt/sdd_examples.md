# SDD Worked Examples

## How to Use These Examples

These examples illustrate nine SDD documents (constitution, spec, requirements, research, plan, data-model, interface-contracts, tasks) for different project types.

### Choosing an Example
- **"logsnap"** — Rust CLI tool. Use for local-first, terminal-based, single-binary projects.
- **"SnippetVault"** — Python REST API. Use for web API, CRUD, database-backed projects.
- **"LegacyPay"** — Brownfield/modernization. Use when extracting a spec from existing code.
- **"EventStream"** — Event-driven architecture. Use for async, message broker, microservices projects.
- **"HabitStack"** — Next.js full-stack web app. Use for frontend + backend, SSR, auth, database projects.

### Adaptation Guidelines
- Adapt **structure**, not content. Every project needs a constitution with Core Principles, a spec with REQ-00X format, a plan with architecture, etc.
- Include **all 9 documents** even if your chosen example doesn't show all sections — extrapolate from similar documents across examples.
- Customize **Build/Verify/Gate** commands in tasks.md to match your project's toolchain.
- Add a **Traceability Matrix** (see examples) mapping REQ → plan phase → task ID → verify → test.

---

## Example: "logsnap" — A Rust CLI Log Parser

---

## constitution.md (example)

```markdown
# Project Constitution — logsnap

## 1. Project Identity
- **Name**: logsnap
- **Purpose**: A CLI tool that parses, filters, and formats log files from the terminal. Designed for developers and SREs who need fast ad-hoc log analysis without leaving the terminal.
- **Owners**: Maintainer team

## 2. Core Principles
- Performance first — parse 100MB+ log files in under 2 seconds
- Unix-philosophy — do one thing well (log parsing), composable with pipes
- No runtime dependencies — single static binary, zero external runtime deps
- Predictable output — same input always produces same output (pure parsing)

## 3. Technology Constraints
### Mandatory Stack
- Language: Rust (stable)
- CLI framework: clap v4 (derive API)
- Output: colored with NO_COLOR support, JSON via --json flag
- Error handling: anyhow, no panics in production paths

### Forbidden Technologies
- No async runtime (not needed for file parsing)
- No external databases or services
- No network calls whatsoever
- No dynamic dispatch in hot paths

## 4. Testing Strategy
- Unit tests: #[cfg(test)] modules in every .rs file
- Integration tests: tests/ directory with sample log fixtures
- Benchmark: criterion benchmarks for parser hot paths
- Coverage target: 85%

## 5. Definition of Done
- [ ] Code compiles without errors (cargo build)
- [ ] Verify step passes (cargo run -- --help works)
- [ ] Unit + integration tests pass (cargo test)
- [ ] No clippy warnings (cargo clippy)
- [ ] At least one benchmark covers the parser
- [ ] Constitution compliance: no async, no network, no databases
```

---

## spec.md (example)

```markdown
# Feature Specification: Log File Parsing
> Technology-agnostic | Version: 1.0 | Status: Approved

## Overview
A CLI tool that reads log files, applies user-specified filters, and outputs the results in one of several formats (text, JSON, CSV).

## Users & Personas
- **Developer**: Debugs application issues by searching logs for error patterns
- **SRE**: Monitors production logs by filtering on severity, service, or timeframe
- **CI Pipeline**: Runs logsnap as part of automated incident analysis scripts

## Functional Requirements

### REQ-001: Log File Reading
**User Story**:
> As a Developer, I want to pass a log file path to logsnap so that the tool reads and parses its contents.

**Acceptance Criteria**:
- WHEN a user provides a valid file path as a positional argument THEN the system SHALL read and parse the file
- WHEN a user provides a non-existent file path THEN the system SHALL exit with a descriptive error message
- WHEN a user provides a pipe via stdin (no file argument) THEN the system SHALL read from stdin
- WHEN the file is larger than 1GB THEN the system SHALL stream-parse without loading the entire file into memory

### REQ-002: Log Filtering
**User Story**:
> As an SRE, I want to filter logs by severity level (INFO, WARN, ERROR) so that I can focus on relevant entries.

**Acceptance Criteria**:
- WHEN a user specifies --level ERROR THEN the system SHALL only output entries with that severity
- WHEN a user specifies --level WARN,ERROR THEN the system SHALL output entries matching any of the listed levels
- WHEN a user specifies an invalid severity level THEN the system SHALL exit with a list of valid levels
- WHEN no --level flag is provided THEN the system SHALL output all entries unfiltered

### REQ-003: Output Formatting
**User Story**:
> As a CI Pipeline, I want logsnap to output in JSON format so that downstream tools can parse the results.

**Acceptance Criteria**:
- WHEN a user specifies --format json THEN the system SHALL output a JSON array of parsed entries
- WHEN a user specifies --format csv THEN the system SHALL output CSV with headers
- WHEN a user specifies --format text (or no format flag) THEN the system SHALL output human-readable colored text
- WHEN NO_COLOR environment variable is set THEN the system SHALL NOT use ANSI color codes

## Non-Functional Requirements
- **Performance**: Parse 100MB log file in < 2 seconds on a modern laptop
- **Reliability**: Zero panics for any valid or invalid input
- **Composability**: Works as a UNIX pipe (stdin → logsnap --level ERROR --json → jq)

## Out of Scope
- Real-time log tailing (-f flag, v2)
- Log file rotation detection (v2)
- Remote log fetching over SSH/HTTP (v2)
```

---

## requirements.md (example)

```markdown
# Requirements Traceability — logsnap

## REQ-001: Log File Reading
| Attribute | Value |
|-----------|-------|
| Priority | P0 |
| Effort | S (2h) |
| Dependencies | None |
| Risk | Low — Rust std::fs::File handles this |
| Verification | cargo test test_read_file |

## REQ-002: Log Filtering
| Attribute | Value |
|-----------|-------|
| Priority | P0 |
| Effort | M (4h) |
| Dependencies | REQ-001 |
| Risk | Low — simple string matching |
| Verification | cargo test test_filter_by_level |

## REQ-003: Output Formatting
| Attribute | Value |
|-----------|-------|
| Priority | P0 |
| Effort | M (4h) |
| Dependencies | REQ-001 |
| Risk | Low — serde for JSON, custom for CSV |
| Verification | cargo test test_json_output |

## REQ-004: Pipe/Stdin Support
| Attribute | Value |
|-----------|-------|
| Priority | P1 |
| Effort | S (1h) |
| Dependencies | REQ-001 |
| Risk | Low — read from stdin if no file arg |
| Verification | cargo test test_stdin_reading |

## REQ-005: Large File Streaming
| Attribute | Value |
|-----------|-------|
| Priority | P1 |
| Effort | M (4h) |
| Dependencies | REQ-001 |
| Risk | Medium — BufReader with line-by-line iteration |
| Verification | cargo test test_large_file --release
```

---

## research.md (example)

```markdown
# Research — logsnap

## Research Question 1: Which Rust CLI framework?
**Question**: Should we use clap v4 (derive), clap v4 (builder), or structopt?

**Investigation**:
- structopt is deprecated in favor of clap v4 derive
- clap derive API is the community standard (90%+ of Rust CLI tools)
- Builder API is more flexible but unnecessary for our simple CLI
- clap v4 supports autocomplete generation (bonus)

**Decision**: Use clap v4 with derive API.

## Research Question 2: Which log line format to support?
**Question**: Should we support only one format, or auto-detect between common formats?

**Formats considered**:
1. Common Log Format (Apache/nginx)
2. JSON lines (each line is a JSON object)
3. Syslog (RFC 5424)
4. Plain text with level prefix [INFO] [WARN] [ERROR]

**Decision**: Support plain text with level prefix as the baseline. JSON lines as auto-detect (try serde_json::from_str per line). CLF and syslog as opt-in via --format auto-detect. Rationale: plain text + JSON covers 90%+ of real-world log files.

## Open Questions
- Should we support glob patterns for multi-file input? (deferred to v2)
- How aggressive should auto-format detection be? (soft fail: try each format, use first that parses all lines)
```

---

## plan.md (example)

```markdown
# Technical Plan — logsnap CLI
> Implements: spec.md | Stack defined in constitution.md

## Tech Stack
| Layer | Technology | Justification |
|-------|-----------|---------------|
| Language | Rust stable | Performance, safety, static binary |
| CLI Framework | clap v4 (derive) | Industry standard, autocomplete |
| JSON output | serde_json | Standard Rust JSON library |
| Error handling | anyhow | Project convention, ergonomic |
| Testing | cargo test + criterion | Built-in + benchmarks for perf |
| Program Type | CLI | Terminal application, no web/API |

## Architecture
```
stdin ─┐
       ├──> clap arg parser ──> LogConfig ──> LogReader ──> Filter ──> Formatter ──> stdout
file ──┘                                   (BufReader)    (level)    (text|json|csv)
```

## Purity Boundaries
| Component | Type | Reason |
|-----------|------|--------|
| `LogConfig::parse` | Pure | Parses CLI args, no I/O |
| `LogReader::read_lines` | Impure | Reads from file or stdin (I/O) |
| `Filter::matches` | Pure | String matching, no side effects |
| `Formatter::to_json` | Pure | Transforms data, no I/O |
| `Formatter::to_text` | Pure | Transforms data, no I/O |
| `main` | Impure | Orchestrates I/O |
| Testing implication: Pure functions need unit tests only; impure components need integration tests with fixtures.

## Implementation Phases

### Phase 1: Foundation (1 day)
- Project scaffolding (cargo init, clap setup, CI config)
- Requirements: satisfies REQ-001 (file reading)

### Phase 2: Core Features (3 days)
- Log parsing with format detection
- Level-based filtering
- Text, JSON, and CSV output formatting
- Requirements: REQ-001, REQ-002, REQ-003

### Phase 3: Polish & Performance (2 days)
- Stdin support, large file streaming
- Color output with NO_COLOR compliance
- CLI autocomplete generation
- Benchmark suite and optimization
- Requirements: REQ-004, REQ-005

## Risks
| Risk | Mitigation |
|------|-----------|
| Large files cause high memory | BufReader + line-by-line, never load all into memory |
| Log format ambiguity | Auto-detect with fallback to user-specified --format |
| Performance regression | Criterion benchmarks in CI, compare against baseline |
```

---

## data-model.md (example)

```markdown
# Data Model — logsnap

## Core Structs

### LogConfig
| Field | Type | Description |
|-------|------|-------------|
| file_path | Option<PathBuf> | Path to log file (None = read stdin) |
| levels | Vec<Level> | Filter by severity (empty = no filter) |
| format | OutputFormat | text / json / csv |
| color | ColorPolicy | auto / always / never |

### LogEntry
| Field | Type | Description |
|-------|------|-------------|
| timestamp | Option<DateTime<Utc>> | Parsed timestamp (None if not found) |
| level | Option<Level> | Parsed severity (None if not found) |
| message | String | The log message body |
| raw | String | Original raw line (for text output) |

### Level (Enum)
- Info
- Warn
- Error
- Debug
- Trace

### OutputFormat (Enum)
- Text
- Json
- Csv

## Indexes & Constraints
For CLI projects without databases, this section documents validation rules instead of DB indexes:
- LogEntry.timestamp: optional — if present, must be valid RFC 3339
- LogEntry.level: optional — if present, must match exactly one of the Level enum variants
- LogEntry.message: never empty (empty messages are skipped with a warning)
- LogConfig.file_path and stdin are mutually exclusive (error if both missing)
```

---

## interface-contracts/cli.md (example)

```markdown
# CLI Interface — logsnap

## Interface Type: CLI

## Usage
```
logsnap [FILE] [OPTIONS]
```

## Commands
No subcommands. Single-shot execution.

## Arguments
| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| FILE | Path | No | Path to log file. Reads from stdin if omitted. |

## Options
| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--level, -l` | String... | (all) | Filter by severity level (INFO, WARN, ERROR, DEBUG, TRACE). Repeatable. |
| `--format, -f` | String | text | Output format: text, json, csv |
| `--color` | String | auto | Color policy: auto, always, never |
| `--json` | Flag | false | Shorthand for --format json |
| `--help` | Flag | — | Print help and exit |
| `--version` | Flag | — | Print version and exit |

## Exit Codes
| Code | Meaning |
|------|---------|
| 0 | Success (at least one entry matched) |
| 1 | No entries matched the filter |
| 2 | Error (file not found, invalid args, parse failure) |

## Examples
```bash
# Parse all entries from a file
logsnap server.log

# Filter errors only, with JSON output
logsnap server.log --level ERROR --json

# Pipe from another command
tail -n 100 app.log | logsnap --level WARN,ERROR --format csv

# Disable color
NO_COLOR=1 logsnap server.log
```
```

---

## tasks.md (example)

```markdown
# Implementation Tasks — logsnap

## Phase 1: Foundation

- [ ] [T001] Initialize Rust project with clap v4 and anyhow — S
  - **Build**: `cargo build` compiles without errors
  - **Verify**: `cargo run -- --help` shows usage string
  - **Gate**: Code compiles, no warnings

- [ ] [T002][P] Define LogConfig struct with clap derive — S (REQ-001)
  - **Build**: `cargo build` passes
  - **Verify**: `cargo test test_config_parse` passes
  - **Gate**: Struct fields match spec (file_path, levels, format, color)

- [ ] [T003][P] Define LogEntry struct and Level enum — S (REQ-001)
  - **Build**: `cargo build` passes
  - **Verify**: `cargo test test_log_entry_creation` passes
  - **Gate**: All Level variants defined, LogEntry has timestamp/level/message/raw

## Phase 2: Core Features

- [ ] [T004] Implement LogReader with BufReader line iteration — M (REQ-001)
  - **Build**: `cargo build` passes
  - **Verify**: `cargo test test_read_file` and `cargo test test_stdin_reading` pass
  - **Gate**: Reads file line by line, returns Vec<LogEntry>

- [ ] [T005][P] Implement format auto-detection (plain text + JSON lines) — M (REQ-001)
  - **Build**: `cargo build` passes
  - **Verify**: `cargo test test_format_detection` passes
  - **Gate**: Correctly identifies format from first few lines

- [ ] [T006] Implement Filter::matches with level filtering — S (REQ-002)
  - **Build**: `cargo build` passes
  - **Verify**: `cargo test test_filter_by_level` passes
  - **Gate**: Only matching levels pass through filter

- [ ] [T007][P] Implement Formatter (text, json, csv) — M (REQ-003)
  - **Build**: `cargo build` passes
  - **Verify**: `cargo test test_json_output` and `cargo test test_csv_output` pass
  - **Gate**: JSON output is valid serde_json, CSV has headers

- [ ] [T008] Implement color output with NO_COLOR support — S (REQ-003)
  - **Build**: `cargo build` passes
  - **Verify**: `NO_COLOR=1 cargo run -- server.log | cat -v` shows no escape codes
  - **Gate**: ColorPolicy logic implemented correctly

## Phase 3: Polish & Performance

- [ ] [T009] Add large file streaming (BufReader, no full load) — M (REQ-005)
  - **Build**: `cargo build` passes
  - **Verify**: `cargo test test_large_file --release` passes (< 2s for 100MB)
  - **Gate**: Memory usage stays under 10MB for any file size

- [ ] [T010] Add shell completion generation — S
  - **Build**: `cargo build` passes
  - **Verify**: `cargo run -- --completions bash > logsnap.bash` works
  - **Gate**: Generated completions parse without errors

- [ ] [T011] Add criterion benchmarks for parser hot paths — S
  - **Build**: `cargo bench` compiles without errors
  - **Verify**: `cargo bench` runs and shows timing
  - **Gate**: Baseline benchmark results recorded

- [ ] [T012] Write integration tests with sample log fixtures — S
  - **Build**: `cargo test` passes
  - **Verify**: All integration tests in tests/ directory pass
  - **Gate**: Test fixtures cover happy path + error cases + edge cases

---

## Progress: 0/12 tasks complete
```

---

## Example: "SnippetVault" — A Python API for Code Snippets

---

## constitution.md (example)

```markdown
# Project Constitution — SnippetVault

## 1. Project Identity
- **Name**: SnippetVault
- **Purpose**: A REST API that lets developers save, tag, search, and share code snippets. Designed for teams that need a shared snippet repository.
- **Owners**: Platform team

## 2. Core Principles
- API-first — every feature is accessible via REST API; any client can be built on top
- Stateless by default — horizontal scaling without session affinity
- Explicit error responses — every error returns a structured JSON body with a machine-readable code
- Data portability — all snippets exportable via a single API call

## 3. Technology Constraints
### Mandatory Stack
- Language: Python 3.12+
- Web framework: FastAPI
- Database: SQLite (development), PostgreSQL (production) via SQLAlchemy
- Testing: pytest with httpx for async API tests

### Forbidden Technologies
- No synchronous database drivers (use asyncpg/aiosqlite, not psycopg2)
- No class-based views (FastAPI dependency injection only)
- No hardcoded secrets — all configuration via environment variables
- No XML responses — JSON only, consistent envelope format

## 4. Testing Strategy
- Unit tests: pytest with pytest-asyncio for all service functions
- Integration tests: httpx AsyncClient against the FastAPI app
- Test database: separate SQLite file per test run (random tmpfile)
- Coverage target: 90% (enforced via pytest-cov in CI)

## 5. Definition of Done
- [ ] Code passes linter (ruff) and type checker (mypy strict)
- [ ] Verify step passes (pytest with httpx client)
- [ ] Unit + integration tests pass (pytest -v --cov)
- [ ] API docs render at /docs (FastAPI auto-generated OpenAPI)
- [ ] No hardcoded secrets or credentials in code
```

---

## spec.md (example)

```markdown
# Feature Specification: Snippet API
> Technology-agnostic | Version: 1.0 | Status: Approved

## Overview
A REST API where developers can create, read, search, update, and delete code snippets. Each snippet has a title, language tag, code body, and optional description.

## Users & Personas
- **Developer**: Creates and searches snippets by language or keyword
- **Team Lead**: Organizes snippets into collections and manages team access (v2)

## Functional Requirements

### REQ-001: Create Snippet
**User Story**:
> As a Developer, I want to create a code snippet with a title, language, and code body so that I can save useful code patterns.

**Acceptance Criteria**:
- WHEN a client POSTs valid snippet data (title, language, code) THEN the system SHALL return the created snippet with a unique ID and 201 status
- WHEN a client POSTs snippet data without a title THEN the system SHALL return 422 with a validation error
- WHEN a client POSTs snippet data with an unsupported language tag THEN the system SHALL return 422 with a list of supported languages

### REQ-002: Search Snippets
**User Story**:
> As a Developer, I want to search snippets by language or keyword so that I can find relevant code quickly.

**Acceptance Criteria**:
- WHEN a client GETs /snippets?q=sorting&lang=python THEN the system SHALL return snippets matching both query and language
- WHEN a client GETs /snippets?q=sorting THEN the system SHALL return snippets matching the query across title, description, and code
- WHEN a client GETs /snippets with no query params THEN the system SHALL return the 20 most recent snippets
- WHEN no snippets match THEN the system SHALL return an empty array (not 404)

### REQ-003: Get Snippet by ID
**User Story**:
> As a Developer, I want to view a specific snippet by its ID so that I can see its full content.

**Acceptance Criteria**:
- WHEN a client GETs /snippets/{id} for an existing snippet THEN the system SHALL return the full snippet with 200 status
- WHEN a client GETs /snippets/{id} for a non-existent snippet THEN the system SHALL return 404 with a descriptive message

## Non-Functional Requirements
- **Performance**: Search returns results in < 200ms for up to 10,000 snippets
- **Consistency**: Snippets are immutable after creation (only title/description can be updated)
- **JSON envelope**: Every response is wrapped in { "status": "ok"|"error", "data": ..., "error": ... }

## Out of Scope
- User authentication and authorization (v2)
- Snippet collections/grouping (v2)
- Syntax highlighting for code preview (v2 — frontend concern)
```

---

## requirements.md (example)

```markdown
# Requirements Traceability — SnippetVault

## REQ-001: Create Snippet
| Attribute | Value |
|-----------|-------|
| Priority | P0 |
| Effort | S (3h) |
| Dependencies | None |
| Risk | Low — standard CRUD pattern |
| Verification | pytest -k test_create_snippet |

## REQ-002: Search Snippets
| Attribute | Value |
|-----------|-------|
| Priority | P0 |
| Effort | M (5h) |
| Dependencies | REQ-001 (data must exist to search) |
| Risk | Low — SQL LIKE + language filter |
| Verification | pytest -k test_search_snippets |

## REQ-003: Get Snippet by ID
| Attribute | Value |
|-----------|-------|
| Priority | P0 |
| Effort | S (1h) |
| Dependencies | REQ-001 |
| Risk | Low — simple SELECT by PK |
| Verification | pytest -k test_get_snippet_by_id |
```

---

## research.md (example)

```markdown
# Research — SnippetVault

## Research Question 1: FastAPI vs Flask for async API
**Question**: Should we use FastAPI or Flask for the REST API layer?

**Investigation**:
- FastAPI has built-in async support, OpenAPI generation, and type validation via Pydantic
- Flask is more mature but requires extensions for async, validation (Flask-RESTful), and OpenAPI
- FastAPI's dependency injection reduces boilerplate compared to Flask's request globals
- Both have excellent SQLAlchemy integration

**Decision**: Use FastAPI. Built-in OpenAPI docs (/docs) and Pydantic validation reduce implementation effort.

## Research Question 2: Full-text search strategy
**Question**: Should we use SQLite FTS5, PostgreSQL tsvector, or Elasticsearch for snippet search?

**Investigation**:
- SQLite FTS5 is sufficient for < 10K snippets (no external dependency)
- PostgreSQL tsvector is more powerful but couples us to PostgreSQL even in dev
- Elasticsearch is overkill for the expected scale and adds operational complexity
- SQLite FTS5 with MATCH queries is simple and effective

**Decision**: Use SQLite FTS5 in development, PostgreSQL tsvector in production, abstracted behind a SearchService interface.

## Open Questions
- Should we support snippet fork/versioning? (deferred, requires storage impact analysis)
- Should we support image attachments? (deferred, separate attachment service)
```

---

## plan.md (example)

```markdown
# Technical Plan — SnippetVault API
> Implements: spec.md | Stack defined in constitution.md

## Tech Stack
| Layer | Technology | Justification |
|-------|-----------|---------------|
| Language | Python 3.12+ | Team standard, async-native |
| Web Framework | FastAPI | Async, OpenAPI gen, type validation |
| ORM | SQLAlchemy 2.0 (async) | Mature, async-native, multi-DB |
| Validation | Pydantic v2 | Built into FastAPI, fast |
| Testing | pytest + httpx | Async test client |
| Database | SQLite dev / PostgreSQL prod | Standard progression |
| Program Type | API | REST JSON API |

## Architecture
```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│ HTTP Client  │────▶│  FastAPI App  │────▶│ SQLAlchemy  │
│ (curl,axios) │◀────│  (async)     │◀────│  (async)    │
└─────────────┘     └──────────────┘     └──────┬──────┘
                                                │
                                        ┌───────┴───────┐
                                        │  SQLite / Pg   │
                                        └───────────────┘
```

## Purity Boundaries
| Component | Type | Reason |
|-----------|------|--------|
| `schemas.py` (Pydantic models) | Pure | Data validation, no I/O |
| `models.py` (SQLAlchemy models) | Pure | ORM mapping, no I/O |
| `repository.py` (DB access) | Impure | Database queries (I/O) |
| `service.py` (business logic) | Pure | Orchestrates repository calls |
| `router.py` (API endpoints) | Impure | HTTP request/response (I/O) |
| `search_service.py` | Impure | FTS/tsvector queries (I/O) |

## Implementation Phases

### Phase 1: Foundation (2 days)
- Project scaffolding (FastAPI app, SQLAlchemy setup, config)
- Database models and migrations
- Requirements: prerequisites for REQ-001

### Phase 2: Core CRUD (3 days)
- Create snippet endpoint
- Get snippet by ID endpoint
- Search snippets endpoint
- Requirements: REQ-001, REQ-002, REQ-003

### Phase 3: Testing & Polish (2 days)
- Full test coverage for all endpoints
- Error handling middleware
- Performance testing with 10K snippets
- OpenAPI docs customization

## Risks
| Risk | Mitigation |
|------|-----------|
| SQLite vs PostgreSQL differences | Abstract DB access behind repository pattern |
| Search performance at scale | SQLite FTS5 is fast enough for < 10K, plan migration to PG tsvector |
| No auth in v1 | Rate limiting per IP to prevent abuse until auth ships |
```

---

## data-model.md (example)

```markdown
# Data Model — SnippetVault

## Entity: Snippet

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, default uuid4 | Unique snippet identifier |
| title | VARCHAR(200) | NOT NULL | Snippet title |
| description | TEXT | NULLABLE | Optional description |
| language | VARCHAR(30) | NOT NULL, INDEX | Programming language tag |
| code | TEXT | NOT NULL | The snippet code body |
| created_at | TIMESTAMP | NOT NULL, default now | Creation timestamp |
| updated_at | TIMESTAMP | NOT NULL, auto-update | Last modification timestamp |

## Indexes & Constraints
- PK: id (UUID, auto-generated)
- INDEX: language (for filtering by language)
- INDEX: created_at (for recent-snippets query)
- CONSTRAINT: title must not be empty (validation in Pydantic schema)
- CONSTRAINT: language must be one of supported_languages list

## Example: SQLAlchemy Model
```python
class Snippet(Base):
    __tablename__ = "snippets"

    id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), primary_key=True, default=uuid.uuid4
    )
    title: Mapped[str] = mapped_column(String(200), nullable=False)
    description: Mapped[Optional[str]] = mapped_column(Text, nullable=True)
    language: Mapped[str] = mapped_column(String(30), nullable=False, index=True)
    code: Mapped[str] = mapped_column(Text, nullable=False)
    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), server_default=func.now()
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), server_default=func.now(), onupdate=func.now()
    )
```

## Example: Pydantic Schema
```python
class SnippetCreate(BaseModel):
    title: str = Field(..., min_length=1, max_length=200)
    description: str | None = None
    language: str = Field(..., pattern=r"^(python|javascript|go|rust|typescript|java)$")
    code: str = Field(..., min_length=1)

class SnippetResponse(BaseModel):
    id: uuid.UUID
    title: str
    description: str | None
    language: str
    code: str
    created_at: datetime
    updated_at: datetime
```
```

---

## interface-contracts/api.md (example)

```markdown
# API Interface — SnippetVault

## Interface Type: API (REST)

## Base URL
```
http://localhost:8000/api/v1
```

## Endpoints

### POST /snippets
Create a new snippet.

**Request Body:**
```json
{
  "title": "Bubble sort in Python",
  "description": "Classic bubble sort implementation",
  "language": "python",
  "code": "def bubble_sort(arr):\n    for i in range(len(arr)):\n        for j in range(len(arr)-1):\n            if arr[j] > arr[j+1]:\n                arr[j], arr[j+1] = arr[j+1], arr[j]\n    return arr"
}
```

**Response** (201):
```json
{
  "status": "ok",
  "data": {
    "id": "a1b2c3d4-...",
    "title": "Bubble sort in Python",
    "description": "Classic bubble sort implementation",
    "language": "python",
    "code": "def bubble_sort(arr):\n    ...",
    "created_at": "2025-01-15T10:30:00Z",
    "updated_at": "2025-01-15T10:30:00Z"
  }
}
```

**Errors**:
| Status | Condition |
|--------|-----------|
| 422 | Missing title, unsupported language, empty code |
| 500 | Database error |

### GET /snippets
Search or list snippets.

**Query Parameters:**
| Param | Type | Required | Description |
|-------|------|----------|-------------|
| q | string | No | Full-text search query |
| lang | string | No | Filter by language tag |
| limit | int | No | Max results (default 20, max 100) |

**Response** (200):
```json
{
  "status": "ok",
  "data": {
    "snippets": [
      {
        "id": "a1b2c3d4-...",
        "title": "Bubble sort in Python",
        "language": "python",
        "created_at": "2025-01-15T10:30:00Z"
      }
    ],
    "total": 1
  }
}
```

### GET /snippets/{id}
Get a snippet by its UUID.

**Response** (200): Full snippet object.
**Error** (404): `{ "status": "error", "error": { "code": "NOT_FOUND", "message": "Snippet not found" } }`
```

---

## tasks.md (example)

```markdown
# Implementation Tasks — SnippetVault

## Phase 1: Foundation

- [ ] [T001] Initialize FastAPI project with async SQLAlchemy — S
  - **Build**: `ruff check .` passes with 0 errors
  - **Verify**: `uvicorn app.main:app --port 8000` starts and /docs returns 200
  - **Gate**: App starts, /docs renders OpenAPI

- [ ] [T002] Define Pydantic schemas (SnippetCreate, SnippetResponse) — S (REQ-001)
  - **Build**: `mypy app/ --strict` passes
  - **Verify**: `pytest -k test_schema_validation -v` passes
  - **Gate**: Schemas include all fields with type annotations

- [ ] [T003] Define SQLAlchemy model and create migration — S (REQ-001)
  - **Build**: `python -m alembic upgrade head` succeeds
  - **Verify**: `sqlite3 test.db .schema` shows snippets table
  - **Gate**: Table has all required columns with correct types

## Phase 2: Core CRUD

- [ ] [T004][P] Implement repository layer (create, get, search) — M (REQ-001, REQ-002, REQ-003)
  - **Build**: `ruff check .` passes
  - **Verify**: `pytest -k test_repository -v` passes
  - **Gate**: All CRUD operations work against SQLite

- [ ] [T005][P] Implement POST /api/v1/snippets endpoint — M (REQ-001)
  - **Build**: `mypy app/ --strict` passes
  - **Verify**: `curl -X POST localhost:8000/api/v1/snippets -H "Content-Type: application/json" -d '{"title":"test","language":"python","code":"print()"}'` returns 201
  - **Gate**: Snippet is persisted and queryable

- [ ] [T006] Implement GET /api/v1/snippets search endpoint — M (REQ-002)
  - **Build**: `ruff check .` passes
  - **Verify**: `pytest -k test_search -v` passes
  - **Gate**: Search by keyword and language works independently and combined

- [ ] [T007] Implement GET /api/v1/snippets/{id} endpoint — S (REQ-003)
  - **Build**: `ruff check .` passes
  - **Verify**: `curl localhost:8000/api/v1/snippets/a1b2c3d4` returns 404 (non-existent)
  - **Gate**: Existing snippet returns 200, non-existent returns 404

## Phase 3: Testing & Polish

- [ ] [T008] Write full test suite for all endpoints — M
  - **Build**: `pytest -v --cov` passes with 90%+ coverage
  - **Verify**: All 15+ tests pass (sad paths + happy paths)
  - **Gate**: Coverage report shows >= 90%

- [ ] [T009] Add error handling middleware for consistent JSON errors — S
  - **Build**: `ruff check .` passes
  - **Verify**: `curl -v localhost:8000/api/v1/nonexistent` returns JSON error, not HTML
  - **Gate**: All errors return `{ status, error: { code, message } }` envelope

- [ ] [T010] Load test with 10K snippets — S
  - **Build**: Seed script inserts 10K snippets without error
  - **Verify**: `GET /snippets?q=search` returns in < 200ms
  - **Gate**: P95 response time under 200ms

---

## Progress: 0/10 tasks complete
```

---

## Example: "LegacyPay" — Extracting a Spec from a Legacy PHP Monolith

---

## Legacy Analysis (example)

```markdown
# Legacy Analysis — LegacyPay

## Source Code Examined
- Repository: github.com/company/payments-legacy
- Language: PHP 7.4 (no types, no tests)
- Age: ~8 years, ~50K lines
- Database: MySQL 5.7

## Extraction Process
1. **Read entry points** (public/index.php, CLI scripts) — identify user-facing features
2. **Trace data flow** — map HTTP routes → controller methods → SQL queries
3. **Infer intent** — from column names, comments, and client usage patterns
4. **Interview stakeholders** — confirm observed behavior matches expected behavior
5. **Write spec** — capture *actual* behavior as the baseline contract

## Key Findings
- The "refund" endpoint actually archives the transaction (doesn't reverse it)
- Error responses vary by controller — some return JSON, some HTML, some plain text
- Three different date formats in use across the codebase
- No input validation on 40% of endpoints (trusting client-side only)
```

## Tech Debt Inventory (example)

```markdown
# Tech Debt Inventory — LegacyPay

| ID | Issue | Location | Severity | Impact |
|----|-------|----------|----------|--------|
| TD-001 | No input validation on POST /payment | PaymentController.php | Critical | SQL injection risk |
| TD-002 | Inconsistent error format (JSON vs HTML) | All controllers | High | Client parsing failures |
| TD-003 | Hardcoded database credentials | config.php | Critical | Security breach risk |
| TD-004 | No transaction atomicity | PaymentService.php | High | Partial writes on failure |
| TD-005 | Mixed date formats (Y-m-d, d/m/Y, timestamp) | DateHelper.php | Medium | Reporting inaccuracies |
| TD-006 | Dead code: legacy SOAP client (unused 3+ years) | soap/ | Low | Maintenance overhead |
```

---

## constitution.md (example)

```markdown
# Project Constitution — LegacyPay (Modernized)

## 1. Project Identity
- **Name**: LegacyPay — Modernization Initiative
- **Purpose**: Extract the specification from an undocumented PHP payment monolith and incrementally migrate to a maintainable architecture.
- **Owners**: Platform team

## 2. Core Principles
- Behavior-preserving — the new system matches the old system's observable behavior exactly (including known bugs, initially)
- Strangler fig — new code grows alongside old code; no big-bang rewrite
- Test-first — every extracted module gets tests before any refactoring
- Transparency — all extracted specs are reviewed with stakeholders for accuracy

## 3. Technology Constraints
### Mandatory Stack
- New services: Go 1.22+ (performance, static typing)
- API gateway: Envoy (route traffic to old vs new services)
- Database: PostgreSQL 16 (migrate from MySQL 5.7)
- Testing: Go testing + testify + dockertest for integration

### Migration Constraints
- Zero downtime during migration
- Old PHP monolith stays in production until all traffic is migrated
- Every new service must pass the legacy test suite before cutover

## 4. Testing Strategy
- Characterization tests: capture current behavior before changes
- Integration tests: dockertest with real PostgreSQL
- Canary testing: route 1% of production traffic to new services

## 5. Definition of Done
- [ ] Legacy characterization test suite passes on new code
- [ ] No regression in existing behavior
- [ ] New service compiles and passes all tests
- [ ] Tech debt item(s) for this module are resolved or documented
```

---

## spec.md (example)

```markdown
# Feature Specification: Payment Processing (Legacy Baseline)
> Technology-agnostic | Version: 1.0 — Legacy Extraction | Status: Draft

## Overview
Process customer payments and refunds. This spec captures the *actual* behavior of the legacy PHP monolith as the baseline contract for modernization.

**Note**: Some behaviors below are buggy or undesirable. They are documented as-is to ensure the new system matches before improvements are introduced.

## Users & Personas
- **Customer**: Initiates payments and refunds via the web frontend
- **Support Agent**: Manually processes failed payments via admin panel
- **Batch Processor**: Scheduled job that retries failed payments

## Functional Requirements

### REQ-001: Process Payment
**User Story**:
> As a Customer, I want to submit a payment so that my order is completed.

**Acceptance Criteria** (Legacy Baseline):
- WHEN a customer submits a payment with valid card details THEN the system SHALL record the transaction and return HTTP 200 with JSON body `{ "status": "success", "transaction_id": "..." }`
- WHEN a customer submits a payment with invalid card details THEN the system SHALL return HTTP 200 with JSON body `{ "status": "failure", "message": "Card declined" }` (note: HTTP 200 even on failure — legacy constraint)
- WHEN a customer submits a payment without an amount THEN the system SHALL return HTTP 500 (known bug: missing validation)
- WHEN the database is unreachable THEN the system SHALL return HTTP 500 with HTML error page (known bug: no JSON error handler)

### REQ-002: Process Refund
**User Story**:
> As a Support Agent, I want to refund a payment so that the customer gets their money back.

**Acceptance Criteria** (Legacy Baseline):
- WHEN a support agent submits a refund request THEN the system SHALL mark the transaction as `archived` (not "reversed" — known behavior)
- WHEN a refund is for an already-archived transaction THEN the system SHALL return `{ "status": "already_archived" }` and take no action

## Out of Scope (Deferred)
- Input validation improvements (Phase 2 after baseline match)
- Consistent error format (Phase 3 after all services migrated)
```
---

## plan.md (example)

```markdown
# Technical Plan — LegacyPay Migration
> Implements: spec.md | Strategy: Strangler Fig

## Migration Architecture
```
┌──────────────┐       ┌──────────────┐
│   Envoy GW   │──────▶│  PHP Monolith │ (existing)
│  (router)    │       └──────────────┘
│              │       ┌──────────────┐
│              │──┐───▶│  Go Service  │ (new)
│              │  │    │  - payments  │
│              │  │    └──────────────┘
│              │  │    ┌──────────────┐
│              │  └───▶│  Go Service  │ (new)
│              │       │  - refunds   │
└──────────────┘       └──────────────┘
```

## Tech Stack
| Layer | New (Go) | Old (PHP) |
|-------|----------|-----------|
| Language | Go 1.22 | PHP 7.4 |
| Database | PostgreSQL 16 | MySQL 5.7 |
| Transport | gRPC + REST | REST |
| Testing | Go test + dockertest | None |

## Implementation Phases

### Phase 1: Characterization (lowest risk)
| Task | Risk | Scope |
|------|------|-------|
| Write characterization tests for payment flow | Low | Read-only, no prod impact |
| Document all error responses per endpoint | Low | Analysis only |
| Extract database schema to migration scripts | Low | Read-only |

### Phase 2: New Payment Service (medium risk)
| Task | Risk | Scope |
|------|------|-------|
| Build Go payment service | Medium | New code, parallel-run with old |
| Dual-write payments to both systems | Medium | Requires sync mechanism |
| Compare outputs in staging | Low | Automated diff |

### Phase 3: Cutover (highest risk — deferred)
| Task | Risk | Scope |
|------|------|-------|
| Envoy route payment traffic to new service | High | Production cutover |
| Monitor for errors and rollback if needed | High | Requires rollback plan |
| Decommission old payment code | Medium | After stability verified |

## Purity Boundaries
| Component | Type | Reason |
|-----------|------|--------|
| Legacy spec extraction | Pure | Documenting existing behavior, no changes |
| Characterization tests | Pure | Read-only, no side effects |
| Envoy routing config | Impure | Affects production traffic |
| Dual-write sync | Impure | Writes to two databases |
```

---

## tasks.md (example)

```markdown
# Implementation Tasks — LegacyPay Migration

## Phase 1: Characterization (Risk: Low)

- [ ] [T001] Write characterization tests for payment flow — S
  - **Build**: `go build ./...` compiles without errors
  - **Verify**: `go test ./characterization/... -v` passes against legacy PHP API
  - **Gate**: All legacy behaviors captured as test assertions

- [ ] [T002] Document all endpoint error responses — S
  - **Build**: Markdown list in `specs/legacy-errors.md`
  - **Verify**: Each endpoint has at least one success and one error documented
  - **Gate**: No undocumented endpoints remain

- [ ] [T003] Extract MySQL schema to migration files — S
  - **Build**: `golang-migrate` source files created
  - **Verify**: `golang-migrate up` creates identical tables in PostgreSQL
  - **Gate**: All tables and indexes present in PostgreSQL

## Phase 2: New Payment Service (Risk: Medium)

- [ ] [T004] Build Go payment service with REST endpoints — M
  - **Build**: `go build ./services/payments/...` passes
  - **Verify**: `go test ./services/payments/...` passes against test PostgreSQL
  - **Gate**: Endpoints match legacy API contract exactly

- [ ] [T005] Implement dual-write strategy (write to both old and new DB) — M
  - **Build**: `go vet ./...` passes
  - **Verify**: Both databases have identical rows after write (integration test)
  - **Gate**: Data consistency verified in staging

- [ ] [T006] Add comparison tool for staging verification — S
  - **Build**: `go build ./cmd/compare` passes
  - **Verify**: Tool runs against staging and reports 0 differences
  - **Gate**: Able to detect drift between old and new

## Phase 3: Cutover (Risk: High)

- [ ] [T007] Configure Envoy routing for payment traffic — S
  - **Build**: Envoy config passes validation (`envoy --mode validate`)
  - **Verify**: Canary traffic (1%) routes to new service without errors
  - **Gate**: 100% of canary requests succeed

- [ ] [T008] Monitor and rollback plan — S
  - **Build**: Rollback script tested in staging
  - **Verify**: `make rollback` restores old routing within 30 seconds
  - **Gate**: Rollback verified in dry-run

- [ ] [T009] Decommission old payment code — S
  - **Build**: Remove old PHP payment controller
  - **Verify**: All traffic routes to Go service for 7 days with zero incidents
  - **Gate**: Old code removal approved by team lead

---

## Progress: 0/9 tasks complete
```

---

## Example: "EventStream" — Event-Driven Architecture with AsyncAPI

---

## constitution.md (example)

```markdown
# Project Constitution — EventStream

## 1. Project Identity
- **Name**: EventStream
- **Purpose**: A real-time event processing platform that ingests, enriches, and routes domain events between microservices using Apache Kafka.
- **Owners**: Platform infrastructure team

## 2. Core Principles
- Eventual consistency — services are eventually consistent; no distributed transactions
- Schema-first — every event has a versioned CloudEvents schema published before any producer or consumer
- At-least-once delivery — consumers must be idempotent (duplicate events are expected and safe)
- No synchronous coupling — services only communicate via events; no HTTP between services

## 3. Technology Constraints
### Mandatory Stack
- Message broker: Apache Kafka 3.6+ with Kraft (no Zookeeper)
- Event format: CloudEvents 1.0 with JSON encoding
- Services: Go 1.22+ with Confluent Kafka client
- Schema registry: Confluent Schema Registry (Avro for binary, JSON Schema for JSON)
- Containers: Docker + docker-compose for local dev, Kubernetes for production

### Forbidden Technologies
- No HTTP calls between services (events only)
- No shared databases between services
- No synchronous transactions spanning multiple services
- No manual topic creation (IaC with Terraform or Kubernetes operator)

## 4. Testing Strategy
- Unit tests: Go testing for individual service logic
- Integration tests: testcontainers-go with real Kafka + Schema Registry
- Contract tests: verify event schemas match between producer and consumer
- End-to-end tests: docker-compose with full service topology

## 5. Definition of Done
- [ ] Event schema published and registered in Schema Registry
- [ ] Producer service emits the event correctly
- [ ] Consumer service processes the event with idempotency
- [ ] Integration test passes with real Kafka
- [ ] E2E docker-compose test passes end-to-end
```

---

## spec.md (example)

```markdown
# Feature Specification: Order Lifecycle Events
> Technology-agnostic | Version: 1.0 | Status: Approved

## Overview
When a customer places an order, the Order Service emits events that other services consume: Inventory (reserve stock), Billing (charge customer), Notification (email confirmation), and Analytics (track metrics).

## Users & Personas
- **Order Service (Producer)**: Emits order.created, order.paid, order.shipped, order.cancelled
- **Inventory Service (Consumer)**: Reserves/releases stock on order.created / order.cancelled
- **Billing Service (Consumer)**: Charges customer on order.paid, issues refund on order.cancelled
- **Notification Service (Consumer)**: Sends email on order.created and order.shipped
- **Analytics Service (Consumer)**: Records all events for dashboards

## Functional Requirements

### REQ-001: Emit order.created Event
**User Story**:
> As the Order Service, I want to emit an order.created event when a customer places an order so that downstream services can react.

**Acceptance Criteria**:
- WHEN a customer successfully places an order THEN the Order Service SHALL emit an event with type `order.created` and a CloudEvents-compliant payload
- WHEN the event is emitted THEN it SHALL contain order_id, customer_id, items[], total_amount, and timestamp
- WHEN the Kafka cluster is unavailable THEN the Order Service SHALL retry with exponential backoff (max 3 retries) before failing the order

### REQ-002: Consume order.created in Inventory
**User Story**:
> As the Inventory Service, I want to consume order.created events so that I can reserve stock for the order items.

**Acceptance Criteria**:
- WHEN the Inventory Service receives an order.created event THEN it SHALL decrement the reserved stock for each item
- WHEN an item has insufficient stock THEN the Inventory Service SHALL emit an inventory.reservation_failed event
- WHEN a duplicate order.created event is received THEN the Inventory Service SHALL be idempotent (no double-decrement)

### REQ-003: Emit order.paid Event
**User Story**:
> As the Billing Service, I want to emit an order.paid event after successful charge so that downstream services can proceed.

**Acceptance Criteria**:
- WHEN the Billing Service successfully charges the customer THEN it SHALL emit an order.paid event
- WHEN the charge fails THEN the Billing Service SHALL emit a billing.charge_failed event (not order.paid)

## Non-Functional Requirements
- **Throughput**: System handles 10,000 events/second at peak (Black Friday)
- **Latency**: Event produced to event consumed in < 500ms (P99)
- **Durability**: No event loss under any failure scenario (acks=all, min.insync.replicas=2)

## Out of Scope
- Event replay / reprocessing UI (v2)
- Dead letter queue management (v2)
- Schema evolution governance beyond compatibility checks (v2)
```

---

## plan.md (example)

```markdown
# Technical Plan — EventStream Order Lifecycle
> Implements: spec.md | Stack defined in constitution.md

## Service Topology
```
┌──────────────┐    ┌──────────────┐    ┌──────────────────┐
│  Order Svc   │───▶│   Kafka      │◀───│  Schema Registry  │
│  (Go)        │    │  (3 brokers) │    │  (Avro + JSON)    │
└──────────────┘    └──────┬───────┘    └──────────────────┘
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
       ┌──────────┐ ┌──────────┐ ┌──────────┐
       │Inventory │ │ Billing  │ │Notific.  │
       │(Go)      │ │(Go)      │ │(Go)      │
       └──────────┘ └──────────┘ └──────────┘
```

## Tech Stack
| Layer | Technology |
|-------|-----------|
| Services | Go 1.22+ |
| Message Broker | Apache Kafka 3.6+ (Kraft) |
| Event Format | CloudEvents 1.0 (JSON) |
| Schema Registry | Confluent Schema Registry |
| Client Library | confluent-kafka-go |
| Testing | testcontainers-go |
| Orchestration | Docker Compose (dev) / K8s (prod) |

## Implementation Phases

### Phase 1: Infrastructure (3 days)
- Kafka cluster with docker-compose
- Schema Registry setup
- Topic creation (IaC scripts)
- Requirements: infrastructure prerequisites for REQ-001

### Phase 2: Core Events (5 days)
- Order Service: emit order.created
- Inventory Service: consume order.created, emit inventory.reservation_failed
- Billing Service: consume order.created, emit order.paid / billing.charge_failed
- Notification Service: consume order.created + order.paid
- Requirements: REQ-001, REQ-002, REQ-003

### Phase 3: Testing & Resilience (3 days)
- Idempotency testing (replay events, verify no side effects)
- Failure scenario testing (Kafka broker down, service crash)
- Load testing (10K events/sec)
- E2E docker-compose test suite

## Purity Boundaries
| Component | Type | Reason |
|-----------|------|--------|
| Event schema definitions | Pure | No I/O, only data structures |
| Event serialization/deserialization | Pure | Transform in/out, no side effects |
| Kafka producer wrapper | Impure | Network I/O to Kafka broker |
| Kafka consumer loop | Impure | Network I/O, stateful offset management |
| Stock reservation logic | Pure | Pure business logic, testable without Kafka |
| Idempotency check | Impure | Reads/writes to state store (I/O) |
```

---

## data-model.md (example)

```markdown
# Data Model — EventStream

## Event Schema (CloudEvents 1.0)

### order.created
```json
{
  "specversion": "1.0",
  "type": "com.eventstream.order.created",
  "source": "/order-service/v1",
  "id": "a1b2c3d4-...",
  "time": "2025-01-15T10:30:00Z",
  "datacontenttype": "application/json",
  "data": {
    "order_id": "ord_12345",
    "customer_id": "cus_67890",
    "items": [
      { "sku": "ABC-123", "quantity": 2, "unit_price": 19.99 }
    ],
    "total_amount": 39.98,
    "currency": "USD",
    "timestamp": "2025-01-15T10:30:00Z"
  }
}
```

### order.paid
```json
{
  "specversion": "1.0",
  "type": "com.eventstream.order.paid",
  "source": "/billing-service/v1",
  "id": "e5f6g7h8-...",
  "time": "2025-01-15T10:30:05Z",
  "datacontenttype": "application/json",
  "data": {
    "order_id": "ord_12345",
    "transaction_id": "txn_98765",
    "amount": 39.98,
    "currency": "USD",
    "charged_at": "2025-01-15T10:30:05Z"
  }
}
```

### inventory.reservation_failed
```json
{
  "specversion": "1.0",
  "type": "com.eventstream.inventory.reservation_failed",
  "source": "/inventory-service/v1",
  "id": "i9j0k1l2-...",
  "time": "2025-01-15T10:30:02Z",
  "datacontenttype": "application/json",
  "data": {
    "order_id": "ord_12345",
    "failed_items": [
      { "sku": "ABC-123", "requested": 2, "available": 0 }
    ],
    "reason": "insufficient_stock"
  }
}
```

## Topics
| Topic | Partitions | Retention | Cleanup Policy |
|-------|-----------|-----------|----------------|
| order.events | 6 | 7 days | delete |
| inventory.events | 3 | 3 days | delete |
| billing.events | 3 | 30 days | compact (audit) |
| notification.events | 1 | 1 day | delete |

## Indexes & Constraints
For event-driven systems, indexes apply to state stores in consumer services:
- Consumer offset: committed to `__consumer_offsets` topic (managed by Kafka)
- Idempotency key: order_id stored in consumer state store (unique constraint)
- Event ID: globally unique across all services (UUID v4)
```

---

## interface-contracts/asyncapi.md (example)

```markdown
# AsyncAPI Interface — EventStream

## Interface Type: AsyncAPI (Event-Driven)

## Server
```
production:
  host: kafka-cluster.prod.example.com:9092
  protocol: kafka
  description: Production Kafka cluster (3 brokers, min.insync.replicas=2)
```

## Channels

### channel: order.events
Publish-subscribe channel for order lifecycle events.

**Publish** (Order Service):
```
subscribe:
  operationId: emitOrderCreated
  message:
    $ref: '#/components/messages/OrderCreated'
```

**Subscribe** (Inventory, Billing, Notification Services):
```
publish:
  operationId: consumeOrderEvents
  message:
    oneOf:
      - $ref: '#/components/messages/OrderCreated'
      - $ref: '#/components/messages/OrderPaid'
      - $ref: '#/components/messages/OrderCancelled'
```

### channel: inventory.events
Events emitted by the Inventory Service.

**Publish**:
```
subscribe:
  operationId: emitReservationFailed
  message:
    $ref: '#/components/messages/ReservationFailed'
```

## Components

### OrderCreated
```
payload:
  type: object
  properties:
    order_id: { type: string, pattern: "^ord_" }
    customer_id: { type: string, pattern: "^cus_" }
    items:
      type: array
      items:
        type: object
        properties:
          sku: { type: string }
          quantity: { type: integer, minimum: 1 }
          unit_price: { type: number, exclusiveMinimum: 0 }
    total_amount: { type: number }
    currency: { type: string, enum: ["USD", "EUR", "GBP"] }
```

### ReservationFailed
```
payload:
  type: object
  properties:
    order_id: { type: string }
    failed_items:
      type: array
      items:
        type: object
        properties:
          sku: { type: string }
          requested: { type: integer }
          available: { type: integer }
    reason: { type: string, enum: ["insufficient_stock", "sku_not_found"] }
```
```

---

## tasks.md (example)

```markdown
# Implementation Tasks — EventStream

## Phase 1: Infrastructure

- [ ] [T001] Set up Kafka cluster with docker-compose (3 brokers, Kraft) — S
  - **Build**: `docker-compose up -d` starts all 3 brokers
  - **Verify**: `kafka-topics --bootstrap-server localhost:9092 --list` succeeds
  - **Gate**: All brokers show in `kafka-broker-api-versions`

- [ ] [T002] Set up Schema Registry with docker-compose — S
  - **Build**: `docker-compose up -d schema-registry`
  - **Verify**: `curl localhost:8081/subjects` returns `[]`
  - **Gate**: Schema Registry is health at / health endpoint

- [ ] [T003] Create topics via Terraform or scripts — S
  - **Build**: `terraform apply` creates order.events, inventory.events, billing.events
  - **Verify**: `kafka-topics --describe --topic order.events` shows 6 partitions, RF=3
  - **Gate**: All expected topics exist with correct config

## Phase 2: Core Events

- [ ] [T004][P] Define CloudEvents schemas and register in Schema Registry — M
  - **Build**: `make register-schemas` succeeds
  - **Verify**: `curl localhost:8081/subjects/order.created-value/versions` returns version 1
  - **Gate**: All event types registered (order.created, order.paid, order.cancelled, reservation_failed)

- [ ] [T005] Build Order Service: emit order.created event — M (REQ-001)
  - **Build**: `go build ./services/order/...` passes
  - **Verify**: `go test ./services/order/... -v` with testcontainers passes
  - **Gate**: Event appears in Kafka topic (verified via test consumer)

- [ ] [T006] Build Inventory Service: consume order.created, emit reservation_failed — M (REQ-002)
  - **Build**: `go build ./services/inventory/...` passes
  - **Verify**: `go test ./services/inventory/... -v` with testcontainers passes
  - **Gate**: Insufficient stock triggers reservation_failed event

- [ ] [T007] Build Billing Service: consume order.created, emit order.paid — M (REQ-003)
  - **Build**: `go build ./services/billing/...` passes
  - **Verify**: `go test ./services/billing/... -v` with testcontainers passes
  - **Gate**: Successful charge emits order.paid, failed charge emits billing.charge_failed

## Phase 3: Testing & Resilience

- [ ] [T008] Write idempotency tests (replay events, verify no side effects) — M
  - **Build**: `go test ./tests/idempotency/... -v` passes
  - **Verify**: Replaying the same event 3 times produces same state
  - **Gate**: No duplicate processing detected

- [ ] [T009] Write failure scenario tests (broker down, service crash) — M
  - **Build**: `go test ./tests/resilience/... -v` passes
  - **Verify**: Service recovers after broker restart without data loss
  - **Gate**: No events lost during broker outage (consumer resumes from last committed offset)

- [ ] [T010] Load test at 10K events/second — M
  - **Build**: `go run ./cmd/loadtest` produces 10K events/sec for 5 minutes
  - **Verify**: P99 latency < 500ms, zero consumer lag growth
  - **Gate**: No memory leak or OOM during sustained load

---

## Progress: 0/10 tasks complete
```

---

## Example: "HabitStack" — A Next.js Full-Stack Habit Tracker

Stack: **Next.js 14 (App Router) + React + TypeScript + Tailwind CSS + Prisma + PostgreSQL + NextAuth.js**

---

## constitution.md (example)

```markdown
# Project Constitution — HabitStack

## 1. Project Identity
- **Name**: HabitStack
- **Purpose**: A full-stack web application for tracking daily habits. Users create habits (e.g., "Read 30 min"), log daily completions, and view streak-based progress analytics.
- **Owners**: Frontend team

## 2. Core Principles
- Server-rendered by default — use React Server Components unless interactivity requires client JS
- Type-safe end-to-end — Prisma types flow into React components via tRPC or Server Actions
- Mobile-first responsive — Tailwind CSS breakpoints, touch-friendly interactions
- Progressive enhancement — core features work without JavaScript

## 3. Technology Constraints
### Mandatory Stack
- Framework: Next.js 14 (App Router)
- Language: TypeScript 5+
- Styling: Tailwind CSS + shadcn/ui components
- Database: PostgreSQL via Prisma ORM
- Auth: NextAuth.js v5 (Auth.js) with Google OAuth
- Testing: Vitest + React Testing Library + Playwright for E2E

### Forbidden Technologies
- No Redux or context-based global state (use URL/search params for shareable state)
- No raw SQL queries (use Prisma exclusively)
- No client-side data fetching unless SSR is impossible
- No CSS-in-JS libraries (Tailwind utility classes only)

## 4. Testing Strategy
- Unit tests: Vitest for pure functions and validation
- Component tests: React Testing Library with vitest-dom matchers
- API/action tests: Vitest mocking Prisma client
- E2E tests: Playwright for critical user flows (sign-up, create habit, log entry)
- Coverage target: 80% (unit + component), 100% of auth flows

## 5. Definition of Done
- [ ] Code compiles (next build passes with strict TypeScript)
- [ ] Verify step passes (vitest run + lint:next)
- [ ] Unit + component tests pass
- [ ] Mobile viewport renders correctly (≥320px width)
- [ ] No hardcoded secrets — all via env vars (AUTH_SECRET, DATABASE_URL)
- [ ] E2E smoke test passes for the feature
```
---

## spec.md (example)

```markdown
# Specification — HabitStack

## Overview
A web app where users sign up, create habit definitions, log daily completions, and view progress streaks.

## Users & Personas
- **Individual User**: wants to build habits, track streaks, stay motivated
- **Guest**: can view a demo (read-only, no persistence)

## Functional Requirements

### REQ-001: User Registration & Authentication
- **User Story**: As a new user, I want to sign in with Google so that I can start tracking habits without creating another password
- **Acceptance Criteria**:
  - WHEN a user clicks "Sign in with Google" THEN the system SHALL redirect to Google OAuth
  - WHEN Google returns a valid token THEN the system SHALL create or retrieve the user record
  - WHEN authenticated THEN the system SHALL redirect to the dashboard
- **Implementation Notes**: Implemented as 3 atomic tasks: NextAuth config, Prisma adapter, login UI

### REQ-002: Create a Habit
- **User Story**: As a logged-in user, I want to define a new habit so that I can track it daily
- **Acceptance Criteria**:
  - WHEN a user navigates to /habits/new THEN the system SHALL display a form (name, description, icon, target frequency)
  - WHEN the user submits the form THEN the system SHALL create the habit and redirect to /habits
  - WHEN the name is empty THEN the system SHALL show a validation error
- **Implementation Notes**: Implemented as 2 atomic tasks: Server Action + form component, Prisma habit model

### REQ-003: Log Daily Entry
- **User Story**: As a user, I want to mark a habit as done for today so that I can track my streak
- **Acceptance Criteria**:
  - WHEN a user clicks "Log" on a habit card THEN the system SHALL create today's entry
  - WHEN the user logs the same habit twice in one day THEN the system SHALL update the existing entry (not duplicate)
  - WHEN the entry is logged THEN the habit card SHALL show a checkmark for today
- **Implementation Notes**: Implemented as 2 atomic tasks: upsert server action, optimistic UI update

### REQ-004: View Progress & Streaks
- **User Story**: As a user, I want to see my current streak and completion history so that I stay motivated
- **Acceptance Criteria**:
  - WHEN a user visits /dashboard THEN the system SHALL show each habit with: total days, current streak, longest streak
  - WHEN a habit has 7+ consecutive days THEN the system SHALL display a "🔥" icon
  - WHEN there are no habits yet THEN the system SHALL show an empty state with a CTA to create one
- **Implementation Notes**: Implemented as 2 atomic tasks: streak query in Prisma, dashboard server component

## Non-Functional Requirements
- Page load: < 2s on 3G (SSR + streaming)
- Auth session: < 500ms overhead per request
- Database: < 100ms per query (p99)

## Out of Scope
- Mobile app (native or PWA)
- Social features (sharing, following)
- Habit templates library
```

---

## requirements.md (example)

```markdown
# Requirements Checklist — HabitStack

## REQ-001: User Registration & Authentication
- [ ] Google OAuth redirect flow works end-to-end
- [ ] User record created on first login
- [ ] Session persists across page reloads
- [ ] Sign-out clears session

## REQ-002: Create a Habit
- [ ] Form rendered at /habits/new
- [ ] Validation rejects empty name
- [ ] Habit appears on dashboard after creation
- [ ] Form is accessible (labels, focus, error announcements)

## REQ-003: Log Daily Entry
- [ ] Clicking "Log" creates entry for today
- [ ] Second click on same day updates (not duplicates)
- [ ] Checkmark appears immediately (optimistic)
- [ ] Entry persists on page refresh

## REQ-004: View Progress & Streaks
- [ ] Dashboard shows all user habits
- [ ] Current streak computed correctly
- [ ] Fire emoji shown for 7+ day streak
- [ ] Empty state shown when no habits exist
```

---

## research.md (example)

```markdown
# Research — HabitStack

## Research Question 1: Auth strategy for Next.js
- Options: NextAuth.js v5 vs Clerk vs Supabase Auth vs custom
- Decision: **NextAuth.js v5** with Google OAuth + Prisma adapter
- Rationale: Free, self-hosted, Prisma adapter matches our stack, no vendor lock-in

## Research Question 2: Server Actions vs API routes for mutations
- Options: Next.js Server Actions vs traditional API routes
- Decision: **Server Actions** for mutations, **Server Components** for reads
- Rationale: Less boilerplate, co-located types, progressive enhancement by default

## Open Questions
- Should habit ordering be drag-and-drop? Deferred to post-MVP.
- Should we support notification reminders? Deferred.
```

---

## plan.md (example)

```markdown
# Plan — HabitStack

## Tech Stack
| Layer | Technology | Justification |
|-------|-----------|---------------|
| Framework | Next.js 14 (App Router) | SSR + Server Components + API in one project |
| Language | TypeScript 5 | End-to-end type safety with Prisma |
| Database | PostgreSQL + Prisma | Type-safe queries, migrations, relationships |
| Auth | NextAuth.js v5 + Google OAuth | Zero-config OAuth, Prisma adapter |
| Styling | Tailwind CSS + shadcn/ui | Rapid UI, accessible components |
| Hosting | Vercel | Native Next.js support, edge functions |
| Testing | Vitest + Playwright | Fast unit tests + reliable E2E |

## Architecture
```
Browser ←→ Next.js Edge/Server
              ├── Server Components (data fetching via Prisma)
              ├── Server Actions (mutations via Prisma)
              └── NextAuth.js (session via Prisma adapter)
                      ↓
              PostgreSQL (via Prisma ORM)
```
All data fetching happens on the server. Client components only handle interactivity (forms, optimistic UI). No REST API layer — Server Actions are the mutation boundary.

## Purity Boundaries
| Component | Type | Reason |
|-----------|------|--------|
| Prisma schema | Pure | Type definitions, no runtime behavior |
| Streak calculation | Pure | Same input → same output, no I/O |
| Server Actions | Impure | Database writes, auth context |
| Dashboard page | Impure | Database reads, session lookup |
| UI components | Pure | Props in → JSX out (no I/O) |

## Implementation Phases

### Phase 1: Foundation (1 day)
- T001: Next.js project scaffold (App Router, Tailwind, shadcn/ui, TypeScript strict)
- T002: Prisma schema + PostgreSQL setup + first migration
- T003: NextAuth.js v5 config with Google OAuth + Prisma adapter

### Phase 2: Core Features (2 days)
- T004: Habit form (Server Action + validation + form component)
- T005: Dashboard page showing habit list (Server Component)
- T006: Daily log upsert (Server Action + optimistic UI)

### Phase 3: Progress & Streaks (1 day)
- T007: Streak query (Prisma raw query or aggregation)
- T008: Dashboard progress display (streak, fire icon, empty state)

### Phase 4: Polish (1 day)
- T009: Error boundaries and loading skeletons
- T010: E2E tests with Playwright (sign-up, create habit, log entry)

## Risks
- Next.js Server Actions are experimental — API may change
- Prisma cold start on Vercel serverless functions (~300ms overhead)
```

---

## data-model.md (example)

```markdown
# Data Model — HabitStack

## Entity: User
| Field | Type | Constraints |
|-------|------|-------------|
| id | String (UUID) | Primary key |
| name | String | Required |
| email | String | Unique, required |
| image | String? | Nullable (Google avatar) |
| createdAt | DateTime | Auto-generated |

## Entity: Habit
| Field | Type | Constraints |
|-------|------|-------------|
| id | String (UUID) | Primary key |
| userId | String (UUID) | Foreign key → User, indexed |
| name | String | Required, max 100 chars |
| description | String? | Nullable, max 500 chars |
| icon | String | Default: "⭐" |
| targetFrequency | String | Default: "daily" |
| createdAt | DateTime | Auto-generated |

## Entity: Entry
| Field | Type | Constraints |
|-------|------|-------------|
| id | String (UUID) | Primary key |
| habitId | String (UUID) | Foreign key → Habit, indexed |
| date | Date | Required |
| createdAt | DateTime | Auto-generated |

## Indexes & Constraints
- `Entry(habitId, date)` — unique compound index (one entry per habit per day)
- `Habit(userId)` — index for dashboard queries
- `User(email)` — unique index for auth lookup

## Example: Prisma Schema
```prisma
model User {
  id        String   @id @default(uuid())
  name      String
  email     String   @unique
  image     String?
  habits    Habit[]
  createdAt DateTime @default(now())
}

model Habit {
  id              String   @id @default(uuid())
  userId          String
  user            User     @relation(fields: [userId], references: [id])
  name            String
  description     String?
  icon            String   @default("⭐")
  targetFrequency String   @default("daily")
  entries         Entry[]
  createdAt       DateTime @default(now())

  @@index([userId])
}

model Entry {
  id        String   @id @default(uuid())
  habitId   String
  habit     Habit    @relation(fields: [habitId], references: [id])
  date      DateTime
  createdAt DateTime @default(now())

  @@unique([habitId, date])
}
```
```

---

## interface-contracts/actions.md (example)

```markdown
# Interface Contracts — HabitStack (Server Actions)

## Interface Type: Server Actions (Next.js)

## Actions

### `createHabit(data: HabitFormData) → Promise<{ habit: Habit, error?: string }>`
- **Method**: Server Action (POST via form action)
- **Input**: `{ name: string, description?: string, icon?: string, targetFrequency?: string }`
- **Output**: `{ habit: Habit }` on success, `{ error: string }` on validation failure
- **Errors**:
  - `UNAUTHENTICATED` — user not logged in
  - `VALIDATION_ERROR` — name is empty or too long
- **Side effects**: Inserts row in Habit table

### `logEntry(habitId: string) → Promise<{ entry: Entry }>`
- **Method**: Server Action (POST via form action or button click)
- **Input**: `habitId: string`
- **Output**: `{ entry: Entry }` — upserts entry for today
- **Errors**:
  - `UNAUTHENTICATED` — user not logged in
  - `NOT_FOUND` — habit does not exist or belongs to another user
- **Side effects**: Upserts row in Entry table

### `getDashboard() → Promise<{ habits: Array<HabitWithStreak> }>`
- **Method**: Server Component (direct Prisma call)
- **Output**: array of habits with computed streak fields
- **Access control**: Returns only the current user's habits
```

---

## tasks.md (example)

```markdown
# Tasks — HabitStack

## Phase 1: Foundation
- [ ] [T001] Next.js project scaffold — S
  - **Build**: `npx create-next-app@latest habitstack --typescript --tailwind --app`
  - **Verify**: `npm run dev` serves a page at localhost:3000
  - **Gate**: Static page renders without errors
- [ ] [T002] Prisma schema + PostgreSQL setup — S
  - **Build**: `npx prisma init && npx prisma db push`
  - **Verify**: `npx prisma studio` shows empty tables
  - **Gate**: Schema compiles and database responds
- [ ] [T003] [P] NextAuth.js with Google OAuth — M
  - **Build**: `npm install next-auth@beta @auth/prisma-adapter`
  - **Verify**: Sign in with Google, dashboard shows user name
  - **Gate**: Auth session persists on page reload

## Phase 2: Core Features
- [ ] [T004] Create habit form + Server Action — M
  - **Build**: Implement createHabit server action + form component
  - **Verify**: Submit form → habit appears on dashboard
  - **Gate**: Validation rejects empty name
- [ ] [T005] Dashboard habit list — M
  - **Build**: Server Component queries habits and renders cards
  - **Verify**: Dashboard shows habits sorted by creation date
  - **Gate**: Empty state renders when no habits exist
- [ ] [T006] Daily log upsert — M
  - **Build**: Implement logEntry server action + optimistic button
  - **Verify**: Click "Log" → checkmark appears; second click updates
  - **Gate**: Duplicate entries are prevented (unique constraint)

## Phase 3: Progress & Streaks
- [ ] [T007] Streak calculation query — M
  - **Build**: Prisma query that groups entries and computes consecutive days
  - **Verify**: 3 consecutive days → streak is 3
  - **Gate**: Gap in entries resets streak to 0
- [ ] [T008] Dashboard progress display — M
  - **Build**: Render streak, total days, fire icon, percentage bar
  - **Verify**: 7+ day streak shows 🔥; 0 entries shows "Start your first habit"
  - **Gate**: All states render without data from other users

## Phase 4: Polish
- [ ] [T009] [P] Error boundaries + loading skeletons — S
  - **Build**: Add error.tsx and loading.tsx at route group level
  - **Verify**: Simulate DB failure → error page shows retry button
  - **Gate**: Skeleton matches card layout (no layout shift)
- [ ] [T010] [P] E2E tests with Playwright — M
  - **Build**: `npm init playwright` + test files for auth + create + log flows
  - **Verify**: `npx playwright test` passes headlessly
  - **Gate**: All 3 critical flows covered

## Progress: 0/10 tasks complete
```
---

## Traceability Matrix — logsnap

| REQ | Plan Phase | Task ID | Verify | Test |
|-----|-----------|---------|--------|------|
| REQ-001: Log File Reading | Phase 1: Foundation | T001, T002, T003 | `cargo test test_config_parse` | Unit: config_parsing |
| REQ-001 (stdin) | Phase 3: Polish | T004 | `cargo test test_stdin_reading` | Integration: stdin |
| REQ-001 (large file) | Phase 3: Polish | T009 | `cargo test test_large_file --release` | Integration: large_file |
| REQ-002: Log Filtering | Phase 2: Core | T006 | `cargo test test_filter_by_level` | Unit: filter |
| REQ-003: Output Formatting | Phase 2: Core | T007, T008 | `cargo test test_json_output` | Unit: formatter |
| REQ-004: Pipe/Stdin | Phase 3: Polish | T004 | `cargo test test_stdin_reading` | Integration: stdin |
| REQ-005: Large File | Phase 3: Polish | T009 | `cargo test test_large_file --release` | Integration: large_file |

Every line of production code in the final implementation should trace back to a requirement in spec.md. This matrix ensures no untracked code.

---

## Traceability Matrix — SnippetVault

| REQ | Plan Phase | Task ID | Verify | Test |
|-----|-----------|---------|--------|------|
| REQ-001: Create Snippet | Phase 1–2 | T001–T005 | `curl -X POST` returns 201 | Integration: test_create_snippet |
| REQ-002: Search Snippets | Phase 2 | T006 | `pytest -k test_search` | Integration: test_search |
| REQ-003: Get by ID | Phase 2 | T007 | `curl /snippets/{id}` returns 200/404 | Integration: test_get_by_id |

Every line of production code in the final implementation should trace back to a requirement in spec.md. This matrix ensures no untracked code.

---

## Traceability Matrix — LegacyPay

| REQ | Plan Phase | Task ID | Verify | Test |
|-----|-----------|---------|--------|------|
| REQ-001: Process Payment | Phase 1–2 | T001, T004 | `go test ./characterization/...` | Integration: payment_flow_test |
| REQ-002: Process Refund | Phase 1–2 | T001, T004 | `go test ./characterization/...` | Integration: refund_flow_test |

Every line of production code in the final implementation should trace back to a requirement in spec.md. This matrix ensures no untracked code.

---

## Traceability Matrix — EventStream

| REQ | Plan Phase | Task ID | Verify | Test |
|-----|-----------|---------|--------|------|
| REQ-001: Emit order.created | Phase 2 | T005 | `go test ./services/order/...` | Integration: order_created_test |
| REQ-002: Consume in Inventory | Phase 2 | T006 | `go test ./services/inventory/...` | Integration: inventory_consumer_test |
| REQ-003: Emit order.paid | Phase 2 | T007 | `go test ./services/billing/...` | Integration: billing_flow_test |

Every line of production code in the final implementation should trace back to a requirement in spec.md. This matrix ensures no untracked code.

---

## Traceability Matrix — HabitStack

| REQ | Plan Phase | Task ID | Verify | Test |
|-----|-----------|---------|--------|------|
| REQ-001: Auth | Phase 1 | T003 | Sign in with Google → dashboard shows name | E2E: auth_flow |
| REQ-002: Create Habit | Phase 2 | T004 | Submit form → habit appears on dashboard | Component: habit_form_test |
| REQ-003: Log Daily Entry | Phase 2 | T006 | Click "Log" → checkmark, second click updates | Component: log_button_test |
| REQ-004: View Progress | Phase 3 | T007, T008 | Dashboard shows streak + fire icon | Integration: streak_query_test |

Every line of production code in the final implementation should trace back to a requirement in spec.md. This matrix ensures no untracked code.
```
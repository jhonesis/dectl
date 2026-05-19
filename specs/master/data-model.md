# Data Model — dectl
> *Defines all persistent entities, their attributes, and relationships.*
> *Last updated: 2026-05-13*

---

## Storage Locations

| Store | Path | Purpose |
|-------|------|---------|
| Memory database | `~/.dectl/memory.db` | All persistent memory entries and embeddings |
| Trust registry | `~/.dectl/trust.toml` | Workflow execution trust decisions |
| Global config | `~/.dectl/config.toml` | User preferences across all projects |
| Project config | `.dec/config/project.toml` | Per-project settings, overrides global |
| Project state | `.dec/state/progress.json` | Feature progress tracking (flat file, not SQLite) |

---

## SQLite Entities (`~/.dectl/memory.db`)

### `memories`

The core memory table. Every entry stored via `dectl memory add` lives here.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | INTEGER PRIMARY KEY | ✅ | Auto-increment. Used in `dectl memory show <id>` |
| `content` | TEXT | ✅ | Markdown-supported content of the memory entry |
| `tags` | TEXT | ❌ | Comma-separated tags. Nullable. e.g. `"architecture,auth"` |
| `project` | TEXT | ❌ | Project name at time of creation. Nullable (global memory if absent) |
| `created_at` | TEXT | ✅ | ISO 8601 timestamp. e.g. `"2026-05-13T14:32:00Z"` |
| `updated_at` | TEXT | ❌ | ISO 8601 timestamp. Set when content is edited. Nullable |

**Indexes**:
- `idx_memories_project` on `project` — speeds up project-scoped queries
- `idx_memories_created_at` on `created_at` — speeds up chronological listing

**Notes**:
- `content` is stored as raw Markdown. Rendering is the responsibility of the consuming tool.
- `tags` stored as comma-separated TEXT (not a separate table) for Phase 1 simplicity. Normalized in Phase 3 if needed.
- A memory with `project = NULL` is considered global and returned in all projects.

---

### `embeddings` *(Phase 2)*

Stores vector embeddings for semantic search. One row per memory entry.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | INTEGER PRIMARY KEY | ✅ | Auto-increment |
| `memory_id` | INTEGER | ✅ | Foreign key → `memories.id`. Cascade delete |
| `model` | TEXT | ✅ | Embedding model used. e.g. `"nomic-embed-text"` |
| `vector` | BLOB | ✅ | Raw float32 array serialized as binary |
| `created_at` | TEXT | ✅ | ISO 8601 timestamp |

**Indexes**:
- `idx_embeddings_memory_id` on `memory_id`

**Notes**:
- SQLite does not have a native vector type. `vector` is stored as BLOB (serialized `Vec<f32>`).
- Semantic search in Phase 2 loads vectors into memory and computes cosine similarity in Rust — no vector DB required at this scale.
- If the embedding model changes, old embeddings are invalidated. `dectl memory embed-all --model <name>` regenerates them.

---

## Relationships

```
memories 1 ──── 0..1 embeddings
   (memory_id FK, cascade delete)
```

Simple by design. A memory entry exists independently of its embedding. Deleting a memory cascades to its embedding. An embedding cannot exist without a memory.

---

## Entity Relationship Diagram

```
┌──────────────────────────────┐
│          memories            │
├──────────────────────────────┤
│ id          INTEGER PK       │
│ content     TEXT NOT NULL    │
│ tags        TEXT             │
│ project     TEXT             │
│ created_at  TEXT NOT NULL    │
│ updated_at  TEXT             │
└──────────────┬───────────────┘
               │ 1
               │
               │ 0..1
┌──────────────▼───────────────┐
│         embeddings           │  ← Phase 2
├──────────────────────────────┤
│ id          INTEGER PK       │
│ memory_id   INTEGER FK       │
│ model       TEXT NOT NULL    │
│ vector      BLOB NOT NULL    │
│ created_at  TEXT NOT NULL    │
└──────────────────────────────┘
```

---

## Flat File Entities

### `trust.toml` (`~/.dectl/trust.toml`)

Tracks which workflows have been explicitly trusted by the user for execution of `action` steps.

```toml
[trusted]
# key = "<project_path>/<workflow_name>"
# value = ISO 8601 timestamp of when trust was granted

"/home/user/projects/myapp/design_architecture" = "2026-05-13T10:00:00Z"
"/home/user/projects/myapp/setup_project" = "2026-05-13T11:30:00Z"
```

**Rules**:
- Trust is scoped to project path + workflow name. The same workflow name in a different project requires separate trust.
- Trust is never granted automatically. Always requires explicit user confirmation (Y/n prompt).
- Revoking trust = deleting the entry from this file.

---

### `progress.json` (`.dec/state/progress.json`)

Tracks feature implementation status within a project. Written by the model or CLI; read by both.

```json
{
  "updated_at": "2026-05-13T14:00:00Z",
  "features": [
    {
      "id": "auth",
      "name": "Authentication",
      "status": "in_progress",
      "notes": "JWT implemented, refresh tokens pending"
    },
    {
      "id": "payments",
      "name": "Payment integration",
      "status": "pending",
      "notes": ""
    },
    {
      "id": "user_profile",
      "name": "User profile",
      "status": "done",
      "notes": ""
    }
  ]
}
```

**Valid status values**: `pending`, `in_progress`, `done`, `blocked`

---

## Migration Strategy

- SQLite schema migrations run automatically on `memory.db` open via a `migrations` table that tracks applied versions.
- Phase 1 ships with migration `0001_initial` (creates `memories` table).
- Phase 2 ships with migration `0002_embeddings` (creates `embeddings` table).
- Migrations are append-only — never modify or drop existing migrations.

```
migrations table:
┌─────────┬──────────────────┬─────────────────────┐
│ version │ name             │ applied_at          │
├─────────┼──────────────────┼─────────────────────┤
│ 1       │ initial          │ 2026-05-13T...      │
│ 2       │ embeddings       │ (Phase 2)           │
└─────────┴──────────────────┴─────────────────────┘
```

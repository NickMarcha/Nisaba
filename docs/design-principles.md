# Design Principles

## Core Philosophy

### Client: Markdown is absolute source of truth

On the **desktop client** (Windows/Mac/Linux), Markdown files are the **final source of truth**. Everything is derived from them:

- The SQLite index is a **derived cache** — never the canonical store
- Views, queries, relational data — all derived from parsing the Markdown vault
- If the index is lost: rebuild from Markdown. No data loss
- The client never treats a database (or any other format) as authoritative over the files

This is non-negotiable for the client.

### Server: Performance + optional Markdown sync

For the **sync server**:

- The server may store data in a database for performance — acceptable
- The server can offer a **Markdown sync option** for users who want Markdown-backed storage:
  - Sync to a GitHub repo (e.g. hourly commits when changes occur)
  - Write Markdown to local storage (filesystem) on the server
- Users choose: DB-only for performance, or DB + Markdown for portability/backup

### Other principles

- **Scripting is optional** — Power users can script; regular users work through the UI
- **UI-first** — Typical users need no scripting or APIs for normal workflows
- **SQLite as power-user interface** — The index is SQLite. Power users and plugins query it directly with SQL. Sensible defaults for most; full access when needed.

## Design Rules

### 1. Round-trip rule

Never allow **data that cannot be represented in Markdown**.

Every feature must round-trip:

```
Markdown → index → views → Markdown
```

If something can't survive that cycle, it shouldn't exist in the system. This prevents the architecture from drifting into a Notion-style proprietary platform.

### 2. Rebuild from Markdown

Always enforce:

```
Markdown → rebuild everything
```

If the SQLite database or server disappears:

```
delete index.db
rebuild from markdown
```

System fully recovers. This protects long-term integrity of user data.

### 3. No custom Markdown dialect

Do not create custom syntax in the body like `((block refs))` or `{{query}}`. Stick to:

- Standard Markdown in body
- YAML frontmatter (metadata, including `block_ids`)
- Wikilinks `[[note]]`

The closer to standard Markdown, the more future-proof the system.

### 4. Block index is internal only

Files are the user-facing model. Blocks are an **internal index** for powering relational views — not a first-class or user-facing concept. Block IDs live in frontmatter (`block_ids` array), mapped to blocks by position. Body stays standard Markdown.

## Data Model

### File-based storage

```
vault/
  note-a.md
  note-b.md
  project-x.md
```

Each Markdown file is the primary object. Relations via `[[links]]`, tags, frontmatter.

### Block index (internal)

For relational queries, the indexer extracts blocks. Block IDs in frontmatter (`block_ids`) map to blocks by order:

```yaml
---
type: task
block_ids: [abc123, def456, ghi789]
---
# Project Alpha
- [ ] Task 1
- [ ] Task 2
- [ ] Task 3
```

Indexer assigns `block_ids[i]` to block at position i. Stable IDs; body stays clean. **Block ID generation:** Generate UUID for new blocks; include `block_ids` in every write (auto-save or explicit). No explicit save required — app auto-saves; block_ids ride along with content.

### Typed properties

Metadata lives in **frontmatter only**. Body is content.

```yaml
---
type: task
status: open
assignee: [[Alice]]
due: 2026-04-01
priority: 2
---
```

Indexer interprets types; SQLite stores typed values. Links `[[Alice]]` and tags `#tag` in the body are indexed (standard Markdown). No inline `Key: value` extraction — keeps round-trip simple and avoids ambiguity.

**Schema definitions** (optional): Notes can define schemas that other notes inherit. Each vault has a schema folder:

```
vault/
  schemas/
    core/       — Built-in (task, project, person). Copied on vault init; editable.
    plugin/     — Schemas added by plugins (e.g. plugin-id/task.md)
    userdefined/ — Custom schemas created by power users
```

Precedence: userdefined > plugin > core. Regular users never touch this; they use templates. Power users and plugins add schemas.

**Schema versioning:** Schemas include a `version` field. Support **migrations** so schema evolution is traceable and can be applied when the format changes.

## Decision Log

| Decision | Rationale |
|----------|-----------|
| **AGPL-3.0 license** | Copyleft for network use. Hosted forks must share source. Protects the project from proprietary SaaS derivatives while allowing self-hosting. |
| **Svelte over React** | Smaller bundle, no virtual DOM overhead, reactive by default. Aligns with lightweight web client. CodeMirror and Yjs are framework-agnostic. |
| **UI + SQLite for queries** | Most users: views via UI only. Power users: direct SQL on index.db. No custom DSL; SQL matches the store. Plugins can use SQL or index API. |
| **Frontmatter only for metadata** | Typed metadata (status, assignee, due) in YAML frontmatter. No inline body extraction. Keeps round-trip unambiguous; body stays content. |
| **Schema folder structure** | `vault/schemas/{core,plugin,userdefined}/`. Core = built-in; plugin = extensions; userdefined = power users. Precedence: userdefined > plugin > core. |
| **Server DB: SQLite** | SQLite for self-hosting (zero config, sufficient). Postgres only for non-self-hosted SaaS with many tenants. |
| **CRDT persistence** | Yjs state in SQLite (blob per doc). Markdown materialization on save + 30s debounce. |
| **Auth: self-hosted** | Users in server DB. No external OAuth required. OAuth optional for teams. |
| **Git backup: both** | Desktop app and server can each run git backup. User configures per context. |
| **Block IDs in frontmatter** | `block_ids` array in YAML; mapped by position. Body stays standard Markdown. |
| **Tauri over Electron** | Smaller binaries, better performance for indexing, Rust excellent for file watching. See [Architecture](architecture.md#tech-choices). |
| **CodeMirror over ProseMirror** | Markdown-first; CodeMirror is modular and has collaborative plugins. ProseMirror is more WYSIWYG-focused. |
| **Yjs over Automerge** | Mature, fast, works with CodeMirror. Better ecosystem support. |
| **SQLite for local index** | Zero config, fast, portable, rebuildable. Only a cache — never source of truth. |
| **File-based + block index** | Files for portability; block index for relational views. Best of both without polluting Markdown. |

## Options Considered

### File-based vs block-based model

- **File-based (Obsidian):** Simple, portable, Git-friendly. Weak for relational queries (tasks inside notes).
- **Block-based (Logseq, Notion):** Powerful queries. Tends to destroy Markdown portability (block IDs in files).

**Chosen:** Hybrid. Markdown files = storage. Blocks = indexed representation in SQLite only. User never sees blocks; they power views.

### Tauri vs Electron

- **Tauri:** Smaller binaries, system webview, Rust backend, better performance, path to mobile.
- **Electron:** Massive ecosystem, JS-only friendly, many editor components. Large bundles, higher RAM.

**Chosen:** Tauri. Performance matters for indexing and file watching.

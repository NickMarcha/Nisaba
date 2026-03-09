# Architecture

## Overview

The system separates **storage**, **indexing**, and **collaboration**.

```
Markdown Files (source of truth)
            │
            ▼
Indexing Engine (SQLite)
            │
            ▼
Query + View Layer (relational views, dashboards)
```

Realtime editing operates separately:

```
Editor
  │
  ▼
Collaboration Layer (CRDT)
  │
  ▼
Sync Server
  │
  ▼
Materialized Markdown Files
```

The database can always be rebuilt from Markdown.

## Components

### Desktop App

Primary interface for editing and managing knowledge.

**Stack:** Tauri, Svelte, Rust, SQLite. Tauri over Electron (smaller binaries, better performance for indexing). Svelte over React (smaller bundle, no virtual DOM). CodeMirror for editing; Yjs for collaboration.

**Responsibilities:**

- Markdown editing
- Local vault management
- Offline support
- Relational queries (via derived index)
- Syncing with servers

**Local storage:**

```
vault/
  *.md
  schemas/
    core/
    plugin/
    userdefined/

index.db
```

The SQLite database indexes (derived from Markdown):

- metadata
- links
- tags
- relations
- blocks (internal, for views; IDs from frontmatter `block_ids`)

**Critical:** On the client, Markdown is the sole source of truth. The index is a derived cache. If `index.db` is deleted, the system rebuilds from `vault/*.md`.

### Collaboration Server

Optional component enabling realtime editing, shared workspaces, web access.

**Stack:** Rust, Axum, Yjs websocket, SQLite (Postgres only for non-self-hosted SaaS).

**Features:** CRDT-based document sync, authentication, permission management. The server does **not replace Markdown storage**; it may use a database for performance.

### Web Client

Browser interface for reading, lightweight editing, collaboration.

**Stack:** Svelte, CodeMirror, Yjs, WebSocket.

**Sync model:** On-demand. Documents sync when opened; no full vault download. Open docs use full CRDT (Yjs) for real-time collaboration. "Lightweight" = don't load everything upfront, not partial/incomplete data.

### Shared Frontend

Tauri desktop app and web client use the **same Svelte source**. One codebase builds for both targets. Shared: components, views, editor, styling. Platform-specific: data layer — Tauri uses Rust IPC (local vault); web uses WebSocket/API (server). Keeps UI consistent across platforms.

## Tech Choices

| Choice | Rationale |
|--------|-----------|
| **Tauri** | Smaller binaries, system webview, Rust backend, better performance. Electron rejected: large bundles, higher RAM. |
| **Svelte** | Smaller bundle, no virtual DOM. React rejected: Svelte fits lightweight web client; CodeMirror and Yjs work with both. |
| **CodeMirror** | Modular, collaborative plugins. ProseMirror rejected: more WYSIWYG-focused; CodeMirror better for Markdown-first. |
| **Yjs** | Mature, fast, works with CodeMirror. Automerge rejected: Yjs has better ecosystem support. |
| **SQLite** | Zero config, fast, portable, rebuildable. Only a cache — never source of truth. |
| **WebSocket (sync)** | Low latency for CRDT sync. No HTTP polling. |

**Indexing:** pulldown-cmark (Markdown), serde_yaml (frontmatter), notify (file watching). Flow: `filesystem change → indexer → update SQLite`.

## Server Markdown Sync

The server may store data in a database for performance. Additionally, it can offer **optional Markdown sync** for users who want Markdown-backed storage:

- **Option A: GitHub sync** — Push vault to a GitHub repo (e.g. hourly). Users configure git remote and SSH key.
- **Option B: Local storage** — Write Markdown to a filesystem path on the server.

Users choose: DB-only for performance, or DB + Markdown for portability and backup.

## Collaboration Architecture

Sync uses **WebSockets** for low latency. No HTTP polling.

```
Desktop App
      │
      │ WebSocket
      v
Sync Server
      │
      v
CRDT Document Store
      │
      v
Markdown Materializer
      │
      v
Markdown Vault (if sync enabled)
```

### CRDT Persistence

**Server storage:** Yjs state in SQLite (doc_id → blob). Use `Y.encodeStateAsUpdate()` / `Y.applyUpdate()`. Persist on each update.

**Markdown materialization** (if sync enabled): CRDT → Markdown → write to vault. Trigger: (1) explicit save from client, (2) debounced after 30s idle. Y.Text holds the document content; conversion to Markdown is direct.

### CRDT and Markdown: Known Limitations

- **Materialization:** Cursor positions and ephemeral state are not persisted.
- **Bidirectional sync:** Edits in Markdown (e.g. from git, external tools) must merge with CRDT state. Conflict resolution is complex.

### CRDT Bidirectional Sync: Merge UX

When Markdown is edited outside the app (git pull, external editor), present an **in-app view** where the user can confirm or reject the external changes before merging into the CRDT state.

If server changes arrive while the user is resolving external changes, use a **temp file** to hold one version and prompt the user with a **merge strategy** (e.g. take external, take server, or manual merge).

## Query System

Queries operate on the SQLite index (derived from Markdown).

**Most users:** UI only. Filters, groupings, view types. No query writing.

**Power users:** Direct SQLite access. The index lives in `index.db`; power users query with SQL. Plugins can use SQL or an index API. No custom DSL — SQL matches the store.

**Possible views:** tables, kanban, calendar, graph.

## Indexing Pipeline

```
markdown file
     ↓
parser
     ↓
block extraction
     ↓
metadata extraction
     ↓
type resolver (schemas from vault/schemas/{core,plugin,userdefined}/)
     ↓
store in SQLite
```

**Indexed data:** files, links, tags, frontmatter, blocks, relations. Metadata (status, assignee, due, etc.) from frontmatter only; links and tags from body.

**Index schema:** Core table is `blocks`. Views like `tasks` are `SELECT * FROM blocks WHERE type='task'`. Kanban, calendar, etc. query blocks filtered by type and properties.

## Server Database

For sync state (Yjs documents), auth, workspace metadata, permissions.

**Self-hosting:** SQLite. Single file, zero config. Sufficient for typical self-hosted deployments.

**Non-self-hosted / SaaS:** Postgres when running a commercial multi-tenant service. Not needed for self-hosters.

## Deployment

Self-hosting requires Docker:

```bash
docker-compose up -d
```

**Services:** app-server, sync-server, database (SQLite; Postgres only for non-self-hosted SaaS).

**Optional:** reverse-proxy, OAuth provider (for teams that want Google/GitHub login).

## Authentication

**Default: self-hosted.** Users and credentials stored in the server's database (SQLite). No external providers. OAuth optional for teams.

## Backup Strategy

Git is **not** part of the editing workflow — only backup.

**Desktop app:** Can run git backup (push vault to remote) on schedule or manual.

**Server (if Markdown sync enabled):** Can also run git backup — push to configured remote (e.g. hourly). Local storage option: write to filesystem path.

Both can run it; user chooses. Backups contain **pure Markdown files** — full portability.

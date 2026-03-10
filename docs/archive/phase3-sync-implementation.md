<!-- Archived: Phase 3 implementation plan. All steps complete. See implementation_handoff_plan for current state. -->

# Phase 3: Sync Server + Web Client — Implementation Plan

Continuation of [implementation_handoff_plan](../../.cursor/plans/implementation_handoff_plan_04fee695.plan.md). Phase 1–2 done. This doc details Phase 3.

---

## Goal

Desktop and web sync; edit on one, see on the other. Server syncs Yjs CRDT; clients connect via WebSocket.

---

## Tech Choices for Phase 3

| Layer | Choice | Rationale |
|-------|--------|-----------|
| **Sync server (Rust)** | `yrs-axum` | Axum WebSocket + Yjs/Yrs protocol. Compatible with JS `y-websocket` clients. |
| **Server persistence** | SQLite (doc_id → blob) | Per [docs/architecture.md](../architecture.md). Use `Y.encodeStateAsUpdate()` / `Y.applyUpdate()`. |
| **Desktop sync** | `y-websocket` (npm) | Connects to server when URL configured. Merge local vault ↔ CRDT. |
| **Web client** | Same Svelte UI as desktop | Shared components via `packages/ui`. Data layer: fetch + Yjs for open docs. |
| **Auth (MVP)** | Single shared secret | Query param or header. No OAuth initially. |

---

## Implementation Order

### Step 3.1: Sync server — WebSocket + Yjs protocol — DONE

**Path:** `apps/sync-server/`

1. Add deps to `Cargo.toml`:
   - `yrs-axum` (Yjs/Yrs protocol over Axum WebSocket)
   - `yrs` (CRDT types)
   - `rusqlite` (persistence)
   - `axum` (already present)
   - `tokio` (already present)

2. Implement:
   - Axum router with WebSocket route (e.g. `/ws` or `/sync`)
   - `yrs-axum` integration: `BroadcastGroup` per document
   - Doc ID from URL path or query (e.g. `/sync?doc=path/to/note.md`)
   - SQLite table: `yjs_docs (doc_id TEXT PRIMARY KEY, state BLOB, updated_at INTEGER)`
   - On connect: load blob, apply to Yrs doc, broadcast to new client
   - On update: persist blob, broadcast to other clients
   - Optional: `NISABA_SECRET` env for auth (reject connections without it)

3. Run server: `npm run dev:sync` or `cd apps/sync-server && cargo run`

**Deliverable:** Server accepts WebSocket at `ws://localhost:8765/sync/{doc_id}`, syncs Yjs docs, persists to SQLite (`sync.db`). Doc ID is URL-encoded (e.g. `notes%2Ffoo.md` for `notes/foo.md`).

---

### Step 3.2: Shared UI package

**Path:** `packages/ui/`

1. Extract shared layout from `apps/desktop/src/App.svelte`:
   - Sidebar (248px), file list, view tabs (Notes/Blocks)
   - Editor toolbar (Raw/Edit/Split/Read)
   - Blocks table view
   - Context menu (New note, Rename)

2. Create platform-agnostic components:
   - `Sidebar.svelte` — props: `files`, `selectedPath`, `viewMode`, `onOpenVault`, `onSelectFile`, etc.
   - `EditorPane.svelte` — wraps `Editor.svelte`, mode buttons
   - `BlocksView.svelte` — table, type filter
   - `AppLayout.svelte` — composes sidebar + main

3. Data layer abstraction:
   - `createDataLayer()` returns `{ files, selectedPath, content, loadFiles, selectFile, saveFile, ... }`
   - Desktop: implementation uses Tauri `invoke`
   - Web: implementation uses fetch + Yjs (Step 3.4)

4. Move `Editor.svelte` and `livePreview.ts` into `packages/ui` or keep in desktop and import from shared — decide based on whether web needs identical editor.

**Deliverable:** `packages/ui` exports layout components. Desktop and web can share the same UI shell.

---

### Step 3.3: Desktop sync client — DONE

**Path:** `apps/desktop/`

1. Add deps:
   - `yjs` (CRDT)
   - `y-websocket` (WebSocket provider)

2. Sync layer:
   - `sync.ts` — `connectSync(serverUrl, docId)` — creates Y.Doc, connects via WebSocket
   - When server URL configured (e.g. settings/store): for each open doc, create sync connection
   - When doc is closed: disconnect, optionally persist Yjs state to server (server already persists)

3. Merge local vault ↔ CRDT:
   - **Open doc:** If sync connected, load from Yjs (merge with local if both exist). Else load from local file.

   - **Save:** On save: (a) write to local file (existing), (b) emit Yjs update to server (new). Debounced.

   - **Incoming:** When Yjs update arrives, update Editor. If Editor has unsaved local changes, conflict resolution: prefer server (or prompt user — MVP: take server).

4. Settings:
   - Add "Sync server URL" in sidebar or settings. Store in `localStorage` or Tauri store.
   - When URL set: enable sync for open docs. When cleared: disable sync, local-only.

5. **Editor integration:** Use `y-codemirror.next` or `@codemirror/collab` with Yjs. Replace current Editor content binding with Y.Text binding when sync is active.

**Deliverable:** Desktop syncs with server when URL configured. Enter `ws://localhost:8765` in the sync input, open a vault, select a note — edits sync in real time via Yjs. Uses `EditorSync.svelte` with y-codemirror.next when sync is on.

---

### Step 3.4: Web client — data layer + shared UI — DONE

**Path:** `apps/web/`

1. Add deps:
   - `yjs`, `y-websocket`
   - `@codemirror/*`, `codemirror` (same as desktop)
   - `marked`, `turndown` (for preview)
   - `@nisaba/ui` (shared components)

2. Data layer:
   - `createWebDataLayer(serverUrl)`:
     - `listFiles()` — fetch from server API (e.g. `GET /api/files`)
     - `readFile(path)` — fetch from API or open Yjs doc and sync
     - `saveFile(path, content)` — for non-synced: `PUT /api/files`. For synced: Yjs update (server persists)

3. Server API (add to sync-server):
   - `GET /api/files` — list doc IDs (from SQLite or filesystem if Markdown sync enabled)
   - `GET /api/files/:path` — return Markdown content (materialize from Yjs or read from DB)
   - `PUT /api/files/:path` — create/update doc (for initial upload or non-Yjs clients)

4. Web client UI:
   - Use `AppLayout` from `packages/ui` with `createWebDataLayer`
   - No vault picker — server URL is config (env or login form)
   - Same sidebar, file list, editor, blocks view

5. Sync: When user opens a doc, connect Yjs WebSocket. Editor binds to Y.Text. Edits sync in real time.

**Deliverable:** Web client connects to server, lists files, edits with Yjs sync. Desktop and web sync.

**Implemented:** Web app uses `connectMeta` and `connectBlocks` for file list and index via WebSocket. Notes/Blocks/Links tabs. Tree file list. Same shared UI as desktop.

---

### Step 3.5: End-to-end — DONE

1. **Doc ID scheme:** Use file path as doc_id (e.g. `notes/foo.md`). Ensure URL-safe encoding.

2. **Server Markdown materialization:** Optional for MVP. If skipped: server stores Yjs blobs only. Web client reads from Yjs when doc is open. API `GET /api/files/:path` can materialize from Yjs on demand.

3. **Index on server:** Server may run indexer on materialized Markdown. Or: web client fetches blocks from `GET /api/blocks` (server indexes Yjs state). Defer if needed.

4. **Scripts:** Add `dev:sync:server`, `dev:web` to root `package.json`. Document: run sync server, then `dev:desktop` or `dev:web`.

---

## Key Files (After Phase 3)

| Purpose | Path |
|---------|------|
| Sync server entry | `apps/sync-server/src/main.rs` |
| Yjs persistence | `apps/sync-server/src/persistence.rs` (new) |
| Shared layout | `packages/ui/src/AppLayout.svelte` (new) |
| Desktop sync | `apps/desktop/src/sync.ts` (new) |
| Web data layer | `apps/web/src/dataLayer.ts` (new) |
| Web entry | `apps/web/src/App.svelte` |

---

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Yjs ↔ Markdown merge conflicts | MVP: prefer server. Later: in-app merge UX per [architecture.md](../architecture.md). |
| Cursor/undo lost on sync | Known limitation. Document. |
| yrs-axum protocol mismatch | Verify compatibility with `y-websocket` protocol. Use same message format. |

---

## References

- [yrs-axum](https://docs.rs/yrs-axum/latest/yrs_axum/) — Axum WebSocket + Yjs
- [y-websocket](https://github.com/yjs/y-websocket) — JS client protocol
- [docs/architecture.md](../architecture.md) — CRDT persistence, materialization
- [.agent/Uncodixfy.md](../../.agent/Uncodixfy.md) — UI for web client

# Roadmap

Decisions (license, stack, data model) are in [Design Principles](design-principles.md#decision-log) and [Architecture](architecture.md).

## MVP Scope — Complete

First version includes:

1. Desktop vault
2. Markdown editing
3. SQLite index (derived from Markdown)
4. Relational queries (via views: Blocks, Links)
5. Sync server + web client (real-time Yjs sync)

**Skipped for MVP:**

- Plugins
- Mobile apps
- Public publishing
- Advanced permissions

## Future Work

Potential future features:

- Plugin system (query API, render hooks, view components — plugins must never modify Markdown format)
- Advanced query language (if not covered by UI views)
- Graph visualization
- Public publishing
- Mobile clients (initially: mobile browser → web client → sync server)

## Difficulty Estimate

Building this is very doable but still a serious project.

**Estimated complexity:**

- MVP: ~4–6 months solo
- Full system: 1–2 years

**Most difficult parts:**

1. CRDT + Markdown reconciliation
2. Fast indexing
3. Query system

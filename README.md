# Nisaba

![Nisaba logo](packages/ui/src/lib/nisaba-logo.svg)

A self-hostable, local-first knowledge system built around **Markdown as the source of truth**, with **real-time collaboration**, **relational views**, and **portable data**.

The goal is to combine the strengths of Obsidian and Notion while avoiding vendor lock-in.

## Motivation

Most knowledge management tools force a trade-off:

| System   | Strength                                 | Weakness               |
| -------- | ---------------------------------------- | ---------------------- |
| Obsidian | Markdown files, local ownership          | Poor collaboration     |
| Notion   | Realtime collaboration, relational views | Proprietary data model |

Nisaba aims to combine both: Markdown portability, local-first editing, realtime collaboration, relational views, self-hostable infrastructure.

## Features

- **Desktop (Tauri):** Open a vault folder, tree file list, Markdown editor (raw/edit/split/read), Blocks and Links views, rename and create notes. Sync indicator for unsynced files. File watcher and close handler.
- **Web:** Connect to sync server, tree file list, Notes/Blocks/Links tabs, editor with real-time sync. No local vault — data from server.
- **Sync server:** Yjs WebSocket, SQLite persistence. `GET/PUT /api/files`. CORS for web client.
- **Relationships:** Wikilink autocomplete, frontmatter relations, Links view (outbound + backlinks).

## Core Principles

- **Client: Markdown is final source of truth** — Everything derived from Markdown. SQLite index is a derived cache.
- **Round-trip rule** — Markdown → index → views → Markdown
- **No custom Markdown dialect** — Standard Markdown body, YAML frontmatter, wikilinks
- **UI-first** — Regular users work through the UI; scripting optional for power users

## Development

```bash
npm install
npm run dev:sync    # Sync server (run first for collaboration)
npm run dev:desktop # Tauri desktop app
npm run dev:web     # Web client
```

Run the sync server before desktop or web if you want real-time collaboration.

## Project structure

```
nisaba/
  apps/
    desktop/       # Tauri + Svelte (local vault, Rust backend)
    web/           # Svelte (connects to sync server)
    sync-server/   # Rust, Axum, WebSocket (CRDT sync)
  packages/
    core/          # Shared types (blocks, relations)
    ui/            # Shared Svelte components
```

## Documentation

- [Architecture](docs/architecture.md) — Components, tech stack, data flow, deployment
- [Design Principles](docs/design-principles.md) — Philosophy, constraints, decision log
- [Roadmap](docs/roadmap.md) — MVP scope, future work
- [Desktop troubleshooting](docs/desktop-troubleshooting.md) — Port 1420, WebView2, dev tips

## License

AGPL-3.0. See [LICENSE](LICENSE).

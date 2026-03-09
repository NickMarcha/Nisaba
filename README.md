# Nisaba

A self-hostable, local-first knowledge system built around **Markdown as the source of truth**, with **real-time collaboration**, **relational views**, and **portable data**.

The goal is to combine the strengths of Obsidian and Notion while avoiding vendor lock-in.

## Motivation

Most knowledge management tools force a trade-off:

| System   | Strength                                 | Weakness               |
| -------- | ---------------------------------------- | ---------------------- |
| Obsidian | Markdown files, local ownership          | Poor collaboration     |
| Notion   | Realtime collaboration, relational views | Proprietary data model |

Nisaba aims to combine both: Markdown portability, local-first editing, realtime collaboration, relational views, self-hostable infrastructure.

## Core Principles

- **Client: Markdown is final source of truth** — Everything derived from Markdown. SQLite index is a derived cache.
- **Round-trip rule** — Markdown → index → views → Markdown
- **No custom Markdown dialect** — Standard Markdown body, YAML frontmatter, wikilinks
- **UI-first** — Regular users work through the UI; scripting optional for power users

## Development

```bash
npm install
npm run dev:desktop   # Tauri desktop app
npm run dev:web       # Web client
```

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

## License

AGPL-3.0. See [LICENSE](LICENSE).

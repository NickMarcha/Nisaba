# Nisaba — Agent Context

This file orients AI agents working on Nisaba. Read it at the start of a session.

## Project Summary

**Nisaba** is a local-first, Markdown-native knowledge base. It combines Obsidian-style file ownership with Notion-style collaboration and relational views. Self-hostable, no vendor lock-in.

**Current state:** Concept phase. Documentation split into `README.md` and `docs/`. No implementation yet. See [docs/roadmap.md](docs/roadmap.md) for MVP scope.

## Key Constraints

- **Client:** Markdown is the sole source of truth. Everything (index, views, queries) is derived from `vault/*.md`. SQLite is a derived cache, never canonical.
- **Server:** May use a database for performance. Optional Markdown sync (GitHub or local storage).
- **Data model:** File-based user-facing; block index is internal only for relational views.
- **No custom Markdown dialect (body).** Standard Markdown body; YAML frontmatter (including `block_ids`); wikilinks.

## Documentation Map

| Document | Use when |
|----------|----------|
| [README.md](README.md) | Project overview, principles |
| [docs/architecture.md](docs/architecture.md) | Components, tech stack, data flow, deployment |
| [docs/design-principles.md](docs/design-principles.md) | Philosophy, design rules, decision log |
| [docs/roadmap.md](docs/roadmap.md) | MVP scope, future work |

## When to Use .agent Files

The `.agent/` folder contains project-specific guidance. **Read and apply** these when the task matches:

### [.agent/Signs_of_AI_writing.md](.agent/Signs_of_AI_writing.md)

**Use when:** Writing significant text — docs, README, comments, user-facing copy, commit messages, or any prose.

**What it does:** Lists patterns to avoid (AI puffery, vague attribution, promotional language, certain vocabulary). Prefer simple words, concrete facts, neutral tone.

### [.agent/Uncodixfy.md](.agent/Uncodixfy.md)

**Use when:** Building UI — editor layout, views (tables, kanban, calendar), dashboards, settings screens, web client, or any visual interface.

**What it does:** Bans generic AI UI (soft gradients, floating panels, oversized corners, hero sections in dashboards). Defines "normal" UI standards (Linear, Raycast, Stripe, GitHub style). Includes color palettes. Read before designing or implementing UI.

## Tech Stack (Planned)

See [docs/architecture.md](docs/architecture.md). Desktop: Tauri, Svelte, Rust, SQLite. Editor: CodeMirror. Collaboration: Yjs. Server: Rust, Axum, Yjs websocket.

## Quick Reference

- Markdown → rebuild everything. Index is disposable.
- Round-trip rule: Markdown → index → views → Markdown. No data that can't survive that cycle.
- UI-first; scripting optional. Regular users need no APIs.
- Writing? Apply `.agent/Signs_of_AI_writing.md`. UI? Apply `.agent/Uncodixfy.md`.

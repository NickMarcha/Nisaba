Here’s a <!-- Archived: Original concept document. Content split into README.md and docs/*.md -->

Here's a **clean, realistic README-style draft** you could start a repo with. I’ll keep it concise but structured like an actual serious open-source project.

---

# Local-First Collaborative Markdown Knowledge Base

A self-hostable, local-first knowledge system built around **Markdown as the source of truth**, with **real-time collaboration**, **relational views**, and **portable data**.

The goal is to combine the strengths of tools like Obsidian and Notion while avoiding vendor lock-in.

---

# Motivation

Most knowledge management tools force a trade-off:

| System   | Strength                                 | Weakness               |
| -------- | ---------------------------------------- | ---------------------- |
| Obsidian | Markdown files, local ownership          | Poor collaboration     |
| Notion   | Realtime collaboration, relational views | Proprietary data model |

This project aims to combine both:

* **Markdown portability**
* **Local-first editing**
* **Realtime collaboration**
* **Relational data views**
* **Self-hostable infrastructure**

Your data should **always remain accessible as plain Markdown files**, regardless of whether this software continues to exist.

---

# Core Principles

### Markdown is the source of truth

All content is stored as:

```
/vault
  note-a.md
  note-b.md
  project-x.md
```

The system **never replaces Markdown with a proprietary format**.

Metadata is stored using frontmatter:

```yaml
---
type: task
status: open
project: [[project-x]]
---
```

---

### Local-first architecture

The desktop application works **fully offline**.

Local storage contains:

```
markdown files
+ local index database
```

The database acts as a **cache and query layer**, not the canonical data store.

---

### Collaboration without lock-in

Realtime collaboration is enabled through a sync server, but the server **does not own the data**.

Markdown files remain exportable at all times.

---

### Self-hostable by default

The entire backend can be deployed using Docker.

Example:

```
docker-compose up
```

Users maintain full control of:

* data
* backups
* authentication
* infrastructure

---

# Architecture Overview

The system separates **storage**, **indexing**, and **collaboration**.

```
Markdown Files (source of truth)
            │
            ▼
Indexing Engine
(SQLite)
            │
            ▼
Query + View Layer
(relational views, dashboards)
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

---

# Components

## Desktop App

Primary interface for editing and managing knowledge.

Responsibilities:

* Markdown editing
* local vault management
* offline support
* relational queries
* syncing with servers

Local storage includes:

```
vault/
  *.md

index.db
```

The SQLite database indexes:

* metadata
* links
* tags
* relations

---

## Collaboration Server

Optional component enabling:

* realtime editing
* shared workspaces
* web access

Features:

* CRDT-based document sync
* authentication
* permission management

The server does **not replace Markdown storage**.

---

## Web Client

A browser interface for:

* reading
* lightweight editing
* collaboration

The web interface does not require downloading the entire vault.

This allows:

* sharing notes
* collaborating with non-technical users
* public documentation pages

---

# Data Model

Relationships are expressed through Markdown conventions.

Examples:

### Internal links

```
[[Project Alpha]]
```

### Metadata

```yaml
---
type: person
team: engineering
---
```

### Query example

```
LIST
FROM type=task
WHERE status=open
```

These queries are executed against the SQLite index.

---

# Backup Strategy

Self-hosters can optionally configure automatic backups to a Git repository.

Example configuration:

```
git remote: git@github.com:user/vault-backup.git
backup schedule: daily
```

Backups contain **pure Markdown files**.

This ensures:

* full portability
* independent backups
* no proprietary export format

---

# Deployment

Self-hosting requires only Docker.

Example:

```
docker-compose up -d
```

Services:

```
app-server
sync-server
database
```

Optional components:

```
reverse-proxy
auth-provider
```

---

# Goals

Primary goals:

* Markdown-native knowledge base
* Local-first editing
* Realtime collaboration
* Relational querying
* Simple self-hosting
* No vendor lock-in

---

# Non-Goals

This project will **not**:

* replace Markdown with a proprietary format
* require cloud infrastructure
* lock users into a specific hosting provider

---

# Future Work

Potential future features:

* plugin system
* advanced query language
* graph visualization
* public publishing
* mobile clients

---

# License

TBD

---

If you want, I can also help with two things that would make this **way more real as a project**:

1. **A concrete tech stack** (Electron vs Tauri, backend language, CRDT choice, etc.)
2. **A system architecture diagram** that would guide actual implementation.


Below is a **pragmatic stack and architecture** that would actually make this buildable by a small team (or one person). The key idea is: **keep the system simple enough that the Markdown layer always stays recoverable**.

---

# Proposed Tech Stack

## Desktop App

Two realistic choices exist:

### Option A — Tauri (recommended)

Pros

* much smaller binaries
* uses system webview
* Rust backend possible
* better performance
* easier path to mobile later

Cons

* slightly smaller ecosystem
* Rust required for backend logic

Typical stack:

```text
Tauri
React / Svelte frontend
Rust core logic
SQLite (local index)
```

---

### Option B — Electron

Pros

* massive ecosystem
* easier for JS-only developers
* many editor components available

Cons

* huge bundle sizes
* higher RAM usage

Typical stack:

```text
Electron
React
Node backend
SQLite
```

---

### Recommendation

Use **Tauri**.

Reasons:

* performance matters when indexing large vaults
* Rust is excellent for file watching and indexing
* binaries stay small
* SQLite + Rust is extremely stable

---

# Editor Layer

You need a good Markdown editor.

Best choice:

* CodeMirror

Reasons:

* modular
* collaborative plugins exist
* powers many modern editors

Alternative:

* ProseMirror

But ProseMirror is more WYSIWYG-focused.

For Markdown-first systems, **CodeMirror works better**.

---

# Collaboration Engine

For realtime editing you should use a CRDT library.

Best choice today:

* Yjs

Why:

* extremely mature
* widely used
* very fast
* works with CodeMirror

Alternatives:

* Automerge

But Yjs has better ecosystem support.

---

# Local Index Layer

Use:

```
SQLite
```

Indexed data:

```
files
links
tags
frontmatter
blocks
relations
```

SQLite advantages:

* zero configuration
* fast
* portable
* rebuildable

Important rule:

> SQLite is **only a cache**, never the source of truth.

---

# File Watching

When files change you must reindex.

Rust libraries:

```
notify
```

Flow:

```
filesystem change
     ↓
indexer
     ↓
update SQLite
```

---

# Backend Server

For the optional collaboration server.

Recommended stack:

```
Rust
Axum web framework
Yjs websocket server
Postgres (optional)
```

You may not even need Postgres initially.

Minimal server:

```
sync server
auth
web client
```

---

# Web Client

Purpose:

* lightweight editing
* sharing notes
* non-technical collaborators

Stack:

```
React / Svelte
CodeMirror
Yjs
WebSocket
```

The web client does **not require downloading the full vault**.

---

# System Architecture

## Local Desktop System

```
+---------------------------+
| Desktop App (Tauri)      |
|                           |
| CodeMirror Editor        |
|                           |
| Local CRDT Document      |
|                           |
| Markdown Files           |
|                           |
| SQLite Index             |
+------------+--------------+
             |
             |
             v
       File Watcher
             |
             v
         Reindexer
```

---

## Collaboration Architecture

```
Desktop App
      |
      | WebSocket
      v
Sync Server
      |
      v
CRDT Document Store
      |
      v
Markdown Materializer
      |
      v
Markdown Vault
```

CRDT keeps documents synchronized.

Markdown is written periodically or on save.

---

# Query System

Queries operate only on SQLite.

Example flow:

```
Markdown files
      ↓
Indexer
      ↓
SQLite
      ↓
Query Engine
      ↓
Views
```

Possible views:

```
tables
kanban
calendar
graph
```

---

# Git Backup Architecture

Git should **not be part of the editing workflow**.

Only backup.

Server job:

```
cron job
     ↓
git add vault/*
     ↓
git commit
     ↓
git push
```

Users configure:

```
git remote
ssh key
backup frequency
```

---

# Indexing Pipeline

```
markdown file
     ↓
parser
     ↓
extract metadata
extract links
extract tags
     ↓
store in SQLite
```

Useful Rust libraries:

```
pulldown-cmark
serde_yaml
```

---

# Plugin System (Future)

Eventually expose:

```
query API
render hooks
view components
```

Plugins should **never modify the markdown format**.

---

# Mobile Strategy

Don't build mobile initially.

Instead:

```
mobile browser
      ↓
web client
      ↓
sync server
```

Reasons:

* vault size can be huge
* mobile editing less common
* development complexity much lower

---

# MVP Scope

First version should only include:

1. desktop vault
2. markdown editing
3. SQLite index
4. relational queries
5. optional sync server

Skip initially:

* plugins
* mobile apps
* public publishing
* advanced permissions

---

# Realistic Difficulty

Building this is **very doable** but still a serious project.

Estimated complexity:

```
MVP: ~4–6 months solo
full system: 1–2 years
```

Most difficult parts:

1. CRDT + Markdown reconciliation
2. fast indexing
3. query system

---

# One Very Important Design Rule

Always enforce:

```
Markdown → rebuild everything
```

Meaning:

If the SQLite database or server disappears:

```
delete index.db
rebuild from markdown
```

System fully recovers.

This rule protects the **long-term integrity of user data**.

---

If you're interested, I can also show you something **very important before starting this project**:

There is one architectural decision that will determine whether the system becomes **simple and powerful** or **a nightmare like Notion internally**. Most projects get this wrong.

The decision that will make or break this system is:

**What is the fundamental unit of data?**

You have two choices.

---

# Option 1 — File-based model

This is what Obsidian does.

```
vault/
  note-a.md
  note-b.md
  project-x.md
```

Each Markdown file is the primary object.

Relations happen via:

```
[[links]]
tags
frontmatter
```

## Advantages

* extremely simple
* perfectly portable
* Git-friendly
* easy to reason about

## Problems

Relational queries are weaker.

Example problem:

```
Task inside note
```

You can't easily query tasks if they live inside random files.

You end up building hacks like:

* block references
* parsing headings
* task extraction

That’s why systems like Obsidian need heavy plugins like Dataview.

---

# Option 2 — Block-based model

This is what Logseq and Notion do internally.

Instead of files being the unit, **blocks are the unit**.

Example markdown:

```
# Project Alpha

- Task 1
- Task 2
```

Internally becomes:

```
block_1: Project Alpha
block_2: Task 1
block_3: Task 2
```

Each block has:

```
id
parent
properties
links
```

Now queries become powerful:

```
find all tasks
find tasks under project X
find tasks assigned to person Y
```

---

# Why block models are powerful

Relational systems need **small addressable objects**.

Files are too big.

Blocks are perfect.

That’s why:

* Notion
* Logseq
* Anytype

all use block/object models internally.

---

# The problem with block models

They tend to **destroy Markdown portability**.

Instead of:

```
note.md
```

you get weird IDs like:

```
block-uuid-8372
```

Which ruins the clean file system.

---

# The architecture that actually solves this

The best design is a **hybrid model**.

```
Markdown files = storage format
Blocks = index layer
```

Meaning:

Markdown stays clean:

```
# Project Alpha

- [ ] Task 1
- [ ] Task 2
```

But when indexed:

```
block_1
  type: heading

block_2
  type: task

block_3
  type: task
```

SQLite stores the blocks.

---

# Result

You get:

### Simple storage

```
vault/*.md
```

### Powerful querying

```
SELECT *
FROM blocks
WHERE type = 'task'
```

### Rebuildable index

```
delete index.db
reindex markdown
```

Everything still works.

---

# How indexing would work

Example pipeline:

```
markdown file
     ↓
parser
     ↓
block extraction
     ↓
block table
```

Example table:

```
blocks
-------------------------
id
file
parent_id
type
content
metadata
```

Another table:

```
links
-------------------------
source_block
target_note
```

---

# Why this decision matters

If you choose **file-based indexing**, you will constantly hit walls:

* tasks
* inline metadata
* nested structures
* relations

If you choose **block-based indexing**, queries become trivial.

But you must **keep Markdown readable**.

That balance is the entire challenge.

---

# My recommendation

Use this model:

```
Markdown files = truth
Blocks = indexed representation
SQLite = query engine
CRDT = collaboration layer
```

Architecture:

```
markdown
   ↓
parser
   ↓
block graph
   ↓
SQLite
   ↓
views / queries
```

---

# One more critical design choice

Avoid this mistake that many systems make:

**Never invent a custom Markdown dialect.**

Do not create weird syntax like:

```
((block refs))
{{query}}
```

Stick to:

```
markdown
yaml frontmatter
wikilinks
```

The closer you stay to standard Markdown, the more future-proof the system is.

---

# The interesting part

If you build this correctly, the result would actually sit **between**:

* Obsidian
* Notion
* Logseq

But with a **cleaner architecture than all three**.

Because:

```
Notion → powerful but locked
Obsidian → portable but weak relationally
Logseq → powerful but messy markdown
```

Your design could realistically hit the sweet spot.

---

If you want, I can also show you **one extremely useful feature** that would make this system *dramatically more powerful than all of these tools* while still staying Markdown-native. It's a small architectural addition but changes how knowledge bases work.


The feature is **treating metadata and relations as first-class data, not just text annotations** — while still storing them in Markdown.

Most Markdown systems treat metadata like this:

```yaml
---
status: open
assignee: alice
project: alpha
---
```

That’s easy to read, but weak structurally. The trick is to make those properties behave like **typed relational fields** internally.

---

# The idea: a typed property system

Every block or note can have **structured properties** that map to database columns.

Example Markdown:

```yaml
---
type: task
status: open
assignee: [[Alice]]
due: 2026-04-01
priority: 2
---
```

Internally the indexer interprets types:

```text
block_id
type: task
status: enum(open)
assignee: relation(person)
due: date
priority: integer
```

SQLite then stores them as typed values.

---

# Why this matters

Without typing, queries become messy:

```sql
status = "open"
```

With typed metadata you get real relational power:

```sql
SELECT *
FROM tasks
WHERE due < today
AND assignee = 'Alice'
```

Now you can build:

* calendars
* kanban boards
* dashboards
* filtered tables

All from plain Markdown.

---

# Property schemas

To make this scalable, notes themselves can define schemas.

Example:

```yaml
---
type: schema
name: task
fields:
  status: enum(open,doing,done)
  assignee: relation(person)
  due: date
  priority: number
---
```

Then any note with:

```yaml
type: task
```

inherits those fields.

---

# Resulting workflow

Users write normal Markdown:

```markdown
# Fix login bug

Status: open  
Assignee: [[Alice]]  
Due: 2026-04-01
```

Indexer extracts:

```
task
status=open
assignee=Alice
due=2026-04-01
```

Now relational views become trivial.

---

# Views become powerful

Example task board:

```
Tasks by status
----------------
open
doing
done
```

Calendar view:

```
tasks grouped by due date
```

People view:

```
tasks grouped by assignee
```

All derived from Markdown.

---

# This solves a major weakness

Pure Markdown systems struggle with **structured data**.

Database systems solve that but sacrifice portability.

Typed properties give you **database power with file storage**.

---

# Architecture impact

The indexer pipeline expands slightly:

```
markdown
   ↓
parser
   ↓
block extraction
   ↓
metadata extraction
   ↓
type resolver
   ↓
SQLite tables
```

Tables might look like:

```
blocks
properties
relations
schemas
```

---

# Why this is a big deal

This enables something most tools struggle with:

**knowledge bases that behave like databases without losing Markdown.**

You can store:

* tasks
* books
* people
* research notes
* projects

All as Markdown, but query them relationally.

---

# What this becomes in practice

A user could have a vault like:

```
/people
/alice.md
/bob.md

/projects
/project-alpha.md

/tasks
/fix-login.md
```

And the system could generate dynamic views:

```
Tasks assigned to Alice
Projects with open tasks
People with overdue tasks
```

All without writing SQL.

---

# Why most tools don't do this well

Tools like:

* Obsidian rely on plugins such as Dataview.
* Notion uses proprietary databases.
* Logseq mixes custom syntax into Markdown.

A clean typed property layer avoids all of those problems.

---

# One final design rule that keeps everything sane

Never allow **data that cannot be represented in Markdown**.

Every feature must round-trip:

```
Markdown → index → views → Markdown
```

If something can't survive that cycle, it shouldn't exist in the system.

That rule prevents the architecture from slowly turning into another Notion-style proprietary platform.

# Phase 2: Persistent Data - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-11
**Phase:** 02-Persistent Data
**Areas discussed:** Schema Evolution, DB Threading Model, Type Alignment, Initial Data Seeding, Conversation List Denormalization, Tauri Command Design, Frontend Integration

---

## Schema Evolution

| Option | Description | Selected |
|--------|-------------|----------|
| Port gtk_ref schema as-is | Battle-tested, covers all tables; no redesign risk | ✓ |
| Redesign schema from scratch | Clean slate based on current frontend needs | |
| Port + evolve | Start with gtk_ref, add columns where frontend types have gaps | |

**Auto-selected:** Port + evolve (recommended) — gtk_ref schema is proven, but the frontend Conversation type reveals missing denormalizations (title, last_message_preview).
**Notes:** FTS5 virtual tables kept from gtk_ref even though search UI is Phase 8. Message branches deferred to Phase 8.

---

## DB Threading Model

| Option | Description | Selected |
|--------|-------------|----------|
| Background worker (mpsc channel) | Dedicated thread owns Connection; commands via channels with oneshot results | ✓ |
| Mutex<Connection> with async | Tauri State wraps Connection in Mutex; commands lock and query | |
| Pool of connections | r2d2 connection pool for concurrent access | |

**Auto-selected:** Background worker (recommended) — SQLite is single-writer, the worker pattern is proven in gtk_ref, rusqlite Connection is !Send so it must stay on one thread.
**Notes:** WAL mode enables concurrent reads without blocking the writer thread.

---

## Type Alignment (Rust ↔ TypeScript)

| Option | Description | Selected |
|--------|-------------|----------|
| Rust models as source of truth | Frontend types updated to match Tauri command returns | ✓ |
| TypeScript types as source of truth | Rust structs mirror the frontend interfaces exactly | |
| Separate models, manual mapping | Both sides have independent types; mapping layer converts | |

**Auto-selected:** Rust models as source of truth (recommended) — the database schema determines what's possible; frontend types should reflect reality.
**Notes:** Timestamps as ISO 8601 strings (not NaiveDateTime). UUIDs as text strings.

---

## Initial Data Seeding

| Option | Description | Selected |
|--------|-------------|----------|
| Empty database | No built-in content; users create or import | ✓ |
| Ship mock characters | Pre-populate with Eleanor, KAI-7, etc. from mock.ts | |
| First-run wizard | Guided setup that offers to import sample characters | |

**Auto-selected:** Empty database (recommended) — mock data served the UI scaffold only. Built-in generation presets (Creative/Balanced/Precise) ARE seeded as utility defaults.

---

## Conversation List Denormalization

| Option | Description | Selected |
|--------|-------------|----------|
| Add title + preview columns to conversations | Denormalized for sidebar display; updated on message save | ✓ |
| Join with messages table at query time | Compute sidebar data from messages on each load | |
| Hybrid: title stored, preview computed | Title is user-editable; preview computed from latest message | |

**Auto-selected:** Add title + preview columns (recommended) — sidebar needs these on every render; joining is expensive at scale. Message count computed dynamically.
**Notes:** message_count via SELECT COUNT(*) per conversation — cheap, avoids stale-count bugs.

---

## Tauri Command Design

| Option | Description | Selected |
|--------|-------------|----------|
| One command per CRUD operation | Flat command surface; DbInput enum for internal routing | ✓ |
| Single generic command | One `db_command` that takes an enum; frontend sends variants | |
| Resource-oriented | REST-like: `characters_list`, `characters_create`, etc. | |

**Auto-selected:** One command per CRUD operation (recommended) — matches Tauri conventions, gives typed returns, and the DbInput enum stays internal.

---

## Frontend Integration

| Option | Description | Selected |
|--------|-------------|----------|
| Replace hydrate() with invoke() calls | Keep AppState structure; change data source | ✓ |
| Full reactive store rewrite | Replace AppState with Tauri-aware reactive queries | |
| Dual mode (mock + real) | Keep mock data as fallback; invoke() when Tauri available | |

**Auto-selected:** Replace hydrate() with invoke() calls (recommended) — minimal change, preserves existing component contracts, just swaps the data source. Add loading state for initial fetch. Remove mock.ts after wiring.

---

## Claude's Discretion

None — all decisions were auto-selected in yolo mode.

## Deferred Ideas

None — all discussion stayed within phase scope.

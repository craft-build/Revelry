# Phase 1: Foundation - Context

**Gathered:** 2025-04-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 1 delivers the foundational infrastructure for Persona: Tauri 2.0 application scaffold, SQLite database with migrations, Rust-TypeScript type sharing, OS keychain integration, Tauri capabilities configuration, error handling framework, full persona CRUD system, and GNOME-inspired UI shell with three primary views. Everything needed to support future character management, chat, and AI provider features in Phases 2+.

</domain>

<decisions>
## Implementation Decisions

### Persona System
- **D-01:** Full 10-field persona model: name, appearance, personality, background, role_description, relationships, speech_style, notes, tags + avatar
- **D-02:** Key-value block prompt injection format: `[Persona]\nName: ...\nAppearance: ...\nPersonality: ...`
- **D-03:** `{{user}}` resolves to persona name only (e.g., `{{user}}` → `"Alice"`). Matches SillyTavern/CharacterCardV2 convention
- **D-04:** Auto-apply default persona when starting a conversation; if no default exists, show persona picker before first message
- **D-05:** Avatars stored as files in app data directory (`$APPDATA/persona/avatars/`), filename referenced in DB via `avatar_filename` column
- **D-06:** No strict avatar upload limits (local-only app). SpicyChat pattern: display small thumbnail/icon inline, show full image on click. Store original file, generate thumbnail (64x64 or 128x128) for inline display
- **D-07:** Stub `active_persona_id` column on conversations table (nullable FK) in Phase 1 schema — Phase 2 adds per-conversation persona UI/logic
- **D-08:** Persona block placed before system prompt in assembled prompt: `[Persona block]\n\n[Character system prompt]`

### Database Schema
- **D-09:** rusqlite_migration crate for versioned SQL migrations (rollback support, validation)
- **D-10:** Single `personas` table with all 10 text fields, `tags` as JSON array column, `is_default` BOOLEAN, `avatar_filename` TEXT
- **D-11:** Create only the personas table in Phase 1; add other tables (characters, conversations, messages, providers) in respective phases via migrations
- **D-12:** Defer FTS5 full-text search to Phase 3 — simple LIKE queries suffice for Phase 1's typical <20 personas
- **D-13:** `is_default` BOOLEAN column on personas table for default persona persistence (single source of truth)

### UI Shell Layout
- **D-14:** Header bar: app name/logo (left), view switcher tabs — Chat, Characters, Settings (center), active persona avatar+name + menu button (right)
- **D-15:** Sidebar: conversations list with character avatar + name. 240px expanded, 48px collapsed (icon-only). Ctrl+B toggle. No sidebar in Settings view
- **D-16:** Phase 1 views: Chat placeholder with persona indicator, Characters view with "My Personas" tab, Settings with provider/keychain setup (INFRA-04/PROV-02)
- **D-17:** Two breakpoints: >=1024px full layout (sidebar + content), <1024px sidebar collapses to overlay

### Project Structure
- **D-18:** Rust backend: feature modules — `commands/`, `db/`, `models/`, `errors/`, `keychain/` subdirectories under `src-tauri/src/`
- **D-19:** Frontend: feature-based — `components/`, `views/`, `hooks/`, `stores/`, `lib/`, `types/` under `src/`
- **D-20:** ts-rs derive macros for Rust↔TypeScript type sharing. Generated `.ts` files in `src/types/generated/`. Build script auto-generates on `cargo build`
- **D-21:** SQL migrations: `src-tauri/migrations/` with numbered SQL files (e.g., `001_create_personas.sql`), embedded at compile time via `include_str!`

### Error Handling
- **D-22:** Snafu for Rust error types (user preference over thiserror). AppError enum with context selectors for Database, Validation, NotFound, Keychain, Io variants
- **D-23:** TanStack Query for frontend error handling — each Tauri command is a query/mutation with `onError` callback. Pattern: `usePersonaQuery`, `useCreatePersonaMutation`
- **D-24:** Toast notifications for user-facing errors (non-modal, auto-dismiss). Matches GNOME Adwaita toast pattern

### Persona Editor UI
- **D-25:** Sliding panel pattern (slides in from right, overlaying content). Non-destructive — can see content behind
- **D-26:** Token estimate: placeholder in Phase 1 (reserved space in editor UI). Wire real counter in Phase 2 with USER-12/USER-13
- **D-27:** Avatar picker: click-to-upload circle at top of editor (GNOME Contacts pattern). Shows current avatar or placeholder icon
- **D-28:** Field layout: grouped sections with progressive disclosure — Identity (name, avatar), Description (appearance, personality, background), Role (role_description, relationships, speech_style), Meta (notes, tags). Collapsible sections

### Tauri Capabilities
- **D-29:** FS access: AppData scope for read/write + `dialog::open` file picker for avatar selection (bypasses FS scope)
- **D-30:** IPC permissions: `core:default` baseline + `sql:default` + `dialog:default` + `store:default`
- **D-31:** Keychain: Rust-only via `keyring` crate — no Tauri plugin or capability grant needed. Frontend calls Tauri command, which calls keyring internally

### IPC Command Design
- **D-32:** Resource-verb command naming: `get_personas`, `create_persona`, `update_persona`, `delete_persona`
- **D-33:** Typed `Result<T, AppError>` responses where T is ts-rs generated type. Frontend gets typed data or typed error
- **D-34:** UUID v4 for entity IDs (INTEGER column storing UUID text, or BLOB — implementer's choice)
- **D-35:** No pagination in Phase 1 — persona lists return all at once. Add cursor-based pagination in Phase 2 for larger datasets

### Dev Environment
- **D-36:** `cargo tauri dev` + Vite HMR for development (standard Tauri dev workflow)
- **D-37:** Keychain service name suffix: `persona-dev` in dev mode, `persona` in production. Prevents dev/prod key collision
- **D-38:** DB location: Tauri pathresolver for AppData dir. Dev mode resolves to dev-specific path (e.g., `com.persona.dev/`), production to standard app data path

### Agent's Discretion
- Exact thumbnail generation implementation (Rust `image` crate vs frontend canvas approach)
- Exact toast notification component (Radix Toast vs custom)
- Sidebar collapse animation details (duration, easing)
- Build script specifics for ts-rs type generation
- Snafu error variant naming and context selector design
- UUID storage format in SQLite (TEXT vs BLOB)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Specification
- `.planning/REQUIREMENTS.md` — All v1/v2/v3 requirements with phase mapping and traceability
- `.planning/PROJECT.md` — Project context, architecture decisions, critical constraints
- `.planning/ROADMAP.md` — 4-phase execution roadmap with dependency graph
- `.planning/STATE.md` — Current project state and open questions

### Technical Research
- `.planning/research/SUMMARY.md` — Stack recommendations, architectural decisions, critical pitfalls, CharacterCardV2 spec details

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
None — greenfield project with no existing source code

### Established Patterns
None — all patterns will be established in Phase 1

### Integration Points
- Tauri 2.0 IPC boundary (commands + channels + events) — primary integration point between Rust backend and React frontend
- SQLite via rusqlite — persistence layer, accessed through Mutex or connection pool
- OS keychain via keyring crate — secure API key storage, no DB involvement
- App data directory — avatar file storage, Tauri pathresolver for platform-specific paths

</code_context>

<specifics>
## Specific Ideas

- SpicyChat-inspired avatar display: small thumbnail/icon inline in sidebar and headers, full image shown on click
- GNOME Contacts-inspired avatar picker: click-to-upload circular avatar area at top of editor
- Key-value persona block format matches SillyTavern convention for AI model compatibility
- Progressive disclosure in persona editor: collapsible sections reduce cognitive load (Identity visible by default, expand for Details/Role/Meta)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 1-Foundation*
*Context gathered: 2025-04-28*

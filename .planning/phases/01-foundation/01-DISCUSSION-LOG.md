# Phase 1: Foundation — Discussion Log

**Date:** 2025-04-28
**Areas:** Persona System, Database Schema, UI Shell Layout, Project Structure, Error Handling, Persona Editor UI, Tauri Capabilities, IPC Command Design, Dev Environment

---

## Persona System

| Q | Question | Options | Decision |
|---|----------|---------|----------|
| 1 | What fields should a persona have? | Full set (10), Core only (4), Core + relationships (6) | **Full set** — name, appearance, personality, background, role_description, relationships, speech_style, notes, tags + avatar |
| 2 | How should persona block be injected into prompt? | Key-value block, Narrative paragraph, XML-tagged sections | **Key-value block** — `[Persona]\nName: ...\nAppearance: ...` |
| 3 | What should `{{user}}` resolve to? | Name only, Name + key fields, Deep attribute access | **Name only** — `{{user}}` → `"Alice"` |
| 4 | What happens with no persona selected? | Auto-apply default, Always prompt, Chat as 'You' | **Auto-apply default** — silent if default exists, picker if none |
| 5 | Where should avatars be stored? | App data dir, SQLite BLOB, Base64 in DB | **App data dir** — `$APPDATA/persona/avatars/`, filename in DB |
| 6 | Avatar upload constraints? | Resize + limit, Limit only, No limits | **No strict limits** — SpicyChat pattern: thumbnail inline, full on click |
| 7 | Per-conversation persona in Phase 1 schema? | Stub column, Schema when needed, Junction table | **Stub column** — `active_persona_id` nullable FK on conversations |
| 8 | Persona block position in prompt? | Before system prompt, After, Embedded | **Before system prompt** — AI knows who it's talking to first |

## Database Schema

| Q | Question | Options | Decision |
|---|----------|---------|----------|
| 1 | Migration tool? | rusqlite_migration, tauri-plugin-sql built-in, Custom | **rusqlite_migration** — versioned SQL, rollback, validation |
| 2 | Personas table granularity? | Single table, Normalized + tags table, Split text fields | **Single table** — all fields + tags as JSON array |
| 3 | How much future schema in Phase 1? | Personas only, All v1 tables, Personas + stubs | **Personas only** — add others via migrations in later phases |
| 4 | FTS5 for personas now? | Defer to Phase 3, Include now | **Defer to Phase 3** — LIKE queries suffice for <20 personas |
| 5 | Default persona persistence? | DB column, Settings table, tauri-plugin-store | **DB column** — `is_default` BOOLEAN on personas table |

## UI Shell Layout

| Q | Question | Options | Decision |
|---|----------|---------|----------|
| 1 | Header bar contents? | View switcher + persona, Minimal GNOME, Sidebar-driven | **View switcher + persona** — name (L), tabs (C), persona+menu (R) |
| 2 | Sidebar contents? | Conversations list, Multi-section, Context-sensitive | **Conversations list** — 240px/48px, Ctrl+B toggle |
| 3 | Phase 1 view content? | Shell + personas focus, All scaffolded, Only functional | **Shell + personas** — Chat placeholder, My Personas tab, Settings/keychain |
| 4 | Responsive breakpoints? | Two (1024px), Three (1280/800), Fixed desktop | **Two breakpoints** — >=1024px full, <1024px sidebar overlay |

## Project Structure

| Q | Question | Options | Decision |
|---|----------|---------|----------|
| 1 | Rust module organization? | Feature modules, Flat modules, Layered architecture | **Feature modules** — commands/, db/, models/, errors/, keychain/ |
| 2 | Frontend organization? | Feature-based, Atomic design, Flat with views | **Feature-based** — components/, views/, hooks/, stores/, lib/, types/ |
| 3 | Type sharing mechanism? | ts-rs derive, specta, Manual sync | **ts-rs** — derive macros, generated to src/types/generated/ |
| 4 | Migrations directory? | src-tauri/migrations/, src/db/migrations/, External | **src-tauri/migrations/** — numbered SQL files, `include_str!` |

## Error Handling

| Q | Question | Options | Decision |
|---|----------|---------|----------|
| 1 | Rust error types? | thiserror enum, Error + kind, anyhow internal + custom IPC | **Snafu** — user preference. Context selectors for typed errors |
| 2 | Frontend error handling? | TanStack Query, Global error boundary, Error store (Zustand) | **TanStack Query** — query/mutation onError callbacks |
| 3 | Error display to user? | Toast notifications, Inline banners, Dialog modals | **Toast notifications** — non-modal, auto-dismiss, GNOME pattern |

## Persona Editor UI

| Q | Question | Options | Decision |
|---|----------|---------|----------|
| 1 | Editor UI pattern? | Sliding panel, Full-page form, Modal dialog | **Sliding panel** — right-side overlay, preserves context |
| 2 | Token estimate display? | Placeholder in Phase 1, Simple char count, Skip entirely | **Placeholder** — reserved space, wire real counter in Phase 2 |
| 3 | Avatar picker? | Click-to-upload circle, Drag-and-drop, Separate button | **Click-to-upload circle** — GNOME Contacts pattern |
| 4 | Field layout? | Grouped sections, Flat scrollable, Tabbed | **Grouped sections** — Identity, Description, Role, Meta (collapsible) |

## Tauri Capabilities

| Q | Question | Options | Decision |
|---|----------|---------|----------|
| 1 | FS access scope? | AppData only, AppData + file picker, Broad FS | **AppData + file picker** — dialog::open for avatar selection |
| 2 | IPC permission grants? | core:default + plugins, Minimal per-command, Allow-all | **core:default + plugins** — sql:default, dialog:default, store:default |
| 3 | Keychain capability? | No — Rust-only, Yes — custom plugin | **Rust-only** — keyring crate, no Tauri plugin/capability needed |

## IPC Command Design

| Q | Question | Options | Decision |
|---|----------|---------|----------|
| 1 | Command naming? | Resource-verb, Verb-resource, Namespace.module | **Resource-verb** — get_personas, create_persona, update_persona, delete_persona |
| 2 | Response format? | Typed Result, Wrapped response, Bare values | **Typed Result** — `Result<T, AppError>` with ts-rs generated types |
| 3 | ID type? | Int auto-increment, UUID v4, ULID | **UUID v4** — user preference for no-enumeration/no-collision |
| 4 | Pagination now? | No pagination yet, Cursor-based from start | **No pagination yet** — add in Phase 2 for larger datasets |

## Dev Environment

| Q | Question | Options | Decision |
|---|----------|---------|----------|
| 1 | Development workflow? | cargo tauri dev + Vite HMR, Separate dev servers, Docker | **cargo tauri dev + Vite HMR** — standard Tauri dev workflow |
| 2 | Keychain dev vs prod? | Service name suffix, Env-based service name, Same service | **Service name suffix** — `persona-dev` in dev, `persona` in prod |
| 3 | DB location? | Tauri resolver, Explicit config, Always standard path | **Tauri resolver** — dev-specific path in dev, standard in prod |

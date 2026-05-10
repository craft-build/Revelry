<!-- refreshed: 2026-05-10 -->
# Architecture

**Analysis Date:** 2026-05-10

## System Overview

Revelry is a SvelteKit + Svelte 5 SPA (`src/`) — the **active codebase** — designed for eventual Tauri embedding. The **archival reference implementation** in `gtk_ref/` is a Rust/GTK4/Relm4 desktop proof-of-concept whose domain models, AI service patterns, database schema, and keyring integration will inform the future Tauri Rust backend. Both implementations share the same data model from the v2.0 design specification.

```
┌─────────────────────────────────────────────────────────────────────┐
│                   ACTIVE: SvelteKit SPA (src/)                      │
│  ┌──────────┐  ┌──────────────┐  ┌──────────┐  ┌───────────────┐  │
│  │ Routes   │  │ Components   │  │ AppState │  │ Types/Mock    │  │
│  │(SvelteKit│  │(Svelte 5     │  │(Rune     │  │(TS translation│  │
│  │ file-based│  │ components)  │  │ store)   │  │ of SQL schema)│  │
│  │ routing)  │  │              │  │          │  │               │  │
│  └────┬─────┘  └──────┬───────┘  └────┬─────┘  └───────┬───────┘  │
│       │               │               │                │          │
│  ┌────┴───────────────┴───────────────┴────────────────┘          │
│  │              $lib/stores/app.svelte.ts                         │
│  │         (Single reactive state tree)                           │
│  └───────────────────────────────────────────────────────────────┘
│       │                           │                                │
│  Browser SPA              Tauri IPC bridge                  ┌──────┘
│  (mock data)              (future target)                    │
│                                                            │
└────────────────────────────────────────────────────────────┘
                                     │
┌────────────────────────────────────┴─────────────────────────────┐
│          ARCHIVAL REFERENCE: GTK4/Rust PoC (gtk_ref/)             │
│          Patterns for future Tauri Rust backend                   │
│  ┌───────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │ UI Layer  │  │ Service      │  │ Domain       │              │
│  │ (Relm4    │  │ (AI, Prompt, │  │ (Character,  │              │
│  │  Elm Arch)│  │  Token, MD,  │  │  Persona,    │              │
│  │           │  │  Regex, CardIO│  │  Provider…)  │              │
│  └─────┬─────┘  └──────┬───────┘  └──────┬───────┘              │
│        │               │                  │                      │
│  ┌─────┴───────────────┴──────────────────┴──────┐              │
│  │           DB Layer (rusqlite + migrations)    │              │
│  │           Worker: background thread async     │              │
│  └──────────────────────────────────────────────┘              │
│                         │                                       │
│  ┌──────────────────────┴──────────────────────┐               │
│  │           Secrets (OS keyring, never on disk)│               │
│  └─────────────────────────────────────────────┘               │
└─────────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

| Component | Responsibility | File |
|-----------|----------------|------|
| AppState | Single reactive state tree for Svelte UI; CRUD actions, derived views | `src/lib/stores/app.svelte.ts` |
| Types | TypeScript mirrors of the SQLite schema from spec §12.1 | `src/lib/types/index.ts` |
| Mock Data | Hard-coded sample characters/personas/conversations for UI dev | `src/lib/data/mock.ts` |
| Utils | `cn()` class merge, `estimateTokens()`, `formatRelativeTime()`, `avatarInitials()` | `src/lib/utils/index.ts` |
| AppHeader | Top bar: app identity, view switcher, sidebar toggle | `src/lib/components/layout/AppHeader.svelte` |
| ConversationSidebar | Left panel: conversation list, search, new chat | `src/lib/components/layout/ConversationSidebar.svelte` |
| ChatHeader | Active conversation header: character info, persona switcher, model badge | `src/lib/components/chat/ChatHeader.svelte` |
| MessageBubble | Single message rendering with inline markdown, avatars, hover actions | `src/lib/components/chat/MessageBubble.svelte` |
| MessageInput | Auto-grow textarea, token estimation, context bar, send/cancel | `src/lib/components/chat/MessageInput.svelte` |
| PersonaSwitcher | Popover for switching active persona in conversation | `src/lib/components/chat/PersonaSwitcher.svelte` |
| CharacterCard | Grid card for character library | `src/lib/components/characters/CharacterCard.svelte` |
| PersonaCard | Grid card for persona library | `src/lib/components/personas/PersonaCard.svelte` |
| UI Primitives | Headless Bits.ui wrappers: Avatar, Button, Input, Tag, Tooltip | `src/lib/components/ui/` |
| AppModel (Rust) ⬡ | Root Relm4 component: view routing, DB worker, chat panel | `gtk_ref/src/app.rs` |
| DbService ⬡ | Background-thread rusqlite worker with input/output message protocol | `gtk_ref/src/db/mod.rs` |
| AiService ⬡ | rig-core streaming client; provider-agnostic chat completion | `gtk_ref/src/service/ai.rs` |
| PromptEngine ⬡ | Prompt assembly with budget-aware persona tiering and lorebook injection | `gtk_ref/src/service/prompt.rs` |
| KeyringStore ⬡ | OS-native API key storage (macOS Keychain, GNOME Keyring, Windows Credential Manager) | `gtk_ref/src/secrets/mod.rs` |
| Migrations ⬡ | rusqlite_migration schema versioning | `gtk_ref/src/db/migrations.rs` |
| DB Row Mappers ⬡ | `row_to_*` conversion functions for all domain types | `gtk_ref/src/db/models.rs` |

> ⬡ = **Archival reference** (`gtk_ref/`). These Rust components are not part of the active codebase. Their patterns (domain models, service abstractions, DB worker protocol, keyring integration) will inform the future Tauri Rust backend.

## Pattern Overview

**Overall:** Elm Architecture (Model–Update–View) — Svelte 5 runes in the active codebase; Relm4 in the archival reference

**Key Characteristics:**
- Svelte 5 runes (`$state`, `$derived`, `$effect`) as Elm-architecture equivalent — single `AppState` class with reactive fields replacing Relm4's Model/Update/View split (active)
- Headless component pattern: Bits.ui provides behavior (focus, ARIA, keyboard), app provides all styling via Tailwind CSS custom properties
- Variant component pattern: `tailwind-variants` (`tv()`) in `<script module>` for type-safe variant styling, re-exported alongside default component
- Centralized state: one `AppState` singleton (`app`) imported everywhere — no prop drilling beyond immediate parent→child
- Mock-first development: all UI flows testable without backend via `src/lib/data/mock.ts`
- DB worker pattern (Rust, ⬡ archival): `DbService` runs on background thread, communicates via `DbInput`/`DbOutput` enums, never blocks main loop — pattern to adapt for Tauri async commands

## Layers

**Presentation / UI Layer:**
- Purpose: Render views, handle user interactions, display state
- Location: `src/routes/`, `src/lib/components/`
- Contains: SvelteKit page routes, Svelte 5 components, Bits.ui wrappers
- Depends on: `AppState`, `$lib/types`, `$lib/utils`, Bits.ui, Lucide icons
- Used by: User (direct interaction)

**State Management Layer:**
- Purpose: Single source of truth for application state; derived computations; action dispatching
- Location: `src/lib/stores/app.svelte.ts`
- Contains: `AppState` class with `$state` fields, `$derived` getters, action methods
- Depends on: `$lib/types`
- Used by: All UI components and routes

**Data / Type Layer:**
- Purpose: Type definitions mirroring the design spec's SQLite schema; mock data seeding
- Location: `src/lib/types/index.ts`, `src/lib/data/mock.ts`
- Contains: TypeScript interfaces (`CharacterCard`, `Message`, `Conversation`, `Provider`, etc.), mock datasets
- Depends on: (none — pure definitions)
- Used by: `AppState`, all components

**Utility Layer:**
- Purpose: Shared helpers for styling, formatting, token estimation
- Location: `src/lib/utils/index.ts`
- Contains: `cn()`, `estimateTokens()`, `formatRelativeTime()`, `avatarInitials()`
- Depends on: `clsx`, `tailwind-merge`
- Used by: All components

### Archival Reference Layers (gtk_ref/)

> The following Rust layers are from the archival `gtk_ref/` proof-of-concept. They are **not part of the active codebase**. Their patterns (domain modeling, service abstractions, DB worker protocol, keyring integration) are preserved here as reference for the future Tauri Rust backend.

**Rust Domain Layer (gtk_ref/) — ⬡ Archival:**
- Purpose: Business entities, validation, domain logic
- Location: `gtk_ref/src/domain/`
- Contains: `CharacterCardV2`, `Conversation`, `Message`, `Persona`, `ProviderConfig`, `GenerationPreset`, `LorebookEntry`, `RegexScript`
- Depends on: `serde`, `chrono`, `uuid`
- Used by: DB layer, service layer, UI layer (within `gtk_ref/`)
- **Pattern for Tauri:** Domain structs and `serde` derive patterns should be reused directly in the Tauri Rust backend.

**Rust Service Layer (gtk_ref/) — ⬡ Archival:**
- Purpose: AI inference, prompt assembly, token counting, markdown rendering, regex processing, card I/O
- Location: `gtk_ref/src/service/`
- Contains: `AiService` (rig-core streaming), `PromptEngine` (budget-aware prompt assembly), `TokenCountService`, `MarkdownService`, `RegexService`, `CardIoService`, `LorebookService`
- Depends on: Domain layer, `rig-core`, `tiktoken-rs`, `pulldown-cmark`, `tokio`, `futures`
- Used by: UI layer (via Relm4 commands, within `gtk_ref/`)
- **Pattern for Tauri:** `AiService` streaming abstraction and `PromptEngine` budget-aware assembly are the primary patterns to port. Replace Relm4 command dispatch with Tauri `invoke()` commands.

**Rust Database Layer (gtk_ref/) — ⬡ Archival:**
- Purpose: Persistent storage via SQLite with schema migrations; background thread worker
- Location: `gtk_ref/src/db/`
- Contains: `DbService` (worker), `migrations`, `models` (row mappers), CRUD modules per entity
- Depends on: Domain layer, `rusqlite`, `rusqlite_migration`
- Used by: `AppModel` via Relm4 worker protocol (within `gtk_ref/`)
- **Pattern for Tauri:** The `DbService` background-thread worker pattern (`DbInput`/`DbOutput` enums) should be adapted for Tauri's async command system. Schema migrations can be reused as-is.

**Rust Secrets Layer (gtk_ref/) — ⬡ Archival:**
- Purpose: API key storage in OS-native keychain — never written to disk
- Location: `gtk_ref/src/secrets/mod.rs`
- Contains: `KeyringStore` (save/load/delete operations)
- Depends on: `keyring`
- Used by: `AppModel`, provider configuration (within `gtk_ref/`)
- **Pattern for Tauri:** `KeyringStore` abstraction should be reused directly. Tauri has its own keyring plugin, but the security invariant (never write keys to disk) must be preserved.

## Data Flow

### Primary Request Path (SvelteKit UI — Chat)

1. User opens app → `+layout.svelte` mounts, hydrates `AppState` from mock data (`src/routes/+layout.svelte:19-25`)
2. User clicks conversation in sidebar → SvelteKit navigates to `/chat/[id]` (`src/lib/components/layout/ConversationSidebar.svelte` renders `<a href="/chat/${conv.id}">`)
3. Route load extracts `id` from params (`src/routes/chat/[id]/+page.ts:3`)
4. `$effect` in chat page calls `app.openConversation(data.id, getMessages(data.id))` (`src/routes/chat/[id]/+page.svelte:24-25`)
5. User types in `MessageInput`, presses Enter → `handleSend(text)` (`src/routes/chat/[id]/+page.svelte:37`)
6. User message appended via `app.appendMessage(userMsg)` (`src/routes/chat/[id]/+page.svelte:47`)
7. AI streaming message created, `app.is_generating = true` (`src/routes/chat/[id]/+page.svelte:49-58`)
8. Mock token stream updates via `app.updateStreamingMessage(aiId, acc)` (`src/routes/chat/[id]/+page.svelte:63-68`)
9. Stream completes: `app.updateStreamingMessage(aiId, acc, true)` sets `is_streaming: false` (`src/routes/chat/[id]/+page.svelte:63`)
10. `$effect` on `app.messages` auto-scrolls to bottom (`src/routes/chat/[id]/+page.svelte:27-33`)

### Reference Implementation Flow (archival, gtk_ref/)

> This flow documents the archival Rust/GTK4 implementation. It is preserved as a **pattern reference** for the future Tauri Rust backend — the streaming, prompt assembly, and DB persistence patterns should be adapted for Tauri `invoke()` commands.

1. User sends message → `ChatPanel` dispatches `ChatPanelMsg::SendMessage` (`gtk_ref/src/ui/chat/`)
2. `AppModel` forwards to `AiService::stream_chat()` (`gtk_ref/src/service/ai.rs:89`)
3. `PromptEngine::assemble_with_budget()` builds context window from character, persona, lorebook, history (`gtk_ref/src/service/prompt.rs:72`)
4. `AiService::run_stream()` calls rig-core provider client, receives token stream (`gtk_ref/src/service/ai.rs:154`)
5. `StreamChunk::Token` chunks forwarded to UI via GLib main loop channel
6. `ChatPanel` appends tokens to message, updates GTK widgets
7. Stream completes → `StreamChunk::Complete` → message saved to DB via `DbService` input

### Persona Switch Flow

1. User opens `PersonaSwitcher` popover (`src/lib/components/chat/PersonaSwitcher.svelte`)
2. User selects persona → `app.switchPersona(persona_id)` (`src/lib/components/chat/PersonaSwitcher.svelte:10`)
3. `AppState.switchPersona()` updates `conversations[id].user_persona_id` (`src/lib/stores/app.svelte.ts:80-83`)
4. `active_persona` derived getter re-computes (`src/lib/stores/app.svelte.ts:42-46`)
5. All subscribed components re-render (ChatHeader, MessageBubble avatars)

**State Management:**
- SvelteKit: Single `AppState` class with Svelte 5 runes. All state is `$state` (reactive). Derived values are `$derived` getters. No stores, no contexts, no prop chains beyond parent→child. One `app` singleton exported from `src/lib/stores/app.svelte.ts`.
- Rust/Relm4 (⬡ archival): Elm Architecture with `AppModel` struct. `AppMsg` enum for updates. `tracker` crate for fine-grained widget re-rendering. DB operations via `DbService` worker on background thread with `DbInput`/`DbOutput` message protocol. — This pattern will inform the Tauri backend's state management.

## Key Abstractions

**AppState (Svelte 5 Rune Store):**
- Purpose: Single source of truth for entire UI — library data, active conversation, UI toggles, derived views
- Examples: `src/lib/stores/app.svelte.ts`
- Pattern: Centralized reactive state object. Components import `app` and read reactive fields directly. Actions are methods on the class that mutate `$state` fields.

**Provider-Agnostic AI Service ⬡ Archival:**
- Purpose: Stream AI completions through any supported provider (20+ types) without provider-specific code in UI
- Examples: `gtk_ref/src/service/ai.rs`
- Pattern: `AiService` wraps rig-core provider clients. `GenerationParams` normalized across providers. `StreamChunk` enum (`Token`/`Complete`/`Error`) abstracts streaming protocol.
- **Pattern for Tauri:** This abstraction should be ported directly as a Tauri command. The `StreamChunk` enum and `GenerationParams` normalization are backend-agnostic.

**Prompt Assembly with Budget ⬡ Archival:**
- Purpose: Build context window within token limits, with persona tiering (full→trimmed→minimal→essential) and lorebook activation
- Examples: `gtk_ref/src/service/prompt.rs`
- Pattern: `PromptEngine::assemble_with_budget()` takes character, persona, history, lorebook, and token budget. Returns `AssembledPrompt` with preamble, chat history, context utilization metrics.
- **Pattern for Tauri:** This is the core prompt engineering logic. Port as a Tauri command that returns the assembled prompt to the frontend for debugging/transparency.

**DB Worker (Background Thread) ⬡ Archival:**
- Purpose: Execute SQLite operations off the main thread to prevent UI blocking
- Examples: `gtk_ref/src/db/mod.rs`
- Pattern: Relm4 `Worker` trait. `DbInput` enum → `DbOutput` enum. `DbService::update()` matches input, executes SQL, sends output back to main thread.
- **Pattern for Tauri:** The `DbInput`/`DbOutput` message protocol should be adapted as Tauri async commands. The background-thread isolation principle remains critical.

**Headless UI Primitives:**
- Purpose: Accessible component behavior (focus, keyboard, ARIA) without imposing visual style
- Examples: `src/lib/components/ui/Avatar.svelte`, `src/lib/components/ui/Button.svelte`
- Pattern: Bits.ui primitive wrapped in Svelte component. Styling via Tailwind CSS custom properties from `src/app.css` theme tokens. Variant API via `tailwind-variants` (`tv()`).

**CharacterCardV2 Domain Model:**
- Purpose: Canonical character format compatible with SillyTavern V2 spec
- Examples: `src/lib/types/index.ts` (TS), `gtk_ref/src/domain/character.rs` (Rust)
- Pattern: Flat data structure with `serde(default)` for partial deserialization. `extensions` field as `Record<string, unknown>` for forward compatibility.

## Entry Points

**SvelteKit Dev Server:**
- Location: `vite.config.ts` → `vite dev` (port 5173)
- Triggers: `npm run dev`
- Responsibilities: Serves SPA at `localhost:5173`, hot module replacement, loads mock data into `AppState`

**SvelteKit Root Layout:**
- Location: `src/routes/+layout.svelte`
- Triggers: App initialization (first page load)
- Responsibilities: Hydrates `AppState` from mock data, registers F9 keyboard listener, renders `AppHeader` + `ConversationSidebar` + child route

**SvelteKit Route Pages:**
- Location: `src/routes/+page.svelte` (welcome), `src/routes/chat/[id]/+page.svelte` (chat), `src/routes/characters/+page.svelte` (library), `src/routes/settings/+page.svelte` (settings)
- Triggers: URL navigation via SvelteKit router
- Responsibilities: Each page renders its view, subscribes to `AppState`

**Rust Application Entry ⬡ Archival:**
- Location: `gtk_ref/src/main.rs`
- Triggers: `cargo run`
- Responsibilities: Initializes tracing, GTK icons, creates `RelmApp`, runs `AppModel`
- **Pattern for Tauri:** The `main.rs` initialization sequence (DB setup, provider loading, secret store init) maps to the Tauri `Builder` setup in `src-tauri/src/main.rs`.

**Rust AppModel Init ⬡ Archival:**
- Location: `gtk_ref/src/app.rs` (`SimpleComponent::init`)
- Triggers: Relm4 application startup
- Responsibilities: Creates GTK window, initializes DB worker, loads providers/characters/conversations from SQLite, sets up component controllers
- **Pattern for Tauri:** The init flow (DB worker spawn → data load → UI hydration) maps to Tauri's `setup()` hook.

## Architectural Constraints

- **Threading:** SvelteKit runs in browser (single-threaded JS). The archival Rust backend (`gtk_ref/`) uses tokio multi-threaded runtime for async AI calls; `DbService` runs on dedicated background thread via Relm4 Worker; GLib main loop bridges tokio futures to GTK via `MainContext::channel()`. The future Tauri backend will use Tauri's async command system with a similar background-thread isolation principle for DB operations.
- **Global state:** SvelteKit: single `app` singleton exported from `src/lib/stores/app.svelte.ts`. Rust (⬡ archival): single `AppModel` struct owned by Relm4 framework.
- **Circular imports:** None known. SvelteKit layering is strictly directional: routes → components → stores → types/utils. Rust layering in `gtk_ref/` (archival): ui → service → domain ← db.

## Anti-Patterns

### Direct API Key Storage on Disk

**What happens:** Storing API keys in SQLite, config files, or environment variables
**Why it's wrong:** Keys would be readable by any process with file access; violates security invariant (PROV-05, D-16)
**Do this instead:** Use `KeyringStore` pattern from `gtk_ref/src/secrets/mod.rs` (archival reference) — keys stored exclusively in OS keychain. The Tauri backend must enforce the same invariant.

### Blocking the Main Thread with DB Operations

**What happens:** Running `rusqlite` queries synchronously on the GTK main loop
**Why it's wrong:** SQLite queries block the UI thread, causing frame drops and unresponsiveness
**Do this instead:** Dispatch all DB operations through a background-thread worker pattern like `DbService` (`gtk_ref/src/db/mod.rs`, archival reference) which runs on a background thread and returns results via message protocol. The Tauri backend must use async commands to achieve the same isolation.

### Prop Drilling State Through Component Hierarchy

**What happens:** Passing `characters`, `messages`, `active_conversation` etc. as props through 3+ levels
**Why it's wrong:** Creates tight coupling, makes refactoring painful, contradicts the single-store design
**Do this instead:** Import `app` from `src/lib/stores/app.svelte.ts` directly in any component that needs state. Only pass data as props for presentational variety (e.g., a reusable `MessageBubble` receives its specific `message` prop).

## Error Handling

**Strategy:** Layered — application-level errors via `anyhow`, domain-level errors via `thiserror`, UI-level graceful degradation

**Patterns:**
- Rust backend (⬡ archival, `gtk_ref/`): `anyhow::Result` for application code, `thiserror::Error` derive for domain types. `DbOutput::Error` variant for DB operation failures. `StreamChunk::Error` for AI streaming failures. These patterns should be reused in the Tauri Rust backend.
- SvelteKit UI: Currently minimal — mock data always succeeds. Production will need IPC error handling from Tauri `invoke()` rejections.
- Validation: `serde(default)` on all `CharacterCardV2` fields for resilient partial deserialization. Row mapper functions convert SQL type errors explicitly.

## Cross-Cutting Concerns

**Logging:** Rust (⬡ archival, `gtk_ref/`): `tracing` crate with `tracing-subscriber` (env-filter). The Tauri backend should use the same. SvelteKit: browser console only (no structured logging yet).
**Validation:** Type safety via TypeScript interfaces and Rust domain structs. `serde(default)` for partial data resilience. No runtime validation library in use yet.
**Authentication:** Not applicable — local-first desktop app. API key authentication handled via `KeyringStore` → rig-core provider clients (archival pattern from `gtk_ref/`). The Tauri backend will use the same keyring → provider flow. No user accounts or login flow.

---

*Architecture analysis: 2026-05-10*

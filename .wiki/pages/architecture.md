---
type: Reference
description: Module map, frontend↔backend data flow, cross-cutting concerns (DbWorker, cancellation, keyring, errors), and subsystem boundaries.
tags:
- architecture
- data-flow
- dbworker
- tauri-ipc
- subsystems
timestamp: 2026-07-12T18:26:11Z
---
# Architecture

Revelry is a layered desktop app. A **SvelteKit 2 + Svelte 5** frontend talks to a **Rust (Tauri v2)** backend exclusively through typed IPC — the frontend never accesses the database, filesystem, or LLM APIs directly. The design mirrors the Elm/Relm4 AppModel pattern: one source of truth on the frontend, a 4-layer backend (commands → service → domain → db).

## Module Map

### Frontend (`src/`)

- **`lib/stores/app.svelte.ts`** — `AppState`, a single Svelte 5 rune-based class instance exported as `app`. All reactive state (`$state`) and derived getters live here; methods call `api.ts` and mutate state directly. No legacy `svelte/store` usage.
- **`lib/stores/toasts.svelte.ts`** — toast notifications state.
- **`lib/data/api.ts`** — the only place `invoke()` is called. Typed wrappers for all Tauri commands; falls back to `lib/data/mock.ts` when `__TAURI_INTERNALS__` is absent (`IS_DEV` guard).
- **`lib/types/index.ts`** — TypeScript domain types mirroring Rust serde exports.
- **`lib/components/`** — UI grouped by domain: `ui/` (bits-ui primitives), `layout/`, `chat/`, `characters/`, `personas/`, `lorebooks/`, `providers/`, `settings/`. Barrel re-exports per scope.
- **`lib/utils/index.ts`** — `cn()`, `formatRelativeTime()`, `estimateTokens()`, `avatarInitials()`.
- **`routes/`** — SvelteKit routes (SSR off): `/` welcome, `/chat/[id]` conversation, `/characters` library, `/settings` preferences.

### Backend (`src-tauri/src/`)

- **`lib.rs`** — Tauri builder. Starts `DbWorker` before the builder, manages `CancelState` (chat) and `GenerateCancelState` (character gen), registers plugins (fs, dialog, shell, http, store), and lists all commands in `invoke_handler`.
- **`main.rs`** — entry point with `windows_subsystem = "windows"` in release.
- **`error.rs`** — `AppError` enum (thiserror) implementing `Serialize` (display string) so errors reach the frontend as strings.
- **`commands/`** — 13 `#[tauri::command]` async handler modules (one per domain): `characters`, `chat`, `conversations`, `generate`, `providers`, `lorebooks`, `personas`, `presets`, `avatars`, `card_io`, `keywords`, `regex`, `app_settings`.
- **`domain/`** — pure data structs, `#[derive(Serialize)]` + `#[serde(rename_all = "camelCase")]`. No I/O.
- **`service/`** — business logic: `PromptEngine` (prompt assembly + budget truncation), `TokenCountService` (tiktoken-rs), persona template compression, regex post-processing, lorebook matching, card import/export.
- **`ai/mod.rs`** — `AiService` wrapping rig-core for streaming (`stream_chat`, `stream_generate_character`) plus `StreamChunk` / `GenerateCharacterChunk` tagged enums.
- **`db/`** — `DbWorker` tokio mpsc actor (`mod.rs`), rusqlite models (`models.rs`), migrations (`migrations.rs`), and CRUD modules per domain.
- **`secrets/mod.rs`** — `KeyringStore`, OS keychain with `is_available()` round-trip check.

## Data Flow

### Request / Response

```
component → app.<method>() → api.<fn>() → invoke("snake_case_cmd", {camelCaseArgs})
    → #[tauri::command] async fn → service / DbWorker → Result<T, AppError>
    → serialized (camelCase) → Promise<T> on frontend
```

Tauri v2 auto-converts snake_case Rust arg names to camelCase for the frontend; command names stay snake_case.

### Streaming

```
invoke("start_chat", { ..., channel }) → Channel<StreamChunk>
    → command spawns tokio task → AiService::stream_chat() pushes chunks via channel.send()
    → frontend onEvent.onmessage handles { event: "token"|"complete"|"error", data: {...} }
```

`StreamChunk` is a `#[serde(tag = "event", content = "data")]` enum; the TypeScript discriminated union must match it exactly (both camelCase).

## Cross-Cutting Concerns

### Database — DbWorker actor

All SQLite access goes through a `DbWorker` running on a background tokio task (`src-tauri/src/db/mod.rs`). Commands send `DbCommand` variants over an mpsc channel and await a `oneshot` reply. This keeps synchronous rusqlite off the Tauri async runtime. The `DbCommand` enum has 50+ variants. Never call `rusqlite` directly from a command.

### Cancellation

- **`CancelState`** (chat) and **`GenerateCancelState`** (character generation) are `Mutex<Option<CancellationToken>>` managed states.
- Starting a new stream cancels any existing one (`let _ = old_token.cancel()`).
- `stop_generation` / `stop_character_generation` take the token and cancel it.
- Spawned streaming tasks check the token regularly.

### Secrets / Keyring

- `KeyringStore` uses service name `"revelry"`.
- API keys **never** touch the database (invariant). The frontend only sees `has_api_key: boolean`.
- All keyring calls are synchronous and must be wrapped in `tokio::task::spawn_blocking()`.
- `check_keyring_available` runs at startup; the result is cached in `AppState.keyring_available`.

### Error handling

`AppError` (`src-tauri/src/error.rs`) variants: `Database`, `DbWorkerUnavailable`, `NotFound`, `Validation`, `Io`, `Serialization`, `CardIo` (anyhow), `Keyring`, `AiService`, `PathTraversal`, `Poisoned`, `Other`. It `Serialize`s to its `Display` string, so `invoke()` rejections arrive as strings on the frontend. Helper constructors: `AppError::db()`, `not_found()`, `validation()`, `ai()`, `keyring()`, `poisoned()`, `other()`.

## Integration Boundaries

- **Commands → Service/AI**: command handlers are thin; they pull managed state, load context, resolve params, then delegate to `service/` or `ai/`.
- **Service → DbWorker**: services send `DbCommand`s and await oneshot replies; they own business logic (prompt assembly, token budgeting, lorebook activation, regex application).
- **Domain ↔ TypeScript**: Rust `domain/` structs with `rename_all="camelCase"` are mirrored 1:1 in `src/lib/types/index.ts`.
- **Frontend → Backend**: the only boundary is `api.ts` ↔ Tauri IPC. Components read/write `app`; `app` calls `api.ts`; `api.ts` calls `invoke`.

## Adding a Tauri Command

1. Define the Rust domain type in `domain/` with `#[derive(Serialize)]` + `rename_all = "camelCase"`.
2. Add the matching TypeScript type in `src/lib/types/index.ts`.
3. Add the `invoke()` wrapper in `src/lib/data/api.ts` (snake_case command name).
4. Add the `AppState` method in `src/lib/stores/app.svelte.ts`.
5. Register the command in `src-tauri/src/lib.rs` → `invoke_handler`.
6. For streaming: add chunk types in `ai/mod.rs` and handle the relevant cancellation state in the command.

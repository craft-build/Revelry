# AGENTS.md — Revelry

> Reference guide for AI coding agents working on this project.
> All paths are relative to the project root unless noted.

---

## Project Overview

**Package name:** `revelry`
**What it is:** AI roleplay desktop application — a Tauri v2 app with a SvelteKit frontend and a Rust backend communicating via Tauri IPC.
**Design spec:** v2.0 (documented in `docs/`).

The app is a single-page application (SSR disabled) built with SvelteKit 2 + Svelte 5 runes for the UI, and Rust (Tauri v2) for the backend. The Rust backend handles all data persistence (SQLite), API key management (OS keyring), LLM API calls (via rig-core), and file I/O. The frontend communicates with the backend exclusively through typed `invoke()` calls and `Channel` streams.

**Both layers must be considered when implementing features.** The frontend never directly accesses the database, filesystem, or LLM APIs — everything goes through Tauri commands. When adding a feature that involves data, AI, or system resources, you must implement both the Rust command and the frontend wiring.

---

## Tech Stack

### Frontend

| Layer           | Technology                 | Notes                                                      |
| --------------- | -------------------------- | ---------------------------------------------------------- |
| Framework       | SvelteKit 2.x + Svelte 5   | Runes mode (`$state`, `$derived`, `$effect`, `$props`)     |
| Language        | TypeScript                 | `strict: true` in `tsconfig.json`                          |
| Styling         | Tailwind CSS v4            | Via `@tailwindcss/vite` plugin, OKLCH color tokens         |
| Components      | bits-ui 2.x                | Headless primitives (Svelte equivalent of Radix/shadcn)    |
| Icons           | @lucide/svelte             | —                                                          |
| Variants        | tailwind-variants (`tv()`) | For component variant definitions (e.g., `buttonVariants`) |
| Class merging   | clsx + tailwind-merge      | Via `cn()` utility in `$lib/utils`                         |
| Bundler         | Vite 8.x                   | —                                                          |
| Package manager | Bun                        | `bun.lock` present                                         |

### Backend

| Layer          | Technology                     | Notes                                                       |
| -------------- | ------------------------------ | ----------------------------------------------------------- |
| Runtime        | Tauri v2                       | Desktop app framework — Rust backend, webview frontend      |
| Language       | Rust                           | Edition 2021                                                |
| Database       | SQLite via rusqlite            | Bundled, migrations via rusqlite_migration                  |
| Keyring        | keyring v4 + keyring-core v1   | OS-native credential store (Keychain/GNOME Keyring/CredMan) |
| AI             | rig-core 0.37                  | Unified LLM client (OpenAI, Anthropic, Ollama, etc.)        |
| Async runtime  | tokio (multi-thread)           | All DB operations via async `DbWorker` actor                |
| Tokenization   | tiktoken-rs 0.11               | Accurate token counting for prompt budgeting                |
| Error handling | thiserror 2                    | Typed `AppError` enum, serialized to frontend               |
| Cancellation   | tokio-util (CancellationToken) | Stream cancellation for chat + generation                   |

---

## Commands

### Frontend

```sh
bun dev          # Start dev server (port 5173) — runs without Tauri backend (uses dev mocks)
bun run build    # Production build
bun run preview  # Preview production build
bun run check    # Type-check with svelte-check
bun run format   # Format all files with Prettier
bun run lint     # Check formatting with Prettier
```

### Backend

```sh
cargo check      # Type-check Rust code (run from src-tauri/)
cargo test       # Run Rust tests (run from src-tauri/)
cargo clippy     # Lint Rust code (run from src-tauri/)
```

### Full app

```sh
bun tauri dev    # Start Tauri dev mode (Rust + frontend together)
bun tauri build  # Production build of the full desktop app
```

---

## Architecture

### Frontend ↔ Backend Communication

The frontend communicates with the Rust backend through **Tauri IPC**:

1. **Request/response** — `invoke("command_name", { args })` returns a `Promise<T>`. Used for CRUD operations, file I/O, settings.
2. **Streaming** — `invoke("command_name", { ..., channel })` where `channel` is a `Channel<T>`. The backend pushes chunks via `channel.send()`. Used for AI streaming (chat, character generation).

**API layer:** `src/lib/data/api.ts` — typed wrappers around all `invoke()` calls. The frontend never calls `invoke()` directly outside this file. Every Tauri command has a corresponding exported async function here.

**Dev-mode fallback:** When `__TAURI_INTERNALS__` is not present in `window` (i.e., running `bun dev` without Tauri), `api.ts` detects this and falls back to mock implementations so the UI can be developed independently. Functions that have mock fallbacks use the `IS_DEV` guard at the top of the function.

### Central state — `AppState`

All application state lives in a single reactive class:

- **File:** `src/lib/stores/app.svelte.ts`
- **Singleton export:** `app` (imported as `import { app } from '$lib/stores/app.svelte'`)
- **Pattern:** Svelte 5 rune-based class — fields use `$state`, getters use `$derived`-style `get`, methods mutate state directly.
- **No legacy Svelte stores.** Do not use `writable()`, `readable()`, or `derived()` from `svelte/store`.
- **Methods call `api.ts` functions**, which in turn call Tauri commands. AppState never calls `invoke()` directly.

This mirrors the Elm/Relm4 AppModel pattern from the design spec — one source of truth, components read from and write to it.

### Type definitions

Frontend types are in `src/lib/types/index.ts`. They mirror the Rust domain types and the SQLite schema. When adding a new Tauri command that returns data, add the corresponding TypeScript type here.

### Backend architecture

The Rust backend follows a layered architecture:

```
src-tauri/src/
├── lib.rs                  # Tauri builder — registers managed state + invoke handler
├── main.rs                 # Entry point
├── error.rs                # AppError enum (thiserror) — all command errors flow through this
├── commands/               # Tauri IPC command handlers (one file per domain)
│   ├── mod.rs              # Re-exports all command modules
│   ├── characters.rs       # Character CRUD + search
│   ├── chat.rs             # start_chat, stop_generation, regenerate, edit_and_resend
│   ├── conversations.rs   # Conversation CRUD, messages, branches, search
│   ├── generate.rs         # generate_character, stop_character_generation
│   ├── providers.rs        # Provider CRUD, API key management, keyring check
│   ├── lorebooks.rs        # Lorebook + entry CRUD, assignment, conversation lorebooks
│   ├── personas.rs         # Persona CRUD + search
│   ├── presets.rs          # Generation preset CRUD
│   ├── avatars.rs          # Upload/delete avatar images
│   ├── card_io.rs          # Import/export character cards (PNG, JSON)
│   ├── keywords.rs         # Lorebook keyword testing
│   ├── regex.rs             # Regex script CRUD + test/validate
│   └── app_settings.rs     # Key-value app settings (theme, code theme)
├── domain/                 # Domain types — pure data, no I/O
│   ├── character.rs        # CharacterCardV2 (+ V1/V2 spec deserialization)
│   ├── conversation.rs     # Conversation, Message, MessageRole
│   ├── provider.rs         # ProviderConfig, ProviderType, GenerationParams
│   ├── persona.rs          # Persona
│   ├── generation.rs       # GenerationPreset
│   ├── lorebook.rs         # Lorebook, LorebookEntry, LorebookActivationDetail
│   └── regex_script.rs     # RegexScript
├── ai/                     # AI service layer — LLM interaction
│   └── mod.rs              # AiService (stream_chat, stream_generate_character),
│                            # StreamChunk, GenerateCharacterChunk, GenerationParams
├── service/                # Business logic services
│   ├── prompt.rs           # PromptEngine — assembles system prompts with persona/lorebook
│   ├── template.rs         # Persona template compression (tiered)
│   ├── token_count.rs      # TokenCountService — tiktoken-rs + context window resolution
│   ├── regex.rs            # apply_regex_scripts — post-processing
│   ├── lorebook.rs         # Lorebook keyword matching
│   └── card_io.rs          # Character card import/export logic
├── db/                     # Database layer (async actor pattern)
│   ├── mod.rs              # DbWorker — tokio mpsc actor; DbCommand enum
│   └── migrations/         # SQLite schema migrations
└── secrets/
    └── mod.rs              # KeyringStore — OS keychain for API keys
```

### Key backend patterns

1. **DbWorker actor pattern.** All database operations go through a `DbWorker` that runs on a background tokio task. Commands are sent via `mpsc` channels (`db.send(DbCommand::...)`) with oneshot replies. This avoids blocking the Tauri async runtime with synchronous SQLite calls.

2. **Command pattern.** Every Tauri command is a `#[tauri::command]` async function in `commands/`. It receives `State<'_, DbWorker>` (and optionally `State<'_, CancelState>`) as the first args, then business parameters. It returns `Result<(), AppError>` (or `Result<T, AppError>` / `Result<T, String>`).

3. **Streaming pattern.** Commands that stream AI responses receive a `Channel<StreamChunk>` (or `Channel<GenerateCharacterChunk>`) parameter named `channel`. They load context, resolve params, then spawn a tokio task that calls `AiService::stream_chat()` (or `AiService::stream_generate_character()`). The AI service pushes chunks through the channel; the frontend's `onEvent.onmessage` handler processes them.

4. **Cancellation pattern.** Chat and generation use a `CancelState` managed state (`Mutex<Option<CancellationToken>>`). Starting a new stream cancels any existing one. `stop_generation` / `stop_character_generation` simply cancel the token.

5. **API key flow.** Provider API keys are NEVER stored in the database. They live exclusively in the OS keychain via `KeyringStore`. Commands load keys via `tokio::task::spawn_blocking(KeyringStore::load_key(...))` since keyring operations are synchronous.

6. **Error handling.** All command errors go through `AppError` (defined in `error.rs`), which implements `Serialize` so errors are sent to the frontend as structured data. The frontend catches these as `string` errors from `invoke()`.

7. **Serde conventions.** Rust types use `#[serde(rename_all = "camelCase")]` for JSON serialization to the frontend. The Tauri v2 IPC bridge auto-converts snake_case Rust function args to camelCase on the frontend side.

### Backend ↔ Frontend type mapping

When adding a new Tauri command that returns structured data:

1. Define the Rust domain type in `domain/` with `#[derive(Serialize)]` and `rename_all = "camelCase"`.
2. Define the matching TypeScript type in `src/lib/types/index.ts`.
3. Add the `invoke()` wrapper in `src/lib/data/api.ts` with the correct Rust command name (snake_case).
4. Add the AppState method in `src/lib/stores/app.svelte.ts` that calls the API function.
5. Register the command in `src-tauri/src/lib.rs` → `invoke_handler`.

### Dev-mode operation

When running `bun dev` (without Tauri), the frontend detects the absence of `__TAURI_INTERNALS__` in `window` and falls back to mock implementations in `api.ts`. This allows UI development without the Rust backend. Mock data is in `src/lib/data/mock.ts`.

**When a feature has both a real Tauri invoke and a mock fallback**, the mock is gated by `IS_DEV`:

```ts
const IS_DEV = typeof window !== "undefined" && !("__TAURI_INTERNALS__" in window);

export async function someCommand(...): Promise<T> {
  if (IS_DEV) {
    return mockSomeCommand(...);
  }
  return invoke<T>("some_command", { ... });
}
```

**Not all functions have mock fallbacks.** Commands that don't have mocks will throw if called in dev mode. This is acceptable for features that require the backend (e.g., keyring operations, file I/O). Add mocks only for features that should be usable during frontend-only development.

---

## Conventions & Patterns

### Svelte 5 runes — mandatory

| Do                                              | Don't                                          |
| ----------------------------------------------- | ---------------------------------------------- |
| `$state()` for reactive state                   | `writable()`, `readable()` from `svelte/store` |
| `$derived` / `get` for computed values          | `derived()` from `svelte/store`                |
| `$effect()` for side effects                    | `$:` reactive statements                       |
| `$props()` for component props                  | `export let`                                   |
| `children?: Snippet` + `{@render children?.()}` | `<slot />`                                     |

### Component props pattern

```svelte
<script lang="ts">
  import type { Snippet } from 'svelte';
  import { cn } from '$lib/utils';

  interface Props {
    class?: string;
    children?: Snippet;
    // ... other props
  }

  let { class: className, children, ...rest }: Props = $props();
</script>
```

### Variant components (module script pattern)

UI primitives that expose variant helpers use a **module script** block:

```svelte
<script lang="ts" module>
  import { tv, type VariantProps } from 'tailwind-variants';

  export const buttonVariants = tv({
    base: '...',
    variants: { variant: { ... }, size: { ... } },
    defaultVariants: { variant: 'secondary', size: 'md' }
  });

  export type ButtonProps = VariantProps<typeof buttonVariants>;
</script>
```

This lets consumers import `buttonVariants` without importing the component.

### Class merging

Every component that accepts a `class` prop must merge it through `cn()`:

```svelte
<button class={cn(buttonVariants({ variant, size }), className)} {...rest}>
```

### SSR is disabled

- `src/routes/+layout.ts` sets `export const ssr = false` and `export const prerender = false`.
- This is a desktop SPA — there is no server rendering.

### Path aliases

- `$lib` → `src/lib` (configured in `svelte.config.js`)

### Formatting

- **Prettier** is the sole formatter for frontend code.
- Plugins: `prettier-plugin-svelte`, `prettier-plugin-tailwindcss`.
- Always run `bun run format` before committing style changes.
- **Rust** uses `cargo fmt` (standard rustfmt). Run `cargo fmt` in `src-tauri/` before committing Rust changes.

---

## Directory Structure

### Frontend (`src/`)

```
src/
├── app.css                          # Tailwind v4 @theme tokens + base styles + custom utilities
├── app.html                         # HTML shell (Geist, Fraunces, Geist Mono from Google Fonts)
├── app.d.ts                         # App type augmentations
├── lib/
│   ├── components/
│   │   ├── ui/                      # Bits.ui-styled primitives
│   │   │   ├── Avatar.svelte
│   │   │   ├── Button.svelte        # (module script: buttonVariants export)
│   │   │   ├── Dialog.svelte
│   │   │   ├── Input.svelte
│   │   │   ├── Select.svelte
│   │   │   ├── Tag.svelte
│   │   │   ├── Textarea.svelte
│   │   │   ├── Tooltip.svelte
│   │   │   └── index.ts             # Barrel exports for all UI primitives
│   │   ├── layout/                  # AppHeader, ConversationSidebar
│   │   ├── chat/                    # ChatHeader, MessageBubble, MessageInput, PersonaSwitcher
│   │   ├── characters/              # CharacterCard, CharacterEditorDialog, CharacterGeneratorDialog
│   │   ├── personas/                # PersonaCard, PersonaEditorDialog
│   │   ├── lorebooks/               # LorebookCard, LorebookEditor
│   │   ├── providers/               # ProviderEditorDialog
│   │   └── settings/                # Settings sections
│   ├── stores/
│   │   └── app.svelte.ts            # AppState — single root state class
│   ├── types/
│   │   └── index.ts                 # All TypeScript type definitions
│   ├── data/
│   │   ├── api.ts                   # Typed Tauri invoke wrappers + dev-mode mocks
│   │   └── mock.ts                  # Mock data for development
│   └── utils/
│       └── index.ts                 # cn(), formatRelativeTime(), estimateTokens(), avatarInitials()
└── routes/
    ├── +layout.svelte               # App shell + state hydration + F9 sidebar toggle listener
    ├── +layout.ts                   # SSR off (ssr=false, prerender=false)
    ├── +page.svelte                  # Welcome page with suggested characters
    ├── chat/[id]/
    │   ├── +page.svelte             # Active conversation view (streaming via backend)
    │   └── +page.ts                 # Load function for conversation data
    ├── characters/
    │   └── +page.svelte             # Library: characters/personas/lorebooks tabs
    └── settings/
        └── +page.svelte             # Preferences (providers, generation, etc.)
```

### Backend (`src-tauri/src/`)

See the "Backend architecture" section above for the full directory tree and module descriptions.

---

## Styling System

### Tailwind v4 theme tokens

Defined in `src/app.css` inside an `@theme { }` block. All tokens use **OKLCH** color space.

| Token group  | Examples                                                                                                              |
| ------------ | --------------------------------------------------------------------------------------------------------------------- |
| Surfaces     | `--color-background`, `--color-surface`, `--color-surface-elevated`, `--color-surface-hover`, `--color-surface-input` |
| Foreground   | `--color-foreground`, `--color-muted`, `--color-subtle`                                                               |
| Borders      | `--color-border`, `--color-border-strong`                                                                             |
| Accent       | `--color-accent`, `--color-accent-fg`, `--color-accent-muted`                                                         |
| Semantic     | `--color-destructive`, `--color-warning`, `--color-success`                                                           |
| Chat bubbles | `--color-bubble-user`, `--color-bubble-character`                                                                     |
| Typography   | `--font-sans` (Geist), `--font-serif` (Fraunces), `--font-mono` (Geist Mono)                                          |
| Radii        | `--radius-sm`, `--radius-md`, `--radius-lg`, `--radius-xl`                                                            |

### Using theme tokens in Tailwind classes

Tailwind v4 syntax for referencing CSS custom properties in utility classes:

```
bg-(--color-surface)       → background: var(--color-surface)
text-(--color-muted)       → color: var(--color-muted)
border-(--color-border)    → border-color: var(--color-border)
rounded-(--radius-md)      → border-radius: var(--radius-md)
```

### Custom utility classes (defined in `@layer utilities`)

| Class              | Purpose                                                                              |
| ------------------ | ------------------------------------------------------------------------------------ |
| `.font-display`    | Fraunces serif for character names; applies OpenType features `ss01`, `ss02`, `liga` |
| `.surface-grain`   | Subtle radial-gradient texture overlay (atmospheric, not noise)                      |
| `.streaming-caret` | Animated `▍` caret on streaming AI responses                                         |
| `.drag-region`     | `-webkit-app-region: drag` for Tauri window dragging                                 |
| `.no-drag`         | `-webkit-app-region: no-drag` to exclude elements from drag region                   |

### Fonts

- **Geist** — sans-serif body font (loaded from Google Fonts in `app.html`)
- **Fraunces** — serif display font for character names (`.font-display`)
- **Geist Mono** — monospace for code/tokens

### Theme

- **Dark by default** (`<html class="dark">` in `app.html`).
- Light theme is implemented (theme state persisted in app settings).

---

## State Management

### `AppState` API

**Import:** `import { app } from '$lib/stores/app.svelte';`

#### State fields (all reactive via `$state`)

| Field                         | Type                             | Description                                         |
| ----------------------------- | -------------------------------- | --------------------------------------------------- |
| `characters`                  | `CharacterCard[]`                | All character cards                                 |
| `personas`                    | `UserPersona[]`                  | All user personas                                   |
| `conversations`               | `Conversation[]`                 | All conversations                                   |
| `providers`                   | `Provider[]`                     | Configured LLM providers                            |
| `presets`                     | `GenerationPreset[]`             | Saved generation parameter presets                  |
| `lorebooks`                   | `Lorebook[]`                     | All lorebooks                                       |
| `active_conversation_id`      | `string \| null`                 | Currently open conversation                         |
| `messages`                    | `Message[]`                      | Messages for the active conversation                |
| `is_generating`               | `boolean`                        | Whether AI is currently streaming a chat response   |
| `is_generating_character`     | `boolean`                        | Whether AI is currently generating a character      |
| `generated_character_preview` | `string`                         | Accumulated streaming text for character generation |
| `generated_character_data`    | `GeneratedCharacterData \| null` | Parsed result after generation completes            |
| `generation_error`            | `string`                         | Error from last character generation attempt        |
| `sidebar_open`                | `boolean`                        | Sidebar visibility                                  |
| `character_search_query`      | `string`                         | Search filter for character library                 |

#### Derived getters

| Getter                           | Returns                      | Logic                                                              |
| -------------------------------- | ---------------------------- | ------------------------------------------------------------------ |
| `active_conversation`            | `Conversation \| undefined`  | Finds by `active_conversation_id`                                  |
| `active_character`               | `CharacterCard \| undefined` | Character from active conversation                                 |
| `active_persona`                 | `UserPersona \| undefined`   | Persona from active conversation (falls back to `default_persona`) |
| `default_persona`                | `UserPersona \| undefined`   | First persona with `is_default: true`                              |
| `filtered_characters`            | `CharacterCard[]`            | Characters filtered by search query                                |
| `available_generation_providers` | `Provider[]`                 | Providers that are enabled AND have an API key                     |

#### Actions

| Method                                       | Description                                                                     |
| -------------------------------------------- | ------------------------------------------------------------------------------- |
| `hydrate()`                                  | Load all data from backend via API calls                                        |
| `openConversation(id, messages)`             | Set active conversation and its messages                                        |
| `closeConversation()`                        | Clear active conversation and messages                                          |
| `appendMessage(message)`                     | Add a message to the active conversation                                        |
| `updateStreamingMessage(id, content, done?)` | Update a streaming message's content; marks `is_generating = false` when `done` |
| `toggleSidebar()`                            | Toggle sidebar visibility                                                       |
| `switchPersona(persona_id)`                  | Change the persona on the active conversation                                   |
| `generateCharacter(idea, providerId)`        | Start AI character generation via backend                                       |
| `clearGeneration()`                          | Reset all character generation state                                            |
| `createCharacter(data)`                      | Save a new character via backend                                                |
| `updateCharacter(character)`                 | Update existing character via backend                                           |
| `deleteCharacter(id)`                        | Delete character via backend                                                    |
| `importCharacterFromFile()`                  | Import character card from file (opens native file dialog)                      |
| `exportCharacter(id)`                        | Export character card to file (opens native save dialog)                        |
| `startChat(content)`                         | Send user message and start AI streaming                                        |
| `stopGeneration()`                           | Cancel active AI stream                                                         |

---

## Data Model Types

All frontend types are in `src/lib/types/index.ts`. All backend domain types are in `src-tauri/src/domain/`.

### Core types

| Type                     | Frontend file    | Backend file             | Purpose                                                          |
| ------------------------ | ---------------- | ------------------------ | ---------------------------------------------------------------- |
| `CharacterCard`          | `types/index.ts` | `domain/character.rs`    | AI character with personality, scenario, first message, etc.     |
| `LorebookEntry`          | `types/index.ts` | `domain/lorebook.rs`     | Context-injection entries with position, depth, activation rules |
| `UserPersona`            | `types/index.ts` | `domain/persona.rs`      | User's role/identity in roleplay                                 |
| `Message`                | `types/index.ts` | `domain/conversation.rs` | Chat message with role, content, token count, metadata           |
| `Conversation`           | `types/index.ts` | `domain/conversation.rs` | Conversation record linked to character and optional persona     |
| `Provider`               | `types/index.ts` | `domain/provider.rs`     | LLM provider configuration (type, base URL, model, gen params)   |
| `GenerationPreset`       | `types/index.ts` | `domain/generation.rs`   | Saved generation parameter preset                                |
| `GeneratedCharacterData` | `types/index.ts` | `ai/mod.rs`              | Partial character data from AI generation (creative fields only) |
| `GenerateCharacterChunk` | `types/index.ts` | `ai/mod.rs`              | Streaming chunk for character generation (token/complete/error)  |
| `StreamChunk`            | `types/index.ts` | `ai/mod.rs`              | Streaming chunk for chat (token/complete/error)                  |

### Union/enum types

| Type                 | Values                                                           |
| -------------------- | ---------------------------------------------------------------- |
| `MessageRole`        | `'user' \| 'assistant' \| 'system'`                              |
| `LorebookPosition`   | `'before_char' \| 'after_char' \| 'at_depth'`                    |
| `LorebookActivation` | `'always' \| 'character_only' \| 'user_only'`                    |
| `ProviderType`       | `'openai' \| 'anthropic' \| 'ollama' \| ...` (15 provider types) |
| `PrimaryView`        | `'chat' \| 'characters' \| 'settings'`                           |
| `CharactersTab`      | `'characters' \| 'personas'`                                     |

---

## Routing

| Route         | File                                 | Description                                      |
| ------------- | ------------------------------------ | ------------------------------------------------ |
| `/`           | `src/routes/+page.svelte`            | Welcome page with suggested characters           |
| `/chat/[id]`  | `src/routes/chat/[id]/+page.svelte`  | Active conversation (AI streaming via backend)   |
| `/characters` | `src/routes/characters/+page.svelte` | Library with characters/personas/lorebooks tabs  |
| `/settings`   | `src/routes/settings/+page.svelte`   | Preferences (providers, generation, theme, etc.) |

---

## What's Wired vs. Stubbed

### ✅ Wired (functional via backend)

- App shell with sidebar and conversation list
- Active conversation with AI streaming, auto-scroll, token counting
- Message input with auto-grow textarea, token estimate, and context bar
- Persona switcher in chat header
- Character editor (create/edit/import) with avatar upload, lorebook assignment, regex scripts
- Persona editor (create/edit) with avatar upload
- Lorebook editor (create/edit) with entry management and keyword testing
- Provider management (CRUD, API key storage in OS keyring)
- Generation presets (CRUD)
- Character import/export (PNG, JSON)
- Chat streaming via all 15 LLM provider types
- Chat regeneration, edit-and-resend, branch forking, branch navigation
- Message search (FTS5), conversation search
- Regex scripts (CRUD, test/validate, live application to AI output)
- Lorebook keyword matching and context injection
- Prompt assembly with persona compression, budget-aware truncation
- Settings persistence (theme, code theme, app settings)
- F9 keyboard shortcut for sidebar toggle
- Enter to send / Ctrl+Enter for newline
- Search/filter in character library
- Character generation (AI-powered) with provider selection and streaming preview

### 🚧 Stubbed (placeholder / not yet functional)

- **Markdown rendering** — naive asterisk-to-italic only; no full parser
- **Light theme** — stubbed in CSS; dark-only in practice
- **Vault mode** — not implemented

---

## Gotchas & Important Notes

1. **No legacy stores.** Never import from `svelte/store`. All state uses Svelte 5 runes. The `app` object in `$lib/stores/app.svelte.ts` is a plain class instance with `$state` fields — it is reactive because Svelte 5 compiles `.svelte.ts` files with rune support.

2. **`$props()` not `export let`.** Svelte 5 component props are declared with `$props()`. The `export let` syntax is legacy and must not be used.

3. **Snippet, not slot.** Children are passed as `children?: Snippet` and rendered with `{@render children?.()}`. The `<slot />` element is legacy.

4. **`.svelte.ts` files.** Files ending in `.svelte.ts` are compiled by Svelte's preprocessor and support runes (`$state`, `$derived`, `$effect`). Regular `.ts` files do not.

5. **Tailwind v4 CSS variable syntax.** When referencing theme tokens in utility classes, use the `-(--token)` syntax: `bg-(--color-surface)`, not `bg-surface`. The old `theme()` function and arbitrary value `bg-[var(--color-surface)]` both work but the `-(--token)` shorthand is the project convention.

6. **OKLCH colors.** All custom colors are defined in OKLCH. When adding new tokens, use OKLCH for consistency. Convert from hex/HSL using a tool if needed.

7. **Dark theme only (for now).** The HTML root has `class="dark"`. Light mode is stubbed. When it's implemented, check that new components work in both themes.

8. **Body scroll is disabled.** `body { overflow: hidden }` in `app.css` — this is intentional for the desktop app feel. Scroll happens inside component containers, not on the page body.

9. **Module script for variant exports.** If a component needs to export a variant helper (like `buttonVariants`), put it in a `<script lang="ts" module>` block. This is required because Svelte component default exports can't include arbitrary values.

10. **Barrel exports.** UI primitives are re-exported from `src/lib/components/ui/index.ts`. When adding a new UI primitive, add it there. Character components from `src/lib/components/characters/index.ts`, etc.

11. **Backend is NOT optional.** This project has a fully functional Rust backend in `src-tauri/`. When implementing a feature that involves data persistence, AI calls, file I/O, or API keys, you MUST implement the Rust command(s) as well as the frontend. The backend is not "planned" — it is live and wired. Always check `src-tauri/src/commands/` for existing patterns before adding new commands. Register new commands in `src-tauri/src/lib.rs` → `invoke_handler`.

12. **Dev-mode mocks are supplementary.** The `IS_DEV` guard in `api.ts` provides mock fallbacks for frontend-only development. These are NOT a replacement for backend implementation. Every feature that has a mock should also have the real Tauri invoke call. The mock is for developer convenience only.

13. **TypeScript strict mode.** `strict: true` is enabled. Avoid `any`; use proper types from `$lib/types`.

14. **No API keys in code.** API keys are stored in the OS keychain via `KeyringStore` (Rust). The frontend never sees raw API keys — it only knows `has_api_key: boolean` from the provider list. Never hardcode keys, never send keys to the frontend.

15. **Tauri command naming.** Rust command functions use snake_case (e.g., `generate_character`). Tauri v2 auto-converts args to camelCase for the frontend. The invoke command name is the snake_case function name.

16. **Streaming chunk types must match exactly.** When adding a new streaming command, the Rust `#[serde(tag = "event", content = "data")]` enum must produce JSON that matches the TypeScript discriminated union exactly. Both use `camelCase` field names. Test with the serde unit test pattern (see `ai/mod.rs` tests).

17. **DbWorker is async.** Never call `rusqlite` directly from a Tauri command. Send `DbCommand` variants through the `db.send()` channel and await the oneshot reply. See any command file for the pattern.

18. **Keyring operations are synchronous.** Always wrap `KeyringStore` calls in `tokio::task::spawn_blocking()` to avoid blocking the Tauri async runtime.

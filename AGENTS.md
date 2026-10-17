# AGENTS.md — Revelry

> Reference guide for AI coding agents working on this project.
> All paths are relative to the project root unless noted.

---

## Project Overview

**Package name:** `revelry`
**What it is:** AI roleplay desktop application — a Tauri v2 app with a SvelteKit frontend and a Rust backend communicating via Tauri IPC.
**Design spec:** v2.0 (documented in `docs/`). **Visual design:** v2 "Night fox" (see [Styling System](#styling-system)). `docs/` still describes an older Geist/Fraunces + teal look; `src/app.css` is the source of truth for visuals.

The app is a single-page application (SSR disabled) built with SvelteKit 2 + Svelte 5 runes for the UI, and Rust (Tauri v2) for the backend. The Rust backend handles all data persistence (SQLite), API key management (OS keyring), LLM API calls (via rig-core), and file I/O. The frontend communicates with the backend exclusively through typed `invoke()` calls and `Channel` streams.

**Both layers must be considered when implementing features.** The frontend never directly accesses the database, filesystem, or LLM APIs — everything goes through Tauri commands. When adding a feature that involves data, AI, or system resources, you must implement both the Rust command and the frontend wiring.

---

## Tech Stack

### Frontend

| Layer           | Technology                 | Notes                                                      |
| --------------- | -------------------------- | ---------------------------------------------------------- |
| Framework       | SvelteKit 2.x + Svelte 5   | Runes mode (`$state`, `$derived`, `$effect`, `$props`)     |
| Language        | TypeScript                 | `strict: true` in `tsconfig.json`                          |
| Styling         | Tailwind CSS v4            | Via `@tailwindcss/vite` plugin, hex tokens in `@theme`     |
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
| AI             | rig 0.44                       | Unified LLM client (OpenAI, Anthropic, Ollama, etc.)        |
| Async runtime  | tokio (multi-thread)           | All DB operations via async `DbWorker` actor                |
| Tokenization   | tiktoken-rs 0.11               | Accurate token counting for prompt budgeting                |
| Error handling | thiserror 2                    | Typed `AppError` enum, serialized to frontend               |
| Cancellation   | tokio-util (CancellationToken) | Stream cancellation for chat + generation                   |

---

## Commands

### Frontend

```sh
bun dev          # Start Vite dev server (port 1420) — without Tauri most data calls fail; see Dev-mode operation
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

**Dev-mode fallback:** When `__TAURI_INTERNALS__` is not present in `window` (i.e., running `bun dev` without Tauri), `api.ts` sets `IS_DEV`. Functions that have mock fallbacks check it at the top of the function. Currently only `generateCharacter` has a mock — everything else calls `invoke()` and fails, so use `bun tauri dev` for real UI work.

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
│   ├── keywords.rs         # Lorebook keyword + semantic activation testing
│   ├── regex.rs             # Regex script CRUD + test/validate
│   ├── embeddings.rs       # Embedding config/status/test/rebuild commands
│   └── app_settings.rs     # Key-value app settings (theme, code theme)
├── domain/                 # Domain types — pure data, no I/O
│   ├── character.rs        # CharacterCardV2 (+ V1/V2 spec deserialization)
│   ├── conversation.rs     # Conversation, Message, MessageRole
│   ├── provider.rs         # ProviderConfig, ProviderType, GenerationParams
│   ├── persona.rs          # Persona
│   ├── generation.rs       # GenerationPreset
│   ├── lorebook.rs         # Lorebook, LorebookEntry, LorebookActivationDetail
│   ├── embedding.rs        # EmbeddingConfig, EmbeddingStatus, EmbeddingTestResult
│   └── regex_script.rs     # RegexScript
├── ai/                     # AI service layer — LLM interaction
│   ├── mod.rs              # AiService facade + re-exports
│   ├── chat.rs             # stream_chat (+ PostStreamHook for post-stream maintenance)
│   ├── generate.rs         # stream_generate_character
│   ├── providers.rs        # rig-core client construction per ProviderType
│   ├── model_list.rs       # Remote model listing / detection
│   ├── embeddings.rs       # EmbeddingRuntime — remote/local embedders, index maintenance
│   ├── summary.rs          # Rolling story-summary generation (memory layer)
│   ├── types.rs            # StreamChunk, GenerateCharacterChunk, GenerationParams
│   └── util.rs             # Shared helpers
├── service/                # Business logic services
│   ├── prompt.rs           # PromptEngine — assembles system prompts with persona/lorebook/memory
│   ├── template.rs         # Persona template compression (tiered)
│   ├── token_count.rs      # TokenCountService — tiktoken-rs + context window resolution
│   ├── regex.rs            # apply_regex_scripts — post-processing
│   ├── lorebook.rs         # Lorebook keyword + semantic activation
│   ├── vector.rs           # Cosine similarity, content hashing, vector encode/decode
│   ├── lorebook_io.rs      # Lorebook import/export
│   └── card_io.rs          # Character card import/export logic
├── db/                     # Database layer (async actor pattern)
│   ├── mod.rs              # DbWorker — tokio mpsc actor; DbCommand enum
│   ├── migrations.rs       # SQLite schema migrations
│   ├── embeddings.rs       # Derived vector index queries (semantic activation/recall/search)
│   └── <domain>.rs         # Per-domain queries (characters, conversations, lorebook, …)
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

When running `bun dev` (without Tauri), the frontend detects the absence of `__TAURI_INTERNALS__` in `window`. Mock implementations live in `src/lib/data/mock.ts`, but today only character generation is mocked — `app.hydrate()` calls real commands, so the app shows the "Unable to load data" screen without the backend. Use `bun tauri dev` to see real screens.

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
├── app.css                          # v2 palette, Tailwind v4 @theme tokens, @font-face, base styles, utilities, markdown prose
├── app.html                         # HTML shell + pre-paint theme script (adds html.light / html.dark)
├── app.d.ts                         # App type augmentations (incl. __APP_VERSION__)
├── lib/
│   ├── components/
│   │   ├── ui/                      # Bits.ui-styled primitives
│   │   │   ├── Avatar.svelte        # Image or hashed tail-tint fallback (tone prop)
│   │   │   ├── AvatarUpload.svelte
│   │   │   ├── Button.svelte        # (module script: buttonVariants export)
│   │   │   ├── Dialog.svelte
│   │   │   ├── Input.svelte / Select.svelte / Textarea.svelte
│   │   │   ├── KebabMenu.svelte (+ KebabMenuItem, KebabMenuSeparator)
│   │   │   ├── Logo.svelte          # static/logo.png — the fox app icon
│   │   │   ├── StepperHeader.svelte
│   │   │   ├── Tag.svelte
│   │   │   ├── ToastContainer.svelte
│   │   │   ├── Tooltip.svelte
│   │   │   └── index.ts             # Barrel exports for all UI primitives
│   │   ├── layout/                  # AppHeader, ConversationSidebar
│   │   ├── chat/                    # ChatHeader, MessageBubble, MessageInput (floating composer w/ persona + model chips),
│   │   │                            # PersonaSwitcher, ProviderModelSelector, Branch*, Lorebook panels, GenerationPanel, MarkdownRenderer
│   │   ├── characters/              # CharacterCard, CharacterEditorDialog, CharacterGeneratorDialog
│   │   ├── personas/                # PersonaCard, PersonaEditorDialog
│   │   ├── lorebooks/               # LorebookCard, LorebookEditor
│   │   ├── providers/               # ProviderEditorDialog, ModelEntryRow, KeyringUnavailableBanner
│   │   └── settings/                # ThemeToggle, CodeThemeSelector, PresetEditorDialog
│   ├── stores/
│   │   └── app.svelte.ts            # AppState — single root state class
│   ├── types/
│   │   └── index.ts                 # All TypeScript type definitions
│   ├── data/
│   │   ├── api.ts                   # Typed Tauri invoke wrappers + dev-mode mocks
│   │   └── mock.ts                  # Mock data for development
│   ├── shiki.svelte.ts              # Shiki highlighter (code blocks in messages)
│   └── utils/
│       └── index.ts                 # cn(), formatRelativeTime(), conversationGroup(), estimateTokens(),
│                                    # avatarInitials(), avatarTone(), resolveAvatarUrl(), parseSteering()
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

The visual language is **v2 "Night fox"**, defined by the `revelry-design` skill (tokens, UI kit, guidelines). Mood: a fox at night — a near-black blue field, one cool blue light, warm embers at the edge. Every brand colour comes from the mascot icon (`static/logo.png`). If a design question isn't answered here, check the skill before inventing something.

### Tokens (`src/app.css`)

- **Raw palette** lives in a plain `:root` block: `--night-950…600`, `--blue-200…800`, `--purple-*`, `--pink-*`, `--ember-*`, `--mint-*`, `--coral-*`, plus the gradients (`--gradient-brand`, `--gradient-avatar-{blue,purple,pink,ember}`). These are in `:root` rather than `@theme` so Tailwind always emits them, even when they're only referenced from inline styles or template strings.
- **Semantic tokens** live in `@theme` and are what components use. Colours are hex/rgba (not OKLCH).

| Token group  | Tokens (dark values)                                                                                                                                                     |
| ------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Surfaces     | `--color-background` #07080d → `--color-surface` #10121c → `--color-surface-elevated` #161928 → `--color-surface-hover` #1e2235; `--color-surface-input` #0b0d15 (inset) |
| Foreground   | `--color-foreground`, `--color-muted`, `--color-subtle`                                                                                                                  |
| Borders      | `--color-border` (white 8%), `--color-border-strong` (white 20%)                                                                                                         |
| Accent       | `--color-accent` fox blue #3365fc (+ `-hover`, `-active`, `-fg`, `-muted`); `--color-accent-vivid` #8fa8ff when accent is text/icon on dark                              |
| Tail accents | `--color-accent-secondary` purple, `--color-accent-tertiary` pink, `--color-accent-warm` ember — brand moments only                                                      |
| Semantic     | `--color-destructive` coral, `--color-warning` ember, `--color-success` mint                                                                                             |
| Chat bubbles | `--color-bubble-user` (solid accent), `--color-bubble-character` (elevated)                                                                                              |
| Typography   | `--font-sans` Public Sans, `--font-serif` Space Grotesk (display), `--font-mono` JetBrains Mono                                                                          |
| Radii        | `--radius-sm` 8, `--radius-md` 14, `--radius-lg` 20, `--radius-xl` 28                                                                                                    |
| Shadows      | `--shadow-sm/md/lg` (override Tailwind's), `--shadow-glow` (blue focus glow), `--shadow-glow-ember`; `--shadow-glow-violet` is a legacy alias                            |

### Using theme tokens in Tailwind classes

```
bg-(--color-surface)       → background: var(--color-surface)
text-(--color-muted)       → color: var(--color-muted)
border-(--color-border)    → border-color: var(--color-border)
rounded-(--radius-md)      → border-radius: var(--radius-md)
shadow-(--shadow-glow)     → box-shadow: var(--shadow-glow)
```

### Custom utility classes (defined in `@layer utilities`)

| Class                  | Purpose                                                                                     |
| ---------------------- | ------------------------------------------------------------------------------------------- |
| `.font-display`        | Space Grotesk for names, titles, initials; OpenType `ss01`, `ss02`, `liga`                  |
| `.surface-grain`       | "Night glow" — blue radial top-left, purple top-right, ember bottom-right (on the app root) |
| `.text-gradient-brand` | Tail-sweep gradient text (used once: "life." on Home)                                       |
| `.animate-pop-in`      | Popover entrance (from -4px / 98%)                                                          |
| `.streaming-caret`     | Blinking blue `▍` caret on streaming AI responses                                           |
| `.thinking-dots`       | Three pulsing dots (blue / purple / pink) for an empty streaming bubble                     |
| `.drag-region`         | `-webkit-app-region: drag` for Tauri window dragging                                        |
| `.no-drag`             | `-webkit-app-region: no-drag` to exclude elements from drag region                          |

`.prose-markdown` (in `@layer components`) styles rendered messages. Its colours come from `--md-*` variables that `MarkdownRenderer` sets inline from the selected code theme (Settings → Appearance), so don't hardcode colours or opacity in the prose rules — that breaks the code-theme feature.

### Visual rules

- **One accent.** Fox blue is the only interactive colour: fills, user bubbles, focus. Tail colours (purple, pink, ember) are for brand moments, avatar tints and the streaming indicator. Coral (destructive) stays distinct from tail pink.
- **Gradient** (`--gradient-brand`) is for brand moments only: the Home headline word, 2–3px hairlines on brand cards (About, the generator stream), and `tone="brand"` avatars. **Never** a button or card fill.
- **Avatars** without an image get one of four 2-stop tints hashed from the name (`avatarTone()`), so the same name always gets the same colour.
- **Borders over shadows** for panel separation. Cards: surface + 1px border + `radius-lg` + 16px padding; on hover, strong border + elevated bg + `shadow-md`. Dialogs, popovers, menus, toasts and the composer use `shadow-lg`.
- **Translucency:** the app header (80%) and sidebars (70%) use `bg-(--color-surface)/NN` + `backdrop-blur-[20px]` so the night glow reads through. Overlays are black/60 + 20px blur. Popovers/menus/tooltips: `border-strong`, elevated bg, 20px blur.
- **Focus:** blue border + `shadow-(--shadow-glow)`.
- **Radii:** sm for menu rows/tooltips, md for inputs/tags/chips/tab tracks, lg for buttons/cards/bubbles/banners, xl for dialogs and the composer. Message bubbles have a 6px "speech" corner toward the speaker, and user bubbles carry a faint blue under-glow.
- **Tabs** (header nav, Library) are a segmented track: `surface-input` + border + 3px padding; the active tab is elevated + `shadow-sm` + inset border ring.
- **Motion:** 150ms colour/opacity, 200ms sidebar + toasts, press = `active:scale-[0.98]`. No bounces.
- **Type:** Space Grotesk 600 with tight tracking (-0.02 to -0.03em) for names and titles. Public Sans 14px UI, chat 15px / 1.65. JetBrains Mono for model IDs, params, token counts, regex, keywords, kbd.
- **Icons:** Lucide only (`@lucide/svelte`), 16px default, 14 in small buttons and menus, 12 in message actions. No emoji.
- **Copy:** sentence case, the user is "you", the app never says "I"/"we", real ellipsis "…", middot "·" separators.

### Layout

48px app header (logo + centred Chat / Library / Settings tabs), 288px collapsible conversation sidebar grouped Today / This week / Earlier (F9), 60px chat header, centred 80ch chat column, a **floating composer** (persona + model chips, mini context meter, send) with the hint line beneath, and a 288px right-hand Generation panel. The body never scrolls.

### Fonts

Self-hosted woff2 in `static/fonts/`, declared with `@font-face` at the top of `app.css`: **Space Grotesk** (display), **Public Sans** (body), **JetBrains Mono** (mono). No Google Fonts.

### Theme

- Dark is the default. A pre-paint script in `app.html` reads `localStorage["revelry-theme"]` (`light` / `dark` / `system`) and adds `html.light` or `html.dark`. `+layout.svelte` follows OS changes when set to `system`.
- Light theme overrides live in `html.light { … }` in `app.css`, using darker steps of the same fox hues. Check new components in both themes.

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
| `embedding_config`            | `EmbeddingConfig`                | Embedding source selection (none/remote/local)      |
| `embedding_status`            | `EmbeddingStatus \| null`        | Indexed counts + stale rows for the current model   |
| `search_semantic`             | `boolean`                        | Semantic toggle for sidebar message search          |

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
| `GeneratedCharacterData` | `types/index.ts` | `ai/types.rs`            | Partial character data from AI generation (creative fields only) |
| `GenerateCharacterChunk` | `types/index.ts` | `ai/types.rs`            | Streaming chunk for character generation (token/complete/error)  |
| `StreamChunk`            | `types/index.ts` | `ai/types.rs`            | Streaming chunk for chat (token/complete/error)                  |

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
- Floating composer with auto-grow textarea, persona + provider/model chips, and mini context meter
- Persona switcher and model selector in the composer
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

- Markdown rendering (marked + DOMPurify) with Shiki code highlighting and selectable code theme
- Light / dark / system theme
- Embeddings (opt-in): remote provider sources (incl. OpenAI-compatible endpoints) or local fastembed models, with Settings management, connection test, and background index rebuilds
- Semantic / hybrid lorebook activation with per-entry thresholds and semantic test preview
- Story memory: rolling conversation summaries (works without embeddings) + semantic recall of out-of-window messages
- Hybrid conversation search (FTS5 + optional semantic toggle in the sidebar)

### 🚧 Stubbed (placeholder / not yet functional)

- **Data settings** (export, backup, restore) — "Coming soon" placeholder in Settings
- **Vault mode** — not implemented

---

## Gotchas & Important Notes

1. **No legacy stores.** Never import from `svelte/store`. All state uses Svelte 5 runes. The `app` object in `$lib/stores/app.svelte.ts` is a plain class instance with `$state` fields — it is reactive because Svelte 5 compiles `.svelte.ts` files with rune support.

2. **`$props()` not `export let`.** Svelte 5 component props are declared with `$props()`. The `export let` syntax is legacy and must not be used.

3. **Snippet, not slot.** Children are passed as `children?: Snippet` and rendered with `{@render children?.()}`. The `<slot />` element is legacy.

4. **`.svelte.ts` files.** Files ending in `.svelte.ts` are compiled by Svelte's preprocessor and support runes (`$state`, `$derived`, `$effect`). Regular `.ts` files do not.

5. **Tailwind v4 CSS variable syntax.** When referencing theme tokens in utility classes, use the `-(--token)` syntax: `bg-(--color-surface)`, not `bg-surface`. The old `theme()` function and arbitrary value `bg-[var(--color-surface)]` both work but the `-(--token)` shorthand is the project convention.

6. **Colours come from the palette.** Don't hardcode hex values in components. Use the semantic tokens; reach for the raw palette (`--blue-400` etc.) only where the design calls for a specific tail colour. New colours should be added as tokens in `app.css` (hex, matching the existing ones) with a light-theme override. Variables referenced only from inline styles or template strings must live in `:root`, not `@theme`, or Tailwind may drop them.

7. **Both themes.** Dark is the default, but light (`html.light`) is live. Check new components in both.

8. **Body scroll is disabled.** `body { overflow: hidden }` in `app.css` — this is intentional for the desktop app feel. Scroll happens inside component containers, not on the page body.

9. **Module script for variant exports.** If a component needs to export a variant helper (like `buttonVariants`), put it in a `<script lang="ts" module>` block. This is required because Svelte component default exports can't include arbitrary values.

10. **Barrel exports.** UI primitives are re-exported from `src/lib/components/ui/index.ts`. When adding a new UI primitive, add it there. Character components from `src/lib/components/characters/index.ts`, etc.

11. **Backend is NOT optional.** This project has a fully functional Rust backend in `src-tauri/`. When implementing a feature that involves data persistence, AI calls, file I/O, or API keys, you MUST implement the Rust command(s) as well as the frontend. The backend is not "planned" — it is live and wired. Always check `src-tauri/src/commands/` for existing patterns before adding new commands. Register new commands in `src-tauri/src/lib.rs` → `invoke_handler`.

12. **Dev-mode mocks are supplementary.** The `IS_DEV` guard in `api.ts` provides mock fallbacks for frontend-only development. These are NOT a replacement for backend implementation. Every feature that has a mock should also have the real Tauri invoke call. The mock is for developer convenience only.

13. **TypeScript strict mode.** `strict: true` is enabled. Avoid `any`; use proper types from `$lib/types`.

14. **No API keys in code.** API keys are stored in the OS keychain via `KeyringStore` (Rust). The frontend never sees raw API keys — it only knows `has_api_key: boolean` from the provider list. Never hardcode keys, never send keys to the frontend.

15. **Tauri command naming.** Rust command functions use snake_case (e.g., `generate_character`). Tauri v2 auto-converts args to camelCase for the frontend. The invoke command name is the snake_case function name.

16. **Streaming chunk types must match exactly.** When adding a new streaming command, the Rust `#[serde(tag = "event", content = "data")]` enum must produce JSON that matches the TypeScript discriminated union exactly. Both use `camelCase` field names. Test with the serde unit test pattern (see the tests in `ai/`).

17. **DbWorker is async.** Never call `rusqlite` directly from a Tauri command. Send `DbCommand` variants through the `db.send()` channel and await the oneshot reply. See any command file for the pattern.

18. **Keyring operations are synchronous.** Always wrap `KeyringStore` calls in `tokio::task::spawn_blocking()` to avoid blocking the Tauri async runtime.

# AGENTS.md — Revelry UI

> Reference guide for AI coding agents working on this project.
> All paths are relative to the project root (`persona/`).

---

## Project Overview

**Package name:** `revelry-ui`
**What it is:** AI roleplay desktop application UI — a front-end that can be embedded in a Tauri window or run standalone in the browser.
**Design spec:** v2.0 (documented in `docs/`).

The app is a single-page application (SSR disabled) built with SvelteKit 2 + Svelte 5 runes. It presents a chat interface for AI character roleplay, with a character/persona library, conversation management, and a settings panel. The current scaffold uses mock data; a real Rust backend (via Tauri IPC) is planned.

---

## Tech Stack

| Layer | Technology | Notes |
|-------|-----------|-------|
| Framework | SvelteKit 2.x + Svelte 5 | Runes mode (`$state`, `$derived`, `$effect`, `$props`) |
| Language | TypeScript | `strict: true` in `tsconfig.json` |
| Styling | Tailwind CSS v4 | Via `@tailwindcss/vite` plugin, OKLCH color tokens |
| Components | bits-ui 2.x | Headless primitives (Svelte equivalent of Radix/shadcn) |
| Icons | @lucide/svelte | — |
| Variants | tailwind-variants (`tv()`) | For component variant definitions (e.g., `buttonVariants`) |
| Class merging | clsx + tailwind-merge | Via `cn()` utility in `$lib/utils` |
| Bundler | Vite 8.x | — |
| Package manager | Bun | `bun.lock` present |

---

## Commands

```sh
bun dev          # Start dev server (port 5173)
bun run build    # Production build
bun run preview  # Preview production build
bun run check    # Type-check with svelte-check
bun run format   # Format all files with Prettier
bun run lint     # Check formatting with Prettier
```

---

## Architecture

### Central state — `AppState`

All application state lives in a single reactive class:

- **File:** `src/lib/stores/app.svelte.ts`
- **Singleton export:** `app` (imported as `import { app } from '$lib/stores/app.svelte'`)
- **Pattern:** Svelte 5 rune-based class — fields use `$state`, getters use `$derived`-style `get`, methods mutate state directly.
- **No legacy Svelte stores.** Do not use `writable()`, `readable()`, or `derived()` from `svelte/store`.

This mirrors the Elm/Relm4 AppModel pattern from the design spec — one source of truth, components read from and write to it.

### Type definitions

All data types live in one file:

- **File:** `src/lib/types/index.ts`
- Types mirror the SQLite schema from §12.1 of the design spec.

### Mock data

- **File:** `src/lib/data/mock.ts`
- Provides `mockCharacters`, `mockPersonas`, `mockConversations`, `mockMessagesForConv01`, `mockProviders`, `mockPresets` for development without a backend.

### Utilities

- **File:** `src/lib/utils/index.ts`
- `cn(...inputs)` — merge Tailwind classes with conflict resolution (clsx + tailwind-merge). Used everywhere.
- `formatRelativeTime(iso)` — smart time formatting for the sidebar (today → time, this week → weekday, older → date).
- `estimateTokens(text)` — 4-char-per-token heuristic. Placeholder for tiktoken-rs via backend.
- `avatarInitials(name)` — initial-letter seed for avatar fallbacks.

---

## Conventions & Patterns

### Svelte 5 runes — mandatory

| Do | Don't |
|----|-------|
| `$state()` for reactive state | `writable()`, `readable()` from `svelte/store` |
| `$derived` / `get` for computed values | `derived()` from `svelte/store` |
| `$effect()` for side effects | `$:` reactive statements |
| `$props()` for component props | `export let` |
| `children?: Snippet` + `{@render children?.()}` | `<slot />` |

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

- **Prettier** is the sole formatter.
- Plugins: `prettier-plugin-svelte`, `prettier-plugin-tailwindcss`.
- Always run `bun run format` before committing style changes.

---

## Directory Structure

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
│   │   │   ├── Input.svelte
│   │   │   ├── Tag.svelte
│   │   │   ├── Tooltip.svelte
│   │   │   └── index.ts             # Barrel exports for all UI primitives
│   │   ├── layout/                  # AppHeader, ConversationSidebar
│   │   ├── chat/                    # ChatHeader, MessageBubble, MessageInput, PersonaSwitcher
│   │   ├── characters/              # CharacterCard
│   │   └── personas/                # PersonaCard
│   ├── stores/
│   │   └── app.svelte.ts            # AppState — single root state class
│   ├── types/
│   │   └── index.ts                 # All TypeScript type definitions
│   ├── data/
│   │   └── mock.ts                  # Mock data for development
│   └── utils/
│       └── index.ts                 # cn(), formatRelativeTime(), estimateTokens(), avatarInitials()
└── routes/
    ├── +layout.svelte               # App shell + state hydration + F9 sidebar toggle listener
    ├── +layout.ts                   # SSR off (ssr=false, prerender=false)
    ├── +page.svelte                  # Welcome page with suggested characters
    ├── chat/[id]/
    │   ├── +page.svelte             # Active conversation view (mock streaming)
    │   └── +page.ts                 # Load function for conversation data
    ├── characters/
    │   └── +page.svelte             # Library: characters & personas tabs (tab synced to URL ?tab=)
    └── settings/
        └── +page.svelte             # Preferences (providers, generation, etc.)
```

---

## Styling System

### Tailwind v4 theme tokens

Defined in `src/app.css` inside an `@theme { }` block. All tokens use **OKLCH** color space.

| Token group | Examples |
|-------------|---------|
| Surfaces | `--color-background`, `--color-surface`, `--color-surface-elevated`, `--color-surface-hover`, `--color-surface-input` |
| Foreground | `--color-foreground`, `--color-muted`, `--color-subtle` |
| Borders | `--color-border`, `--color-border-strong` |
| Accent | `--color-accent`, `--color-accent-fg`, `--color-accent-muted` |
| Semantic | `--color-destructive`, `--color-warning`, `--color-success` |
| Chat bubbles | `--color-bubble-user`, `--color-bubble-character` |
| Typography | `--font-sans` (Geist), `--font-serif` (Fraunces), `--font-mono` (Geist Mono) |
| Radii | `--radius-sm`, `--radius-md`, `--radius-lg`, `--radius-xl` |

### Using theme tokens in Tailwind classes

Tailwind v4 syntax for referencing CSS custom properties in utility classes:

```
bg-(--color-surface)       → background: var(--color-surface)
text-(--color-muted)       → color: var(--color-muted)
border-(--color-border)    → border-color: var(--color-border)
rounded-(--radius-md)      → border-radius: var(--radius-md)
```

### Custom utility classes (defined in `@layer utilities`)

| Class | Purpose |
|-------|---------|
| `.font-display` | Fraunces serif for character names; applies OpenType features `ss01`, `ss02`, `liga` |
| `.surface-grain` | Subtle radial-gradient texture overlay (atmospheric, not noise) |
| `.streaming-caret` | Animated `▍` caret on streaming AI responses |
| `.drag-region` | `-webkit-app-region: drag` for Tauri window dragging |
| `.no-drag` | `-webkit-app-region: no-drag` to exclude elements from drag region |

### Fonts

- **Geist** — sans-serif body font (loaded from Google Fonts in `app.html`)
- **Fraunces** — serif display font for character names (`.font-display`)
- **Geist Mono** — monospace for code/tokens

### Theme

- **Dark by default** (`<html class="dark">` in `app.html`).
- Light theme toggle is stubbed (not yet implemented).

---

## State Management

### `AppState` API

**Import:** `import { app } from '$lib/stores/app.svelte';`

#### State fields (all reactive via `$state`)

| Field | Type | Description |
|-------|------|-------------|
| `characters` | `CharacterCard[]` | All character cards |
| `personas` | `UserPersona[]` | All user personas |
| `conversations` | `Conversation[]` | All conversations |
| `providers` | `Provider[]` | Configured LLM providers |
| `prompt_presets` | `PromptPreset[]` | Saved prompt/generation presets |
| `active_conversation_id` | `string | null` | Currently open conversation |
| `messages` | `Message[]` | Messages for the active conversation |
| `is_generating` | `boolean` | Whether AI is currently streaming a response |
| `sidebar_open` | `boolean` | Sidebar visibility |
| `character_search_query` | `string` | Search filter for character library |

#### Derived getters

| Getter | Returns | Logic |
|--------|---------|-------|
| `active_conversation` | `Conversation | undefined` | Finds by `active_conversation_id` |
| `active_character` | `CharacterCard | undefined` | Character from active conversation |
| `active_persona` | `UserPersona | undefined` | Persona from active conversation (falls back to `default_persona`) |
| `default_persona` | `UserPersona | undefined` | First persona with `is_default: true` |
| `filtered_characters` | `CharacterCard[]` | Characters filtered by `character_search_query` (searches name, description, tags) |

#### Actions

| Method | Description |
|--------|-------------|
| `hydrate(data)` | Initialize all state from data object (used at app startup with mock data) |
| `openConversation(id, messages)` | Set active conversation and its messages |
| `closeConversation()` | Clear active conversation and messages |
| `appendMessage(message)` | Add a message to the active conversation |
| `updateStreamingMessage(id, content, done?)` | Update a streaming message's content; marks `is_generating = false` when `done` |
| `toggleSidebar()` | Toggle sidebar visibility |
| `switchPersona(persona_id)` | Change the persona on the active conversation |

---

## Data Model Types

All types are in `src/lib/types/index.ts`.

### Core types

| Type | Purpose |
|------|---------|
| `CharacterCard` | AI character with personality, scenario, first message, etc. (includes `CharacterExtensions`) |
| `LorebookEntry` | Context-injection entries with position, depth, and activation rules |
| `UserPersona` | User's role/identity in roleplay (appearance, personality, speech style, etc.) |
| `Message` | Chat message with role, content, token count, metadata, and `is_streaming` flag |
| `Conversation` | Conversation record linked to a character and optional persona |
| `Provider` | LLM provider configuration (type, base URL, model, generation params) |
| `PromptPreset` | Saved generation parameter preset |

### Union/enum types

| Type | Values |
|------|--------|
| `MessageRole` | `'user' | 'character' | 'system'` |
| `LorebookPosition` | `'before_char' | 'after_char' | 'at_depth'` |
| `LorebookActivation` | `'always' | 'character_only' | 'user_only'` |
| `ProviderType` | `'openai' | 'anthropic' | 'ollama' | 'gemini' | 'cohere' | 'mistral' | 'deepseek' | 'groq' | 'azure_openai' | 'openrouter' | 'together' | 'huggingface' | 'perplexity' | 'xai' | 'custom_openai_compatible'` |
| `PrimaryView` | `'chat' | 'characters' | 'settings'` |
| `CharactersTab` | `'characters' | 'personas'` |

---

## Routing

| Route | File | Description |
|-------|------|-------------|
| `/` | `src/routes/+page.svelte` | Welcome page with suggested characters |
| `/chat/[id]` | `src/routes/chat/[id]/+page.svelte` | Active conversation (chat view with mock streaming) |
| `/characters` | `src/routes/characters/+page.svelte` | Library with characters/personas tabs (`?tab=characters` or `?tab=personas`) |
| `/settings` | `src/routes/settings/+page.svelte` | Preferences with section navigation |

---

## What's Wired vs. Stubbed

### ✅ Wired (functional)

- App shell with sidebar and conversation list
- Active conversation with message rendering and auto-scroll
- Message input with auto-grow textarea, token estimate, and context bar
- Mock streaming AI response (simulated in `chat/[id]/+page.svelte`)
- Persona switcher in chat header
- Character and persona library with tabs
- Settings shell with section navigation
- F9 keyboard shortcut for sidebar toggle
- Enter to send / Ctrl+Enter for newline
- Search/filter in character library

### 🚧 Stubbed (placeholder / not yet functional)

- **Backend bridge** — mock streaming is in `chat/[id]/+page.svelte`; no Tauri IPC yet
- **Markdown rendering** — naive asterisk-to-italic only; no full parser
- **Token counting** — `estimateTokens()` uses 4-char heuristic; real counting needs tiktoken-rs via backend
- **Character editor** — no create/edit UI
- **Persona editor** — no create/edit UI
- **Lorebook UI** — type defined, no interface
- **Theme toggle** — dark only; light mode not yet implemented
- **Branch navigation** — `parent_id` / `branch_index` fields exist on `Message` but no branching UI
- **Regex scripting** — not implemented
- **Full-text search** — not implemented
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

10. **Barrel exports.** UI primitives are re-exported from `src/lib/components/ui/index.ts`. When adding a new UI primitive, add it there.

11. **Mock data hydration.** The app hydrates from `src/lib/data/mock.ts` in the root layout (`+layout.svelte`). When adding new data fields to `AppState`, update the mock data accordingly.

12. **Conversation load pattern.** `chat/[id]/+page.ts` exports a `load` function that returns conversation data. The page component receives it and calls `app.openConversation()`. When wiring the real backend, this is the integration point.

13. **TypeScript strict mode.** `strict: true` is enabled. Avoid `any`; use proper types from `$lib/types`.

14. **No API keys in code.** Provider configuration is for the settings UI only. Any real API keys will flow through the Tauri backend — never hardcode them in the front-end.

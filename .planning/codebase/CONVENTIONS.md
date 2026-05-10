# Coding Conventions

**Analysis Date:** 2026-05-10

## Naming Patterns

**Files:**
- Svelte components: PascalCase (e.g., `CharacterCard.svelte`, `MessageBubble.svelte`, `AppHeader.svelte`)
- TypeScript modules: camelCase or kebab-case (e.g., `app.svelte.ts`, `mock.ts`, `index.ts`)
- Route files: SvelteKit convention (`+page.svelte`, `+page.ts`, `+layout.svelte`, `+layout.ts`)
- Barrel/index files: `index.ts` in each domain directory (e.g., `src/lib/components/ui/index.ts`, `src/lib/types/index.ts`, `src/lib/utils/index.ts`)

**Functions:**
- Utility functions: camelCase (e.g., `cn()`, `formatRelativeTime()`, `estimateTokens()`, `avatarInitials()`, `getMessages()`)
- Event handlers: `handle` prefix (e.g., `handleSend()`, `handleSubmit()`, `handleKey()`, `handleCancel()`)
- Callback props: `on` prefix (e.g., `onchat`, `onedit`, `onsend`, `oncancel`)
- Class methods: camelCase (e.g., `hydrate()`, `openConversation()`, `appendMessage()`, `updateStreamingMessage()`, `toggleSidebar()`, `switchPersona()`)

**Variables:**
- State variables: camelCase with `$state()` rune (e.g., `search`, `text`, `tab`, `active`)
- Derived values: camelCase with `$derived()` rune (e.g., `filtered`, `tokens`, `isUser`, `contextPct`, `showConvSidebar`)
- DOM refs: camelCase (e.g., `scrollEl`, `textarea`)
- Constants: camelCase for local (e.g., `contextLimit`), UPPER_SNAKE_CASE not observed
- Mock data exports: camelCase with `mock` prefix (e.g., `mockCharacters`, `mockPersonas`, `mockConversations`, `mockProviders`, `mockPresets`)

**Types:**
- Interfaces: PascalCase (e.g., `CharacterCard`, `UserPersona`, `Conversation`, `Message`, `Provider`, `PromptPreset`, `GenerationParams`, `MessageMetadata`, `CharacterExtensions`, `LorebookEntry`)
- Type aliases: PascalCase (e.g., `MessageRole`, `ProviderType`, `LorebookPosition`, `LorebookActivation`, `PrimaryView`, `CharactersTab`)
- Props interfaces defined inline within `<script>` using `interface Props` (e.g., in `MessageBubble.svelte`, `MessageInput.svelte`, `CharacterCard.svelte`)
- Variant props from tailwind-variants exported as `ButtonProps` type alias from `VariantProps<typeof buttonVariants>`

## Code Style

**Formatting:**
- Tool: Prettier (`prettier` v3.8+)
- Plugins: `prettier-plugin-svelte` v3.5, `prettier-plugin-tailwindcss` v0.8
- Config: No `.prettierrc` file found — uses Prettier defaults (2-space indent, single quotes for JS/TS observed in `svelte.config.js`, double quotes in `package.json`)
- Svelte components: `<script lang="ts" module>` before `<script lang="ts">` before template before `<style>` (module script used in `Button.svelte` for variant definitions)

**Linting:**
- Tool: Prettier check mode only (`npm run lint` → `prettier --check .`)
- No ESLint configured — linting is purely formatting-based
- `svelte-check` for TypeScript/Svelte type checking (`npm run check`)
- TypeScript strict mode enabled (`"strict": true` in `tsconfig.json`)
- `checkJs: true` — JavaScript files also type-checked

## Import Organization

**Order:**
1. SvelteKit framework imports (e.g., `{ tick }` from `'svelte'`, `{ onMount }` from `'svelte'`, `type { Snippet }` from `'svelte'`)
2. Third-party library imports (e.g., `import { clsx }` from `'clsx'`, `import { Popover }` from `'bits-ui'`, `import { Sparkles }` from `'@lucide/svelte'`)
3. Internal `$lib` imports (e.g., `import { app }` from `'$lib/stores/app.svelte'`, `import { cn }` from `'$lib/utils'`, `import { Avatar }` from `'$lib/components/ui'`)
4. SvelteKit app imports (e.g., `import { page }` from `'$app/state'`, `import { goto }` from `'$app/navigation'`)
5. Type-only imports grouped with `type` keyword (e.g., `import type { CharacterCard }` from `'$lib/types'`)

**Path Aliases:**
- `$lib` → `src/lib` (configured in `svelte.config.js` via `kit.alias`)
- `$app/state` — SvelteKit page state
- `$app/navigation` — SvelteKit navigation utilities

## Error Handling

**Patterns:**
- No explicit error handling patterns found (try/catch, Result types, Error boundaries)
- Guard clauses used instead: early returns with condition checks (e.g., `if (!trimmed || app.is_generating) return;` in `MessageInput.svelte`, `if (!app.active_conversation_id) return;` in `app.svelte.ts`)
- Optional chaining used extensively for safe property access (e.g., `app.active_persona?.name`, `char?.name`, `persona?.avatar_url`)
- Nullish coalescing for defaults (e.g., `persona?.name ?? 'You'`, `char?.name ?? '?'`, `provider.default_params?.temperature ?? '—'`)
- `@html` directive used with explicit `<!-- eslint-disable-next-line svelte/no-at-html-tags -->` comment in `MessageBubble.svelte`
- No custom error types or error boundary components observed
- Streaming errors handled by marking streaming as done (e.g., `app.updateStreamingMessage(last.id, last.content, true)` in cancel handler)

## Logging

**Framework:** console (no logging library)

**Patterns:**
- No logging statements observed in the codebase
- Debug information surfaced via UI (context bar in `MessageInput.svelte` shows token count)
- Comments serve as documentation, not runtime logging

## Comments

**When to Comment:**
- Section dividers in state classes using em-dash banners (e.g., `// ── Library ────`, `// ── Actions ────` in `app.svelte.ts`)
- JSDoc on public utility functions explaining purpose and future replacement (e.g., `/** Approximate token count using a 4-char-per-token heuristic. The real app will swap this for tiktoken-rs via the backend bridge. */`)
- Inline comments for spec references (e.g., `// F9 toggles sidebar (per spec §4.1.5)`)
- Architecture notes explaining design decisions (e.g., `/** Root application state. Mirrors the AppModel/ChatModel/CharacterModel/SettingsModel split from §6.2 of the design spec… */`)
- Stub markers for unfinished features (e.g., `<!-- Right cluster — placeholder for future actions -->`, `_(stub — wire via Bits.ui Toast in Phase 2)_`)

**JSDoc/TSDoc:**
- Used on exported utility functions in `src/lib/utils/index.ts`
- Used on the `AppState` class in `src/lib/stores/app.svelte.ts`
- Used on the `renderInlineMarkdown` function in `MessageBubble.svelte`
- Not used on Svelte component props (interfaces are self-documenting)
- `@type` annotations used in Svelte config (`/** @type {import('@sveltejs/kit').Config} */`)

## Function Design

**Size:** Functions are small and focused — typically 5-15 lines. The longest functions are event handlers like `handleSend()` in `src/routes/chat/[id]/+page.svelte` (~40 lines) which includes mock streaming logic.

**Parameters:** 
- Object destructuring for props (Svelte 5 `$props()` pattern): `let { character, onchat, onedit }: Props = $props();`
- Rest props pattern for HTML attribute passthrough: `let { class: className, ...rest }: Props = $props();` then `{...rest}` on element
- Callback props use `on` prefix with optional chaining for invocation (e.g., `onsend?.(trimmed)`, `onchat?.(character.id)`)

**Return Values:** 
- Utility functions return primitives or arrays directly
- `cn()` returns `string`
- `formatRelativeTime()` returns `string`
- `estimateTokens()` returns `number`
- `avatarInitials()` returns `string`
- Class getters for derived state (e.g., `get active_conversation()`, `get filtered_characters()`)
- Svelte component module exports (e.g., `buttonVariants` and `ButtonProps` type from `Button.svelte`)

## Module Design

**Exports:**
- Named exports preferred (all utility functions, types, mock data)
- Single class export for state: `export const app = new AppState();`
- Default exports used for Svelte components (standard Svelte convention)
- Barrel files re-export defaults: `export { default as Avatar } from './Avatar.svelte';`
- Variant definitions exported from module scripts: `export const buttonVariants = tv({...});`

**Barrel Files:** Used at every `index.ts` level:
- `src/lib/components/ui/index.ts` — re-exports all UI primitives
- `src/lib/types/index.ts` — all type definitions in one file
- `src/lib/utils/index.ts` — all utility functions in one file
- Enables clean imports: `import { Avatar, Button, Input } from '$lib/components/ui';`

---

*Convention analysis: 2026-05-10*

# Testing Patterns

**Analysis Date:** 2026-05-10

## Test Framework

**Runner:**
- None configured. No test runner, test framework, or test scripts found in `package.json`.
- Config: No test config file found (no `vitest.config.*`, `jest.config.*`, `playwright.config.*`)

**Assertion Library:**
- None installed or configured.

**Run Commands:**
```bash
# No test commands exist. Available verification commands:
npm run check       # svelte-check + tsc type checking
npm run lint        # prettier --check . (format only)
npm run build       # vite build (compile-time verification)
```

## Test File Organization

**Location:**
- No test files exist in the repository.
- No `__tests__` directories, `*.test.*` files, or `*.spec.*` files found.
- No `tests/` or `test/` directory at any level.

**Naming:**
- N/A — no tests present.

**Structure:**
```
# No test directory structure exists
```

## Test Structure

**Suite Organization:**
```typescript
// No test suites found in the codebase
```

**Patterns:**
- No setup, teardown, or assertion patterns established.
- The closest thing to "testing" is the mock streaming system in `src/routes/chat/[id]/+page.svelte` which simulates AI responses for manual UI verification.

## Mocking

**Framework:** None configured.

**Patterns:**
```typescript
// No automated mocking patterns exist.
// Manual mock data is used as a substitute for real backend data:
// src/lib/data/mock.ts provides:
// - mockCharacters: CharacterCard[]
// - mockPersonas: UserPersona[]
// - mockConversations: Conversation[]
// - mockMessagesForConv01: Message[]
// - mockProviders: Provider[]
// - mockPresets: PromptPreset[]
// - getMessages(conversation_id: string): Message[]
```

**What to Mock:**
- Based on the codebase architecture, the following would need mocking when tests are added:
  - Backend bridge (Tauri IPC `invoke()` calls) — currently stubbed in `src/routes/chat/[id]/+page.svelte` `handleSend()`
  - API providers (Anthropic, OpenAI, Ollama) — provider list in `src/lib/data/mock.ts`
  - Token counting — currently a heuristic in `src/lib/utils/index.ts` `estimateTokens()`
  - Markdown rendering — currently naive in `src/lib/components/chat/MessageBubble.svelte` `renderInlineMarkdown()`
  - Database queries — `getMessages()` in `src/lib/data/mock.ts` is a stub

**What NOT to Mock:**
- UI component rendering and state reactivity (these are what tests should verify)
- `cn()` utility — it's a pure function (`clsx` + `twMerge`), safe to test directly
- Type definitions in `src/lib/types/index.ts` — pure data shapes, no behavior to mock

## Fixtures and Factories

**Test Data:**
```typescript
// src/lib/data/mock.ts serves as the de facto fixture system:
// - Hand-authored character cards with realistic roleplay content
// - Conversations with pre-written message threads
// - Providers with realistic model configurations
// - Prompt presets (Creative, Balanced, Precise)
// Timestamps computed relative to "now" using Date arithmetic:
const now = new Date().toISOString();
const yesterday = new Date(Date.now() - 86_400_000).toISOString();
const lastWeek = new Date(Date.now() - 7 * 86_400_000).toISOString();
```

**Location:**
- `src/lib/data/mock.ts` — centralized mock/fixture data file
- Inline mock streaming data in `src/routes/chat/[id]/+page.svelte` (`sample` variable in `handleSend()`)

## Coverage

**Requirements:** None enforced. No coverage tooling installed.

**View Coverage:**
```bash
# No coverage command available
```

## Test Types

**Unit Tests:**
- Not present. Candidates when added:
  - `src/lib/utils/index.ts` — pure functions (`cn()`, `formatRelativeTime()`, `estimateTokens()`, `avatarInitials()`) are ideal unit test targets
  - `src/lib/stores/app.svelte.ts` — `AppState` class methods (`hydrate()`, `openConversation()`, `appendMessage()`, `updateStreamingMessage()`, `toggleSidebar()`, `switchPersona()`) and derived getters
  - `src/lib/data/mock.ts` — `getMessages()` function

**Integration Tests:**
- Not present. Candidates when added:
  - Component interaction flows (e.g., typing in `MessageInput` → message appears in list, clicking persona in `PersonaSwitcher` → active persona changes)
  - Route navigation (e.g., `/chat/[id]` loads conversation, `/characters?tab=personas` switches tab)
  - State hydration flow in `+layout.svelte` (`onMount` → `app.hydrate()`)

**E2E Tests:**
- Not used. No Playwright, Cypress, or similar framework configured.

## Common Patterns

**Async Testing:**
```typescript
// No async test patterns exist.
// The codebase uses async in:
// - $effect() with tick() for post-render scroll (src/routes/chat/[id]/+page.svelte)
// - setTimeout-based simulated streaming (src/routes/chat/[id]/+page.svelte handleSend())
// These patterns would need adaptation for test environments.
```

**Error Testing:**
```typescript
// No error testing patterns exist.
// The codebase uses guard clauses rather than thrown exceptions,
// so error testing would focus on:
// - Verifying early returns on invalid input
// - Verifying optional chaining falls through gracefully
// - Verifying streaming cancellation marks messages as done
```

## Observations

**Current State:** This is a UI scaffold (v0.1.0) with no automated testing infrastructure. All verification is manual via `npm run dev` and visual inspection. The mock data system in `src/lib/data/mock.ts` makes the UI navigable without a backend, effectively serving as a manual test harness.

**Type Safety as Testing:** The project uses TypeScript strict mode (`"strict": true` in `tsconfig.json`) and `svelte-check` (`npm run check`) as the primary automated verification — catching type errors, unused variables, and Svelte template issues at compile time.

**Recommended Additions:**
- Vitest is the natural fit (already using Vite) — add `vitest` and `@testing-library/svelte` for component testing
- Unit tests for pure utilities in `src/lib/utils/index.ts` are the lowest-effort starting point
- The `AppState` class in `src/lib/stores/app.svelte.ts` is well-structured for isolated unit testing

---

*Testing analysis: 2026-05-10*

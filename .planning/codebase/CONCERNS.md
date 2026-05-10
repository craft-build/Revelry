# Codebase Concerns

**Analysis Date:** 2026-05-10

## Tech Debt

**Stray directory `src/{lib/`:**
- Issue: A malformed directory `src/{lib/{components/{ui,chat,characters,personas,settings,layout},stores,types,data,utils},routes` exists alongside the real `src/lib/`. Appears to be a brace-expansion artifact from a shell command run against the source tree.
- Files: `src/{lib/`
- Impact: No runtime impact (not imported), but clutters the repo and could confuse tooling or new contributors. If someone ever auto-imports from the wrong path, it would silently resolve to a duplicate with no contents.
- Fix approach: Delete the entire `src/{lib/` directory tree. Verify no import references it.

**Mock data as sole data source:**
- Issue: The entire app hydrates from hardcoded mock data in `src/lib/data/mock.ts`. No API layer, no persistence, no Tauri IPC bridge exists yet. Every page load resets state.
- Files: `src/lib/data/mock.ts`, `src/lib/stores/app.svelte.ts`, `src/routes/+layout.svelte`
- Impact: All user actions (new conversations, persona switches, message sends) are lost on reload. The app is a UI-only shell with no functional backend.
- Fix approach: Implement the Tauri IPC bridge (or HTTP API adapter) behind a service layer. `AppState.hydrate()` should call the backend; `handleSend` in `chat/[id]/+page.svelte` should dispatch `invoke()` calls. Keep mock data behind a dev-mode flag for standalone UI work.

**Naive markdown rendering:**
- Issue: `MessageBubble.svelte` uses a hand-rolled regex for `*italic*` and `**bold**` only. No code blocks, lists, links, headers, or images. The function also does its own HTML escaping before regex substitution, which is a fragile pattern.
- Files: `src/lib/components/chat/MessageBubble.svelte` (L49-56)
- Impact: AI roleplay responses commonly contain code blocks, markdown links, and structured formatting. Users will see raw markdown syntax instead of rendered content.
- Fix approach: Replace with a proper markdown parser (`marked`, `markdown-it`, or `pulldown-cmark` via the Rust backend). Sanitize the output with DOMPurify to prevent XSS.

**Hardcoded streaming simulation:**
- Issue: `chat/[id]/+page.svelte` simulates token streaming with `setTimeout` and a fixed sample string. No real LLM integration.
- Files: `src/routes/chat/[id]/+page.svelte` (L50-71)
- Impact: Chat is non-functional beyond the demo. Every AI response is the same canned text.
- Fix approach: Replace with a real streaming SSE/WebSocket/IPC connection to the backend. The `updateStreamingMessage` API in `AppState` is already correctly shaped for incremental token delivery — only the transport layer needs wiring.

**Hardcoded model name in ChatHeader:**
- Issue: The model badge displays the hardcoded string `opus-4.7` rather than reading from `app.active_conversation` or provider state.
- Files: `src/lib/components/chat/ChatHeader.svelte` (L36-42)
- Impact: Model name shown is always wrong unless the user happens to be using that exact model.
- Fix approach: Derive model from `app.active_conversation` metadata or from the active provider's `model` field.

**Token estimation is a 4-char heuristic:**
- Issue: `estimateTokens()` divides character count by 4. Real token counts vary wildly by language and tokenizer (BPE vs. unigram, code vs. prose, CJK text).
- Files: `src/lib/utils/index.ts` (L31-33)
- Impact: Context utilization bar in `MessageInput.svelte` is inaccurate. Users may hit context limits unexpectedly or see false warnings.
- Fix approach: Backend should return `token_count` per message (already in the `Message` type). For client-side estimation, integrate `tiktoken` WASM or accept the backend's count via IPC.

**No adapter-static — adapter-auto is wrong for Tauri:**
- Issue: `svelte.config.js` uses `@sveltejs/adapter-auto`, which tries to detect the deployment platform. For a Tauri-packaged SPA, `adapter-static` is the correct choice (pre-rendered static files served by the webview).
- Files: `svelte.config.js`
- Impact: `npm run build` may produce an incorrect output format. Adapter-auto can fall back to adapter-node (requiring a server) instead of static files.
- Fix approach: Switch to `@sveltejs/adapter-static` with `fallback: 'index.html'` since all routes are SPA client-side.

**Settings UI buttons are non-functional:**
- Issue: All buttons in the Settings page (Edit provider, Add provider, Export ZIP, Backup DB, Clear history) are present but have no click handlers or `onclick` bindings.
- Files: `src/routes/settings/+page.svelte`
- Impact: Users can see settings but cannot interact with any of them. Creates a false impression of functionality.
- Fix approach: Wire each button to appropriate actions — dialogs for editing, IPC calls for data operations. Consider disabling or hiding non-functional buttons until the backend is connected.

**Character card "Start chat" button not wired:**
- Issue: `CharacterCard.svelte` accepts `onchat` and `onedit` callback props, but `characters/+page.svelte` never passes them. Clicking "Start chat" or "Edit" does nothing.
- Files: `src/lib/components/characters/CharacterCard.svelte`, `src/routes/characters/+page.svelte`
- Impact: Users cannot navigate from the character library to a new conversation.
- Fix approach: Pass `onchat` that creates a new conversation (or navigates to an existing one for that character) and `onedit` that opens a character editor dialog.

## Known Bugs

**All suggested character links route to the same conversation:**
- Symptoms: Clicking any character on the home page navigates to `/chat/conv_01` regardless of which character is clicked.
- Files: `src/routes/+page.svelte` (L32)
- Trigger: Open the app root page; click any of the three suggested character cards.
- Workaround: Navigate manually via the sidebar conversation list.

**Persona switcher deep-link may not activate the correct tab:**
- Symptoms: The "Manage personas" and "Create new persona" links in `PersonaSwitcher.svelte` navigate to `/characters?tab=personas`, but the tab sync logic in `characters/+page.svelte` reads the query param only on initial load. If already on `/characters` with the characters tab active, the URL updates but the tab may not visually switch due to `$effect` short-circuit.
- Files: `src/routes/characters/+page.svelte` (L9-13), `src/lib/components/chat/PersonaSwitcher.svelte`
- Trigger: Navigate to `/characters`, click the characters tab, then use the persona switcher's "Manage personas" link.
- Workaround: Reload the page or manually click the personas tab.

**`getMessages()` only returns data for one conversation:**
- Symptoms: Opening any conversation other than `conv_01` shows "No messages yet" because the mock lookup returns an empty array.
- Files: `src/lib/data/mock.ts` (L139-141)
- Trigger: Navigate to `/chat/conv_02` or `/chat/conv_03` from the sidebar.
- Workaround: Only `conv_01` (Eleanor Vance) has messages in mock data.

## Security Considerations

**`{@html}` rendering of user/AI content:**
- Risk: `MessageBubble.svelte` renders message content as raw HTML via `{@html renderInlineMarkdown(message.content)}`. Although the function does HTML entity escaping first, the escaping is minimal (`&`, `<`, `>`) and the regex replacements re-introduce unsanitized HTML tags (`<em>`, `<strong>`). A crafted message containing `*<img src=x onerror=alert(1)>*` would pass through: the angle brackets get escaped, then the `*...*` regex wraps it in `<em>`, which is safe in this specific case, but the pattern is fragile and will break when a real markdown parser is introduced.
- Files: `src/lib/components/chat/MessageBubble.svelte` (L78)
- Current mitigation: Manual HTML entity escaping before regex substitution. The regex `replace(/\*([^*]+?)\*/g, ...)` only matches content between single asterisks, which limits injection surface.
- Recommendations: When switching to a real markdown parser, add DOMPurify sanitization on the rendered HTML. Never trust LLM output or user-authored content as safe HTML.

**API key display in settings:**
- Risk: The settings page renders a fake API key value `"sk-…example…not-stored"` in an `<input type="password">` field with a reveal toggle. While currently a placeholder, the pattern of storing/displaying keys in client-side state is dangerous. If real keys ever flow through `AppState`, they'd be inspectable in browser devtools.
- Files: `src/routes/settings/+page.svelte` (L79-96)
- Current mitigation: Keys are not stored in the front-end. The design doc specifies OS keychain storage via the Rust backend.
- Recommendations: Never allow real API keys into `AppState`. All key operations (store, retrieve, test) must go through Tauri IPC to the Rust backend, which uses the OS keychain (keyring crate). The settings UI should only show key status (set/not set), never the value.

**External font loading:**
- Risk: Google Fonts are loaded via `<link>` tags in `app.html`, which makes network requests to `fonts.googleapis.com` and `fonts.gstatic.com` on every page load. This leaks user IP and browsing context to Google.
- Files: `src/app.html` (L10-17)
- Current mitigation: None. This is standard web practice but conflicts with the "data never leaves the machine" privacy posture.
- Recommendations: For the Tauri desktop build, bundle the font files locally (self-host Geist, Fraunces, Geist Mono). Remove the Google Fonts CDN links. This also eliminates the network dependency for first load.

**Missing Content Security Policy:**
- Risk: No CSP headers are set, meaning the app is vulnerable to inline script injection and data exfiltration via XSS (especially relevant given the `{@html}` usage).
- Files: `src/app.html`, `svelte.config.js`
- Current mitigation: SvelteKit sets some default security headers, but CSP is not customized.
- Recommendations: Add a strict CSP via SvelteKit's `handle` hook or Tauri's config. For a desktop app that should never make external requests (except user-configured API endpoints), a restrictive `default-src 'self'` with exceptions for the configured provider endpoints is appropriate.

## Performance Bottlenecks

**Full-array replacement on every message update:**
- Problem: `updateStreamingMessage()` and `appendMessage()` create new array copies with spread syntax (`[...this.messages]`, `this.messages.map(...)`). During streaming, this runs every 30ms, creating a new array and new object for every token chunk.
- Files: `src/lib/stores/app.svelte.ts` (L82-89, L78-80)
- Cause: Svelte 5 `$state` requires reassignment for reactivity detection; the spread/`map` pattern is the idiomatic way to trigger updates. However, for long conversations (hundreds of messages), copying the entire array on every token chunk becomes O(n) per chunk.
- Improvement path: For streaming, update the specific message object in-place (mutate the `content` and `is_streaming` fields) — Svelte 5's fine-grained reactivity tracks property-level changes on `$state` objects. Only create a new array reference for structural changes (append, delete, reorder).

**Context utilization bar recalculates on every keystroke:**
- Problem: `MessageInput.svelte` derives `usedContext` by reducing over all messages on every keystroke in the input field.
- Files: `src/lib/components/chat/MessageInput.svelte` (L19-22)
- Cause: `$derived` recalculates whenever `app.messages` or `text` changes. For a 100-message conversation, this is 100 iterations per keystroke.
- Improvement path: Cache the sum of existing message token counts in `AppState` (update when messages change), and only add the current input's token estimate. This reduces the per-keystroke work to O(1).

**Conversation sidebar renders all conversations unconditionally:**
- Problem: The sidebar renders the full `filtered` list with avatar lookups (`characterFor` does a `.find()` per conversation). For large conversation counts, this is O(n*m) where n=conversations and m=characters.
- Files: `src/lib/components/layout/ConversationSidebar.svelte` (L12-13, L38-47)
- Cause: No virtualization; every conversation is in the DOM.
- Improvement path: Add virtual scrolling (e.g., `svelte-virtual-list` or a custom solution) for the conversation list. Build a character lookup map (Map<string, CharacterCard>) in `AppState` to avoid repeated `.find()` calls.

## Fragile Areas

**`AppState` as a god object:**
- Files: `src/lib/stores/app.svelte.ts`
- Why fragile: All application state (characters, personas, conversations, providers, presets, UI state, active conversation, messages) lives in a single class. Every component imports the entire state tree. Changes to any field trigger re-renders in components that subscribe to unrelated fields.
- Safe modification: Add new state fields as `$state` properties. Extract sub-stores (e.g., `ChatState`, `LibraryState`, `SettingsState`) if the class grows beyond ~20 fields. Use `$derived` for computed values to avoid manual sync.
- Test coverage: Zero — `AppState` has no unit tests.

**Message branching model defined but unused:**
- Files: `src/lib/types/index.ts` (L80-81, L87-88 — `parent_id`, `branch_index`, `active_branch_tip`)
- Why fragile: The `Message` type includes `parent_id`, `branch_index` for a tree-based branching model, and `Conversation` includes `active_branch_tip`. None of this is used in the UI or state logic. The message list is rendered as a flat array. If branching is implemented later, the flat-array rendering and state management will need significant refactoring.
- Safe modification: Do not remove these fields (they match the design spec schema), but do not add branching logic until the UI is designed. When implementing, the message list will need to become a tree traversal with branch selection.
- Test coverage: No tests for branching logic (since none exists).

**`formatRelativeTime()` edge cases:**
- Files: `src/lib/utils/index.ts` (L15-29)
- Why fragile: The function compares `date.getDate() === now.getDate()` to detect "today", but this fails across month boundaries (e.g., a message from May 31 at 23:59 and current time June 1 at 00:01 — `diffDays < 1` but dates differ, so it falls through to the weekday format instead of showing time). The "this week" check uses `diffDays < 7` but a message from 6 days ago in a different ISO week would show as "weekday" which may be confusing.
- Safe modification: Use `Intl.RelativeTimeFormat` or a small date library (`date-fns`) for robust relative time formatting. Alternatively, compare calendar dates explicitly.
- Test coverage: No unit tests for `formatRelativeTime`.

## Scaling Limits

**In-memory-only state:**
- Current capacity: Limited by browser/WebView memory. Current mock data is ~10KB. Real usage with thousands of conversations and tens of thousands of messages would consume significant memory.
- Limit: Without pagination or virtualization, the conversation list and message list will become sluggish at ~500+ conversations and ~1000+ messages per conversation.
- Scaling path: Implement SQLite persistence via the Rust backend with lazy loading. Only load conversation metadata for the sidebar; load messages on demand when a conversation is opened. Add virtual scrolling for both lists.

**Single active conversation model:**
- Current capacity: One conversation loaded at a time (`active_conversation_id` + `messages[]`).
- Limit: Users cannot reference or switch between multiple conversations without losing the current message list (since `openConversation` replaces the entire `messages` array).
- Scaling path: Add a conversation cache (LRU) in `AppState` that keeps the last N conversations' messages in memory. Unload least-recently-used conversations when the cache is full.

**No search indexing:**
- Current capacity: Search is done via `.filter()` with `.includes()` over in-memory arrays.
- Limit: Linear scan over all conversations/characters. Becomes noticeably slow at ~10K+ items.
- Scaling path: Implement full-text search via SQLite FTS5 on the backend. The front-end sends search queries to the backend and receives ranked results.

## Dependencies at Risk

**`adapter-auto` instead of `adapter-static`:**
- Risk: The project is designed as a Tauri-packaged SPA but uses `adapter-auto`, which may not produce the correct static output format.
- Impact: Production builds may not work correctly in Tauri's webview.
- Migration plan: Install `@sveltejs/adapter-static` and update `svelte.config.js` to use it with `fallback: 'index.html'`.

**`bits-ui` v2 is evolving rapidly:**
- Risk: `bits-ui@^2.18` is in active development with breaking changes between minor versions. The API used here (Avatar, Popover, Tooltip, Tabs) may shift.
- Impact: `npm install` could pull a version with incompatible changes. Currently used: Avatar, Popover, Tooltip, Tabs primitives.
- Migration plan: Pin the exact version in `package.json` (remove `^`). Monitor the bits-ui changelog. Test after any version bump.

**`@lucide/svelte` at `^1.14` may not be compatible with Svelte 5:**
- Risk: Lucide's Svelte package may still use legacy Svelte 4 component patterns (`export let`). Svelte 5 supports these via compatibility mode, but this is not guaranteed long-term.
- Impact: Icons may break in a future Svelte 5 release that drops legacy mode.
- Migration plan: Verify that the installed version uses Svelte 5 `$props()` syntax. If not, contribute a PR or fork the icon set. Alternatively, switch to inline SVGs or a different icon library with Svelte 5 support.

## Missing Critical Features

**No persistence layer:**
- Problem: All data is lost on page reload. No SQLite, no localStorage, no IndexedDB fallback.
- Blocks: Real-world usage. Users cannot save conversations, characters, or settings.

**No character/persona editor:**
- Problem: Characters and personas can be viewed but not created, edited, or deleted. The design spec (§7.4) describes a multi-step wizard.
- Blocks: User onboarding — users cannot add their own characters or customize personas.

**No authentication/API key management:**
- Problem: Provider settings show a placeholder key field. No mechanism to store, validate, or rotate API keys.
- Blocks: Any real LLM integration. The app cannot make API calls without credentials.

**No error handling or toast notifications:**
- Problem: There is no error boundary, no toast system, and no error state in `AppState`. Network failures, IPC errors, and invalid inputs have nowhere to surface.
- Blocks: User-facing error feedback. Broken actions fail silently.

**No light theme:**
- Problem: Dark theme is the only option. The `<html class="dark">` is hardcoded.
- Blocks: Users who prefer light mode or have accessibility needs for higher contrast.

## Test Coverage Gaps

**Zero test infrastructure:**
- What's not tested: Everything. No test framework is configured, no test files exist, and no CI pipeline runs checks.
- Files: Entire `src/` directory
- Risk: Any refactoring could introduce regressions silently. The reactive state in `AppState` is particularly vulnerable — Svelte 5 runes have subtle reactivity edge cases that are easy to break.
- Priority: High

**`AppState` logic untested:**
- What's not tested: All `AppState` methods (`hydrate`, `openConversation`, `appendMessage`, `updateStreamingMessage`, `switchPersona`, derived getters like `filtered_characters`, `active_character`).
- Files: `src/lib/stores/app.svelte.ts`
- Risk: State mutations could break without detection. The `updateStreamingMessage` spread/replace pattern is especially fragile.
- Priority: High

**Utility functions untested:**
- What's not tested: `cn()`, `formatRelativeTime()`, `estimateTokens()`, `avatarInitials()`.
- Files: `src/lib/utils/index.ts`
- Risk: Edge cases in date formatting, token estimation, and initials extraction could produce wrong output in production.
- Priority: Medium

**Component rendering untested:**
- What's not tested: Any component rendering, prop handling, event dispatching, conditional rendering.
- Files: All `.svelte` files in `src/lib/components/` and `src/routes/`
- Risk: UI regressions from styling or logic changes. Broken links and non-functional buttons could be introduced without detection.
- Priority: Medium

**Type definitions untested:**
- What's not tested: Type exhaustiveness — whether the TypeScript types cover all fields from the design spec's SQLite schema. Runtime validation of incoming data shapes.
- Files: `src/lib/types/index.ts`
- Risk: Backend data that doesn't match the expected shape could cause runtime errors in components that assume field existence.
- Priority: Low (type-level concern, but runtime validation is important when backend connects)

---

*Concerns audit: 2026-05-10*

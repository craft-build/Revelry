# Project Research Summary

**Project:** Revelry
**Domain:** AI roleplay desktop application (native Rust/GTK4, local-first, multi-provider)
**Researched:** 2026-04-28
**Confidence:** HIGH

## Executive Summary

Revelry is a native desktop AI roleplay application built with Rust, GTK4/libadwaita, and Relm4 — the only app in this space using a fully native widget toolkit rather than web-based rendering. Every major competitor (SillyTavern, RisuAI, Agnai, Character.ai, SpicyChat) is web-based, making Revelry's GTK4 native desktop experience a genuine product differentiator, not just a technical choice. The domain is well-understood: the CharacterCardV2 spec, lorebook/WorldInfo systems, conversation branching, and prompt assembly patterns are all battle-tested in the SillyTavern ecosystem, providing clear implementation references.

The recommended approach is a layered architecture following Relm4's Elm pattern: UI components (ChatPanel, LibraryPanel, SettingsPanel) communicate via typed messages through a service layer (AiService, PromptEngine, ImportService) to a data layer (DbService Worker, KeyringStore, FileStore). The critical integration point is the tokio↔GLib streaming bridge — AI tokens arrive via rig's streaming API on the tokio runtime and must be forwarded to the GTK main loop for rendering without blocking the UI. Relm4's `sender.command()` with `ShutdownReceiver` provides both the multi-message streaming channel and cancellation support, eliminating the need for manual glib channel setup.

The key risks are: (1) blocking the GTK main loop with synchronous DB or streaming operations — mitigated by enforcing background-thread patterns from day one, (2) token counting mismatches across providers — mitigated by designing budget systems with safety margins and per-provider approximations, (3) conversation branching implemented as a flat list instead of a tree — mitigated by designing the tree data model before building any chat UI, and (4) CharacterCardV2 spec drift causing broken imports — mitigated by validating against real SillyTavern community cards.

## Key Findings

### Recommended Stack

All crate versions verified via crates.io and Context7. The gtk-rs ecosystem (gtk4, libadwaita, pango, sourceview5) must share the same 0.11.x/0.22.x minor version. Relm4 0.11 aligns with gtk4 0.11.x. rig-core 0.36 provides unified streaming across 20+ providers. rusqlite 0.39 with `bundled` feature eliminates external SQLite dependency. **Avoid keyring 4.0.0** (released 2 days ago, breaking changes unvalidated) — pin to 3.6.3.

**Core technologies:**
- **gtk4 0.11.3 + libadwaita 0.9.1 + Relm4 0.11.0**: Native GNOME UI with Elm Architecture state management. Relm4's `libadwaita` feature flag is required for Adwaita widget support in view macros.
- **rig-core 0.36.0**: Provider-agnostic AI abstraction with 20+ native clients (OpenAI, Anthropic, Gemini, Ollama, etc.). Unified `StreamingPrompt` trait. Agent builder pattern for preamble + temperature + tools.
- **rusqlite 0.39.0 (bundled) + rusqlite_migration 2.5.0**: Zero-config SQLite with FTS5, window functions, schema migration via `user_version`.
- **keyring 3.6.3**: OS-native credential storage (macOS Keychain, GNOME Keyring, Windows Credential Manager). Platform-specific feature flags.
- **pulldown-cmark 0.13.3 + pango 0.22.6**: Markdown→Pango markup pipeline for AI response rendering.
- **tiktoken-rs 0.11.0**: BPE tokenization for context budget tracking. OpenAI-compatible; approximate for other providers.
- **aes-gcm 0.10.3 + argon2 0.5.3**: AES-256-GCM vault encryption with OWASP-recommended Argon2id key derivation.

### Expected Features

See [FEATURES.md](./FEATURES.md) for full detail.

**Must have (table stakes):**
- Multi-provider AI connection with streaming — without this, nothing else matters
- Character card CRUD + CharacterCardV2 spec — core data model
- Character card import/export (PNG + JSON) — ecosystem interoperability
- Character library with search & tags — users accumulate hundreds of characters
- Conversation history with message CRUD — basic expectation
- Conversation branching (swipes) — THE core interaction pattern for roleplay
- Streaming responses with cancellation — non-negotiable in 2026
- User personas with structured fields — key differentiator, inject into prompt
- Template variable resolution (`{{user}}`, `{{char}}`, `{{persona.*}}`) — cards break without this
- Lorebook/WorldInfo with keyword activation — the most powerful prompt engineering tool in roleplay
- Token counting & context budget — essential for managing limited context windows
- Generation parameter presets — expected by all users
- Markdown rendering — AI output is unreadable without it
- Local data storage — privacy dealbreaker if missing
- No content filtering — target audience explicitly rejects this

**Should have (competitive differentiators):**
- Native GTK4/libadwaita desktop UI — no competitor does native desktop
- Structured persona system with compression tiers — no competitor compresses personas
- Persona-aware lorebook activation — structured personas enable more precise matching
- AES-256-GCM vault encryption — no competitor offers local encryption
- OS-native credential storage — no competitor uses OS keychain
- FTS5 full-text search across conversations — practical differentiator for power users

**Defer (v2+):**
- Group chat — complex multi-character prompt management (Phase 2 of next milestone)
- Image generation, TTS, character sprites — not core to text roleplay
- RAG/Data Bank — lorebook covers 80% of knowledge injection
- Extension/plugin system — ship solid core first
- STscript/scripting — enormous scope, slash commands suffice
- Cloud sync, online accounts — contradicts local-first architecture

### Architecture Approach

Three-layer architecture following Relm4's Elm pattern: UI Layer (GTK4/libadwaita components with typed message buses) → Service Layer (async business logic on tokio runtime) → Data Layer (SQLite via background Worker, OS keychain, file I/O). The critical pattern is the streaming bridge: `sender.command()` provides a `Sender<CommandOutput>` that emits token chunks from rig's streaming API back to the GLib main loop, with `ShutdownReceiver` for cancellation. Database access uses a dedicated Relm4 `Worker` that owns the `!Send` rusqlite `Connection` on its own thread. Prompt assembly is a pure-function pipeline (system → character → persona → lorebook → truncated history) that runs inside `command()` closures before calling rig.

**Major components:**
1. **AppWindow** — Root component with `AdwOverlaySplitView`, routes messages between panels
2. **ChatPanel** — Conversation display, streaming token rendering, branch navigation via `FactoryVecDeque`
3. **LibraryPanel** — Character card grid (`FactoryVecDeque`), search, tags, import/export
4. **SettingsPanel** — Provider config, presets, persona management, vault via `AdwPreferencesWindow`
5. **AiService** — rig agent construction per-request, streaming execution on tokio
6. **PromptEngine** — Pure-function prompt assembly pipeline with token budgeting
7. **DbService** — SQLite CRUD on dedicated background Worker thread

### Critical Pitfalls

See [PITFALLS.md](./PITFALLS.md) for full detail with prevention strategies.

1. **Blocking the GTK main loop** — Never call rusqlite, keyring, or any blocking I/O from `update()`. Always use `command()`, `spawn_oneshot_command()`, or `Worker`. Profile with `GTK_DEBUG=interactive`.
2. **Token counting mismatch across providers** — tiktoken-rs only matches OpenAI. Design budget system with 10-15% safety margin. Use per-provider encoding selection. Show approximate ranges, not exact counts.
3. **Conversation branching without tree structure** — Design the tree data model (`parent_id`, `branch_index`, recursive CTEs) before any chat UI. A flat list cannot be retrofitted without a migration nightmare.
4. **Context window "just send everything"** — Implement layered priority from day one: sticky (system+character), semi-sticky (persona with compression tiers), activatable (lorebook with budget caps), volatile (history). Compute budget BEFORE assembly.
5. **Lorebook recursive activation bomb** — Hard-cap recursion at 3-5 levels. Hard-cap total activated lorebook tokens at 30% of context budget. Track already-activated entries. Two-pass assembly: activate first, then select by priority.
6. **Streaming cancellation without state cleanup** — Use `CancellationToken` not `abort()`. Persist partial responses as draft rows. Model streaming state as `Idle | Streaming | Cancelling` enum. Test cancel at 1%, 50%, 99%.
7. **CharacterCardV2 spec drift** — Validate against actual SillyTavern codebase, not blog posts. Full `data` object including `extensions`, `alternate_greetings`, `character_book.entries[].extensions`. Test with real community cards.
8. **macOS GTK4 quirks** — Test on macOS from day one. Platform-aware keyboard shortcuts (Cmd vs Ctrl). Use `AdwApplication`. Expect scrollbar and font rendering differences.

## Implications for Roadmap

Based on research, suggested phase structure follows the architecture's build order (domain types → DB → services → UI) and respects feature dependencies (characters before chat, token counting before context management):

### Phase 1: Foundation & App Shell
**Rationale:** Everything depends on domain types and the database. The app scaffold establishes the async patterns (Worker, command) that all features must follow. Getting this wrong means retrofitting later — the single most expensive mistake identified in pitfalls research.
**Delivers:** Runnable GTK4/Adwaita app window with `AdwOverlaySplitView`, all domain type definitions, SQLite schema + migrations, DbService Worker, macOS platform handling
**Addresses:** Domain types, database schema, app shell from ARCHITECTURE.md
**Avoids:** Pitfall 3 (SQLite on main thread), Pitfall 8 (macOS quirks)

### Phase 2: Character System & Library
**Rationale:** Characters are the core data model — conversations, personas, and lorebooks all reference character definitions. Import/export is critical for ecosystem adoption (users start by importing their existing SillyTavern cards).
**Delivers:** CharacterCardV2 spec types + CRUD, PNG/JSON import with validation + security checks, character library UI (grid + search + tags), character creation wizard
**Addresses:** Character card CRUD, import/export, character library from FEATURES.md
**Avoids:** Pitfall 7 (CharacterCardV2 spec drift), security mistakes (PNG chunk limits, ReDoS validation)
**Uses:** gtk4, libadwaita, relm4, relm4-icons, png, image, base64, serde_json, regex

### Phase 3: AI Connection & Core Chat Loop
**Rationale:** This is the product's core value proposition — connecting to AI providers and chatting with characters. Streaming is the critical integration point (tokio↔GLib bridge) that must be architected correctly from the start. Conversation branching requires the tree data model before any chat UI is built.
**Delivers:** Multi-provider connection via rig, streaming token-by-token rendering with cancellation, conversation persistence with tree-structured branching, message CRUD, basic prompt assembly (character + history, minimal persona), Markdown→Pango rendering
**Addresses:** Multi-provider connection, streaming, conversation history + branching, markdown rendering from FEATURES.md
**Avoids:** Pitfall 1 (blocking GTK main loop), Pitfall 6 (streaming cancellation), Pitfall 10 (branching without tree)
**Uses:** rig-core, tokio, futures, pulldown-cmark, pango, glib

### Phase 4: Context Engineering Layer
**Rationale:** Prompt assembly is the architectural core of a roleplay app. Template variables must resolve before token counting. Persona compression must exist before lorebook budget allocation works correctly. This layer transforms "chat works" into "roleplay works" — without it, characters go out of character in long conversations.
**Delivers:** Template variable resolution (`{{user}}`, `{{char}}`, `{{persona.*}}`), structured persona system with compression tiers, lorebook with keyword activation + depth/budget caps, token counting via tiktoken-rs, context budget management with layered priority, token budget gauge UI, persona switching mid-conversation
**Addresses:** Template variables, personas, lorebook, token counting, context management from FEATURES.md
**Avoids:** Pitfall 2 (token counting mismatch), Pitfall 4 (context window mismanagement), Pitfall 5 (lorebook recursion bomb), Pitfall 9 (template variable edge cases)
**Uses:** tiktoken-rs, tracker (fine-grained UI updates for budget gauge)

### Phase 5: Settings, Security & Polish
**Rationale:** Security and configuration features complete the application. OS keychain must be in place before vault encryption makes sense. Backup/restore enables safe data portability. FTS5 search requires the conversation data from Phase 3. Generation presets are expected but not blocking for core functionality.
**Delivers:** Provider configuration UI, generation parameter presets, OS keychain for API keys, AES-256-GCM vault encryption, backup/restore, FTS5 conversation search, batch import/export, regex scripts per character
**Addresses:** Presets, OS keychain, vault encryption, FTS5 search, backup/restore, regex scripts from FEATURES.md
**Avoids:** Security mistakes (API keys in SQLite, weak KDF parameters), performance traps (un-indexed FTS5, un-cached prepared statements)
**Uses:** keyring, aes-gcm, argon2, zip

### Phase Ordering Rationale

- **Foundation first** — Domain types have zero dependencies. Everything else depends on them. The DB Worker pattern and async boundaries must be established before any feature code is written, because retrofitting from synchronous to async DB access is the highest-cost recovery identified in pitfalls research.
- **Characters before chat** — Conversations reference character IDs. The library UI validates the Relm4 component + Worker + Factory patterns before the more complex ChatPanel.
- **Chat before context engineering** — A basic chat loop (character + history, no persona/lorebook) is testable and deliverable. Context engineering adds sophistication to working chat rather than blocking it.
- **Context engineering as a dedicated phase** — Prompt assembly is the most complex pure-logic layer (template resolution, persona compression, lorebook activation, token budgeting). It deserves focused attention, not interleaving with UI work.
- **Settings and security last** — API keys can use `from_env()` initially. Vault encryption is a differentiator, not a blocker. FTS5 requires existing conversation data to be useful.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 2 (Character System):** CharacterCardV2 spec needs verification against actual SillyTavern codebase — the spec is community-driven, not formally documented. Must read `public/scripts/common-char-data.js` from SillyTavern repo.
- **Phase 3 (AI Streaming):** rig 0.36.0 was published today (2026-04-28). Streaming patterns and `MultiTurnStreamItem` variants need verification against the actual API. The Context7 docs were checked but rig is evolving rapidly.
- **Phase 4 (Context Engineering):** Persona compression tiers (full→trimmed→minimal→essential) are a novel feature with no reference implementation. Token budget allocation strategy needs design validation.
- **Phase 4 (Lorebook):** SillyTavern's WorldInfo is extremely feature-rich. Need to define which subset to implement in v1.

Phases with standard patterns (skip research-phase):
- **Phase 1 (Foundation):** Relm4 app scaffold, Worker pattern, SQLite migrations are well-documented in official docs.
- **Phase 5 (Settings/Security):** AdwPreferencesWindow, keyring crate, aes-gcm/argon2 are well-established patterns with good documentation.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All crate versions verified via crates.io. Feature flags and version compatibility confirmed. gtk-rs ecosystem alignment validated. rig streaming patterns verified via Context7. |
| Features | HIGH | SillyTavern, RisuAI, Agnai documentation provides comprehensive feature landscape. Feature presence matrix cross-referenced across 5 competitors. MVP prioritization is clear. |
| Architecture | HIGH | Relm4 book thoroughly documents Component, Worker, command, Factory patterns. Streaming bridge pattern is canonical. Build order follows dependency graph. Anti-patterns well-documented. |
| Pitfalls | MEDIUM-HIGH | Most pitfalls derived from official docs and well-established patterns. CharacterCardV2 spec drift risk is real — spec is community-driven, not standardized. rig 0.36.0 is very new. macOS GTK4 quirks based on general knowledge, not hands-on testing yet. |

**Overall confidence:** HIGH

### Gaps to Address

- **CharacterCardV2 spec accuracy:** The spec lives in SillyTavern's codebase, not a formal document. Must read actual parsing code during Phase 2 planning. Community cards have undocumented edge cases that only real import testing reveals.
- **rig 0.36.0 API stability:** Published same day as research. Streaming API, `MultiTurnStreamItem` variants, and `ShutdownReceiver` integration patterns should be re-verified at Phase 3 planning time.
- **Persona compression effectiveness:** Full→trimmed→minimal→essential compression is a novel design. No reference implementation exists. The actual compression ratios and field selection strategy need validation during Phase 4.
- **macOS GTK4 rendering quality:** General knowledge of macOS quirks, but no hands-on testing with the specific gtk4-rs 0.11 + libadwaita 0.9 combination on macOS. First UI phase must include macOS testing.
- **tiktoken-rs accuracy for non-OpenAI providers:** tiktoken-rs only provides OpenAI tokenizers. The ~10-20% divergence for Anthropic/local models is estimated, not measured. Phase 4 should include provider-reported usage comparison testing.

## Sources

### Primary (HIGH confidence)
- crates.io — gtk4 0.11.3, libadwaita 0.9.1, relm4 0.11.0, rig-core 0.36.0, rusqlite 0.39.0, keyring 3.6.3, and all other dependency versions verified
- Context7 — Relm4 book (Component model, Worker, command, Factory, async patterns), gtk4-rs book (main loop, threading), rusqlite docs (Connection, WAL, prepared statements), tiktoken-rs docs (encoding selection)
- Relm4 Book: https://relm4.org/book/stable/ — Component, Worker, command, async patterns
- rig GitHub: https://github.com/0xplaygrounds/rig — Provider list, streaming patterns, agent builder
- SillyTavern Documentation: https://docs.sillytavern.app/ — WorldInfo, personas, character design, macros, group chats

### Secondary (MEDIUM confidence)
- SillyTavern GitHub: https://github.com/SillyTavern/SillyTavern — CharacterCardV2 implementation reference
- RisuAI GitHub: https://github.com/kwaroran/Risuai — Feature comparison
- Agnai GitHub: https://github.com/agnaistic/agnai — Feature comparison
- rig GitHub issues — #1599 (system_message), #1556 (Chat trait) — open issues that may affect implementation

### Tertiary (LOW confidence)
- Character.ai: https://character.ai/ — 403 on direct fetch, feature knowledge from training data
- SpicyChat: https://spicychat.ai/ — minimal content retrieved, feature knowledge from training data
- macOS GTK4 rendering behavior — general knowledge, not empirically tested with current versions

---
*Research completed: 2026-04-28*
*Ready for roadmap: yes*

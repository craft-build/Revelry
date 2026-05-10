# Project Research Summary

**Project:** Revelry
**Domain:** Tauri desktop AI roleplay application
**Researched:** 2026-05-10
**Confidence:** HIGH

## Executive Summary

Revelry is a cross-platform desktop AI roleplay app being built by wrapping an existing SvelteKit SPA in a Tauri v2 shell, then porting a complete Rust backend (AI inference, SQLite persistence, keyring secrets, prompt engineering, token counting, markdown rendering, card I/O, lorebook, regex scripting) from an archival `gtk_ref/` codebase. This is a well-trodden architecture — Tauri v2 has first-class SvelteKit support, a robust IPC system with commands (request-response), channels (streaming), and events (broadcasts), and cross-platform distribution for macOS, Linux, and Windows.

The recommended approach is: establish the Tauri shell first (adapter-static switch, permissions, dev/prod parity), then build the backend layer-by-layer following the dependency chain (DB → keyring → AI streaming → prompt engine → supporting services). The `gtk_ref/` codebase provides proven patterns for every service — the migration is primarily about adapting the Relm4 message-passing pattern to Tauri's command/channel IPC model, not inventing new architecture.

Key risks center on the IPC boundary: synchronous DB calls blocking the main thread, confusing Tauri channels vs events for streaming, and SSR/adapter mismatches between SvelteKit and Tauri. All are preventable with established patterns documented in the research.

## Key Findings

### Recommended Stack

Tauri v2 + SvelteKit with adapter-static is the standard approach for this type of app. rig-core provides 18 native AI providers with streaming support. rusqlite with bundled feature gives zero-config cross-platform SQLite. The keyring crate handles OS-native secrets on all three platforms.

**Core technologies:**
- **Tauri v2**: Desktop shell — revamped IPC, channel API for streaming, cross-platform builds
- **rig-core ^0.36**: AI provider abstraction — 18 providers, streaming, custom OpenAI-compatible
- **rusqlite ^0.39 (bundled)**: SQLite persistence — zero-config, WAL mode, background-thread worker
- **keyring ^3.6**: OS-native secrets — macOS Keychain, GNOME Keyring, Windows Credential Manager
- **tiktoken-rs ^0.11**: Token counting — BPE encoding matching OpenAI's tiktoken
- **pulldown-cmark ^0.13**: Markdown rendering — events-based parsing for AI output
- **@sveltejs/adapter-static**: Required for Tauri — SPA from webview, no SSR

### Expected Features

**Must have (table stakes):**
- Streaming AI chat with token-by-token output — users expect real-time typewriter
- Character cards with TavernAI V2 import/export — the de facto sharing standard
- Persona management with per-conversation switching
- Conversation persistence and history
- Budget-aware prompt engineering with token counting
- Provider configuration with OS keychain for API keys
- Markdown rendering for AI responses

**Should have (competitive):**
- Lorebook (world info) with keyword-triggered context injection
- Conversation branching with parallel timelines
- Regex scripting for output post-processing
- Full-text search across conversations (FTS5)
- Character/persona/lorebook editors (multi-step wizards)
- Theme toggle (dark/light)

**Explicitly excluded:**
- Vault/encryption mode — was speculative, not a real requirement
- Cloud sync — conflicts with local-first privacy proposition
- Mobile app — desktop only

### Architecture Approach

Two-layer architecture: SvelteKit frontend (webview) communicates with Rust backend (Tauri) via IPC. Frontend holds UI state in Svelte 5 runes (AppState); backend holds service instances in tauri::State; SQLite is the source of truth for persistent data. The Relm4 Model/Update/Message pattern from `gtk_ref/` maps to Tauri's command (request-response) and channel (streaming) patterns.

**Major components:**
1. **Tauri Commands** — IPC handlers that delegate to services (#[tauri::command])
2. **Service Layer** — AiService (rig-core), DbService (SQLite), KeyringStore, PromptEngine, TokenService, MarkdownService, CardIO, LorebookService, RegexService
3. **Domain Models** — Character, Conversation, Persona, Provider, Generation, Lorebook, RegexScript, AppSettings (port from gtk_ref/)
4. **Frontend AppState** — Svelte 5 runes, existing mock patterns adapted for real invoke() calls

### Critical Pitfalls

1. **Synchronous DB in commands** — Never call rusqlite from #[tauri::command]; use spawn_blocking or background worker
2. **Channel vs Event confusion** — Use Channel<T> for streaming (ordered, typed, scoped); emit/listen for broadcasts only
3. **SSR/adapter mismatch** — Must use adapter-static with fallback: 'index.html'; current ssr=false is necessary but not sufficient
4. **API key exposure** — All keys via keyring crate, never in DB/config/logs; hard security constraint
5. **Tauri v2 permissions** — More restrictive than v1; each command needs explicit capability allowlist

## Implications for Roadmap

### Phase 1: Tauri Shell + SvelteKit Adapter
**Rationale:** Foundation — everything else depends on the app running in a Tauri window
**Delivers:** App launches as native desktop window; dev and production builds work
**Addresses:** Existing SvelteKit SPA running in Tauri
**Avoids:** SSR conflicts (Pitfall 3), permissions surprises (Pitfall 12), dev/prod drift (Pitfall 11)

### Phase 2: Domain Models + Database Layer
**Rationale:** All services depend on persisted data; models define the data contract
**Delivers:** SQLite DB with migrations, domain model structs, CRUD operations for all entities
**Uses:** rusqlite (bundled), dirs crate for platform paths, tokio spawn_blocking
**Implements:** DbService component, domain model port from gtk_ref/
**Avoids:** Sync DB on main thread (Pitfall 1), cross-platform path issues (Pitfall 8)

### Phase 3: Keyring + Provider Configuration
**Rationale:** AI service depends on provider configs and API keys
**Delivers:** Provider CRUD with API key storage in OS keychain; provider list in settings UI wired to real backend
**Uses:** keyring crate, rig-core provider types
**Avoids:** API key exposure (Pitfall 4), Linux keyring absence (Pitfall 6)

### Phase 4: AI Service + Streaming
**Rationale:** Core value proposition — real AI conversations replacing mock streaming
**Delivers:** Streaming AI chat via Tauri channels; stop/regenerate; all rig-core providers working
**Uses:** rig-core stream_chat/stream_completion, Tauri Channel<T>
**Avoids:** Channel vs Event confusion (Pitfall 2), streaming backpressure (Pitfall 10), rig-core API drift (Pitfall 5)

### Phase 5: Prompt Engine + Token Counting
**Rationale:** Makes AI conversations work correctly — budget-aware context assembly
**Delivers:** Prompt assembly from character+persona+history; token counting display; context utilization bar with real data
**Uses:** tiktoken-rs, prompt engine port from gtk_ref/
**Avoids:** Token count mismatch (Pitfall 7)

### Phase 6: Markdown + Card I/O
**Rationale:** Polish AI output rendering; enable character card sharing
**Delivers:** Real markdown rendering via pulldown-cmark; TavernAI V2 card import/export
**Uses:** pulldown-cmark, png crate for card metadata
**Avoids:** PNG metadata corruption (Pitfall 9)

### Phase 7: Editors (Character, Persona, Lorebook)
**Rationale:** Users need to create/edit entities — currently only view-only cards exist
**Delivers:** Character editor wizard, persona editor, lorebook editor (Bits.ui Dialog-based)
**Uses:** Bits.ui Dialog, existing domain models and CRUD services

### Phase 8: Conversation Branching + Full-Text Search
**Rationale:** Advanced conversation features that differentiate from basic chat
**Delivers:** Fork conversations from any message; FTS5 search across all conversations
**Uses:** Tree-structured message schema, SQLite FTS5

### Phase 9: Regex Scripting
**Rationale:** Output post-processing for formatting, find/replace, trigger warnings
**Delivers:** Regex script CRUD, ordered application to AI output, enable/disable toggles
**Avoids:** ReDoS (Pitfall 14)

### Phase 10: Theme Toggle + Polish
**Rationale:** User preference for light mode; final polish before distribution
**Delivers:** Dark/light theme with system preference detection; any remaining UI refinements

### Phase 11: Design Document Update + Cross-Platform Distribution
**Rationale:** Spec should reflect reality; app must build and distribute on all three platforms
**Delivers:** Updated design spec (GTK→SvelteKit/Tauri); macOS/Linux/Windows builds
**Can run in parallel** with other phases (design doc update is documentation, not blocking code)

### Phase Ordering Rationale

- Phases 1-3 are sequential foundation: shell → data → credentials
- Phases 4-5 are the critical path: AI streaming is the core value, prompt engine makes it correct
- Phases 6-9 can partially overlap: markdown, card I/O, editors, and advanced features are relatively independent
- Phase 10 (theme) is independent and low-effort
- Phase 11 (design doc + distribution) can run in parallel with phases 6-10

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 4 (AI Streaming):** Complex integration — verify rig-core streaming API with Tauri channels, test with multiple providers
- **Phase 6 (Card I/O):** Niche domain — verify TavernAI V2 PNG spec, test with real character cards from community
- **Phase 11 (Distribution):** Cross-platform builds — macOS signing/notarization, Linux packaging formats, Windows MSVC requirements

Phases with standard patterns (skip research-phase):
- **Phase 1 (Tauri Shell):** Well-documented in Tauri official docs, SvelteKit guide exists
- **Phase 2 (DB Layer):** Standard rusqlite pattern, proven in gtk_ref/
- **Phase 3 (Keyring):** Standard keyring crate usage, proven in gtk_ref/
- **Phase 5 (Prompt Engine):** Port from gtk_ref/ with known working implementation

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All technologies verified via Context7 and official docs; most already proven in gtk_ref/ |
| Features | HIGH | Clear from existing codebase, design spec, and established AI roleplay apps (SillyTavern) |
| Architecture | HIGH | Tauri v2 + SvelteKit pattern is well-documented; gtk_ref/ provides proven service patterns |
| Pitfalls | HIGH | Most pitfalls are well-known in Tauri ecosystem; verified via official docs |

**Overall confidence:** HIGH

### Gaps to Address

- **tiktoken-rs exact version:** MEDIUM confidence on 0.11 — verify current crate version and API compatibility before Phase 5
- **rig-core 0.36 API stability:** Verify streaming API surface hasn't changed since gtk_ref/ was written — test in Phase 4
- **Tauri v2 plugin ecosystem:** Some functionality (e.g., autoupdate, system tray) may use plugins not yet researched — assess as needed during build
- **macOS code signing:** Not researched in detail — needed for Phase 11 distribution; may require Apple Developer account

## Sources

### Primary (HIGH confidence)
- Context7 /websites/v2_tauri_app — Tauri v2 commands, channels, state, SvelteKit integration, permissions
- Context7 /websites/rs_tauri_2_9_5 — Tauri v2 API reference, Channel, Emitter, Listener
- Context7 /websites/rs_rig-core — rig-core providers, streaming, agent API, DefaultProviders enum
- Existing codebase (.planning/codebase/) — Architecture, stack, structure analysis

### Secondary (MEDIUM confidence)
- gtk_ref/ archival codebase — Proven service patterns, domain models, DB schema
- SillyTavern feature set — Reference for AI roleplay table stakes features

### Tertiary (LOW confidence)
- TavernAI V2 character card spec — PNG metadata format; needs validation with real cards
- Cross-platform distribution specifics — macOS signing, Linux packaging; needs research in Phase 11

---
*Research completed: 2026-05-10*
*Ready for roadmap: yes*

# Revelry

## What This Is

Revelry is a cross-platform desktop AI roleplay application built with Tauri and SvelteKit. It lets users create and inhabit characters and personas, then converse with AI-powered characters through a rich, content-first interface. The SvelteKit frontend provides the UI; the Tauri Rust backend handles AI inference, data persistence, secrets, and all non-UI logic.

## Core Value

Users can have immersive, streaming AI conversations with custom characters through personas — and the whole thing runs locally as a native desktop app.

## Requirements

### Validated

<!-- Inferred from existing codebase — already built and working -->

- ✓ SvelteKit SPA shell with header, sidebar, and view switching — existing
- ✓ Conversation list with relative timestamps and search — existing
- ✓ Active conversation view with message rendering and auto-scroll — existing
- ✓ Message input with auto-grow, token estimate, and context utilization bar — existing
- ✓ Mock streaming response (fake token-by-token with typewriter animation) — existing
- ✓ Persona switcher popover with active-persona indication — existing
- ✓ Character & persona library with tab switching, search, and grid layout — existing
- ✓ Settings shell with provider list and shortcut reference — existing
- ✓ Keyboard shortcuts (F9 sidebar toggle, Ctrl+Enter / Enter send) — existing
- ✓ Svelte 5 rune-based centralized state management (AppState) — existing
- ✓ TypeScript type definitions mirroring design spec schema — existing
- ✓ Bits.ui headless components with Adwaita-inspired styling — existing
- ✓ Tailwind v4 theme tokens (dark by default, Fraunces for character names, Geist for UI) — existing
- ✓ Rust domain models (Character, Conversation, Persona, Provider, Generation, Lorebook, RegexScript, AppSettings) — existing (archival gtk_ref/)
- ✓ Rig-core provider-agnostic AI service with streaming support (15+ providers) — existing (archival gtk_ref/)
- ✓ SQLite persistence layer with background-thread worker — existing (archival gtk_ref/)
- ✓ OS-native keyring integration (macOS Keychain, GNOME Keyring, Windows Credential Manager) — existing (archival gtk_ref/)
- ✓ Prompt engine with budget-aware assembly — existing (archival gtk_ref/)
- ✓ BPE tokenization via tiktoken-rs for context window tracking — existing (archival gtk_ref/)
- ✓ Markdown rendering via pulldown-cmark — existing (archival gtk_ref/)
- ✓ Card I/O (character/persona import-export) — existing (archival gtk_ref/)
- ✓ Lorebook service — existing (archival gtk_ref/)
- ✓ Regex scripting service — existing (archival gtk_ref/)

### Active

<!-- Current scope — porting and wiring to create the full Tauri app -->

- ✓ Tauri desktop shell wrapping the SvelteKit SPA — Phase 1
- ✓ Migrate Rust domain models from gtk_ref/ to src-tauri/ — Phase 2
- [ ] Migrate rig-core AI service to src-tauri/ with full provider support
- ✓ Migrate SQLite persistence layer to src-tauri/ — Phase 2
- ✓ Character and persona editors — Phase 3
- ✓ Lorebook editor with keyword testing — Phase 3
- ✓ Avatar upload and display via Tauri asset protocol — Phase 3
- ✓ Migrate Rust domain models from gtk_ref/ to src-tauri/ — Phase 2
- [ ] Migrate OS keyring secrets integration to src-tauri/
- [ ] Migrate prompt engine to src-tauri/
- [ ] Migrate token counting (tiktoken-rs) to src-tauri/
- [ ] Migrate markdown rendering (pulldown-cmark) to src-tauri/
- [ ] Migrate card I/O service to src-tauri/
- [ ] Migrate lorebook service to src-tauri/
- [ ] Migrate regex scripting service to src-tauri/
- ✓ Wire SvelteKit frontend to Tauri backend via invoke() commands — Phase 2
- [ ] Replace mock streaming with real AI provider streaming via Tauri IPC
- ✓ Replace mock data with real SQLite persistence via Tauri IPC — Phase 2
- [ ] Replace token estimate heuristic with real tiktoken-rs counts via Tauri IPC
- [ ] Replace naive markdown rendering with pulldown-cmark output via Tauri IPC
- [ ] Character and persona editors (multi-step wizards from spec §7.4)
- [ ] Lorebook editor UI
- [ ] Theme toggle (dark/light)
- [ ] Branch navigation for conversation branching
- [ ] Full-text search across conversations
- [ ] Update design specification (docs/) to replace GTK4/GNOME references with SvelteKit/Tauri equivalents
- [ ] Cross-platform build and distribution (macOS, Linux, Windows)

### Out of Scope

<!-- Explicit boundaries — not building these -->

- Vault mode (AES-256-GCM encryption with Argon2id key derivation) — was speculative, not a real requirement
- Multi-user / cloud sync — local-first single-user desktop app
- Mobile app — desktop only
- Web-hosted deployment — Tauri native distribution only

## Context

**Prior work:** The project began as a Rust/GTK4/libadwaita desktop app (archived in `gtk_ref/`). A complete proof-of-concept was built with Relm4, rig-core, rusqlite, and OS keyring integration. The decision was then made to shift the UI layer to SvelteKit + Bits.ui for faster iteration and Tauri compatibility. The SvelteKit frontend scaffold is now built with mock data and mock streaming; the Rust backend logic in `gtk_ref/` is preserved as an archival reference for migration.

**Design specification:** A v2.0 design spec exists at `docs/Revelry_Design_Specification_v2.0.docx`. It was written for GTK4/GNOME and needs updating to reflect the SvelteKit/Tauri architecture. The update scope is primarily replacing GTK/GNOME references with SvelteKit/Tauri equivalents — not a fundamental redesign.

**Rig-core provider support:** The most-used provider will be custom OpenAI-compatible endpoints, but rig-core supports 15+ providers natively. The full rig-core abstraction should be preserved — no reason to subset.

**SQLite schema:** The schema in `gtk_ref/` worked well. It can be used as-is or evolved if a better representation is found during implementation.

**UI state:** The SvelteKit frontend scaffold is solid — no changes needed to the existing UI patterns. The work is primarily backend migration and wiring.

## Constraints

- **Tech Stack**: Tauri + SvelteKit + Svelte 5 (runes) + Bits.ui + Tailwind v4 — UI is already built in this stack; backend must be Rust via Tauri
- **Cross-Platform**: Must run on macOS, Linux, and Windows — affects keyring, file paths, and distribution
- **AI Provider**: rig-core is the provider abstraction — all 15+ providers supported, custom OpenAI-compatible is priority
- **Local-First**: All data stored locally via SQLite; no cloud backend; API keys stored in OS keychain
- **Streaming**: AI conversations must stream token-by-token to the UI with low latency
- **Data Compatibility**: The `gtk_ref/` SQLite schema is the starting point; can evolve but should not unnecessarily diverge

## Key Decisions

| Decision                              | Rationale                                                               | Outcome                                    |
| ------------------------------------- | ----------------------------------------------------------------------- | ------------------------------------------ |
| SvelteKit over GTK4 for UI            | Faster iteration, Tauri-compatible, web-native tooling                  | ✓ Good — UI scaffold is solid              |
| Bits.ui over styled component library | Headless = full styling control for Adwaita aesthetic                   | ✓ Good — matches design intent             |
| Tauri for desktop shell               | Embeds the existing SPA, Rust backend for native logic                  | ✓ Good — Phase 1 verified, builds on macOS |
| Bundle identifier com.revelry.desktop | Changed from com.revelry.app to avoid macOS .app suffix conflict        | ✓ Good — clean build, no warnings          |
| DateTime<Utc> with .to_rfc3339()      | Better IPC portability than NaiveDateTime                               | ✓ Good — Phase 2                           |
| Single collapsed migration            | No data to preserve, cleaner than 9 sequential migrations               | ✓ Good — Phase 2                           |
| MessageRole Assistant (not Character) | Aligned with rig-core convention                                        | ✓ Good — Phase 2                           |
| rig-core for AI providers             | Provider-agnostic, supports 15+ providers with streaming                | — Pending                                  |
| Avatar via Tauri asset protocol       | Native file picker + absolute path + convertFileSrc()                   | ✓ Good — Phase 3                           |
| Dark theme by default                 | Creative roleplay users work at night; respects content-first principle | ✓ Good                                     |
| Exclude vault mode                    | Was speculative, adds significant complexity without confirmed need     | — Pending                                  |
| Preserve full gtk_ref feature set     | No subsetting — complete the project as designed                        | — Pending                                  |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):

1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):

1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---

_Last updated: 2026-05-18 after Phase 7

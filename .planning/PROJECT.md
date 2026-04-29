# Revelry

## What This Is

Revelry is a cross-platform desktop application for immersive AI roleplay with AI-powered characters. Built entirely in Rust with GTK4/libadwaita for a native GNOME-inspired desktop experience and the rig crate ecosystem for true provider-agnostic AI inference across 20+ providers — including cloud (OpenAI, Anthropic, Gemini) and local (Ollama, LM Studio, vLLM). All user data is stored locally in SQLite with no mandatory cloud synchronization, no account creation, and no content filtering.

## Core Value

Users can roleplay with AI characters through any AI provider they choose — cloud or local — with complete creative freedom and complete data privacy. The provider layer must be truly agnostic: no provider is privileged, local inference is first-class, and switching between providers is seamless.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Connect to any AI provider through rig's unified abstraction (OpenAI, Anthropic, Ollama, Gemini, Cohere, Mistral, Groq, DeepSeek, OpenRouter, Azure OpenAI, Together AI, HuggingFace, Perplexity, xAI, and custom OpenAI-compatible endpoints)
- [ ] Stream AI responses token-by-token with smooth typewriter rendering via glib channel bridge
- [ ] Create, edit, and delete AI character cards following the CharacterCardV2 specification
- [ ] Import character cards from PNG (embedded metadata) and JSON formats
- [ ] Export character cards as PNG and JSON
- [ ] Display character library in a searchable, tag-filterable grid
- [ ] Maintain conversation history with full message CRUD
- [ ] Branch conversations by editing or regenerating any message, preserving original branches
- [ ] Navigate between conversation branches with visual branch indicators
- [ ] Create, edit, and delete user personas with structured fields (name, appearance, personality, background, role, relationships, speech style)
- [ ] Inject active user persona data into AI prompt as a structured block
- [ ] Resolve {{user}}, {{char}}, and {{persona.*}} template variables at prompt assembly time
- [ ] Assign user personas per-conversation, per-character, and as global default
- [ ] Switch active persona mid-conversation via popover UI
- [ ] Compress persona block when token budget is tight (full → trimmed → minimal → essential)
- [ ] Create, edit, and delete prompt presets (temperature, top_p, max_tokens, penalties, stop sequences)
- [ ] Assign prompt presets at character and conversation levels
- [ ] Manage lorebook entries per character with keyword activation, secondary terms, insertion position, depth, and activation state
- [ ] Support persona-aware lorebook activation (entries activate based on active persona data)
- [ ] Support recursive lorebook activation with configurable depth limit
- [ ] Implement multi-strategy context window management (sticky system prompt + character card, semi-sticky persona block, activated lorebooks, truncated conversation history)
- [ ] Count tokens using tiktoken-rs BPE tokenization for context budget tracking
- [ ] Configure multiple AI providers simultaneously with provider-specific settings
- [ ] Store API keys in OS-native credential store (GNOME Keyring, macOS Keychain, Windows Credential Manager) via keyring crate
- [ ] Conduct group chats with multiple AI characters in a single conversation
- [ ] Define regex find-and-replace scripts per character with ReDoS validation
- [ ] Search across all conversations using SQLite FTS5 full-text search
- [ ] Encrypt database and character files with AES-256-GCM vault mode (Argon2id key derivation)
- [ ] Track persona evolution with timestamped snapshots linked to conversations
- [ ] Create child personas inheriting from parent personas with field-level fallback
- [ ] Use dual-persona mode to portray two identities in a single conversation
- [ ] Build UI with GTK4/libadwaita native widgets (AdwHeaderBar, AdwOverlaySplitView, AdwAvatar, AdwPreferencesWindow, AdwToastOverlay, AdwClamp)
- [ ] Implement adaptive layout (sidebar alongside content in wide windows, overlay in narrow)
- [ ] Follow GNOME design principles: content-first, progressive disclosure, consistent visual language
- [ ] Render AI responses as formatted text using pulldown-cmark + Pango markup
- [ ] Support all keyboard navigation per GNOME accessibility standards
- [ ] Bridge tokio runtime and GLib main loop via glib::MainContext channels for non-blocking UI
- [ ] Run database operations on background threads to keep UI responsive
- [ ] Create characters via guided multi-step wizard (AdwPreferencesWindow pattern)
- [ ] Import from SillyTavern and CharacterAI community export formats
- [ ] Batch import/export for character libraries
- [ ] Backup/restore application data as portable SQL dump or ZIP archive
- [ ] Validate and sanitize imported character cards (schema validation, PNG chunk size limits, ReDoS detection for regex scripts)

### Out of Scope

- Mobile application — desktop-first, mobile is a separate future project
- Cloud synchronization or online account system — local-first architecture
- Built-in content moderation or filtering — users have full creative control
- Web-based UI (Electron/Tauri/WebView) — native GTK4 only
- Proprietary AI provider lock-in — must support open provider ecosystem

## Context

- **Personal hobby project** — no corporate timeline, scope pressure, or stakeholder approval. Build quality over speed.
- **Developing on macOS** — GTK4/libadwaita works on macOS but the native GNOME feel is strongest on Linux. Cross-platform testing needed.
- **GNOME design is strong inspiration, not hard HIG requirement** — will not sacrifice features or usability for strict GNOME Circle compliance, but the visual language and interaction patterns follow GNOME conventions.
- **Privacy and creative freedom are inseparable** — both are core values, not just practical details. No content policy, no telemetry, no cloud dependency.
- **True provider agnosticism** — no provider is privileged. Local models (Ollama) are first-class, not an afterthought. Switching providers should feel seamless.
- **CharacterCardV2 spec** — compatibility with SillyTavern ecosystem for maximum interoperability with existing character card libraries.
- **Design specification** — comprehensive v2.0 spec in `docs/Revelry_Design_Specification_v2.0.docx` covers architecture, UI, data model, features, and security in detail. This document is the canonical reference.

## Constraints

- **Tech Stack**: Pure Rust — GTK4/libadwaita + Relm4 (UI), rig (AI inference), SQLite/rusqlite (storage), keyring (secrets), pulldown-cmark (markdown), tokio (async runtime)
- **No IPC Bridge**: Must be a single-process Rust application — no Tauri, no Electron, no web frontend communicating with a Rust backend
- **Local-First**: All data in local SQLite — no mandatory cloud sync, no account creation, no telemetry
- **API Key Security**: Keys stored in OS keychain only — never in database, config files, or logs
- **Content Policy**: No built-in filtering or moderation — user has complete creative control
- **Cross-Platform**: Must run on Linux, macOS, and Windows — GTK4/libadwaita on all platforms
- **Streaming**: AI responses must stream token-by-token with cancellation support

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Pure Rust (no Tauri/Electron) | Eliminates IPC bridge complexity, single-language codebase, lower latency, smaller binary | — Pending |
| GTK4/libadwaita (not web UI) | Native GNOME widgets, automatic HIG compliance, proper accessibility, GNOME Circle eligibility | — Pending |
| Relm4 for declarative UI | Elm Architecture provides deterministic state, clean separation, type-safe view macros | — Pending |
| rig framework (not raw reqwest) | Native client implementations for 20+ providers, unified API, agent abstractions, structured output, streaming | — Pending |
| SQLite/rusqlite (bundled) | Zero-config local storage, portable, fast, single-file database | — Pending |
| OS keychain via keyring crate | Most secure credential storage per-platform, keys never in application memory longer than needed | — Pending |
| CharacterCardV2 spec compatibility | Maximum interoperability with SillyTavern ecosystem and existing character card libraries | — Pending |
| glib::MainContext channel bridge | Proven pattern for tokio ↔ GTK main loop communication, keeps UI responsive | — Pending |
| GNOME design as strong inspiration (not hard HIG requirement) | Will not sacrifice features for strict compliance, but follows GNOME visual language and interaction patterns | — Pending |

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
*Last updated: 2026-04-28 after initialization*

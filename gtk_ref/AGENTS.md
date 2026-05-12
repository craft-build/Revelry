<!-- GSD:project-start source:PROJECT.md -->

## Project

**Revelry**

Revelry is a cross-platform desktop application for immersive AI roleplay with AI-powered characters. Built entirely in Rust with GTK4/libadwaita for a native GNOME-inspired desktop experience and the rig crate ecosystem for true provider-agnostic AI inference across 20+ providers — including cloud (OpenAI, Anthropic, Gemini) and local (Ollama, LM Studio, vLLM). All user data is stored locally in SQLite with no mandatory cloud synchronization, no account creation, and no content filtering.

**Core Value:** Users can roleplay with AI characters through any AI provider they choose — cloud or local — with complete creative freedom and complete data privacy. The provider layer must be truly agnostic: no provider is privileged, local inference is first-class, and switching between providers is seamless.

### Constraints

- **Tech Stack**: Pure Rust — GTK4/libadwaita + Relm4 (UI), rig (AI inference), SQLite/rusqlite (storage), keyring (secrets), pulldown-cmark (markdown), tokio (async runtime)
- **No IPC Bridge**: Must be a single-process Rust application — no Tauri, no Electron, no web frontend communicating with a Rust backend
- **Local-First**: All data in local SQLite — no mandatory cloud sync, no account creation, no telemetry
- **API Key Security**: Keys stored in OS keychain only — never in database, config files, or logs
- **Content Policy**: No built-in filtering or moderation — user has complete creative control
- **Cross-Platform**: Must run on Linux, macOS, and Windows — GTK4/libadwaita on all platforms
- **Streaming**: AI responses must stream token-by-token with cancellation support
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->

## Technology Stack

## Recommended Stack

### Core Framework (UI Layer)

| Technology  | Version | Purpose                         | Why                                                                                                                                                                                                                      | Confidence |
| ----------- | ------- | ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------- |
| gtk4        | 0.11.3  | GTK4 Rust bindings              | The only mature Rust binding for GTK4. Maintained by gtk-rs project. Required by Relm4.                                                                                                                                  | HIGH       |
| libadwaita  | 0.9.1   | Adwaita widget bindings         | Provides AdwHeaderBar, AdwOverlaySplitView, AdwAvatar, AdwPreferencesWindow, AdwToastOverlay, AdwClamp, AdwBreakpoint. Enables GNOME-adaptive layouts. Feature flags: `v1_5` minimum for breakpoint support.             | HIGH       |
| relm4       | 0.11.0  | Declarative UI framework        | Elm Architecture provides deterministic state management, type-safe view macros, clean Model-Update-View separation. The `libadwaita` feature flag enables Adwaita widget support. Version 0.11 aligns with gtk4 0.11.x. | HIGH       |
| relm4-icons | 0.11.0  | Icon bundles for Relm4          | Bundled symbolic icons matching Adwaita icon theme. Version-locked to relm4 0.11.                                                                                                                                        | MEDIUM     |
| tracker     | 0.2.2   | Fine-grained UI update tracking | `#[tracker::track]` macro on model structs + `#[track]` in view macros = only re-render widgets whose data changed. Critical for perf with complex character cards.                                                      | HIGH       |
| pango       | 0.22.6  | Text layout/rendering           | Pango markup for formatted AI responses. Works with pulldown-cmark for Markdown→Pango conversion.                                                                                                                        | HIGH       |
| sourceview5 | 0.11.0  | Syntax-highlighted text editor  | GTK SourceView widget for message editing with syntax awareness. Optional — can defer to a later phase if plain GtkTextView suffices initially.                                                                          | MEDIUM     |

### AI Inference Layer

| Technology | Version | Purpose                          | Why                                                                                                                                                                                                                                                                                                                       | Confidence |
| ---------- | ------- | -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- |
| rig-core   | 0.36.0  | Provider-agnostic AI abstraction | Native client implementations for 19+ providers (OpenAI, Anthropic, Gemini, Ollama, Cohere, Mistral, Groq, DeepSeek, OpenRouter, xAI, Perplexity, Together, Azure OpenAI, HuggingFace). Unified `Prompt`, `Chat`, and `StreamingPrompt` traits. Agent builder pattern with preamble, temperature, and tool configuration. | HIGH       |
| tokio      | 1.52.1  | Async runtime                    | Required by rig-core for async HTTP. Relm4 commands run on tokio. The `rt-multi-thread` and `macros` features are required.                                                                                                                                                                                               | HIGH       |
| futures    | 0.3.32  | Stream combinators               | `StreamExt` for processing rig's streaming responses chunk-by-chunk. `StreamMap` for parallel provider health checks.                                                                                                                                                                                                     | HIGH       |

### Database Layer

| Technology         | Version | Purpose          | Why                                                                                                                                           | Confidence |
| ------------------ | ------- | ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------- | ---------- |
| rusqlite           | 0.39.0  | SQLite bindings  | Ergonomic Rust wrapper. The `bundled` feature compiles SQLite from C source — zero external dependency, consistent behavior across platforms. | HIGH       |
| rusqlite_migration | 2.5.0   | Schema migration | Uses SQLite's `user_version` for fast migration tracking. Atomic schema updates. Simpler than writing manual migration logic.                 | HIGH       |

- `bundled`: Compiles SQLite from source. **Required** for cross-platform zero-config builds.
- `window`: Window functions for advanced queries (conversation analytics).
- `functions`: Custom SQL functions (e.g., token counting in SQL).
- `vtab`: Virtual table support (FTS5 full-text search is built into SQLite itself, but `vtab` enables custom virtual tables if needed later).

### Security & Secrets

| Technology | Version | Purpose                      | Why                                                                                                                                                                                                                                                                                    | Confidence |
| ---------- | ------- | ---------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- |
| keyring    | 3.6.3   | OS-native credential storage | macOS Keychain, GNOME Keyring (via Secret Service), Windows Credential Manager. Feature flags: `apple-native`, `linux-native`, `windows-native`. **Do NOT use 4.0.0** — released 2 days ago (2026-04-26) with empty feature flags, likely a breaking API change that needs validation. | HIGH       |
| aes-gcm    | 0.10.3  | AES-256-GCM encryption       | Standard authenticated encryption for vault mode. The 0.11.x line is pre-release (all `-rc`/`-pre` versions). 0.10.3 is the stable release.                                                                                                                                            | HIGH       |
| argon2     | 0.5.3   | Key derivation (Argon2id)    | Password hashing / key derivation for vault encryption. Configurable memory, iterations, parallelism. The 0.6.x line is pre-release. 0.5.3 is stable.                                                                                                                                  | HIGH       |

### Data & Content Processing

| Technology     | Version | Purpose                   | Why                                                                                                                                                                                                      | Confidence |
| -------------- | ------- | ------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- |
| pulldown-cmark | 0.13.3  | Markdown → events         | Parse AI response Markdown into events, then convert to Pango markup. Pull parser design = no allocation for skipped elements.                                                                           | HIGH       |
| tiktoken-rs    | 0.11.0  | BPE tokenization          | OpenAI-compatible token counting for context window budget tracking. Same tokenizer as GPT-3.5/4. Note: Anthropic models use a different tokenizer — tiktoken is an approximation for non-OpenAI models. | MEDIUM     |
| png            | 0.18.1  | PNG chunk reading/writing | Read/write tEXt/iTXt chunks for CharacterCardV2 embedded metadata in PNGs. The `image` crate doesn't expose chunk-level control — need `png` directly.                                                   | HIGH       |
| image          | 0.25.10 | Image processing          | Character card avatar display, resizing, format conversion. Handles JPEG, PNG, WebP.                                                                                                                     | HIGH       |
| base64         | 0.22.1  | Base64 encode/decode      | Character card image data encoding/decoding in V2 spec JSON fields.                                                                                                                                      | HIGH       |
| regex          | 1.12.3  | Regex find-and-replace    | Character-specific regex scripts with ReDoS validation (use `regex::RegexBuilder::size_limit()`).                                                                                                        | HIGH       |
| serde          | 1.0.228 | Serialization framework   | Derive macros for all data models (character cards, personas, presets, lorebooks).                                                                                                                       | HIGH       |
| serde_json     | 1.0.149 | JSON serialization        | CharacterCardV2 spec is JSON. Import/export. Configuration files.                                                                                                                                        | HIGH       |

### Application Infrastructure

| Technology         | Version | Purpose                      | Why                                                                                                                                                | Confidence |
| ------------------ | ------- | ---------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- |
| anyhow             | 1.0.102 | Error handling (application) | Ergonomic error propagation for application code. Use `thiserror` for library-type modules, `anyhow` for the app binary.                           | HIGH       |
| thiserror          | 2.0.18  | Error handling (library)     | Derive `Error` for domain error types (provider errors, migration errors, card validation errors). Version 2.0 changed some macro syntax from 1.x. | HIGH       |
| tracing            | 0.1.44  | Structured logging           | Correlation IDs for tracing AI requests across provider → streaming → UI pipeline.                                                                 | HIGH       |
| tracing-subscriber | 0.3.23  | Log output configuration     | `fmt` + `env-filter` subscribers for development. File logging for diagnostics.                                                                    | HIGH       |
| clap               | 4.6.1   | CLI argument parsing         | Command-line flags (--debug, --data-dir, --config). Relm4 supports passing GTK args through.                                                       | HIGH       |
| dirs               | 6.0.0   | Platform directory paths     | XDG on Linux, ~/Library on macOS, %APPDATA% on Windows. For data/config/cache directories.                                                         | HIGH       |
| uuid               | 1.23.1  | Unique identifiers           | `v4` feature for random IDs on characters, conversations, messages.                                                                                | HIGH       |
| chrono             | 0.4.44  | Timestamps                   | Message timestamps, persona evolution snapshots, conversation metadata.                                                                            | HIGH       |
| zip                | 8.6.0   | Archive import/export        | Batch character card import/export, backup/restore as ZIP archives.                                                                                | MEDIUM     |
| toml               | 1.1.2   | Configuration files          | If application settings are stored as TOML (alternative to SQLite settings table).                                                                 | MEDIUM     |

### GLib/GTK Build Infrastructure

| Technology       | Version | Purpose                          | Why                                                                                                                        | Confidence |
| ---------------- | ------- | -------------------------------- | -------------------------------------------------------------------------------------------------------------------------- | ---------- |
| glib             | 0.22.6  | GLib core bindings               | `MainContext::channel()` for tokio → GLib main loop bridge. `glib::spawn_future_local()` for async tasks on the main loop. | HIGH       |
| gio              | 0.22.6  | GIO bindings                     | File monitoring, async I/O, application registration.                                                                      | HIGH       |
| glib-build-tools | 0.22.0  | Build-time GResource compilation | Compile icon/resource bundles into the binary via `build.rs`.                                                              | HIGH       |

## Alternatives Considered

| Category       | Recommended             | Alternative              | Why Not                                                                                                                                                              |
| -------------- | ----------------------- | ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| UI Framework   | GTK4/libadwaita + Relm4 | Tauri                    | Constraint: "No IPC bridge" — Tauri requires Rust backend + WebView frontend communicating via IPC                                                                   |
| UI Framework   | GTK4/libadwaita + Relm4 | Iced                     | Iced is pure-Rust but lacks libadwaita, GNOME integration, and Adwaita widgets. No native adaptive layout. Smaller ecosystem.                                        |
| UI Framework   | GTK4/libadwaita + Relm4 | Slint                    | Slint is declarative but not GTK-native. Renders its own UI. No Adwaita. Not GNOME Circle eligible.                                                                  |
| UI Framework   | GTK4/libadwaita + Relm4 | Dioxus (desktop)         | WebView-based (uses wry/tao). Same IPC problem as Tauri.                                                                                                             |
| AI Abstraction | rig-core                | Raw reqwest per-provider | Reimplementing provider clients, streaming parsers, error handling for 19+ providers = months of work. rig gives unified API for free.                               |
| AI Abstraction | rig-core                | llm-chain                | llm-chain is less mature, fewer providers, no streaming support as mature as rig's. Lower activity.                                                                  |
| Database       | rusqlite (bundled)      | diesel + SQLite          | Diesel is an ORM — overkill for a local-first app with simple schema. Rusqlite gives raw SQL control for FTS5, window functions, custom functions.                   |
| Database       | rusqlite (bundled)      | sqlx + SQLite            | sqlx is async-native but adds significant compile time. Rusqlite + background threads via Relm4 commands achieves the same non-blocking behavior with simpler build. |
| Database       | rusqlite (bundled)      | sled                     | sled is pure-Rust but lacks FTS5, is alpha-quality, and has had data-loss bugs. SQLite is battle-tested.                                                             |
| Secrets        | keyring (3.6.3)         | keyring (4.0.0)          | 4.0.0 released 2 days ago with empty feature flags — unstable, API likely changed. Wait for stabilization.                                                           |
| Secrets        | keyring (3.6.3)         | Manual keychain FFI      | Reinventing platform-specific keychain access is error-prone and a security risk. keyring handles it correctly.                                                      |
| Markdown       | pulldown-cmark          | comrak                   | comrak is C-dependent (libcmark-gfm). pulldown-cmark is pure Rust, actively maintained, pull-parser design fits streaming.                                           |
| Tokenizer      | tiktoken-rs             | tokenizers (HuggingFace) | tokenizers crate is heavier (full BPE/WordPiece/BPE SentencePiece). tiktoken-rs matches the OpenAI tokenizer most roleplay APIs expect.                              |
| Encryption     | aes-gcm (0.10.3)        | ring                     | ring is a general crypto library — overkill for just AES-256-GCM. aes-gcm from RustCrypto is focused, auditable, no ASM dependency.                                  |
| Key Derivation | argon2 (0.5.3)          | scrypt                   | Argon2id is the PHC winner and recommended by OWASP. scrypt is older. Argon2id provides better resistance to GPU/ASIC attacks.                                       |

## Architecture Integration Points

### tokio ↔ GLib Bridge

### rig Streaming Integration

### SQLite Background Operations

## Installation

# Core UI

# AI inference

# Database

# Security

# Data processing

# Infrastructure

# Build tools (in [build-dependencies])

### Cargo.toml Skeleton

# UI

# AI

# Database

# Security

# Data

# Infrastructure

# Platform-specific keyring features

## Version Pinning Notes

| Crate                     | Pin Strategy                  | Rationale                                                                                                                              |
| ------------------------- | ----------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| gtk4 / libadwaita / relm4 | Exact match to 0.11.x / 0.9.x | gtk-rs ecosystem crates must share the same major.minor version. Relm4 0.11 requires gtk4 0.11.x.                                      |
| rig-core                  | Compatible with 0.36.x        | Rig is evolving rapidly (0.36.0 published 2026-04-28). Minor versions may add providers. Pin to 0.36 for stability during initial dev. |
| rusqlite                  | Compatible with 0.39.x        | Schema compatibility matters. Minor rusqlite updates rarely break.                                                                     |
| keyring                   | Exact = 3.6.3                 | 4.0.0 is a breaking release too new to trust. Pin to 3.x.                                                                              |
| aes-gcm / argon2          | Compatible                    | RustCrypto crates are mature and stable. Pre-release versions exist but are clearly marked.                                            |

## Sources

- gtk4-rs: https://crates.io/crates/gtk4 (v0.11.3) — **HIGH** (crates.io verified)
- libadwaita-rs: https://crates.io/crates/libadwaita (v0.9.1) — **HIGH** (crates.io verified)
- Relm4: https://crates.io/crates/relm4 (v0.11.0) — **HIGH** (crates.io + Context7 docs verified)
- Relm4 Book: https://relm4.org/book/stable/ — **HIGH** (official docs, verified async/command patterns)
- rig-core: https://crates.io/crates/rig-core (v0.36.0) — **HIGH** (crates.io + Context7 docs verified)
- rig GitHub: https://github.com/0xplaygrounds/rig — **HIGH** (provider list, streaming patterns verified)
- rusqlite: https://crates.io/crates/rusqlite (v0.39.0) — **HIGH** (crates.io + Context7 verified)
- keyring: https://crates.io/crates/keyring (v3.6.3) — **HIGH** (crates.io + docs.rs verified)
- tiktoken-rs: https://crates.io/crates/tiktoken-rs (v0.11.0) — **MEDIUM** (crates.io verified, API not checked via Context7)
- pulldown-cmark: https://crates.io/crates/pulldown-cmark (v0.13.3) — **MEDIUM** (crates.io verified)
- RustCrypto (aes-gcm, argon2): https://crates.io/ — **HIGH** (well-established ecosystem, stable versions verified)
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->

## Conventions

Conventions not yet established. Will populate as patterns emerge during development.

<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->

## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.

<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->

## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, `.github/skills/`, or `.codex/skills/` with a `SKILL.md` index file.

<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->

## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:

- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.

<!-- GSD:workflow-end -->

<!-- GSD:profile-start -->

## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.

<!-- GSD:profile-end -->

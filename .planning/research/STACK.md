# Technology Stack

**Project:** Revelry
**Researched:** 2026-04-28
**Confidence:** HIGH (all versions verified via crates.io and Context7)

## Recommended Stack

### Core Framework (UI Layer)

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| gtk4 | 0.11.3 | GTK4 Rust bindings | The only mature Rust binding for GTK4. Maintained by gtk-rs project. Required by Relm4. | HIGH |
| libadwaita | 0.9.1 | Adwaita widget bindings | Provides AdwHeaderBar, AdwOverlaySplitView, AdwAvatar, AdwPreferencesWindow, AdwToastOverlay, AdwClamp, AdwBreakpoint. Enables GNOME-adaptive layouts. Feature flags: `v1_5` minimum for breakpoint support. | HIGH |
| relm4 | 0.11.0 | Declarative UI framework | Elm Architecture provides deterministic state management, type-safe view macros, clean Model-Update-View separation. The `libadwaita` feature flag enables Adwaita widget support. Version 0.11 aligns with gtk4 0.11.x. | HIGH |
| relm4-icons | 0.11.0 | Icon bundles for Relm4 | Bundled symbolic icons matching Adwaita icon theme. Version-locked to relm4 0.11. | MEDIUM |
| tracker | 0.2.2 | Fine-grained UI update tracking | `#[tracker::track]` macro on model structs + `#[track]` in view macros = only re-render widgets whose data changed. Critical for perf with complex character cards. | HIGH |
| pango | 0.22.6 | Text layout/rendering | Pango markup for formatted AI responses. Works with pulldown-cmark for Markdown→Pango conversion. | HIGH |
| sourceview5 | 0.11.0 | Syntax-highlighted text editor | GTK SourceView widget for message editing with syntax awareness. Optional — can defer to a later phase if plain GtkTextView suffices initially. | MEDIUM |

**Feature flag configuration for Relm4:**
```toml
relm4 = { version = "0.11.0", features = ["libadwaita", "macros"] }
```

The `libadwaita` feature is **required** — it pulls in `libadwaita` crate and enables Adwaita widget support in view macros. The `macros` feature enables `#[relm4::component]` derive macros. The `gnome_47` or `gnome_48` features can be added to target specific GNOME runtime versions for Flatpak distribution.

### AI Inference Layer

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| rig-core | 0.36.0 | Provider-agnostic AI abstraction | Native client implementations for 19+ providers (OpenAI, Anthropic, Gemini, Ollama, Cohere, Mistral, Groq, DeepSeek, OpenRouter, xAI, Perplexity, Together, Azure OpenAI, HuggingFace). Unified `Prompt`, `Chat`, and `StreamingPrompt` traits. Agent builder pattern with preamble, temperature, and tool configuration. | HIGH |
| tokio | 1.52.1 | Async runtime | Required by rig-core for async HTTP. Relm4 commands run on tokio. The `rt-multi-thread` and `macros` features are required. | HIGH |
| futures | 0.3.32 | Stream combinators | `StreamExt` for processing rig's streaming responses chunk-by-chunk. `StreamMap` for parallel provider health checks. | HIGH |

**Feature flag configuration for rig-core:**
```toml
rig-core = { version = "0.36.0", features = ["derive"] }
```

The `derive` feature enables derive macros for custom provider implementations. The `reqwest` feature is default and handles HTTP. For custom OpenAI-compatible endpoints, rig supports building providers via `ProviderBuilder` trait — no separate crate needed.

**Key rig patterns for Revelry:**

1. **Streaming:** `agent.stream_prompt(msg).await` → iterate `MultiTurnStreamItem` with `StreamExt` for token-by-token rendering
2. **Chat history:** `agent.chat(msg, history).await` — history is `Vec<Message>`, managed by the app
3. **Multiple providers:** `openai::Client::from_env()`, `anthropic::Client::from_env()`, `ollama::Client::from_env()` — all produce agents with the same `Prompt`/`StreamingPrompt` trait
4. **Custom endpoints:** Implement `Provider` + `ProviderBuilder` traits for OpenAI-compatible endpoints (vLLM, LM Studio, local servers)

### Database Layer

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| rusqlite | 0.39.0 | SQLite bindings | Ergonomic Rust wrapper. The `bundled` feature compiles SQLite from C source — zero external dependency, consistent behavior across platforms. | HIGH |
| rusqlite_migration | 2.5.0 | Schema migration | Uses SQLite's `user_version` for fast migration tracking. Atomic schema updates. Simpler than writing manual migration logic. | HIGH |

**Feature flag configuration:**
```toml
rusqlite = { version = "0.39.0", features = ["bundled", "window", "functions", "vtab"] }
```

- `bundled`: Compiles SQLite from source. **Required** for cross-platform zero-config builds.
- `window`: Window functions for advanced queries (conversation analytics).
- `functions`: Custom SQL functions (e.g., token counting in SQL).
- `vtab`: Virtual table support (FTS5 full-text search is built into SQLite itself, but `vtab` enables custom virtual tables if needed later).

**FTS5 note:** SQLite FTS5 is compiled into the bundled SQLite by default (it's a compile-time option in SQLite, not a rusqlite feature). The `bundled` feature uses SQLite with all default compile-time options including FTS5.

### Security & Secrets

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| keyring | 3.6.3 | OS-native credential storage | macOS Keychain, GNOME Keyring (via Secret Service), Windows Credential Manager. Feature flags: `apple-native`, `linux-native`, `windows-native`. **Do NOT use 4.0.0** — released 2 days ago (2026-04-26) with empty feature flags, likely a breaking API change that needs validation. | HIGH |
| aes-gcm | 0.10.3 | AES-256-GCM encryption | Standard authenticated encryption for vault mode. The 0.11.x line is pre-release (all `-rc`/`-pre` versions). 0.10.3 is the stable release. | HIGH |
| argon2 | 0.5.3 | Key derivation (Argon2id) | Password hashing / key derivation for vault encryption. Configurable memory, iterations, parallelism. The 0.6.x line is pre-release. 0.5.3 is stable. | HIGH |

**Feature flag configuration for keyring:**
```toml
[target.'cfg(target_os = "macos")'.dependencies]
keyring = { version = "3.6.3", features = ["apple-native"] }
[target.'cfg(target_os = "linux")'.dependencies]
keyring = { version = "3.6.3", features = ["linux-native"] }
[target.'cfg(target_os = "windows")'.dependencies]
keyring = { version = "3.6.3", features = ["windows-native"] }
```

Platform-specific dependency configuration ensures only the relevant backend is compiled on each platform. The `linux-native` feature uses the native Linux keyutils interface (no D-Bus dependency). If GNOME Keyring integration via Secret Service is preferred on Linux, use `sync-secret-service` instead.

### Data & Content Processing

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| pulldown-cmark | 0.13.3 | Markdown → events | Parse AI response Markdown into events, then convert to Pango markup. Pull parser design = no allocation for skipped elements. | HIGH |
| tiktoken-rs | 0.11.0 | BPE tokenization | OpenAI-compatible token counting for context window budget tracking. Same tokenizer as GPT-3.5/4. Note: Anthropic models use a different tokenizer — tiktoken is an approximation for non-OpenAI models. | MEDIUM |
| png | 0.18.1 | PNG chunk reading/writing | Read/write tEXt/iTXt chunks for CharacterCardV2 embedded metadata in PNGs. The `image` crate doesn't expose chunk-level control — need `png` directly. | HIGH |
| image | 0.25.10 | Image processing | Character card avatar display, resizing, format conversion. Handles JPEG, PNG, WebP. | HIGH |
| base64 | 0.22.1 | Base64 encode/decode | Character card image data encoding/decoding in V2 spec JSON fields. | HIGH |
| regex | 1.12.3 | Regex find-and-replace | Character-specific regex scripts with ReDoS validation (use `regex::RegexBuilder::size_limit()`). | HIGH |
| serde | 1.0.228 | Serialization framework | Derive macros for all data models (character cards, personas, presets, lorebooks). | HIGH |
| serde_json | 1.0.149 | JSON serialization | CharacterCardV2 spec is JSON. Import/export. Configuration files. | HIGH |

### Application Infrastructure

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| anyhow | 1.0.102 | Error handling (application) | Ergonomic error propagation for application code. Use `thiserror` for library-type modules, `anyhow` for the app binary. | HIGH |
| thiserror | 2.0.18 | Error handling (library) | Derive `Error` for domain error types (provider errors, migration errors, card validation errors). Version 2.0 changed some macro syntax from 1.x. | HIGH |
| tracing | 0.1.44 | Structured logging | Correlation IDs for tracing AI requests across provider → streaming → UI pipeline. | HIGH |
| tracing-subscriber | 0.3.23 | Log output configuration | `fmt` + `env-filter` subscribers for development. File logging for diagnostics. | HIGH |
| clap | 4.6.1 | CLI argument parsing | Command-line flags (--debug, --data-dir, --config). Relm4 supports passing GTK args through. | HIGH |
| dirs | 6.0.0 | Platform directory paths | XDG on Linux, ~/Library on macOS, %APPDATA% on Windows. For data/config/cache directories. | HIGH |
| uuid | 1.23.1 | Unique identifiers | `v4` feature for random IDs on characters, conversations, messages. | HIGH |
| chrono | 0.4.44 | Timestamps | Message timestamps, persona evolution snapshots, conversation metadata. | HIGH |
| zip | 8.6.0 | Archive import/export | Batch character card import/export, backup/restore as ZIP archives. | MEDIUM |
| toml | 1.1.2 | Configuration files | If application settings are stored as TOML (alternative to SQLite settings table). | MEDIUM |

### GLib/GTK Build Infrastructure

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| glib | 0.22.6 | GLib core bindings | `MainContext::channel()` for tokio → GLib main loop bridge. `glib::spawn_future_local()` for async tasks on the main loop. | HIGH |
| gio | 0.22.6 | GIO bindings | File monitoring, async I/O, application registration. | HIGH |
| glib-build-tools | 0.22.0 | Build-time GResource compilation | Compile icon/resource bundles into the binary via `build.rs`. | HIGH |

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| UI Framework | GTK4/libadwaita + Relm4 | Tauri | Constraint: "No IPC bridge" — Tauri requires Rust backend + WebView frontend communicating via IPC |
| UI Framework | GTK4/libadwaita + Relm4 | Iced | Iced is pure-Rust but lacks libadwaita, GNOME integration, and Adwaita widgets. No native adaptive layout. Smaller ecosystem. |
| UI Framework | GTK4/libadwaita + Relm4 | Slint | Slint is declarative but not GTK-native. Renders its own UI. No Adwaita. Not GNOME Circle eligible. |
| UI Framework | GTK4/libadwaita + Relm4 | Dioxus (desktop) | WebView-based (uses wry/tao). Same IPC problem as Tauri. |
| AI Abstraction | rig-core | Raw reqwest per-provider | Reimplementing provider clients, streaming parsers, error handling for 19+ providers = months of work. rig gives unified API for free. |
| AI Abstraction | rig-core | llm-chain | llm-chain is less mature, fewer providers, no streaming support as mature as rig's. Lower activity. |
| Database | rusqlite (bundled) | diesel + SQLite | Diesel is an ORM — overkill for a local-first app with simple schema. Rusqlite gives raw SQL control for FTS5, window functions, custom functions. |
| Database | rusqlite (bundled) | sqlx + SQLite | sqlx is async-native but adds significant compile time. Rusqlite + background threads via Relm4 commands achieves the same non-blocking behavior with simpler build. |
| Database | rusqlite (bundled) | sled | sled is pure-Rust but lacks FTS5, is alpha-quality, and has had data-loss bugs. SQLite is battle-tested. |
| Secrets | keyring (3.6.3) | keyring (4.0.0) | 4.0.0 released 2 days ago with empty feature flags — unstable, API likely changed. Wait for stabilization. |
| Secrets | keyring (3.6.3) | Manual keychain FFI | Reinventing platform-specific keychain access is error-prone and a security risk. keyring handles it correctly. |
| Markdown | pulldown-cmark | comrak | comrak is C-dependent (libcmark-gfm). pulldown-cmark is pure Rust, actively maintained, pull-parser design fits streaming. |
| Tokenizer | tiktoken-rs | tokenizers (HuggingFace) | tokenizers crate is heavier (full BPE/WordPiece/BPE SentencePiece). tiktoken-rs matches the OpenAI tokenizer most roleplay APIs expect. |
| Encryption | aes-gcm (0.10.3) | ring | ring is a general crypto library — overkill for just AES-256-GCM. aes-gcm from RustCrypto is focused, auditable, no ASM dependency. |
| Key Derivation | argon2 (0.5.3) | scrypt | Argon2id is the PHC winner and recommended by OWASP. scrypt is older. Argon2id provides better resistance to GPU/ASIC attacks. |

## Architecture Integration Points

### tokio ↔ GLib Bridge

This is the most critical integration point. Relm4 0.11 runs its own tokio runtime internally for commands. The bridge pattern:

```
rig (tokio async) → glib::MainContext::channel() → Relm4 update()
```

**Two patterns, one choice:**

1. **Relm4 Commands (recommended):** `sender.oneshot_command(async { ... })` — Relm4 manages the tokio runtime and automatically marshals the `CommandOutput` back to the GLib main loop. Zero manual channel management.

2. **Manual glib channels:** `glib::MainContext::channel()` — for cases where you need to send incremental updates (streaming tokens) rather than a single result. Create a `glib::Sender<String>`, pass it to the async task, call `send(token)` from the tokio context, and receive via `gtk::idle_add()` or Relm4's input mechanism.

**For streaming specifically:** Use the manual glib channel approach for token-by-token updates. The streaming loop runs on tokio (via `StreamExt::next()`), and each token chunk is sent through a `glib::MainContext::channel()` to update the UI on the GLib main loop. This avoids blocking the UI while providing smooth typewriter rendering.

### rig Streaming Integration

```rust
// Pseudocode for the streaming bridge
let (sender, receiver) = glib::MainContext::channel::<StreamChunk>(glib::Priority::DEFAULT);

// On tokio runtime (via Relm4 command or manual spawn):
let mut stream = agent.stream_prompt(message).await?;
while let Some(chunk) = stream.next().await {
    match chunk? {
        MultiTurnStreamItem::StreamAssistantItem(
            StreamedAssistantContent::Text(text)
        ) => {
            sender.send(StreamChunk::Token(text.text))?;
        }
        MultiTurnStreamItem::FinalResponse(resp) => {
            sender.send(StreamChunk::Complete(resp))?;
        }
        _ => {}
    }
}

// On GLib main loop:
receiver.attach(None, move |chunk| {
    // Update UI model and widgets
    glib::ControlFlow::Continue
});
```

### SQLite Background Operations

Use Relm4's `spawn_oneshot_command()` for synchronous rusqlite operations (since rusqlite is sync). The operation runs on a thread pool, and the result is delivered back as a `CommandOutput`.

```rust
// In update():
sender.spawn_oneshot_command(|| {
    let conn = Connection::open(path)?;
    let results = conn.query_row(...)?;
    CommandMsg::QueryResult(results)
});

// In update_cmd():
fn update_cmd(&mut self, msg: CommandMsg, ...) {
    match msg {
        CommandMsg::QueryResult(data) => self.data = Some(data),
    }
}
```

## Installation

```bash
# Core UI
cargo add gtk4 --features v4_14
cargo add libadwaita --features v1_5
cargo add relm4 --features libadwaita,macros
cargo add relm4-icons
cargo add pango
cargo add sourceview5  # Optional: defer if not needed immediately

# AI inference
cargo add rig-core --features derive
cargo add tokio --features macros,rt-multi-thread
cargo add futures

# Database
cargo add rusqlite --features bundled,window,functions
cargo add rusqlite_migration

# Security
cargo add keyring --features apple-native  # per-platform
cargo add aes-gcm
cargo add argon2

# Data processing
cargo add pulldown-cmark
cargo add tiktoken-rs
cargo add png
cargo add image
cargo add base64
cargo add regex
cargo add serde --features derive
cargo add serde_json

# Infrastructure
cargo add anyhow
cargo add thiserror
cargo add tracing
cargo add tracing-subscriber --features env-filter
cargo add clap --features derive
cargo add dirs
cargo add uuid --features v4
cargo add chrono --features serde
cargo add zip

# Build tools (in [build-dependencies])
cargo add glib-build-tools --build
```

### Cargo.toml Skeleton

```toml
[package]
name = "revelry"
version = "0.1.0"
edition = "2021"

[dependencies]
# UI
gtk4 = { version = "0.11.3", features = ["v4_14"] }
libadwaita = { version = "0.9.1", features = ["v1_5"] }
relm4 = { version = "0.11.0", features = ["libadwaita", "macros"] }
relm4-icons = "0.11.0"
pango = "0.22.6"
sourceview5 = "0.11.0"

# AI
rig-core = { version = "0.36.0", features = ["derive"] }
tokio = { version = "1.52", features = ["macros", "rt-multi-thread"] }
futures = "0.3.32"

# Database
rusqlite = { version = "0.39.0", features = ["bundled", "window", "functions"] }
rusqlite_migration = "2.5.0"

# Security
keyring = { version = "3.6.3" }  # Add per-platform features via target cfg
aes-gcm = "0.10.3"
argon2 = "0.5.3"

# Data
pulldown-cmark = "0.13.3"
tiktoken-rs = "0.11.0"
png = "0.18.1"
image = "0.25.10"
base64 = "0.22.1"
regex = "1.12.3"
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"

# Infrastructure
anyhow = "1.0.102"
thiserror = "2.0.18"
tracing = "0.1.44"
tracing-subscriber = { version = "0.3.23", features = ["env-filter"] }
clap = { version = "4.6.1", features = ["derive"] }
dirs = "6.0.0"
uuid = { version = "1.23.1", features = ["v4"] }
chrono = { version = "0.4.44", features = ["serde"] }
zip = "8.6.0"

[build-dependencies]
glib-build-tools = "0.22.0"

# Platform-specific keyring features
[target.'cfg(target_os = "macos")'.dependencies]
keyring = { version = "3.6.3", features = ["apple-native"] }
[target.'cfg(target_os = "linux")'.dependencies]
keyring = { version = "3.6.3", features = ["linux-native"] }
[target.'cfg(target_os = "windows")'.dependencies]
keyring = { version = "3.6.3", features = ["windows-native"] }
```

## Version Pinning Notes

| Crate | Pin Strategy | Rationale |
|-------|-------------|-----------|
| gtk4 / libadwaita / relm4 | Exact match to 0.11.x / 0.9.x | gtk-rs ecosystem crates must share the same major.minor version. Relm4 0.11 requires gtk4 0.11.x. |
| rig-core | Compatible with 0.36.x | Rig is evolving rapidly (0.36.0 published 2026-04-28). Minor versions may add providers. Pin to 0.36 for stability during initial dev. |
| rusqlite | Compatible with 0.39.x | Schema compatibility matters. Minor rusqlite updates rarely break. |
| keyring | Exact = 3.6.3 | 4.0.0 is a breaking release too new to trust. Pin to 3.x. |
| aes-gcm / argon2 | Compatible | RustCrypto crates are mature and stable. Pre-release versions exist but are clearly marked. |

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

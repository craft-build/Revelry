# Technology Stack

**Domain:** Tauri desktop AI roleplay app with SvelteKit frontend

## Recommended Stack

### Core Framework

| Technology | Version | Confidence | Rationale |
|------------|---------|------------|-----------|
| **Tauri** | v2.x (latest stable) | HIGH | Desktop shell for the existing SvelteKit SPA. V2 has revamped IPC with custom protocols (faster than v1 string serialization), built-in channel API for streaming, and cross-platform support (macOS, Linux, Windows). Verified via Context7 and official docs. |
| **SvelteKit** | ^2.59 | HIGH | Already in use. Switch from `adapter-auto` to `@sveltejs/adapter-static` with `fallback: 'index.html'` for Tauri SPA compatibility. No SSR in Tauri — static adapter serves the SPA from the webview. |
| **Svelte** | ^5.55 (runes) | HIGH | Already in use. `$state`, `$derived`, `$effect` runes map directly from the Relm4 AppModel pattern. No change needed. |

### Database

| Technology | Version | Confidence | Rationale |
|------------|---------|------------|-----------|
| **rusqlite** | ^0.39 | HIGH | Bundled SQLite via `features = ["bundled"]` — zero-config cross-platform builds. Already proven in `gtk_ref/`. Use WAL mode for concurrent read/write. Background-thread worker pattern from `gtk_ref/` maps to a tokio task in Tauri. |

### AI Provider Layer

| Technology | Version | Confidence | Rationale |
|------------|---------|------------|-----------|
| **rig-core** | ^0.36 | HIGH | Provider-agnostic AI abstraction. Supports 18 providers natively: Anthropic, OpenAI, Gemini, Cohere, DeepSeek, Azure, Groq, Mistral, Ollama, OpenRouter, Together, xAI, HuggingFace, Moonshot, Perplexity, Galadriel, Hyperbolic, Mira. Custom OpenAI-compatible endpoints via the `openai` client with custom base URL. Streaming via `stream_chat` / `stream_completion`. Already proven in `gtk_ref/`. |
| **tokio** | ^1.52 | HIGH | Async runtime for rig-core AI calls and background DB operations. Tauri v2 uses tokio internally — integrate naturally. |

### Security

| Technology | Version | Confidence | Rationale |
|------------|---------|------------|-----------|
| **keyring** | ^3.6 | HIGH | OS-native credential storage: macOS Keychain, GNOME Keyring (via Secret Service), Windows Credential Manager. Already proven in `gtk_ref/`. Caveat: Linux requires a running keyring daemon (gnome-keyring or kwallet); fails gracefully if absent. |

### Text Processing

| Technology | Version | Confidence | Rationale |
|------------|---------|------------|-----------|
| **tiktoken-rs** | ^0.11 | MEDIUM | BPE tokenization for context window budget tracking. Matches OpenAI's tiktoken. Already proven in `gtk_ref/`. MEDIUM confidence on exact version — verify current crate version before use. |
| **pulldown-cmark** | ^0.13 | HIGH | Markdown→events parser for AI response rendering. Already proven in `gtk_ref/`. Produces events that can be rendered to HTML or custom output. |

### Utilities

| Technology | Version | Confidence | Rationale |
|------------|---------|------------|-----------|
| **anyhow** | ^1.0 + **thiserror** | ^2.0 | HIGH | Error handling. `anyhow` for application errors, `thiserror` for domain error types. Already proven in `gtk_ref/`. |
| **tracing** | ^0.1 + **tracing-subscriber** | ^0.3 | HIGH | Structured logging in Rust backend. Already proven in `gtk_ref/`. |
| **serde** | ^1.0 + **serde_json** | ^1.0 | HIGH | Serialization for Tauri IPC (all command params/returns must implement Serialize/Deserialize). |
| **uuid** | ^1.0 | HIGH | ID generation for entities. |
| **chrono** | ^0.4 | HIGH | Timestamp handling for conversations and messages. |
| **dirs** | ^6.0 | HIGH | Cross-platform data/config directories (XDG on Linux, ~/Library on macOS, %APPDATA on Windows). Already proven in `gtk_ref/`. |

### Frontend (Tauri-specific additions)

| Technology | Version | Confidence | Rationale |
|------------|---------|------------|-----------|
| **@tauri-apps/api** | ^2.x | HIGH | TypeScript API for Tauri IPC — `invoke()` for commands, `Channel` for streaming, `listen()` for events. Verified via Context7. |
| **@sveltejs/adapter-static** | ^3.x | HIGH | Required for Tauri — serves the SPA from the webview. Switch from `adapter-auto`. |

### NOT Recommended

| Technology | Why Not |
|------------|---------|
| **tauri-plugin-sql** | Adds a plugin abstraction over SQLite but loses fine-grained control over migrations, WAL mode, background-thread patterns, and connection management. `rusqlite` directly gives more control and matches the `gtk_ref/` architecture. |
| **tauri-plugin-store** | Key-value store, not suitable for relational data with conversations, messages, characters, and personas. Use SQLite. |
| **tauri-plugin-keyring** | No official Tauri keyring plugin exists with feature parity to the `keyring` crate. Use `keyring` directly. |
| **SQLite via Tauri HTTP** | Unnecessary network overhead. Direct rusqlite with bundled feature is simpler and faster. |
| **diesel** | ORM overhead for a schema that's already well-defined in `gtk_ref/`. rusqlite gives direct SQL control. |
| **sqlx** | Async SQLite adds complexity without benefit for a local desktop app. rusqlite with a background thread is simpler and proven. |

## Installation

### Core (Cargo.toml for src-tauri/)

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-shell = "2"
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
rig-core = "0.36"
rusqlite = { version = "0.39", features = ["bundled"] }
keyring = "3.6"
tiktoken-rs = "0.11"
pulldown-cmark = "0.13"
anyhow = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = "0.3"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
dirs = "6"
```

### Frontend additions (package.json)

```json
{
  "devDependencies": {
    "@tauri-apps/cli": "^2",
    "@sveltejs/adapter-static": "^3"
  },
  "dependencies": {
    "@tauri-apps/api": "^2"
  }
}
```

## Sources

- Tauri v2 official docs (Context7: /websites/v2_tauri_app) — SvelteKit integration, commands, channels, state, permissions
- Tauri v2 API docs (Context7: /websites/rs_tauri_2_9_5) — Channel API, event emit/listen, streaming patterns
- rig-core API docs (Context7: /websites/rs_rig-core) — Provider support, streaming, agent API
- Existing codebase: `.planning/codebase/STACK.md`, `gtk_ref/Cargo.toml`

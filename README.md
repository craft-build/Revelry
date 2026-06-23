# Revelry

AI roleplay desktop application — immersive, private, provider-agnostic.

Built with **Tauri v2** (Rust backend) and **SvelteKit 2 + Svelte 5** (frontend). All data stays local in SQLite. API keys live in your OS keychain, never in the database or code. Supports 15 LLM providers out of the box.

## Features

- **Multi-provider AI streaming** — OpenAI, Anthropic, Ollama, Gemini, Cohere, Mistral, Groq, DeepSeek, OpenRouter, xAI, Perplexity, Together, Azure OpenAI, HuggingFace, and any OpenAI-compatible endpoint
- **Character cards** — full CharacterCardV2 spec compatibility; import/export PNG and JSON
- **AI character generation** — describe an idea, pick a model, get a complete character card
- **Chat branching** — fork conversations at any message, navigate between branches
- **Message editing** — edit-and-resend, regenerate, delete
- **Message search** — full-text search across all conversations
- **Lorebooks** — keyword-activated context injection per character or per conversation
- **Regex scripts** — post-processing rules applied to AI output in real time
- **User personas** — switchable identity profiles with tiered template compression
- **Prompt engine** — budget-aware assembly with persona compression, lorebook injection, and oldest-first truncation
- **Generation presets** — save and reuse temperature, top-p, frequency/presence penalty, stop sequences
- **Token counting** — accurate tiktoken-based counts with context bar in chat input
- **Markdown rendering** — full GFM with syntax highlighting (Shiki)
- **Cross-platform** — macOS (DMG), Linux (AppImage), Windows (NSIS)

## Tech Stack

| Layer | Technology |
| --- | --- |
| Frontend framework | SvelteKit 2 + Svelte 5 (runes) |
| Frontend language | TypeScript (strict) |
| Styling | Tailwind CSS v4 (OKLCH tokens) |
| UI primitives | bits-ui 2 |
| Icons | Lucide |
| Build/bundler | Vite 8 |
| Package manager | Bun |
| Desktop runtime | Tauri v2 |
| Backend language | Rust (2021 edition) |
| Database | SQLite (rusqlite, bundled) |
| Migrations | rusqlite-migration |
| AI client | rig-core 0.37 |
| Tokenization | tiktoken-rs 0.11 |
| Async runtime | tokio (multi-thread) |
| Error handling | thiserror 2 |
| API key storage | OS keychain (keyring v4) |

## Getting Started

### Prerequisites

- [Bun](https://bun.sh/) ≥ 1
- [Rust](https://rustup.rs/) stable toolchain
- [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform

### Install dependencies

```sh
bun install
```

### Development

**Frontend only** (no Rust backend — uses mock data):

```sh
bun dev
```

**Full app** (Rust + frontend together):

```sh
bun tauri dev
```

### Production build

```sh
bun tauri build
```

Cross-compile for Windows from other platforms:

```sh
bun run win:build
```

## Commands

| Command | Description |
| --- | --- |
| `bun dev` | Start frontend dev server (mock backend) |
| `bun run build` | Production frontend build |
| `bun run check` | Type-check with svelte-check |
| `bun run format` | Format all files with Prettier |
| `bun run lint` | Check formatting |
| `bun tauri dev` | Start Tauri dev mode (full stack) |
| `bun tauri build` | Production desktop build |
| `cargo check` | Type-check Rust (from `src-tauri/`) |
| `cargo test` | Run Rust tests (from `src-tauri/`) |
| `cargo clippy` | Lint Rust code (from `src-tauri/`) |

## Architecture

```
revelry/
├── src/                      # SvelteKit frontend
│   ├── lib/
│   │   ├── components/       # UI components (chat, characters, settings, etc.)
│   │   ├── stores/           # AppState — single reactive root (Svelte 5 runes)
│   │   ├── types/            # TypeScript type definitions
│   │   ├── data/             # Tauri invoke wrappers + dev-mode mocks
│   │   └── utils/            # cn(), formatRelativeTime(), etc.
│   └── routes/               # SvelteKit routes (SPA, SSR disabled)
├── src-tauri/                # Rust backend
│   └── src/
│       ├── commands/         # Tauri IPC command handlers
│       ├── domain/           # Domain types (character, conversation, provider, etc.)
│       ├── ai/               # AiService — LLM streaming via rig-core
│       ├── service/          # PromptEngine, TokenCountService, regex, lorebook matching
│       ├── db/               # DbWorker async actor + SQLite migrations
│       └── secrets/          # KeyringStore — OS keychain for API keys
└── docs/                     # Design specification v2.0
```

### Frontend ↔ Backend communication

All communication goes through **Tauri IPC** — the frontend never accesses the database, filesystem, or LLM APIs directly.

- **Request/response**: `invoke("command_name", { args })` for CRUD and settings
- **Streaming**: `invoke("command_name", { ..., channel })` with `Channel<T>` for AI responses

The API layer (`src/lib/data/api.ts`) provides typed wrappers for every command. In dev mode (`bun dev` without Tauri), mock implementations allow UI development without the Rust backend.

### Key backend patterns

- **DbWorker actor** — all SQLite operations run on a background tokio task via `mpsc` channels with oneshot replies, keeping the Tauri async runtime non-blocking
- **Cancellation** — chat and character generation use `CancellationToken`; starting a new stream cancels any active one
- **API key isolation** — keys are stored exclusively in the OS keychain via `KeyringStore`, never in the database or sent to the frontend (providers expose only `has_api_key: boolean`)

## Data Storage

| Data | Location |
| --- | --- |
| Conversations, characters, personas, lorebooks, presets, providers, regex scripts, settings | SQLite database at `$DATA_DIR/com.revelry.desktop/revelry.db` |
| API keys | OS keychain (Keychain Access on macOS, GNOME Keyring on Linux, Credential Manager on Windows) |
| Avatar images | `$APPDATA/avatars/` |

## Design Documentation

The full design specification v2.0 lives in `docs/`:

- `01-executive-summary.md` — product overview
- `05-technology-stack.md` — detailed tech decisions
- `06-system-architecture.md` — system design
- `07-character-card-system.md` — character card spec
- `08-ai-provider-integration.md` — provider integration details
- `11-feature-specifications.md` — feature specs
- `12-data-model-and-storage.md` — data model and schema
- `14-development-roadmap.md` — roadmap
- `15-rig-provider-matrix.md` — provider support matrix

## Status

Revelry is in early development (v0.1.0). Core chat, character management, lorebooks, regex scripts, branching, search, and multi-provider streaming are functional. Markdown rendering, theme switching, and AI character generation are wired. Light theme, vault mode, and some polish items are not yet implemented.

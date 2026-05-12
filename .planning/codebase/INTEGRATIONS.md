# External Integrations

**Analysis Date:** 2026-05-10

## APIs & External Services

> **Note:** The active codebase is a SvelteKit SPA in `src/` using mock data — it does **not** currently integrate with any AI providers. The integrations below are **architectural patterns** from the archival reference implementation (`gtk_ref/`) that will be reimplemented in the future Tauri Rust backend.

**AI Providers — Planned (reference pattern in `gtk_ref/`) — via rig-core ^0.36.0:**

- OpenAI — Cloud LLM inference (GPT-4o, GPT-4o-mini)
  - SDK/Client: `rig::providers::openai::Client` (`gtk_ref/src/service/provider_test.rs`)
  - Auth: OS keychain (key `provider_id` via `KeyringStore` in `gtk_ref/src/secrets/mod.rs`)
- Anthropic — Cloud LLM inference (Claude Sonnet, Claude Opus)
  - SDK/Client: `rig::providers::anthropic::Client` (`gtk_ref/src/service/provider_test.rs`)
  - Auth: OS keychain
- Ollama — Local LLM inference (Llama 3.2, etc.)
  - SDK/Client: `rig::providers::ollama::Client` (`gtk_ref/src/service/provider_test.rs`)
  - Auth: None required (local, default `http://localhost:11434`)
- Gemini — Cloud LLM inference (Gemini 2.0 Flash)
  - SDK/Client: `rig::providers::gemini::Client` (`gtk_ref/src/service/provider_test.rs`)
  - Auth: OS keychain
- Cohere — Cloud LLM inference (Command-R-Plus)
  - SDK/Client: `rig::providers::cohere::Client` (`gtk_ref/src/service/provider_test.rs`)
  - Auth: OS keychain
- Mistral — Cloud LLM inference (Mistral Large)
  - SDK/Client: `rig::providers::mistral::Client` (`gtk_ref/src/service/provider_test.rs`)
  - Auth: OS keychain
- Groq — Cloud LLM inference (Llama 3.3 70B)
  - SDK/Client: `rig::providers::groq::Client` (`gtk_ref/src/service/provider_test.rs`)
  - Auth: OS keychain
- DeepSeek — Cloud LLM inference (DeepSeek Chat)
  - SDK/Client: `rig::providers::deepseek::Client` (`gtk_ref/src/service/provider_test.rs`)
  - Auth: OS keychain
- OpenRouter — Cloud LLM gateway (multi-provider routing)
  - SDK/Client: `rig::providers::openrouter::Client` (`gtk_ref/src/service/provider_test.rs`)
  - Auth: OS keychain
- xAI — Cloud LLM inference (Grok 3 Mini)
  - SDK/Client: `rig::providers::xai::Client` (`gtk_ref/src/service/provider_test.rs`)
  - Auth: OS keychain
- Perplexity — Cloud LLM inference (Sonar Pro)
  - SDK/Client: `rig::providers::perplexity::Client` (`gtk_ref/src/service/provider_test.rs`)
  - Auth: OS keychain
- Together AI — Cloud LLM inference (Llama 3.1 70B Turbo)
  - SDK/Client: `rig::providers::together::Client` (`gtk_ref/src/service/provider_test.rs`)
  - Auth: OS keychain
- Azure OpenAI — Cloud LLM inference (enterprise OpenAI deployment)
  - SDK/Client: `rig::providers::openai::Client` with custom base_url (`gtk_ref/src/service/provider_test.rs`)
  - Auth: OS keychain + custom deployment URL
- HuggingFace — Cloud LLM inference (Llama 3.1 8B Instruct)
  - SDK/Client: `rig::providers::huggingface::Client` (`gtk_ref/src/service/provider_test.rs`)
  - Auth: OS keychain
- Custom OpenAI-Compatible — Any provider exposing OpenAI-compatible API
  - SDK/Client: `rig::providers::openai::Client` with custom base_url (`gtk_ref/src/service/provider_test.rs`)
  - Auth: OS keychain (optional)

**Fonts:**

- Google Fonts — Geist, Fraunces, Geist Mono typefaces loaded via `<link>` in `src/app.html`
  - SDK/Client: External stylesheet (`https://fonts.googleapis.com/css2?...`)
  - Auth: None

**AI Streaming — Planned (reference pattern in `gtk_ref/`):**

- Streaming chat via rig-core `StreamingChat` trait — token-by-token AI responses with cancellation
  - SDK/Client: `rig::streaming::StreamingChat` (`gtk_ref/src/service/ai.rs`)
  - Auth: Via provider-specific keychain entries (reference pattern — not yet active)

## Data Storage

> **Note:** The active SvelteKit app currently uses **in-memory stores with mock data** (`src/lib/stores/`). No database is integrated yet. SQLite below is a reference pattern from `gtk_ref/` that will be reimplemented in the Tauri backend.

**Databases — Planned (reference pattern in `gtk_ref/`):**

- SQLite (local, via rusqlite ^0.39.0 with `bundled` feature)
  - Connection: Local file in platform data directory (`dirs` crate: XDG on Linux, ~/Library on macOS, %APPDATA% on Windows)
  - Client: `rusqlite` with `rusqlite_migration` ^2.5.0 for schema versioning (`gtk_ref/src/db/migrations.rs`)
  - Tables: providers, app_settings, characters, personas, conversations, messages, lorebook, generation presets, regex_scripts (per `gtk_ref/src/db/`)
  - WAL mode enabled (`PRAGMA journal_mode=WAL` in migration 1)
  - FTS5 full-text search planned (per design spec)

**File Storage — Planned (reference pattern in `gtk_ref/`):**

- Local filesystem only — character card avatars, PNG card imports/exports with embedded metadata (`gtk_ref/src/service/card_io.rs`)
- ZIP archives for batch import/export and backup/restore (`gtk_ref/Cargo.toml` — `zip` ^8.6.0)

**Caching:**

- None — Local-first architecture; all data will be persistent in SQLite (currently mock data in SvelteKit stores)

## Authentication & Identity

> **Note:** The active SvelteKit app does **not** currently handle API keys. Keyring/keychain integration below is a reference pattern from `gtk_ref/` that will be reimplemented in the Tauri backend.

**Auth Provider — Planned (reference pattern in `gtk_ref/`):**

- Custom — No user accounts, no cloud auth. The application is local-first with zero account creation.
  - Implementation: API keys for AI providers stored in OS-native keychain via `keyring` crate (`gtk_ref/src/secrets/mod.rs`). The `KeyringStore` uses service name `"revelry"` with provider ID as the username. Keys are never stored in the database (security invariant PROV-05, D-16).
  - Vault mode encryption: AES-256-GCM (`aes-gcm` ^0.10.3) with Argon2id key derivation (`argon2` ^0.5.3) for encrypting local data at rest.

## Monitoring & Observability

> **Note:** `tracing` is a reference pattern from the archival Rust backend (`gtk_ref/`), not currently active in the SvelteKit app. The SvelteKit app uses browser developer tools for logging.

**Error Tracking:**

- None — Local desktop application, no remote error tracking

**Logs — Planned (reference pattern in `gtk_ref/`):**

- `tracing` ^0.1.44 + `tracing-subscriber` ^0.3.23 — Structured logging in Rust backend with env-filter (`gtk_ref/Cargo.toml`)
- Error redaction: API keys redacted from error messages to prevent credential leakage (`gtk_ref/src/service/provider_test.rs` — `redact_error()` function)

## CI/CD & Deployment

**Hosting:**

- Desktop application — future target is **Tauri** desktop app (Rust backend + SvelteKit SPA frontend), not the archival GTK4/libadwaita native binary from `gtk_ref/`
- Reference: Tauri embedding for SvelteKit SPA (`README.md` — `@tauri-apps/cli`, `frontendDist` pointing to `build/`)
- Alternative: Standalone web app against any HTTP backend exposing types from `src/lib/types/index.ts`

**CI Pipeline:**

- None — No CI/CD configuration found in the repository

## Environment Configuration

**Currently required (SvelteKit SPA):**

- No required environment variables — the app runs with mock data and no backend
- `.env` file exists but is gitignored (existence noted, contents not read per security rules)

**Will be required when Tauri backend exists (reference pattern from `gtk_ref/`):**

- No environment variables for API keys (keys managed via OS keychain, not env vars)
- OS-native keychain — macOS Keychain, GNOME Keyring (Secret Service), Windows Credential Manager
  - Managed via `keyring` crate ^3.6.3 (`gtk_ref/src/secrets/mod.rs`)
  - Service namespace: `"revelry"`, key entries keyed by provider ID
  - **Never** stored in database, config files, environment variables, or logs

## Webhooks & Callbacks

**Incoming:**

- None — Local desktop application with no inbound webhook endpoints

**Outgoing:**

- None — AI provider requests are standard request/response and streaming HTTP calls, not webhooks

---

_Integration audit: 2026-05-10_

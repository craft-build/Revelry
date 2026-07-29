---
type: Note
description: Domain terms, project jargon, and acronyms used across Revelry's Rust and TypeScript codebase.
tags:
- glossary
- domain-terms
- jargon
- acronyms
timestamp: 2026-07-12T18:26:11Z
---
# Glossary

Domain terms and project-specific jargon for Revelry. Each entry notes where the term is defined.

## Core Domain Types

- **CharacterCard (CharacterCardV2)** — canonical V2 character card spec with personality, scenario, first message, etc. Includes V1/V2 spec deserialization. (`src-tauri/src/domain/character.rs`, `src/lib/types/index.ts`)
- **CharaCardV2Spec / CharaCardV2SpecData** — PNG-embedded character card wrapper spec used for import/export. (`src-tauri/src/domain/character.rs`)
- **CharacterV1** — legacy V1 character card importer format. (`src-tauri/src/domain/character.rs`)
- **Persona (UserPersona)** — the user's role/identity in a roleplay; one can be marked `is_default`. (`src-tauri/src/domain/persona.rs`)
- **Lorebook** — a collection of world-info entries attached to a character or conversation. (`src-tauri/src/domain/lorebook.rs`)
- **LorebookEntry** — a single lorebook entry with keyword triggers, position, activation rules, and recursive depth limits. (`src-tauri/src/domain/lorebook.rs`)
- **LorebookPosition** — where an entry injects: `before_char` | `after_char` | `at_depth`. (`src-tauri/src/domain/lorebook.rs`)
- **LorebookActivation** — when an entry applies: `always` | `character_only` | `user_only`. (`src-tauri/src/domain/lorebook.rs`)
- **LorebookActivationDetail** — per-entry activation info returned with a chat completion. (`src-tauri/src/domain/lorebook.rs`)
- **Provider (ProviderConfig)** — LLM provider configuration (type, base URL, model, generation params). (`src-tauri/src/domain/provider.rs`)
- **ProviderType** — enum of 15 LLM provider types (`openai`, `anthropic`, `ollama`, ...). (`src-tauri/src/domain/provider.rs`)
- **GenerationParams** — runtime generation settings: temperature, top_p, max_tokens, penalties, stop sequences. (`src-tauri/src/domain/provider.rs`, `src-tauri/src/ai/mod.rs`)
- **GenerationPreset** — saved generation parameter preset; `Creative`, `Balanced`, `Precise` are built-in immutable presets. (`src-tauri/src/domain/generation.rs`)
- **RegexScript** — per-character find-and-replace script with regex pattern, flags (`gi`), and sort order; supports SillyTavern extensions JSON import. (`src-tauri/src/domain/regex_script.rs`)
- **Conversation** — conversation record linked to a character and optional persona, with preset and token overrides. (`src-tauri/src/domain/conversation.rs`)
- **Message** — chat message with role, content, token count, metadata. (`src-tauri/src/domain/conversation.rs`)
- **MessageRole** — `'user' | 'assistant' | 'system'`. (`src-tauri/src/domain/conversation.rs`, `src/lib/types/index.ts`)
- **MessageBranch** — divergent conversation path for forking/regeneration. (`src-tauri/src/domain/conversation.rs`)

## Streaming Types

- **StreamChunk** — streaming chunk for chat: `Token` | `Complete` | `Error` (tagged enum, `event`/`data`, camelCase). (`src-tauri/src/ai/mod.rs`)
- **GenerateCharacterChunk** — streaming chunk for character generation (same shape). (`src-tauri/src/ai/mod.rs`)
- **GeneratedCharacterData** — partial character data returned after AI generation completes (creative fields only). (`src-tauri/src/ai/mod.rs`, `src/lib/types/index.ts`)
- **ContextBudget** — token usage returned after generation: `tokensUsed`, `tokensLimit`, `truncatedCount`, `lorebookDetails[]`. Mirrors `StreamChunk::Complete`. (`src/lib/types/index.ts`)

## Backend Components

- **DbWorker** — tokio mpsc actor that owns the SQLite connection; all DB ops send `DbCommand` variants and await `oneshot` replies to keep rusqlite off the async runtime. (`src-tauri/src/db/mod.rs`)
- **DbCommand** — enum (50+ variants) of database operations dispatched to `DbWorker`. (`src-tauri/src/db/mod.rs`)
- **KeyringStore** — OS keychain interface for API keys (service name `"revelry"`); keys never stored in the DB. (`src-tauri/src/secrets/mod.rs`)
- **PromptEngine** — assembles system prompts from persona, lorebook context, and character data with budget-aware truncation. (`src-tauri/src/service/prompt.rs`)
- **TokenCountService** — tiktoken-rs service for token counting and context window resolution. (`src-tauri/src/service/token_count.rs`)
- **AiService** — rig-core wrapper handling LLM streaming (`stream_chat`, `stream_generate_character`). (`src-tauri/src/ai/mod.rs`)
- **apply_regex_scripts** — service that applies a character's regex scripts to chat output in `sort_order` (pipe chain). (`src-tauri/src/service/regex.rs`)
- **CancelState / GenerateCancelState** — `Mutex<Option<CancellationToken>>` managing cancellation for chat and generation streams; starting a new stream cancels the existing one. (`src-tauri/src/commands/chat.rs`, `src-tauri/src/commands/generate.rs`)
- **AppError** — thiserror enum for all command errors; serializes to a display string for the frontend. (`src-tauri/src/error.rs`)

## Frontend Components

- **AppState** — single Svelte 5 rune-based class instance (`app`) holding all reactive frontend state and actions. (`src/lib/stores/app.svelte.ts`)
- **IS_DEV** — guard (`!("__TAURI_INTERNALS__" in window)`) routing `api.ts` calls to mock implementations when running without the Tauri backend. (`src/lib/data/api.ts`)
- **PrimaryView** — `'chat' | 'characters' | 'settings'`. (`src/lib/types/index.ts`)
- **CharactersTab** — `'characters' | 'personas'`. (`src/lib/types/index.ts`)

## Acronyms

- **IPC** — Inter-Process Communication; Tauri's `invoke()` and `Channel<T>` APIs bridging frontend and Rust.
- **SSR** — Server-Side Rendering; disabled in this desktop SPA (`ssr=false`, `prerender=false`).
- **FTS5** — SQLite Full-Text Search extension; used for message and conversation-title search. (`src-tauri/src/domain/conversation.rs`)
- **OKLCH** — perceptual color space used for all Tailwind v4 theme tokens. (`src/app.css`)
- **OS keyring** — OS-native credential store (macOS Keychain / GNOME Keyring / Windows Credential Manager) accessed via the `keyring` crate.

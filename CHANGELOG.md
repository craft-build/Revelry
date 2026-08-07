# Changelog

All notable changes to **Revelry** are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1]

### Changed

- Split the backend AI module (previously a single large file) into focused
  submodules for chat, character generation, provider client building, and
  utilities. No behavior change.

### Fixed

- Chat now sends the conversation's selected model to the LLM instead of the
  provider's default model. Previously the per-conversation model override was
  ignored at the streaming boundary, so the wrong model was used on every send,
  regenerate, and edit-and-resend (most visible on Ollama via `ollama ps`).
- macOS release build no longer fails on a missing signing identity. Code
  signing env/secrets were removed from the release workflow; the app now
  builds as an ad-hoc/unsigned bundle.

## [0.2.0]

### Added

- Dismissable avatar lightbox for enlarged character/persona images.

### Changed

- UI redesign around the new logo colorspace.

### Fixed

- Closed several unwired/incomplete gaps across chat, lorebook, and prompt
  assembly.

## [0.1.0] - Initial Release

The first functional build of Revelry — an AI roleplay desktop app built on
Tauri v2, SvelteKit 2 + Svelte 5, and a Rust backend.

### Added

#### App shell & navigation

- Application shell with conversation sidebar and conversation list.
- F9 keyboard shortcut to toggle the sidebar.
- Welcome landing page with suggested characters.

#### Chat

- Active conversation view with AI streaming, auto-scroll, and live token
  counting.
- Message input with auto-growing textarea, token estimate, and context budget
  bar.
- Chat streaming across all 15 supported LLM provider types.
- Message regeneration, edit-and-resend, conversation forking, and branch
  navigation.
- Full-text message search (FTS5) and conversation search.
- Animated thinking indicator during streaming.

#### Characters

- Character editor (create / edit / import) with avatar upload.
- Character import/export (PNG + JSON card formats).
- Lorebook assignment and regex script attachment per character.
- AI-powered character generation with provider selection and streaming preview.
- Search/filter in the character library.

#### Personas

- Persona editor (create / edit) with avatar upload.
- Persona switcher in the chat header.
- Persona template compression (tiered) with budget-aware truncation.

#### Lorebooks

- Lorebook editor (create / edit) with entry management and keyword testing.
- JSON import for CharacterBook V2 format.
- Lorebook keyword matching and budget-aware context injection (before-char,
  after-char, at-depth positions; recursive activation).

#### Providers

- Provider management (CRUD) with API key storage in the OS keychain
  (Keychain / GNOME Keyring / Credential Manager).
- Auto-detection of available models from provider APIs.
- Generation parameter presets (CRUD).

#### Regex scripts

- Regex script CRUD with test/validate and live application to AI output.

#### Prompt engine

- Prompt assembly with persona compression and budget-aware history truncation.

#### Settings & theming

- Light and dark themes with flash-prevention and persistence.
- Syntax highlighting theme selection (Shiki) for code blocks.
- Key-value app settings persistence.

#### Backend

- Layered Rust backend: Tauri commands, domain types, services, async
  `DbWorker` actor (SQLite + rusqlite + migrations), AI service
  (`rig-core`), tokenization (`tiktoken-rs`), and OS keyring secrets.
- Cancellation support for chat and character generation streams.
- Typed `AppError` propagated to the frontend.

### Known limitations

- Markdown rendering is partial (naive asterisk-to-italic only; no full parser).
- Vault mode is not yet implemented.

[Unreleased]: https://github.com/craft-build/Revelry/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/craft-build/Revelry/releases/tag/v0.2.0
[0.1.0]: https://github.com/craft-build/Revelry/releases/tag/v0.1.0

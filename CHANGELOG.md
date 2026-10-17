# Changelog

All notable changes to **Revelry** are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0]

### Added

- Opt-in embeddings: remote provider sources or local fastembed models,
  semantic lorebook activation with per-entry thresholds, rolling story
  summaries with semantic recall of out-of-window messages, and a
  Settings page with connection test and background index rebuilds.
- Hybrid conversation search — FTS5 plus an optional semantic toggle in
  the sidebar.
- Cross-conversation character memory (off by default): characters
  recall moments from other conversations with them, with controls to
  forget/restore a character's cross-chat memories, exclude a single
  chat via "start fresh", toggle memory mid-chat from the chat header,
  and a "Carry over memories?" prompt when starting a new chat.

### Changed

- UI migrated to design v2 "Night fox" — palette rebuilt from the fox
  app icon (night surfaces, fox-blue accent, tail accents), translucent
  header/sidebar, floating composer with persona + model chips and a
  mini context meter, and name-hashed avatar tints.
- Chat bubble backgrounds now derive from the selected code theme so
  markdown prose colors always render with the theme's designed
  contrast (user bubbles keep a subtle accent tint).
- Upgraded rig 0.42 to 0.44, porting the AI layer to the rewritten
  client/streaming API. IPC wire formats unchanged.
- Regenerated all app icons from the squircle logo; system prompt now
  includes formatting instructions.

### Fixed

- OpenAI providers stream through Chat Completions again — rig 0.44's
  completion path silently switched them to the Responses API, which
  rejected the penalty params presets always set and broke chat-only
  proxies and non-Responses models.
- Cross-conversation echoes now inject on a new conversation's first
  turn, reserve recall budget, and appear in the MemoryInfo badge;
  semantic activation previews gate by secondary keys and no longer
  panic on keyless entries.
- Editing a provider no longer clears its API key status — saved
  providers re-report keyring state, so chat kept working after an
  edit-save without waiting for a restart.
- Forgotten-memory indicators update reactively (plain Maps held in
  $state were never deeply proxied), and the generation panel extends
  to the full chat height.

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

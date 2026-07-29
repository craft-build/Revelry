---
type: Note
description: One-paragraph overview of Revelry linking to the tech-stack, architecture, design-decisions, and glossary pages.
tags:
- overview
- project-context
- onboarding
timestamp: 2026-07-12T18:26:30Z
---
# Project Context\n\n**Revelry** is an AI roleplay desktop application built with **Tauri v2** (Rust backend) and **SvelteKit 2 + Svelte 5** (TypeScript frontend). It is a single-page desktop app (SSR disabled) where the Rust backend handles all data persistence (SQLite via rusqlite), API key management (OS keychain), LLM API calls (rig-core, 15 provider types), and file I/O. The frontend communicates with the backend exclusively through typed Tauri IPC — `invoke()` for request/response and `Channel<T>` for AI streaming — and never touches the database, filesystem, or LLM APIs directly.\n\nThe design follows the Elm/Relm4 AppModel pattern: a single reactive `AppState` singleton (`src/lib/stores/app.svelte.ts`) is the source of truth on the frontend, while the backend is layered (commands → service → domain → db) with a `DbWorker` actor keeping synchronous SQLite off the Tauri async runtime. Every feature that involves data, AI, or system resources must be implemented on both sides.\n\n## Where to go next\n\n- [Tech Stack](tech-stack) — languages, frameworks, key dependencies, package layout, entry points.\n- [Architecture](architecture) — module map, frontend↔backend data flow, cross-cutting concerns, subsystem boundaries.\n- [Design Decisions](design-decisions) — Svelte 5 runes, `AppState` singleton, Tauri IPC conventions, `DbWorker`, keyring, OKLCH styling, error handling, testing.\n- [Glossary](glossary) — domain terms, project jargon, and acronyms.\n\nThe primary reference for agents working in this repo is `AGENTS.md` at the project root.\n
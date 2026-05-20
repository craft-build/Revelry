<!-- Revelry Design Specification v2.0 — Section 14 -->
<!-- Last updated: 2026-05-20 -->

# Development Roadmap

## 14.1 Phase 1: Tauri Shell + SvelteKit Scaffold

Phase 1 established the technical foundation. This included setting up the Tauri v2 project structure with SvelteKit frontend, implementing the SQLite database schema and migration system through rusqlite with a `DbWorker` background thread, wiring the `invoke()` IPC bridge between frontend and backend, and creating the basic UI shell with `AppHeader`, `ConversationSidebar`, and view routing. The essential chat functionality stub was created in this phase with mock data.

Deliverables: Functional Tauri application that builds on macOS with a SvelteKit SPA shell, basic layout, and mock data hydration.

## 14.2 Phase 2: Backend Migration + IPC Wiring

Phase 2 migrated the complete Rust backend from the archival `gtk_ref/` codebase to `src-tauri/`. This included domain models, SQLite persistence layer, character/persona CRUD, conversation management, provider configuration, and the full `invoke()` command set. The frontend was wired to real data through Tauri IPC, replacing all mock data with live SQLite queries. DateTime serialization switched to RFC 3339 for IPC portability.

Deliverables: Fully functional application with real data persistence, character/persona management, and provider configuration — no mock data remaining.

## 14.3 Phase 3: Character Editor + Lorebook + Avatars

Phase 3 built the character creation/editing wizard (`CharacterEditorDialog`), persona editor (`PersonaEditorDialog`), lorebook management (`LorebookEditor`), and avatar upload via Tauri's asset protocol. The multi-step wizard pattern with `StepperHeader` was established for complex editing flows.

Deliverables: Full content creation capability — users can create, edit, and delete characters, personas, and lorebooks with avatar support.

## 14.4 Phase 4: Chat Streaming + AI Integration

Phase 4 wired the rig-core AI service for real streaming chat responses. The `ChatContext` loading pattern, `PromptEngine` assembly with lorebook injection, and token-by-token streaming via Tauri `Channel<StreamChunk>` were implemented. Message branching (fork, regenerate, edit-and-resend) was completed.

Deliverables: Real AI conversations with streaming, branching, and regeneration.

## 14.5 Phases 5-9: Polish and Advanced Features

Phases 5-9 implemented remaining features: prompt presets, context bar, markdown rendering (marked + DOMPurify + Shiki), regex scripting with ReDoS protection, and lorebook activation with recursive keyword matching. Each phase built incrementally on the established patterns.

## 14.6 Phase 10: Theme Polish

Phase 10 implemented the dark/light theme toggle (`ThemeToggle` component), ensuring all OKLCH color tokens work in both modes. CSS utility refinements and visual polish were applied.

## 14.7 Phase 11: Spec Update and Distribution (Current)

Phase 11 updates the design specification to reflect the actual SvelteKit/Tauri implementation, and prepares cross-platform build and distribution for macOS, Linux, and Windows.

Deliverables: Updated Markdown documentation, cross-platform builds.

## 14.8 Roadmap Summary

| **Phase**  | **Focus**                   | **Key Deliverables**                                       |
| ---------- | --------------------------- | ---------------------------------------------------------- |
| Phase 1    | Tauri shell + scaffold      | SvelteKit SPA in Tauri window, layout, mock data           |
| Phase 2    | Backend migration + IPC     | Rust backend, SQLite, real data, full command set          |
| Phase 3    | Character editor + lorebook | Multi-step wizards, avatar upload, lorebook management     |
| Phase 4    | Chat streaming + AI         | rig-core streaming, PromptEngine, message branching        |
| Phases 5-9 | Advanced features           | Presets, context bar, markdown, regex, lorebook activation |
| Phase 10   | Theme polish                | Dark/light toggle, OKLCH token refinement                  |
| Phase 11   | Spec update + distribution  | Markdown docs, cross-platform builds                       |

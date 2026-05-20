<!-- Revelry Design Specification v2.0 — Section 6 -->
<!-- Last updated: 2026-05-20 -->

# System Architecture

## 6.1 High-Level Architecture

Revelry's architecture eliminates the IPC bridge layer present in Tauri-based designs. Instead of a React frontend communicating with a Rust backend through typed invoke commands, Revelry uses a unified Rust process where the Relm4 UI layer communicates with the business logic through native Rust function calls and channel-based message passing. The architecture is organized into four major layers that all exist within the same process.

The Presentation Layer consists of Relm4 components that render the GNOME-inspired UI using GTK4/libadwaita widgets and handle user interactions through the message dispatch system. The State Layer (Relm4 Model) manages application state, including active conversations, loaded characters, and UI preferences, all within a single Rust struct that updates atomically through message processing. The Service Layer contains the business logic modules for AI inference, database access, file operations, and encryption, exposed as regular Rust functions and structs. The Data Layer manages SQLite access through rusqlite, rig client configuration, filesystem operations, and OS keychain integration.

When a user sends a message, the flow proceeds as follows: the Relm4 component dispatches a SendMessage message to the model. The model's update function calls the chat-engine module, which constructs the full prompt (combining system prompt, character card, lorebook entries, user persona data, conversation history, and user message) and submits it to the configured rig agent. The rig agent's streaming completion runs on the tokio runtime, emitting token chunks through a glib::MainContext channel back to the UI thread. The model receives each token chunk via a StreamToken message and updates the active message in the model state. Relm4's view! macro reactively updates the GtkLabel showing the AI's response, producing a smooth typewriter effect.

## 6.2 Module Architecture

| **Module** | **Layer** | **Responsibility** |
|---|---|---|
| chat-engine | Service | Prompt assembly, context window management, token counting, message dispatch via rig |
| rig-adapter | Service | rig agent construction, provider configuration, streaming completion coordination |
| card-service | Service | Character card CRUD, import/export (PNG V2, JSON), validation, migration |
| persona-service | Service | User persona CRUD, assignment resolution, prompt block construction |
| db-service | Service | SQLite operations, migrations, FTS5 search, backup/restore |
| crypto-service | Service | AES-256-GCM encryption for optional vault mode, key derivation |
| file-service | Service | Image handling, log management, export bundling, config files |
| AppModel | State (Relm4) | Root application model: conversations, characters, personas, UI state |
| ChatModel | State (Relm4) | Active conversation state, message history, branch pointers, generation status |
| CharacterModel | State (Relm4) | Loaded characters, search filters, selected card, edit state |
| SettingsModel | State (Relm4) | All user preferences, provider configs, UI settings |
| ChatView | Presentation (Relm4) | Conversation display, message input, streaming render, branch navigation |
| CharacterManager | Presentation (Relm4) | Character list, card editor, import wizard, avatar management |
| PersonaManager | Presentation (Relm4) | Persona library, editor, switcher popover, assignment resolution |
| SettingsPanel | Presentation (Relm4) | Provider configuration, prompt presets, theme selection, keyboard shortcuts |

## 6.3 Thread Architecture

Rig's async operations require the tokio runtime, while GTK4 requires the GLib main loop. Revelry bridges these two runtimes using a well-established channel pattern. A single multi-threaded tokio runtime is spawned on a background thread at application startup. When the UI needs to initiate an AI completion, it sends a request through an async-channel to the tokio runtime, which processes it using rig's streaming API. Token chunks flow back through a glib::MainContext channel to the GLib main loop, where the Relm4 model processes them and updates the UI.

Database operations follow a similar pattern: SQLite queries are dispatched to a dedicated database thread that holds the rusqlite connection, and results flow back through glib::MainContext channels. This ensures that neither AI inference nor database operations ever block the GTK main loop, maintaining a responsive UI at all times. The keyring crate's synchronous API calls are also dispatched to background threads to prevent credential lookup from causing UI freezes.

## 6.4 Context Window Management

One of the most critical service responsibilities is context window management. AI models have finite context windows (ranging from 4K tokens for older models to 128K+ for modern ones), and Revelry must ensure that the assembled prompt fits within the selected model's limit while preserving the most relevant conversation context. The chat-engine module implements a multi-strategy context manager that combines character card data, user persona data, lorebook entries, and conversation history into a prioritized token budget.

The context assembly process works in layers. First, the system prompt and character card data (name, personality, scenario, mes_example) are marked as sticky, meaning they always occupy their portion of the context. Next, the user persona block is inserted (semi-sticky, with compression levels if the budget is tight). Then, activated lorebook entries (matched by keyword scanning of the recent conversation and persona fields) are inserted. Finally, conversation history fills the remaining budget, with older messages being truncated first. Token counting uses tiktoken-rs for OpenAI-compatible BPE tokenization, and the rig framework's response metadata provides actual token usage for runtime validation.

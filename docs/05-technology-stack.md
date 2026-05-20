<!-- Revelry Design Specification v2.0 — Section 5 -->
<!-- Last updated: 2026-05-20 -->

# Technology Stack

The technology choices for Revelry are driven by three core requirements: native GNOME desktop integration with authentic HIG compliance, a single-language pure Rust codebase that eliminates IPC bridge complexity, and the rig framework's multi-provider AI inference support. The following stack represents the optimal intersection of these requirements.

## 5.1 Framework: GTK4 + libadwaita + Relm4

The UI is built using GTK4 with libadwaita, providing native GNOME widgets and automatic HIG compliance. Relm4 serves as the declarative UI framework on top of gtk-rs, implementing the Elm Architecture (Model-Update-View) with Rust's type system. The view! macro provides declarative UI definition that maps naturally to GNOME's design system, while the message-passing architecture ensures clean separation between UI logic and business logic. Relm4's AsyncComponent support handles the async operations needed for AI inference without blocking the UI thread.

Unlike a Tauri-based approach that renders UI in a WebView (essentially a mini browser), GTK4 renders native platform widgets. This means native Adwaita theming, proper accessibility support through ATK/AT-SPI, native input method support, and the visual behavior that GNOME users expect. The gtk4 crate (v0.9.x) provides near-complete coverage of the GTK4 C API, and the libadwaita crate (v0.7.x) provides all Adwaita widgets including AdwApplicationWindow, AdwHeaderBar, AdwAvatar, AdwNavigationView, AdwOverlaySplitView, AdwPreferencesWindow, AdwToastOverlay, and AdwClamp.

## 5.2 AI Inference: rig Framework

The rig crate (v0.36.0) provides the AI inference layer, offering native client implementations for over twenty providers through a unified abstraction. This is a significant upgrade from the original Tauri design's approach of manually constructing OpenAI-compatible HTTP requests with reqwest. Rig handles provider-specific authentication, request formatting, SSE streaming, and response parsing for each provider, eliminating the need to maintain custom HTTP adapters.

Rig's agent abstraction combines a completion model with a system prompt (preamble), dynamic context (for RAG), and tool definitions. The builder pattern allows constructing agents declaratively, and the Prompt trait provides both single-shot and streaming completion. The streaming API integrates with futures::Stream for ergonomic token-by-token consumption, which feeds into the GTK4 UI through glib::MainContext channels for smooth typewriter-style rendering. Rig also provides structured output extraction through the Extractor pattern, which uses schemars for type-safe JSON deserialization of AI responses.

## 5.3 State Management: Relm4 Model/Message Architecture

State management in Revelry follows the Elm Architecture through Relm4's Model and Message traits. The application model is a single Rust struct containing all application state, and state changes occur exclusively through message dispatch. This architecture provides deterministic state transitions, easy debugging through message logging, and automatic UI reactivity through Relm4's view! macro. Unlike Zustand in a Tauri application, which requires IPC calls to persist state to the Rust backend, Revelry's model updates happen in-process with direct access to the SQLite database and rig clients.

## 5.4 Database: SQLite via rusqlite

All persistent data is stored in a local SQLite database managed through the rusqlite crate with the bundled feature for zero-configuration deployment. SQLite operations run on background threads using std::thread, communicating results back to the UI thread through glib::MainContext channels. This pattern keeps the GTK main loop responsive even during heavy database operations like full-text search across large conversation histories. The database schema supports the full feature set including branching conversations, lorebooks, user personas, and provider configurations.

## 5.5 Markdown Rendering: pulldown-cmark + Pango

GTK4 does not include a native Markdown widget. Revelry implements Markdown rendering by parsing content with pulldown-cmark (a mature CommonMark parser) and rendering the resulting AST as Pango markup within GtkLabel widgets. Pango supports bold, italic, spans with custom colors, and nested formatting, covering the subset of Markdown most relevant to AI chat responses. For more complex content (tables, code blocks), the renderer falls back to GtkTextView with text tag-based formatting. This approach avoids the overhead of embedding a WebView solely for Markdown rendering while providing readable, well-formatted AI responses.

## 5.6 Technology Summary

| **Layer** | **Technology** | **Version** | **Purpose** |
|---|---|---|---|
| UI Framework | GTK4 + libadwaita | 4.x / 1.x | Native GNOME widgets and HIG compliance |
| Declarative UI | Relm4 | 0.10+ | Elm Architecture for Rust + GTK4 |
| AI Inference | rig-core | 0.36+ | Multi-provider LLM abstraction (20+ providers) |
| Async Runtime | tokio | 1.x | Async runtime for rig and HTTP operations |
| Database | rusqlite (bundled) | 0.31+ | Local persistent SQLite storage |
| Secret Storage | keyring | 3.x | OS keychain access (GNOME Keyring, etc.) |
| Markdown | pulldown-cmark | 0.10+ | CommonMark parsing for AI response rendering |
| HTTP Client | reqwest (via rig) | 0.13+ | Async HTTP with SSE streaming |
| Thread Bridge | glib::MainContext channels | --- | tokio ↔ GTK main loop communication |
| Encryption | ring | 0.17+ | AES-256-GCM for optional vault mode |
| Token Counting | tiktoken-rs | --- | BPE tokenization for context window management |

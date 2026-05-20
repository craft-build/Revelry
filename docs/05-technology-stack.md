<!-- Revelry Design Specification v2.0 — Section 5 -->
<!-- Last updated: 2026-05-20 -->

# Technology Stack

The technology choices for Revelry are driven by three core requirements: a cross-platform native desktop shell with minimal resource consumption, a modern reactive frontend framework for rapid UI iteration, and the rig framework's multi-provider AI inference support. The following stack represents the actual implementation, a Tauri v2 shell embedding a SvelteKit SPA with a Rust backend.

## Frontend Framework: SvelteKit 2 + Svelte 5

The UI is built using SvelteKit 2.x with Svelte 5's runes mode, providing a component-based reactive UI without virtual DOM overhead. Svelte 5 runes (`$state`, `$derived`, `$effect`, `$props`) replace the legacy reactive statements and stores, offering fine-grained reactivity with a clean mental model. The app is built as a single-page application using `@sveltejs/adapter-static` with SSR disabled (`ssr: false`), producing static files served by the Tauri webview.

Vite 8.x handles the frontend build, providing fast hot module replacement during development and optimized production builds. Bun is the package manager, chosen for speed and compatibility with the Node.js ecosystem. TypeScript with `strict: true` provides type safety across the frontend codebase.

## UI Components: bits-ui + Tailwind CSS v4

The component layer uses bits-ui 2.x, a headless UI primitive library for Svelte (the Svelte equivalent of Radix UI/shadcn). Headless primitives provide full keyboard navigation, ARIA attributes, and screen reader support without imposing visual styles, giving complete control over the component aesthetic. Components include Dialog, Select, Tooltip, and other interactive patterns.

Styling is handled by Tailwind CSS v4 with the `@tailwindcss/vite` plugin, using OKLCH color tokens defined in a `@theme { }` block. This provides perceptual uniformity across dark and light themes. Additional styling tools include:

- **tailwind-variants** (`tv()`) for component variant definitions (e.g., `buttonVariants` with variant and size options)
- **clsx + tailwind-merge** combined into a `cn()` utility for conflict-free class merging
- **@lucide/svelte** for the icon library

## State Management: Svelte 5 Runes (AppState)

All application state lives in a single reactive class (`AppState`) in `src/lib/stores/app.svelte.ts`, following an Elm-inspired single source of truth pattern. Svelte 5 runes provide the reactivity:

- `$state()` for reactive state fields (characters, conversations, messages, UI state)
- `get` accessors for derived values (active conversation, filtered characters, default persona)
- Methods for state mutations (openConversation, appendMessage, updateStreamingMessage)

No legacy Svelte stores (`writable`, `readable`, `derived` from `svelte/store`) are used. The state class is a plain TypeScript class compiled with Svelte's rune preprocessor (the `.svelte.ts` file extension enables rune support). Components import the singleton `app` instance and read/write state directly.

## Backend Runtime: Tauri v2 + Rust

The desktop shell is provided by Tauri v2, which embeds the SvelteKit SPA in a native webview window. The Rust backend runs in Tauri's app thread and is exposed to the frontend via `#[tauri::command]` IPC handlers:

- **Synchronous calls:** `invoke('command_name', { args })` for request/response operations
- **Streaming:** `Channel<T>` for token-by-token AI response streaming

The backend is organized into layered modules:

- `commands/` — Tauri IPC handlers (app_settings, avatars, card_io, characters, chat, conversations, keywords, lorebooks, personas, presets, providers, regex)
- `service/` — Business logic (card_io, lorebook, prompt, regex, template, token_count)
- `db/` — SQLite access layer with migrations (app_settings, characters, conversations, generation, lorebook, models, personas, providers, regex_scripts)
- `domain/` — Data types (character, conversation, generation, lorebook, persona, provider, regex_script)
- `ai/` — AI provider integration via rig-core
- `secrets/` — OS keychain access via keyring v4

SQLite persistence uses rusqlite 0.31 with the `bundled` feature for zero-configuration static compilation. A background thread `DbWorker` handles all database operations to avoid blocking the Tauri main thread.

## AI Integration: rig-core 0.37

The rig-core crate (v0.37) provides the AI inference layer, offering native client implementations for over twenty providers through a unified builder pattern API. Each provider client is constructed using a builder pattern with provider-specific configuration. The streaming API integrates with Tauri's `Channel<T>` for token-by-token delivery to the frontend.

Supported providers include: OpenAI, Anthropic, Ollama, Gemini, Cohere, Mistral, DeepSeek, Groq, Azure OpenAI, OpenRouter, Together AI, HuggingFace, Perplexity, xAI (Grok), and custom OpenAI-compatible endpoints (LM Studio, vLLM, LocalAI, etc.).

## Markdown Rendering: marked + DOMPurify + Shiki

Markdown rendering in chat messages uses a web-based pipeline rather than native text rendering:

- **marked v18** — GitHub Flavored Markdown parsing
- **DOMPurify v3** — HTML sanitization to prevent XSS from untrusted content
- **Shiki v4** — Syntax highlighting with the vitesse-dark theme

During streaming, raw text is displayed directly. When the AI response completes, the `MarkdownRenderer` component renders the full markdown with syntax highlighting. This conditional approach avoids rendering overhead during the streaming phase.

## Secrets: keyring v4

API keys are stored in the operating system's native credential store through the keyring crate (v4.0): macOS Keychain Services, Windows Credential Manager, or Linux Secret Service API. The keyring integration uses `use_native_store(true)` for initialization and `release_store()` for clean shutdown. Keys are loaded into memory only when needed and dropped immediately after use.

## Technology Summary

| Layer | Technology | Version | Purpose |
|---|---|---|---|
| Shell | Tauri | 2.x | Native desktop webview |
| Frontend | SvelteKit + Svelte | 2.x / 5.x | UI framework (runes mode) |
| Bundler | Vite | 8.x | Frontend build |
| Styling | Tailwind CSS | 4.x | Utility-first CSS with OKLCH tokens |
| Components | bits-ui | 2.x | Headless UI primitives |
| Icons | @lucide/svelte | 1.x | Icon library |
| Markdown | marked + DOMPurify + Shiki | 18 / 3 / 4 | GFM rendering + syntax highlighting |
| Backend | Rust + Tauri | stable / 2.x | Native backend |
| Database | SQLite (rusqlite bundled) | 0.31 | Local persistence |
| AI | rig-core | 0.37 | Multi-provider LLM integration |
| Secrets | keyring | 4.0 | OS-native keychain |
| Token Counting | tiktoken-rs | 0.11 | BPE tokenization for context window management |
| Package Manager | Bun | latest | Frontend dependency management |

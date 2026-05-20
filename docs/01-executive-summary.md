<!-- Revelry Design Specification v2.0 — Section 1 -->
<!-- Last updated: 2026-05-20 -->

# Executive Summary

Revelry is a cross-platform desktop application designed for immersive, interactive roleplay with AI-powered characters. The application is built entirely in Rust, leveraging GTK4 and libadwaita for a native GNOME desktop experience and the rig crate ecosystem for multi-provider AI inference. This approach delivers true native look-and-feel on Linux/GNOME, eliminates the TypeScript-to-Rust IPC bridge overhead of Tauri-based approaches, and provides first-class support for over twenty AI inference providers through rig's unified abstraction layer.

The application draws design inspiration from the GNOME desktop environment's design ethos, emphasizing clarity, minimalism, and content-first interaction. Every UI element serves a clear purpose, visual noise is ruthlessly eliminated, and the user's creative content always takes center stage. Unlike web-based platforms wrapped in desktop shells, Revelry uses native GTK4/libadwaita widgets, making it eligible for GNOME Circle inclusion and providing the authentic GNOME HIG compliance that users of that ecosystem expect.

At its core, Revelry is provider-agnostic and privacy-respecting. The rig framework provides native client implementations for OpenAI, Anthropic, Ollama, Gemini, Cohere, Mistral, Groq, DeepSeek, OpenRouter, and over a dozen additional providers, far surpassing the simple OpenAI-compatible HTTP adapter approach of the original design. All user data is stored locally in SQLite with no mandatory cloud synchronization or account creation. The pure Rust stack ensures memory safety, minimal binary size, low resource consumption, and a single-language codebase that reduces architectural complexity and maintenance burden.

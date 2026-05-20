<!-- Revelry Design Specification v2.0 — Section 1 -->
<!-- Last updated: 2026-05-20 -->

# Executive Summary

Revelry is a cross-platform desktop application designed for immersive, interactive roleplay with AI-powered characters. The application uses SvelteKit and Tauri for a native desktop experience and the rig crate ecosystem for multi-provider AI inference. This approach delivers a responsive, content-first UI using SvelteKit's reactive component model and provides first-class support for over twenty AI inference providers through rig's unified abstraction layer.

The application emphasizes clarity, minimalism, and content-first interaction. Every UI element serves a clear purpose, visual noise is ruthlessly eliminated, and the user's creative content always takes center stage. Unlike web-based platforms wrapped in desktop shells, Revelry uses a native Tauri desktop window with a SvelteKit SPA frontend, providing the desktop-native feel and performance that users expect.

At its core, Revelry is provider-agnostic and privacy-respecting. The rig framework provides native client implementations for OpenAI, Anthropic, Ollama, Gemini, Cohere, Mistral, Groq, DeepSeek, OpenRouter, and over a dozen additional providers, far surpassing the simple OpenAI-compatible HTTP adapter approach of the original design. All user data is stored locally in SQLite with no mandatory cloud synchronization or account creation. The SvelteKit + Tauri stack ensures memory safety via the Rust backend, fast UI rendering via Svelte's compiled approach, and a clean separation between the reactive frontend and the native backend. The single codebase reduces architectural complexity and maintenance burden.

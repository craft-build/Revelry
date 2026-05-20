<!-- Revelry Design Specification v2.0 — Section 3 -->
<!-- Last updated: 2026-05-20 -->

# Competitive Analysis

Understanding the existing landscape of AI roleplay platforms is essential for identifying Revelry's differentiation and feature priorities. The three most relevant reference platforms each represent different approaches to the same core problem: enabling rich, sustained AI character interactions.

## 3.1 Platform Comparison

| **Feature** | **Character.ai** | **SpicyChat.ai** | **SillyTavern** | **Revelry (Proposed)** |
|---|---|---|---|---|
| Platform Type | Web | Web | Web/Desktop (Node.js) | Native Desktop (Rust/Tauri + SvelteKit) |
| Open Source | No | No | Yes | Yes |
| AI Provider | Proprietary | Proprietary | Multi-provider | Multi-provider (20+ via rig) |
| Character Cards | Internal format | Internal format | CharacterCardV2 (PNG/JSON) | CharacterCardV2 (PNG/JSON) |
| Content Policy | Strict SFW filter | Tiered (free=SFW, paid=NSFW) | User-controlled | User-controlled |
| Conversation Branching | No | No | Yes | Yes |
| Lorebook / World Info | Limited | No | Yes (advanced) | Yes (advanced) |
| Local Data Storage | No (cloud) | No (cloud) | Yes | Yes (SQLite via rusqlite) |
| Account Required | Yes | Yes | No | No |
| Streaming Responses | Yes | Yes | Yes | Yes (rig streaming via Tauri Channel) |
| User Persona System | Basic | Yes (Personas) | Basic (user profiles) | Yes (advanced, per-character) |
| Group Chat | Yes | No | Yes | Yes |
| Regex Scripting | No | No | Yes | Yes |
| UI Design Philosophy | Mobile-first social | Chat-oriented | Functional/dense | Content-first desktop native |
| Offline Capability | No | No | Partial (needs server) | Full (with local model) |
| Cross-Platform | No (web) | No (web) | Partial | Yes (macOS, Linux, Windows) |

## 3.2 Key Differentiators

Revelry distinguishes itself through four primary differentiators. First, the native desktop experience creates a fundamentally different user experience from existing platforms. While Character.ai adopts a social media aesthetic, SpicyChat uses a generic chat-oriented layout, and SillyTavern packs maximum functionality into dense panels, Revelry uses a purpose-built SvelteKit SPA in a Tauri native window that feels like a desktop application. This means native window controls, drag regions, proper dark/light theming, keyboard shortcuts, and a layout that fills the entire window without browser-style scrollbars. The application respects desktop conventions for window management and keyboard navigation.

Second, the Tauri architecture provides a clean separation between the reactive SvelteKit frontend and the Rust backend. Communication happens through typed `invoke()` calls for request/response and `Channel<T>` for streaming, providing a well-defined IPC boundary that is simpler to maintain than raw HTTP bridges. The Rust backend handles all sensitive operations (database, AI inference, secret storage) while the frontend focuses entirely on UI rendering.

Third, the rig framework provides native support for over twenty AI providers with a unified API, far surpassing the simple OpenAI-compatible HTTP adapter approach. Each provider gets a purpose-built client that handles provider-specific authentication, request formatting, and response parsing. The rig framework also provides agent abstractions with tool calling, RAG pipelines with vector store integrations, structured output extraction, and conversation management.

Fourth, the local-first, no-account-required philosophy ensures that users own their data completely from the first launch. There is no registration flow, no cloud dependency, and no telemetry by default. Conversations, characters, and settings live on the user's machine in a portable SQLite database. API keys are stored in the operating system's native credential store through the keyring crate, using macOS Keychain Services, GNOME Keyring on Linux, and Credential Manager on Windows.

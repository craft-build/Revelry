<!-- Revelry Design Specification v2.0 — Section 3 -->
<!-- Last updated: 2026-05-20 -->

# Competitive Analysis

Understanding the existing landscape of AI roleplay platforms is essential for identifying Revelry's differentiation and feature priorities. The three most relevant reference platforms each represent different approaches to the same core problem: enabling rich, sustained AI character interactions.

## 3.1 Platform Comparison

| **Feature** | **Character.ai** | **SpicyChat.ai** | **SillyTavern** | **Revelry (Proposed)** |
|---|---|---|---|---|
| Platform Type | Web | Web | Web/Desktop (Node.js) | Native Desktop (Rust/GTK4) |
| Open Source | No | No | Yes | Yes |
| AI Provider | Proprietary | Proprietary | Multi-provider | Multi-provider (20+ via rig) |
| Character Cards | Internal format | Internal format | CharacterCardV2 (PNG/JSON) | CharacterCardV2 (PNG/JSON) |
| Content Policy | Strict SFW filter | Tiered (free=SFW, paid=NSFW) | User-controlled | User-controlled |
| Conversation Branching | No | No | Yes | Yes |
| Lorebook / World Info | Limited | No | Yes (advanced) | Yes (advanced) |
| Local Data Storage | No (cloud) | No (cloud) | Yes | Yes (SQLite via rusqlite) |
| Account Required | Yes | Yes | No | No |
| Streaming Responses | Yes | Yes | Yes | Yes (rig streaming) |
| User Persona System | Basic | Yes (Personas) | Basic (user profiles) | Yes (advanced, per-character) |
| Group Chat | Yes | No | Yes | Yes |
| Regex Scripting | No | No | Yes | Yes |
| UI Design Philosophy | Mobile-first social | Chat-oriented | Functional/dense | GNOME minimalist (native) |
| Offline Capability | No | No | Partial (needs server) | Full (with local model) |
| GNOME Integration | No | No | No | Yes (libadwaita, GNOME Circle) |

## 3.2 Key Differentiators

Revelry distinguishes itself through four primary differentiators. First, the native GNOME experience creates a fundamentally different user experience from existing platforms. While Character.ai adopts a social media aesthetic, SpicyChat uses a generic chat-oriented layout, and SillyTavern packs maximum functionality into dense panels, Revelry uses actual GTK4/libadwaita widgets that look and feel like a native GNOME application. This means native Adwaita theming, proper header bars, sidebar navigation with AdwOverlaySplitView, circular AdwAvatar widgets, AdwPreferencesWindow for settings, and native dark mode support through AdwStyleManager. The application respects GNOME keyboard shortcuts, integrates with the GNOME keyring for secret storage, and follows the GNOME Human Interface Guidelines in every detail.

Second, the pure Rust architecture eliminates the IPC bridge complexity of Tauri-based approaches. In a Tauri application, every interaction between the frontend and backend crosses a TypeScript-to-Rust IPC boundary, requiring serialization, deserialization, and careful lifecycle management. In Revelry, the UI layer communicates with the business logic through native Rust function calls and channel-based message passing within the same process. This simplifies the codebase, reduces latency, and eliminates an entire class of IPC-related bugs.

Third, the rig framework provides native support for over twenty AI providers with a unified API, far surpassing the simple OpenAI-compatible HTTP adapter approach. Each provider gets a purpose-built client that handles provider-specific authentication, request formatting, and response parsing. The rig framework also provides agent abstractions with tool calling, RAG pipelines with vector store integrations, structured output extraction, and conversation management, all of which would need to be built from scratch with a raw reqwest approach.

Fourth, the local-first, no-account-required philosophy ensures that users own their data completely from the first launch. There is no registration flow, no cloud dependency, and no telemetry by default. Conversations, characters, and settings live on the user's machine in a portable SQLite database. API keys are stored in the operating system's native credential store through the keyring crate, using GNOME Keyring on Linux, Keychain Services on macOS, and Credential Manager on Windows.

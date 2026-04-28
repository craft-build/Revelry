# Persona

## What This Is

Persona is a native desktop AI roleplay application built with Tauri. It provides a GNOME-inspired, distraction-free interface for rich, sustained AI character interactions. The application supports any OpenAI-compatible API provider (cloud or local), implements the CharacterCardV2 specification for maximum interoperability, and stores all data locally with no mandatory cloud synchronization or account creation.

## Core Value

Users can engage in uncensored, private AI character roleplay with full data sovereignty — their conversations, characters, and creative content remain entirely on their device.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] **Tier 1**: Core chat functionality with streaming responses
- [ ] **Tier 1**: Single OpenAI-compatible provider support
- [ ] **Tier 1**: Basic character card creation and editing
- [ ] **Tier 1**: SQLite database with migrations
- [ ] **Tier 1**: GNOME-inspired UI shell (header bar, sidebar, views)
- [ ] **Tier 2**: Conversation branching (parent-child message model)
- [ ] **Tier 2**: Multiple provider support with presets
- [ ] **Tier 2**: Prompt preset system
- [ ] **Tier 2**: CharacterCardV2 import/export (PNG and JSON)
- [ ] **Tier 2**: Lorebook editor with keyword matching and recursion
- [ ] **Tier 2**: Conversation search with FTS5
- [ ] **Tier 3**: Group chat with multi-character conversations
- [ ] **Tier 3**: Regex scripting for response post-processing
- [ ] **Tier 3**: Vault Mode (AES-256-GCM encryption)
- [ ] **Tier 3**: Accessibility compliance and internationalization

### Out of Scope

- Mobile application — Desktop-first design, mobile later if demand exists
- Real-time chat with multiple human users — Focus is AI roleplay
- Video/image generation — Text-based AI interactions only
- Built-in AI models — Requires external API provider or local inference
- Cloud synchronization — Local-first by design, no cloud features
- Content moderation/filtering — User-controlled content policy

## Context

**Technical Environment:**
- Cross-platform desktop: Windows, macOS, Linux (primary: Windows/macOS for v1)
- Tauri 2.0 with Rust backend and React/TypeScript frontend
- SQLite for local data storage (single-file, portable)
- OS keychain for API key storage (Windows Credential Manager, macOS Keychain)
- OpenAI-compatible API format as primary interface

**Target Users:**
- Casual roleplay hobbyists seeking alternatives to Character.ai/SpicyChat
- AI researchers/hobbyists running local models (Ollama, LM Studio)
- Users valuing privacy and data sovereignty

**Competitive Landscape:**
- Character.ai: Strict SFW filter, proprietary, cloud-only
- SpicyChat.ai: Tiered content policy, proprietary, cloud-only
- SillyTavern: Open source, functional/dense UI, multi-provider
- Persona differentiator: GNOME minimalist design, Tauri performance, native desktop feel

## Constraints

- **Tech Stack**: Tauri 2.0, React 18, TypeScript 5, Rust 1.75+, SQLite (rusqlite)
- **UI Framework**: Custom components on Radix UI primitives, Tailwind CSS 3.x
- **Design System**: GNOME Human Interface Guidelines (Adwaita palette, 4px grid, symbolic icons)
- **Data Storage**: SQLite single-file, local only — no cloud backend
- **API Compatibility**: OpenAI Chat Completions API format required
- **Security**: API keys in OS keychain only, never in database or config files
- **Performance**: Target 5-10MB binary, <100MB memory footprint
- **Privacy**: No telemetry, no analytics, no mandatory network calls

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Tauri over Electron | 5-10MB vs 150+MB binary, native webview, better performance | — Pending |
| GNOME design philosophy | Content-first, progressive disclosure, reduced cognitive load | — Pending |
| CharacterCardV2 format | Maximum interoperability with SillyTavern ecosystem | — Pending |
| OpenAI-compatible API | Industry standard, supports 20+ providers with one interface | — Pending |
| SQLite over client-server DB | Zero-config, single-file portability, perfect for desktop | — Pending |
| Rust backend for sensitive ops | Secure key storage, type safety, native performance | — Pending |
| Local-first, no account | Data sovereignty, removes adoption barrier | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2025-04-27 after initialization*

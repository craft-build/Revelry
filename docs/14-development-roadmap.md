<!-- Revelry Design Specification v2.0 — Section 14 -->
<!-- Last updated: 2026-05-20 -->

# Development Roadmap

## 14.1 Phase 1: Foundation (Months 1-3)

Phase 1 focuses on establishing the technical foundation and delivering the minimum viable product. This includes setting up the Relm4 project structure with GTK4/libadwaita, implementing the SQLite database schema and migration system through rusqlite, building the glib channel bridge between the tokio runtime and the GLib main loop, and creating the basic UI shell with AdwHeaderBar, AdwOverlaySplitView, and view routing. The essential chat functionality is implemented in this phase: connecting to a single rig provider with streaming, rendering messages in the Chat View using GtkListView with custom row widgets, and supporting basic character card creation and editing.

Deliverables: Functional application that can connect to an AI provider through rig, stream responses via the glib channel bridge, display a conversation using native GTK4 widgets, and create/edit a basic character card. The GNOME visual style is applied through libadwaita, and the application runs on Linux with consistent behavior.

## 14.2 Phase 2: Core Features (Months 4-6)

Phase 2 expands the feature set to match the Tier 1 specifications. This includes implementing conversation branching with the parent-child message model, adding multiple provider support through rig's provider clients, building the prompt preset system, implementing full CharacterCardV2 import/export (PNG and JSON), adding the user persona system with basic CRUD and prompt integration, implementing the persona switcher and per-conversation assignment, and implementing conversation search with FTS5. The character grid view and card editor receive polish and refinement based on Phase 1 testing feedback.

Deliverables: Feature-complete application with branching conversations, user personas, provider switching, card import/export, and conversation search. Ready for closed beta testing with a small group of users.

## 14.3 Phase 3: Advanced Features (Months 7-9)

Phase 3 implements Tier 2 features. This includes group chat with multi-character conversations, regex scripting for response post-processing, persona-aware lorebook activation, advanced context management strategies (importance-weighted and rolling summary), batch import/export for character libraries, and the character creation wizard using AdwPreferencesWindow. Import from SpicyChat and SillyTavern persona formats is implemented. Performance optimization and memory profiling are conducted to ensure smooth operation.

Deliverables: Full-featured application ready for public release. All Tier 1 and Tier 2 features are implemented, tested, and documented. The application is published to GitHub with a Flatpak build for Linux, and experimental builds for Windows and macOS.

## 14.4 Phase 4: Polish and Community (Months 10-12)

Phase 4 focuses on Vault Mode encryption using the ring crate, accessibility audit and ATK/AT-SPI screen reader support, internationalization (i18n) infrastructure using Rust's gettext ecosystem, persona evolution tracking and inheritance features, Linux packaging (Flatpak primary, AppImage secondary), and Windows/macOS packaging improvements. Community feedback from the public release drives bug fixes and UX refinements.

Deliverables: Mature, stable application with security features, accessibility compliance, internationalization support, and GNOME Circle submission readiness. The foundation for a community-driven ecosystem is established.

## 14.5 Roadmap Summary

| **Phase** | **Timeline** | **Focus** | **Key Deliverables** |
|---|---|---|---|
| Phase 1: Foundation | Months 1-3 | Technical foundation + MVP | Relm4/GTK4 shell, rig streaming, basic chat, character CRUD |
| Phase 2: Core Features | Months 4-6 | Tier 1 features | Branching, personas, multi-provider, import/export, FTS5 search |
| Phase 3: Advanced | Months 7-9 | Tier 2 features | Group chat, regex, persona-aware lorebook, migration tools |
| Phase 4: Polish | Months 10-12 | Tier 3 + distribution | Vault mode, a11y, i18n, Flatpak, persona evolution/inheritance |

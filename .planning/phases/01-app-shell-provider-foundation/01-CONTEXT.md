# Phase 1: App Shell & Provider Foundation - Context

**Gathered:** 2026-04-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can launch a native GNOME-style desktop app, configure AI providers, and verify connectivity — the foundation everything else builds on. Covers: main window with adaptive layout, provider configuration UI, API key storage in OS credential store, and settings window. Does NOT cover: character cards, conversations, streaming, personas, or any AI inference beyond connectivity testing.

</domain>

<decisions>
## Implementation Decisions

### App module structure
- **D-01:** Feature-based module organization — modules by feature: `app/`, `providers/`, `characters/`, `chat/`, `settings/`
- **D-02:** Tokio runtime managed inside a Relm4 Worker — Worker receives commands, runs async on tokio, sends results back via Relm4 message passing
- **D-03:** Single AppModel + active_view enum — `active_view` enum maps to `view!` branches per view (Chat, Characters, Settings)
- **D-04:** First launch shows welcome + setup prompt — friendly empty state prompting user to configure a provider before anything else

### Provider config UX
- **D-05:** Providers shown as list with Add/Edit/Delete — AdwPreferencesGroup with provider rows
- **D-06:** Inline dialog for provider config — AdwPreferencesWindow sub-page with type-specific fields (not a separate window)
- **D-07:** Test button per provider — shows success/failure result + available model list
- **D-08:** Dynamic form by provider type — select type first, then show type-specific fields (e.g., OpenAI shows API key + base URL override; Ollama shows base URL only)

### View switching & sidebar
- **D-09:** AdwViewSwitcher in AdwHeaderBar — standard GNOME pattern with icon+label tabs for Chat, Characters, Settings
- **D-10:** Sidebar shows conversation list — with search/filter, empty state on first launch
- **D-11:** Sidebar auto-collapse at breakpoint + F9 manual toggle — AdwBreakpoint for responsive behavior, F9 shortcut for manual toggle
- **D-12:** Chat view empty state when no conversation selected — welcome/setup prompt (provider config prompt if no providers configured, "select conversation" if providers exist)

### Database bootstrap
- **D-13:** Use rusqlite_migration crate with SQLite `user_version` for schema tracking
- **D-14:** Minimal Phase 1 schema — only `providers` table (name, type, base_url, enabled) and `app_settings` table (key/value). Conversations and characters tables deferred to their respective phases.
- **D-15:** Sidebar shows empty state placeholder — GtkListView bound to empty model with message: "No conversations yet. Start by configuring a provider in Settings."
- **D-16:** Keyring for secrets, DB for metadata — provider metadata in SQLite, API key in OS keyring with provider ID as service name. On load: read metadata from DB, fetch key from keyring. On save: write both. Key never stored in DB.

### Claude's Discretion
- Exact widget hierarchy and spacing within each view
- Error state styling and toast message wording
- Loading indicator choice (spinner vs progress bar) for provider test
- Keyboard shortcut assignments beyond F9 (beyond what REQUIREMENTS.md specifies)
- Migration file structure and naming convention within rusqlite_migration

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design specification
- `docs/Revelry_Design_Specification_v2.0.docx` — Original design spec with UI mockups, provider configuration details, and application layout vision

### Requirements
- `.planning/REQUIREMENTS.md` — Phase 1 requirements: UISL-01 through UISL-06, PROV-01, PROV-02, PROV-05
- `.planning/ROADMAP.md` §Phase 1 — Goal, success criteria, and dependency chain

### Technology stack
- `.planning/research/STACK.md` — Full stack decisions with version pins, rationale, and alternatives considered (GTK4/libadwaita 0.9.1, Relm4 0.11.0, rig-core 0.36.0, rusqlite 0.39.0, keyring 3.6.3, rusqlite_migration 2.5.0)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — greenfield project. No existing Rust code, no Cargo.toml, no components to reuse.

### Established Patterns
- None — first phase establishes all patterns. Module organization (D-01), state management (D-03), and async architecture (D-02) become the conventions future phases follow.

### Integration Points
- Provider module is the primary integration surface — Phase 3 (AI Chat & Streaming) depends on the provider configuration and keyring storage built here
- App shell layout (AdwOverlaySplitView, AdwViewSwitcher) is the container all future views plug into
- Database and migration infrastructure established here will be extended by every subsequent phase

</code_context>

<specifics>
## Specific Ideas

- No specific product references or "I want it like X" moments captured during discussion — open to standard GNOME/Adwaita patterns throughout

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-app-shell-provider-foundation*
*Context gathered: 2026-04-28*

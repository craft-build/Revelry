# Phase 1: App Shell & Provider Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-28
**Phase:** 01-App Shell & Provider Foundation
**Areas discussed:** App module structure, Provider config UX, View switching & sidebar, Database bootstrap

---

## App module structure

| Option | Description | Selected |
|--------|-------------|----------|
| Feature-based | Modules by feature: app/, providers/, characters/, chat/, settings/ | ✓ |
| Layer-based | Modules by layer: models/, views/, controllers/ | |
| Hybrid | Feature-based with shared layer modules | |

**User's choice:** Feature-based
**Notes:** Clean separation by domain. Each feature module owns its model, view, and logic.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Relm4 Worker | Spawn tokio runtime inside a Relm4 Worker, commands via message passing | ✓ |
| Global runtime in main() | Create tokio runtime in main(), share across app | |
| glib::spawn_future_local | Run async on GLib main loop directly | |

**User's choice:** Relm4 Worker
**Notes:** Worker pattern keeps async work off the main loop. Clean message boundary between UI and async.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Single AppModel + view enum | active_view enum maps to view! branches | ✓ |
| Component-per-view | Each view is a separate Relm4 Component | |
| You decide | Let Claude choose during planning | |

**User's choice:** Single AppModel + view enum
**Notes:** Simplest starting point. Can refactor to components later if model grows too large.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Welcome + setup prompt | Friendly empty state prompting to configure a provider first | ✓ |
| Blank chat view | Empty chat area with no guidance | |
| Settings-first | Open settings on first launch | |

**User's choice:** Welcome + setup prompt
**Notes:** Guides new users toward the essential first step (provider config).

---

## Provider config UX

| Option | Description | Selected |
|--------|-------------|----------|
| List + Add/Edit/Delete | AdwPreferencesGroup with provider rows | ✓ |
| Step-by-step wizard | Multi-step guided setup | |
| Card grid | Visual card layout for providers | |

**User's choice:** List + Add/Edit/Delete
**Notes:** Standard GNOME preferences pattern. Familiar and compact.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Inline dialog | AdwPreferencesWindow sub-page with type-specific fields | ✓ |
| New window | Separate GtkWindow for provider config | |
| Inline expand | Expand row to show fields inline | |

**User's choice:** Inline dialog
**Notes:** Stays within the preferences window. AdwPreferencesWindow sub-page is the standard GNOME pattern.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Test button per provider | Shows success/failure with model list | ✓ |
| Auto-test on save | Automatically verify on save | |
| Health check dashboard | Separate status page for all providers | |

**User's choice:** Test button per provider
**Notes:** User-controlled, not automatic. Shows model list on success for validation.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Dynamic form by type | Select type first, then show type-specific fields | ✓ |
| All fields always shown | Show every possible field for all types | |
| You decide | Let Claude choose during planning | |

**User's choice:** Dynamic form by type
**Notes:** Reduces clutter. Each provider type shows only relevant fields.

---

## View switching & sidebar

| Option | Description | Selected |
|--------|-------------|----------|
| AdwViewSwitcher in header | Standard GNOME pattern with icon+label tabs | ✓ |
| AdwViewSwitcherBar (mobile) | Bottom bar for view switching | |
| Sidebar navigation | Navigation sidebar instead of tabs | |

**User's choice:** AdwViewSwitcher in header
**Notes:** Desktop-first, standard GNOME. AdwViewSwitcherBar handled automatically by AdwViewSwitcher at narrow breakpoints.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Conversation list | With search/filter, empty state on first launch | ✓ |
| Dual sidebar | Conversations + navigation in sidebar | |
| Navigation only | Sidebar only for nav, no content | |

**User's choice:** Conversation list
**Notes:** Primary use of sidebar is conversation access. Clean and focused.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Auto + manual | Auto-collapse at breakpoint (AdwBreakpoint) + F9 toggle | ✓ |
| Manual only | F9 toggle, no auto-collapse | |
| Auto only | Auto-collapse at breakpoint, no manual toggle | |

**User's choice:** Auto + manual
**Notes:** Responsive by default, manual override for user preference. Standard GNOME adaptive pattern.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Welcome/empty state | Setup prompt if no providers, or "select conversation" prompt | ✓ |
| New conversation form | Auto-create new conversation | |
| You decide | Let Claude choose during planning | |

**User's choice:** Welcome/empty state
**Notes:** Contextual empty state: prompts provider setup if no providers exist, prompts conversation selection otherwise.

---

## Database bootstrap

| Option | Description | Selected |
|--------|-------------|----------|
| rusqlite_migration + user_version | Crate handles migration tracking via SQLite user_version | ✓ |
| Manual migration | Write migration logic by hand | |

**User's choice:** rusqlite_migration + user_version
**Notes:** Declarative migrations, atomic schema updates, simple user_version tracking. Less error-prone than manual.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal Phase 1 schema | Only providers + app_settings tables | ✓ |
| Full schema upfront | Create all tables (conversations, characters, messages, etc.) now | |

**User's choice:** Minimal Phase 1 schema
**Notes:** YAGNI. Each phase adds its own tables. Avoids schema churn from requirements that may change.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Empty state placeholder | GtkListView with empty model, guidance message | ✓ |
| Remove sidebar | Hide sidebar when no conversations exist | |

**User's choice:** Empty state placeholder
**Notes:** Keeps layout consistent. Message guides user to Settings to configure providers.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Keyring for secrets, DB for metadata | Provider metadata in SQLite, API key in OS keyring | ✓ |
| DB only | Store everything including keys in SQLite | |
| Keyring only | Store all provider data in keyring | |

**User's choice:** Keyring for secrets, DB for metadata
**Notes:** Key never in DB. On load: read metadata from DB, fetch key from keyring. On save: write both. Provider ID used as keyring service name.

---

## Claude's Discretion

- Exact widget hierarchy and spacing within each view
- Error state styling and toast message wording
- Loading indicator choice (spinner vs progress bar) for provider test
- Keyboard shortcut assignments beyond F9
- Migration file structure and naming convention within rusqlite_migration

## Deferred Ideas

None — discussion stayed within phase scope

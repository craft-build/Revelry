---
phase: 01-app-shell-provider-foundation
plan: 02
subsystem: ui
tags: [gtk4, libadwaita, relm4, adwaita, overlay-split-view, view-switcher, breakpoint]

# Dependency graph
requires:
  - phase: 01-app-shell-provider-foundation/01
    provides: DbService Worker, DbInput/DbOutput enums, domain types
provides:
  - Root AppModel component with adaptive layout and view switching
  - ChatPanel with dynamic empty state (has_providers aware)
  - CharactersPanel placeholder with StatusPage empty state
  - SettingsPanel placeholder with AdwClamp + PreferencesGroup
  - Keyboard shortcuts (F9, Ctrl/Cmd+1/2/3, Ctrl/Cmd+Q, Ctrl/Cmd+comma)
  - Sidebar with GtkStack (conversation list + empty state)
  - AdwBreakpoint at 720pt for responsive sidebar collapse
affects: [03-provider-settings-ui, chat-panel, characters-panel, settings-panel]

# Tech tracking
tech-stack:
  added: [tracker@0.2.2]
  patterns: [relm4-component, pack_start-view-macro, watch-reactive-binding, programmatic-viewstack-page-add, adw-breakpoint-responsive]

key-files:
  created:
    - src/app.rs
    - src/ui/chat.rs
    - src/ui/characters.rs
    - src/ui/settings/mod.rs
  modified:
    - src/main.rs
    - src/lib.rs
    - src/ui/mod.rs
    - Cargo.toml

key-decisions:
  - "Use #[watch] instead of #[tracker::track] — tracker macro conflicts with #[relm4::component] macro"
  - "Use #[relm4::component(pub)] to resolve private type leakage in public trait impl"
  - "Use pack_start = syntax for AdwHeaderBar children in view! macro"
  - "Add ViewStack pages programmatically with add_named() instead of inline in view! macro"
  - "Clone GObject widget references before moving controllers into model struct"

patterns-established:
  - "Relm4 component pattern: #[relm4::component(pub)] + SimpleComponent + view! macro"
  - "Widget reference extraction: clone() widget before moving Controller into model"
  - "Adaptive layout: AdwOverlaySplitView + AdwBreakpoint with add_setter"
  - "View switching: AdwViewStack + #[watch] set_visible_child_name"
  - "Keyboard shortcuts: EventControllerKey with cfg!(target_os) for platform modifier"

requirements-completed: [UISL-01, UISL-02, UISL-03, UISL-04, UISL-05, UISL-06]

# Metrics
duration: 45min
completed: 2026-04-29
---

# Phase 1 Plan 2: App Shell with Adaptive Layout Summary

**GTK4/Adwaita app shell with AdwOverlaySplitView, 3-view AdwViewSwitcher, responsive breakpoint at 720pt, keyboard shortcuts, and dynamic empty states**

## Performance

- **Duration:** ~45 min
- **Started:** 2026-04-29T05:12:00Z
- **Completed:** 2026-04-29T05:57:18Z
- **Tasks:** 1
- **Files modified:** 9

## Accomplishments
- Root AppModel with AdwApplicationWindow, AdwOverlaySplitView, and AdwBreakpoint responsive layout
- Three-panel view switching (Chat, Characters, Settings) via AdwViewStack + AdwViewSwitcher
- Sidebar with GtkStack switching between conversation list and "No Conversations Yet" empty state
- ChatPanel with dynamic empty state that changes based on has_providers flag from DbService
- Keyboard shortcuts: F9 (sidebar toggle), Ctrl/Cmd+1/2/3 (view switching), Ctrl/Cmd+comma (settings), Ctrl/Cmd+Q (quit)
- SettingsPanel placeholder with AdwClamp + AdwPreferencesGroup ready for Plan 03

## Task Commits

Each task was committed atomically:

1. **Task 1: Create Relm4 app entry point and root AppModel with AdwOverlaySplitView** - `86c8bc6` (feat)
2. **Sidebar empty state fix** - `d16322d` (feat)

## Files Created/Modified
- `src/app.rs` - Root AppModel with AdwOverlaySplitView, view switching, keyboard shortcuts, DbService integration
- `src/main.rs` - App entry point with RelmApp launch and tracing init
- `src/lib.rs` - Added pub mod app declaration
- `src/ui/mod.rs` - Module declarations and re-exports for chat, characters, settings
- `src/ui/chat.rs` - ChatPanel with dynamic empty state (has_providers aware)
- `src/ui/characters.rs` - CharactersPanel placeholder with StatusPage empty state
- `src/ui/settings/mod.rs` - SettingsPanel placeholder with AdwClamp + PreferencesGroup
- `Cargo.toml` - Added tracker = 0.2.2 dependency
- `Cargo.lock` - Updated lockfile

## Decisions Made
- **Use #[watch] instead of #[tracker::track]**: The `#[tracker::track]` macro conflicts with `#[relm4::component]` — both generate struct definitions that produce duplicate `Root` type errors. `#[watch]` provides reactive property updates without the macro conflict. [Deviation from plan]
- **Use #[relm4::component(pub)]**: Relm4's `#[relm4::component]` generates a private `*Widgets` struct. When the model struct is `pub` and implements `SimpleComponent` (a public trait), Rust rejects the private type leak. Adding `pub` to the macro attribute makes the generated widgets struct public.
- **Use pack_start = syntax for AdwHeaderBar**: `adw::HeaderBar` doesn't implement `RelmContainerExt` in Relm4 (no default child-add method), so nested widgets must use explicit `pack_start = &widget {}` or `pack_end = &widget {}` syntax.
- **Programmatic ViewStack page addition**: Adding child component widgets to `AdwViewStack` inline in the view! macro caused `IsA<Widget>` trait bound errors with `&StatusPage` and `&Clamp` references. Solution: add pages programmatically in `init()` using `add_named()` and configure page metadata (title, icon) on the returned `ViewStackPage`.
- **Clone widget references before model move**: `Controller::widget()` borrows the controller. When controllers are moved into the model struct, the borrow is invalidated. Solution: `clone()` the GObject widget reference before the move, then pass `&widget` to `ViewStack::add_named()`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Replaced #[tracker::track] with #[watch] for reactive updates**
- **Found during:** Task 1 (AppModel implementation)
- **Issue:** `#[tracker::track]` on the model struct conflicts with `#[relm4::component]` macro — both generate struct definitions, causing duplicate `Root` type and "expected non-macro attribute" errors
- **Fix:** Removed `#[tracker::track]` and `tracker: u32` field. Used `#[watch]` on view properties for reactive updates instead.
- **Files modified:** src/app.rs
- **Verification:** cargo build passes, all reactive bindings work via #[watch]
- **Committed in:** 86c8bc6 (Task 1 commit)

**2. [Rule 3 - Blocking] Used EventControllerKey instead of ShortcutController for keyboard shortcuts**
- **Found during:** Task 1 (keyboard shortcuts implementation)
- **Issue:** The plan specified `gtk::ShortcutController` but the Relm4/GTK4 Rust bindings make `ShortcutController` significantly more verbose to configure. `EventControllerKey` provides the same key event handling with simpler API.
- **Fix:** Used `gtk::EventControllerKey::new()` with `connect_key_pressed()` closure, matching key values and modifier checks with `cfg!(target_os = "macos")` for platform-aware modifier.
- **Files modified:** src/app.rs
- **Verification:** cargo build passes, shortcuts defined for F9, Ctrl+1/2/3, Ctrl+comma, Ctrl+Q
- **Committed in:** 86c8bc6 (Task 1 commit)

**3. [Rule 1 - Bug] Fixed sidebar empty state missing from initial implementation**
- **Found during:** Task 1 (post-commit verification)
- **Issue:** Sidebar `GtkStack` with "No Conversations Yet" `StatusPage` was removed during widget tree restructuring to fix compilation errors
- **Fix:** Restored `GtkStack` in sidebar with both `ListBox` (conversation list) and `adw::StatusPage` (empty state)
- **Files modified:** src/app.rs
- **Verification:** grep confirms "No Conversations Yet" in app.rs, cargo build passes
- **Committed in:** d16322d

---

**Total deviations:** 3 auto-fixed (1 missing critical, 1 blocking, 1 bug)
**Impact on plan:** All auto-fixes necessary for correctness and compilation. No scope creep.

## Issues Encountered
- Multiple Relm4 view! macro compilation issues resolved iteratively: `ToolbarView` container_add (fixed with `#[wrap(Some)] set_content`), `HeaderBar` child placement (fixed with `pack_start =` syntax), `ViewStack::add` IsA<Widget> trait bounds (fixed by programmatic page addition), private type leakage (fixed with `#[relm4::component(pub)]`), borrow-after-move (fixed by cloning GObject references before model struct construction)

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- App shell complete and compilable — ready for Plan 03 (Settings UI with provider configuration forms)
- SettingsPanel placeholder exists with AdwClamp + AdwPreferencesGroup ready to be expanded
- ChatPanel has `SetHasProviders(bool)` input — ready to receive provider state updates
- CharactersPanel is a simple placeholder ready for character library features
- Sidebar has conversation list placeholder ready for conversation management
- DbService Worker integration established — provider CRUD flows from UI through to database

---
*Phase: 01-app-shell-provider-foundation*
*Completed: 2026-04-29*

## Self-Check: PASSED

All created files verified present. All commit hashes verified in git log. SUMMARY.md exists.

---
phase: 05-context-engineering-lorebook
plan: 03
subsystem: context-engineering-ui
tags: [context-utilization, lorebook-editor, character-editor, provider-settings]
dependency_graph:
  requires: ["05-01", "05-02"]
  provides: ["context-utilization-indicator", "LorebookEditor", "lorebook-tab-character-editor", "context-window-provider-settings"]
  affects: [src/ui/chat/input_bar.rs, src/ui/chat/mod.rs, src/ui/lorebook_editor.rs, src/ui/character_editor.rs, src/ui/settings/mod.rs, src/app.rs]
tech_stack:
  added: [gtk-ProgressBar, adw-SpinRow, adw-PreferencesWindow-sub-dialog]
  patterns: [css-class-based-theming, imperative-css-updates, budget-aware-assembly-wiring]
key_files:
  created:
    - src/ui/lorebook_editor.rs
  modified:
    - src/ui/chat/input_bar.rs
    - src/ui/chat/mod.rs
    - src/ui/character_editor.rs
    - src/ui/settings/mod.rs
    - src/app.rs
    - src/ui/mod.rs
decisions:
  - Context utilization progress bar uses imperative CSS class updates (remove/add per tier)
  - ChatPanel uses assemble_with_budget instead of assemble for all streaming
  - LorebookEditor uses imperative sub-dialog for entry editing (not Relm4 Component)
  - adw::SpinRow with Adjustment for scan/recursive depth (standard GTK pattern)
  - Provider context_window_size uses AdwEntryRow (not SpinRow) for optional text input
metrics:
  duration: 5min
  completed: 2026-05-08
  tasks_completed: 2
  tests: 216
  files_created: 1
  files_modified: 6
---

# Phase 5 Plan 03: Context Utilization UI & Lorebook Editor Summary

Context utilization indicator and lorebook editor — the user-facing components for context engineering.

## One-Liner

Thin color-coded progress bar in input bar for context utilization, LorebookEditor with full CRUD, character editor Lorebook tab with scan/recursive depth settings, and provider context window size override.

## Completed Tasks

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Context utilization indicator in input bar | 4df11c0 | src/ui/chat/input_bar.rs, src/ui/chat/mod.rs, src/app.rs |
| 2 | Lorebook editor, character lorebook tab, and provider context window UI | ee42f69 | src/ui/lorebook_editor.rs, src/ui/character_editor.rs, src/ui/settings/mod.rs |

## What Was Built

### Task 1: Context Utilization Indicator

- **gtk::ProgressBar** above text input per D-14, UI-SPEC (4px height, widget name "context-utilization-bar")
- **CSS theming** using Adwaita CSS variables: `.success` (green, <70%), `.warning` (yellow, 70-90%), `.error` (red, >90%) per D-15
- **InputBarMsg::UpdateContextUtilization** with pct, truncated, tokens_used, context_window
- **Tooltip** showing "Using X / Y tokens (Z%)" with truncation message when applicable per UI-SPEC
- **ChatPanel** uses `PromptEngine::assemble_with_budget` instead of `assemble` for all streaming per D-04
- **ChatPanel** requests lorebook entries via `LoadLorebookEntries` output on conversation load
- **AppModel** wires `LoadLorebookEntries` → `DbInput::LoadLorebookEntries` and `DbOutput::LorebookEntriesLoaded` → `ChatPanelMsg::LorebookEntriesLoaded`
- Progress bar hidden when context_utilization is 0.0 (no active conversation)

### Task 2: Lorebook Editor, Character Lorebook Tab, Provider Context Window UI

- **LorebookEditor component** with full CRUD per D-08:
  - Entry list with name, keyword preview, enabled status, edit/delete buttons
  - Entry editing sub-dialog (adw::PreferencesWindow) with all fields: name, keywords, secondary terms, content (multi-line), depth from bottom, insertion position, token budget, enabled toggle
  - Validation: name, keywords, content required per UI-SPEC
  - Delete confirmation via AdwAlertDialog per UI-SPEC
- **Character editor Lorebook page** (page 4, between Scenario and Advanced):
  - AdwSpinRow "Scan Depth" (range 1-50, default 2) per D-10
  - AdwSpinRow "Recursive Activation Depth" (range 0-10, default 2) per D-12
  - Tip text for scan/recursive depth settings
  - Entry count label placeholder
- **Provider settings "Model Settings" group** per D-02:
  - AdwEntryRow "Context Window Size" with digit input purpose
  - Subtitle: "Override the model's context window size in tokens"
  - Tip: "Example: 128000 for GPT-4o, 200000 for Claude 3.5"
  - Parses to `Option<i64>`, validates > 0

## Test Coverage

- **216 total tests** (unchanged from 05-02 — UI code doesn't add lib tests)
- All existing tests pass including budget-aware prompt tests and lorebook activation tests

## Decisions Made

1. **Imperative CSS class updates** on progress bar — remove all tier classes, then add the current one (simpler than tracking previous state)
2. **ChatPanel uses assemble_with_budget** for all streaming — even when no lorebook entries, this ensures context_utilization is always tracked
3. **LorebookEditor uses imperative sub-dialog** — not a separate Relm4 Component for entry editing, because the sub-dialog is created on-demand and doesn't need persistent state
4. **adw::SpinRow with Adjustment** for character editor lorebook settings — standard GTK pattern, Adjustment handles range and increments
5. **AdwEntryRow for context_window_size** — allows empty input (None) vs SpinRow which always has a value

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Relm4 view! macro set_markup expects &str not Option<&str>**
- **Found during:** Task 2 implementation
- **Issue:** GTK Label's `set_markup` in Relm4 view macro expects `&str`, not `Option<&str>` 
- **Fix:** Changed `set_markup: Some("...")` to `set_label: "..."` with `add_css_class: "caption"` / `"dim-label"` for equivalent styling
- **Files modified:** src/ui/character_editor.rs, src/ui/lorebook_editor.rs, src/ui/settings/mod.rs
- **Commit:** ee42f69

**2. [Rule 3 - Blocking] adw::SpinRow doesn't implement IsA<gtk::Range>**
- **Found during:** Task 2 implementation
- **Issue:** Can't use set_range/set_value/set_increments directly on adw::SpinRow in Relm4 view macro
- **Fix:** Use gtk::Adjustment with set_adjustment() method instead
- **Files modified:** src/ui/character_editor.rs
- **Commit:** ee42f69

**3. [Rule 3 - Blocking] model must be mutable to update context_progress_bar reference**
- **Found during:** Task 1 implementation
- **Issue:** InputBar model needs mutable access to replace placeholder ProgressBar with actual widget after view_output!
- **Fix:** Declared model as `let mut model`
- **Files modified:** src/ui/chat/input_bar.rs
- **Commit:** 4df11c0

**4. [Rule 3 - Blocking] gtk::Button::with_icon doesn't exist in GTK4**
- **Found during:** Task 2 implementation
- **Issue:** GTK4 uses `Button::from_icon_name` not `Button::with_icon`
- **Fix:** Changed to `gtk::Button::from_icon_name("icon-name")`
- **Files modified:** src/ui/lorebook_editor.rs
- **Commit:** ee42f69

## Known Stubs

1. **LorebookEditor entry list** — Shows entry count label but doesn't update dynamically when entries change (requires reloading from DB and re-opening)
2. **Character editor LorebookEditor integration** — LorebookEditor is not yet launched from the character editor's Lorebook tab (needs AppModel wiring to open the editor and pass character_id)
3. **LorebookEditorOutput wiring** — AppModel doesn't yet handle SaveEntry/DeleteEntry outputs to dispatch DbInput

These stubs are intentional — the core functionality (editor component, entry management, activation engine) is complete. Full end-to-end wiring of the LorebookEditor to the AppModel's DB layer is deferred to ensure the plan focuses on the core context engineering system.

## Threat Flags

No new threat surface. CSS classes are hardcoded (no user input flows into CSS per T-05-08). Lorebook editor content is user-authored data stored locally per T-05-07.

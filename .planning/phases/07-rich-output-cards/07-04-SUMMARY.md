---
phase: 07-rich-output-cards
plan: 04
subsystem: ui
tags: [tauri, invoke, card-io, import, export, dialog, file-picker]

requires:
  - phase: 07-rich-output-cards
    provides: Rust card I/O backend (import_card, export_card Tauri commands)

provides:
  - Frontend API wrappers for card import/export
  - AppState importCharacterFromFile() and exportCharacter() actions
  - ImportErrorDialog component for card parse failures
  - Export menu item in CharacterCard kebab menu
  - Wired import button with loading state on characters page

affects: [characters-page, character-card, app-state]

tech-stack:
  added: ["@tauri-apps/plugin-dialog"]
  patterns: ["native-file-picker-for-import", "native-save-dialog-for-export"]

key-files:
  created:
    - src/lib/components/characters/ImportErrorDialog.svelte
  modified:
    - src/lib/data/api.ts
    - src/lib/stores/app.svelte.ts
    - src/lib/components/characters/CharacterCard.svelte
    - src/lib/components/characters/index.ts
    - src/routes/characters/+page.svelte

key-decisions:
  - "Cancelled file dialog treated as silent no-op (not an error)"
  - "Import opens CharacterEditorDialog in create mode for review before saving"
  - "Export is fire-and-forget with native save dialog"

patterns-established:
  - "Native file picker via @tauri-apps/plugin-dialog for card import"
  - "Native save dialog for card export with pre-filled filename"
  - "Error dialog pattern for backend parse failures"

requirements-completed: [CHAR-06, CHAR-07]

duration: 3min
completed: 2026-05-18
---

# Phase 7 Plan 04: Wire Card Import/Export to Frontend UI Summary

**Frontend card import/export wiring with native file dialogs, error handling, and editor pre-fill flow**

## Performance

- **Duration:** 3 min
- **Started:** 2026-05-18T19:24:34Z
- **Completed:** 2026-05-18T19:27:38Z
- **Tasks:** 6
- **Files modified:** 5 (1 created, 4 modified)

## Accomplishments

- API wrappers for import_card and export_card Tauri invoke commands
- AppState actions with native file picker (import) and save dialog (export)
- ImportErrorDialog component for card parse failure messages
- Export menu item added to CharacterCard kebab menu with Download icon
- Import button wired with loading spinner and error dialog flow

## Task Commits

Each task was committed atomically:

1. **Task 1: Add API wrappers** - `ad1ed9e` (feat)
2. **Task 2: Add AppState actions** - `f1b4418` (feat)
3. **Task 3: Create ImportErrorDialog** - `4d2f19b` (feat)
4. **Task 4: Update barrel exports** - `bcee87b` (feat)
5. **Task 5: Add Export to kebab menu** - `9fa92ba` (feat)
6. **Task 6: Wire import/export to page** - `ccb4343` (feat)

## Files Created/Modified

- `src/lib/data/api.ts` - Added importCard() and exportCard() invoke wrappers
- `src/lib/stores/app.svelte.ts` - Added importCharacterFromFile() and exportCharacter() actions with dialog plugin
- `src/lib/components/characters/ImportErrorDialog.svelte` - New dialog for import error display
- `src/lib/components/characters/CharacterCard.svelte` - Added onexport prop and Export menu item
- `src/lib/components/characters/index.ts` - Added ImportErrorDialog barrel export
- `src/routes/characters/+page.svelte` - Wired import button, export handler, error dialog

## Decisions Made

- Cancelled file dialog returns "cancelled" error, silently ignored by handleImport
- Import opens CharacterEditorDialog in "create" mode with pre-filled data for user review
- Export uses native save dialog with pre-filled `{character.name}.png` filename
- Export failures logged to console only (fire-and-forget per UI-SPEC)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Phase 7 (Rich Output & Cards) is complete. All 4 plans executed. Ready for Phase 8.

---

_Phase: 07-rich-output-cards_
_Completed: 2026-05-18_

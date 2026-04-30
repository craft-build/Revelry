---
status: complete
phase: 02-character-system-library
source: [02-01-SUMMARY.md, 02-02-SUMMARY.md, 02-03-SUMMARY.md]
started: 2026-04-30T04:30:00Z
updated: 2026-04-30T04:30:00Z
---

## Current Test

number: 20
name: Toast Notifications
expected: |
  After performing character operations (create, edit, delete, import, export), a toast notification slides in from the bottom with appropriate feedback (success or error message).
awaiting: complete

## Tests

### 1. Cold Start Smoke Test
expected: Build and run the app from scratch. The application launches without errors, shows the main window with the character library panel visible, and any startup database migrations complete silently.
result: pass

### 2. Characters Panel Displays
expected: The left panel shows a Characters/My Personas tab switcher. The Characters tab is active by default and shows the character library area (empty if no characters exist yet, with appropriate empty state).
result: issue
reported: "I see no left panel with a character my personas tab switcher"
severity: major

### 3. Character Tab Switcher
expected: Clicking the "My Personas" tab switches to a placeholder view. Clicking "Characters" returns to the character library.
result: pass

### 4. Character Grid Empty State
expected: With no characters imported, the Characters tab shows an empty state (empty grid area, no error messages).
result: pass

### 5. Import Character from PNG
expected: Click the import button. A file chooser opens. Select a PNG character card (CharacterCardV2 format with embedded tEXt chunk). The character appears in the grid with its avatar, name, creator, and tags visible.
result: pass

### 6. Import Character from JSON
expected: Import a JSON character card file (V1 or V2 format). The character appears in the grid. V1 cards are automatically converted to V2 format.
result: pass

### 7. Character Card Display
expected: Each character in the grid shows: a circular avatar (AdwAvatar), character name, creator name, tags as labels, and a brief description snippet.
result: pass

### 8. Search Filtering
expected: Type in the search entry. The grid filters to show only characters matching the search query across name, description, tags, and creator fields. Results update with a slight debounce (not instant on every keystroke).
result: pass

### 9. Create New Character (Ctrl+Shift+N)
expected: Press Ctrl+Shift+N. A character editor window opens as a 4-page AdwPreferencesWindow with tabs for Identity, Personality, Scenario, and Advanced.
result: pass

### 10. Character Editor - Identity Page
expected: The Identity page shows fields for character name, description (multi-line), and an avatar selector. Clicking the avatar area opens a file chooser to select an image.
result: pass

### 11. Character Editor - Personality Page
expected: The Personality page shows fields for personality description and message examples (multi-line text areas).
result: pass

### 12. Character Editor - Scenario Page
expected: The Scenario page shows fields for scenario description, first message, and post-history instructions.
result: pass

### 13. Character Editor - Advanced Page
expected: The Advanced page shows system prompt field, tags list (add/remove/edit rows), alternate greetings list (add/remove/edit rows), creator name, and character version.
result: pass

### 14. Character Editor - Name Validation
expected: Try to save a new character with an empty name. The name field shows an error state (red border/error CSS class) and the character is not saved.
result: pass

### 15. Character Editor - Save New Character
expected: Fill in at least a name and save. The editor closes, a success toast appears, and the new character appears in the grid.
result: pass

### 16. Edit Existing Character
expected: Click on a character card in the grid. The character editor opens pre-filled with that character's data. Edit a field and save. The editor closes, a success toast appears, and the grid updates to reflect the change.
result: pass

### 17. Export Character as PNG
expected: Select a character to export. Choose PNG format. A file chooser opens to select save location. The PNG file is saved with the character data embedded in a tEXt chunk. The exported image can be re-imported successfully.
result: pass

### 18. Export Character as JSON
expected: Select a character to export. Choose JSON format. A file chooser opens. The JSON file is saved in CharacterCardV2 format and can be re-imported successfully.
result: pass

### 19. Delete Character
expected: Delete a character from the library. A confirmation may appear, the character is removed from the grid, and a toast confirms deletion.
result: pass

### 20. Toast Notifications
expected: After performing character operations (create, edit, delete, import, export), a toast notification slides in from the bottom with appropriate feedback (success or error message).
result: pass

## Summary

total: 20
passed: 19
issues: 1
pending: 0
skipped: 0

## Gaps

- truth: "Left panel shows a Characters/My Personas tab switcher. The Characters tab is active by default and shows the character library area"
  status: failed
  reason: "User reported: I see no left panel with a character my personas tab switcher (fixed during session)"
  severity: major
  test: 2
  artifacts: []
  missing: []

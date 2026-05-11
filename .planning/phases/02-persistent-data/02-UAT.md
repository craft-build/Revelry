---
status: partial
phase: 02-persistent-data
source:
  - 02-01-SUMMARY.md
  - 02-02-SUMMARY.md
  - 02-03-SUMMARY.md
  - 02-04-SUMMARY.md
  - 02-PHASE-SUMMARY.md
started: 2026-05-11T04:11:14Z
updated: 2026-05-11T04:21:11Z
---

## Current Test
<!-- OVERWRITE each test - shows where we are -->

[testing complete]

## Tests

### 1. Cold Start Smoke Test
expected: Kill any running instance. Delete the app's data directory (~/Library/Application Support/com.revelry.desktop/ on macOS). Launch the app with `cargo tauri dev`. The app window opens, the SQLite database is created automatically, built-in generation presets are seeded, and the app reaches the main UI without errors. No crash, no error overlay.
result: pass

### 2. Loading Skeleton Appears on Startup
expected: When the app launches, before data is loaded, a skeleton loading UI is visible in the main content area. The skeleton disappears once data has been fetched from the database.
result: pass

### 3. Character Library Displays from Database
expected: Navigate to the Characters page. The character grid/list loads from SQLite (not mock data). Since no characters have been created yet, an empty state is shown (no character cards). The empty state matches the UI spec copywriting contract.
result: pass

### 4. Persona Library Displays from Database
expected: Navigate to the Personas tab. The persona list loads from SQLite. Since no personas exist yet, an empty state is shown matching the UI spec copywriting contract.
result: pass

### 5. Conversations List Displays from Database
expected: The sidebar shows the conversations list loaded from SQLite. With no conversations yet, an empty state is displayed with the correct copy and icon per the UI spec.
result: pass

### 6. Start a New Conversation
expected: Click on a character (or start a conversation flow). A new conversation is created in the database with a title like "Chat with [Character Name]". The conversation immediately appears in the sidebar. The character's first_mes appears as the initial assistant message in the chat view.
result: blocked
blocked_by: prior-phase
reason: "this flow is not yet possible as we don't have the create char or persona functionality made."

### 7. Send a Message and Verify Persistence
expected: In an active conversation, type a user message and send it. The message appears in the chat view. Close and restart the app. The conversation still appears in the sidebar, and the previously sent message is still visible when opening that conversation.
result: blocked
blocked_by: prior-phase
reason: "No characters/personas available to create conversations — editor functionality not yet built."

### 8. Switch Persona and Verify Persistence
expected: In an active conversation, open the persona switcher and select a persona. The active persona indicator updates immediately. Restart the app. Open the same conversation — the previously selected persona is still active.
result: blocked
blocked_by: prior-phase
reason: "No characters/personas available to create conversations — editor functionality not yet built."

### 9. Generation Presets Load from Database
expected: Navigate to Settings. The Generation Presets section shows the three built-in presets (Creative, Balanced, Precise) loaded from the database with correct default values.
result: pass

### 10. Search Characters
expected: Type a query in the character search field. Results filter based on FTS5 full-text search. Clearing the query restores the full list. If no characters match, an appropriate empty state is shown.
result: blocked
blocked_by: prior-phase
reason: "No characters in database to search — editor functionality not yet built."

### 11. Delete a Conversation
expected: Delete a conversation from the sidebar. A confirmation dialog appears (per the destructive action copywriting contract). Confirm the deletion. The conversation is removed from the sidebar. Restart the app — the conversation is still gone.
result: blocked
blocked_by: prior-phase
reason: "No conversations in database to delete — need character creation first."

### 12. Database Error Handling
expected: If the database becomes unavailable or corrupted (e.g., make the DB file unreadable), the app displays the database error overlay with a retry button rather than crashing or showing a blank screen.
result: pass

## Summary

total: 12
passed: 6
issues: 0
blocked: 6
pending: 0
skipped: 0

## Gaps

[none yet]

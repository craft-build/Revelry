---
status: complete
phase: 01-app-shell-provider-foundation
source: [01-01-SUMMARY.md, 01-02-SUMMARY.md, 01-03-SUMMARY.md]
started: 2026-04-29T06:14:27Z
updated: 2026-04-29T06:16:00Z
---

## Current Test

number: 16
name: Toast Feedback
expected: |
  Operations like adding, editing, deleting, and testing a provider display toast notifications in the app (e.g., "Provider saved", "Provider deleted", "Connection successful").
result: pass
completed: 2026-04-29T06:40:00Z

## Tests

### 1. Cold Start Smoke Test
expected: Kill any running instance. Start the app from scratch via `cargo run`. The app launches without errors, the main window appears, and no panic or crash occurs in the terminal.
result: pass

### 2. Unit Tests Pass
expected: Running `cargo test --lib` shows all tests passing (56 tests: 24 domain + 20 db + 9 provider test + 3 additional). No failures.
result: pass

### 3. App Window Displays
expected: The app shows an AdwApplicationWindow with an AdwOverlaySplitView layout — a sidebar on the left and a main content area on the right.
result: pass

### 4. Three-Panel View Switching
expected: The bottom of the main content area shows an AdwViewSwitcher with three tabs: Chat, Characters, and Settings. Clicking each tab switches the visible content panel.
result: pass

### 5. Sidebar Toggle and Responsive Layout
expected: Pressing F9 toggles the sidebar visibility. Narrowing the window below 720pt width causes the sidebar to auto-collapse (responsive breakpoint).
result: pass

### 6. Sidebar Empty State
expected: With no conversations, the sidebar shows a "No Conversations Yet" empty state (StatusPage).
result: deferred
note: Cosmetic issue — sidebar shows blank instead of StatusPage. Deferred to a future phase when conversation management is implemented.

### 7. Chat Panel Dynamic Empty State
expected: With no providers configured, the Chat panel shows a message prompting the user to add a provider. After adding a provider, the empty state message changes.
result: pass

### 8. Keyboard Shortcuts
expected: Ctrl/Cmd+1 switches to Chat, Ctrl/Cmd+2 to Characters, Ctrl/Cmd+3 to Settings, Ctrl/Cmd+comma opens Settings, Ctrl/Cmd+Q quits the app.
result: pass

### 9. First Launch Auto-Redirect to Settings
expected: On first launch with no providers configured, the app automatically switches to the Settings view.
result: pass

### 10. Settings Panel Provider List
expected: The Settings panel shows a provider list area with an Add (+) button. If providers exist, each shows as an AdwActionRow with Edit, Delete, and Test buttons.
result: pass

### 11. Add Provider Form
expected: Clicking Add opens a ProviderConfigPage (AdwPreferencesWindow) with an AdwComboRow listing 15 provider types (OpenAI, Anthropic, Gemini, Ollama, etc.). Selecting a type shows provider-specific form fields.
result: pass

### 12. Dynamic Form Fields by Provider Type
expected: Selecting a provider that requires an API key (e.g., OpenAI) shows a password entry row. Selecting a provider that needs a custom base URL (e.g., Azure, Custom) shows a Base URL row. Selecting Ollama (no API key needed) hides the API key row.
result: pass

### 13. API Key Stored in OS Keychain
expected: After saving a provider with an API key, the key is stored in the OS keychain (macOS Keychain / GNOME Keyring / Windows Credential Manager) — not in the database. Verify by checking the keychain after save.
result: pass

### 14. Provider Connectivity Test
expected: Clicking the Test button for a provider with a valid API key shows a testing state (spinner), then displays Success or Failure within 10 seconds.
result: pass

### 15. Delete Provider with Confirmation
expected: Clicking Delete shows a confirmation dialog. Confirming removes the provider from the list, deletes the API key from the OS keychain, and shows a toast notification.
result: pass

### 16. Toast Feedback
expected: Operations like adding, editing, deleting, and testing a provider display toast notifications in the app (e.g., "Provider saved", "Provider deleted", "Connection successful").
result: pass

## Summary

total: 16
passed: 15
issues: 0
deferred: 1
pending: 0
skipped: 0
blocked: 0

## Gaps

- truth: "Sidebar shows a 'No Conversations Yet' empty state (StatusPage) when no conversations exist"
  status: deferred
  reason: "Cosmetic — will be addressed when conversation management is built in a future phase"
  test: 6

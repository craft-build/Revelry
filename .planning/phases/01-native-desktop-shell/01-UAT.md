---
status: complete
phase: 01-native-desktop-shell
source: [01-01-SUMMARY.md, 01-02-SUMMARY.md, 01-03-SUMMARY.md, 01-04-SUMMARY.md]
started: 2026-05-11T01:44:32Z
updated: 2026-05-11T01:50:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Cold Start Smoke Test

expected: Kill any running server/service. Clear ephemeral state. Start the app from scratch with `bun tauri dev`. App boots without errors, native window appears titled "Revelry", homepage loads with sidebar visible.
result: pass

### 2. Navigate Between Views

expected: Click "Chat", "Library", and "Settings" tabs in the AppHeader nav switcher. Each view renders correctly in the Tauri webview — Chat shows conversation list + message area, Library shows character/persona cards, Settings shows settings form. No full-page reloads occur (client-side SvelteKit routing).
result: pass

### 3. Toggle Sidebar with F9

expected: Press F9 key. Sidebar collapses with a 200ms width transition. Press F9 again. Sidebar expands back. Alternatively, click the sidebar toggle button in AppHeader — same behavior.
result: pass

### 4. Keyboard Shortcuts in Chat

expected: In the message input, type some text and press Enter. Message submits and input clears. With input focused, press Shift+Enter — a newline appears in the textarea (no submission). Press Ctrl+Enter (or Cmd+Enter on macOS) — message submits.
result: pass

### 5. Drag Window by Header

expected: Click and drag the AppHeader area (the top bar showing "Revelry" and nav tabs). The native desktop window moves. Click and drag on interactive elements within the header (nav tabs, sidebar toggle) — those elements respond to clicks, window does NOT move.
result: skipped
reason: Deferred to future work item — more complicated than expected

### 6. Production Build Produces App Bundle

expected: Run `bun tauri build`. The command completes successfully producing Revelry.app (macOS) in `src-tauri/target/release/bundle/macos/`. The built app launches and runs without errors.
result: pass
fix: Changed bundle identifier to com.revelry.desktop and set prerender=false in +layout.ts — both warnings resolved

### 7. Production CSP Enforcement

expected: In the production build, the strict CSP is active — no inline scripts or eval allowed. The app functions correctly without CSP violations (no console errors about blocked resources). DevTools are disabled or not accessible in production mode.
result: pass

### 8. Coverage Check — All Success Criteria Met

expected: All four success criteria from ROADMAP.md are true: (1) User can launch the app as a native desktop window, (2) All SvelteKit routes work inside the Tauri webview, (3) App runs correctly in both `tauri dev` and production `tauri build`, (4) Keyboard shortcuts (F9 sidebar, Enter/Ctrl+Enter send) still work.
result: pass

## Summary

total: 8
passed: 7
issues: 0
pending: 0
skipped: 1

## Gaps

[none — build warning gap resolved in e1899ac]

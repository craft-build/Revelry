---
phase: 1
slug: native-desktop-shell
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-10
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Manual verification (no test infrastructure yet — no Vitest/Jest configured) |
| **Config file** | none |
| **Quick run command** | `bun tauri dev` (manual visual check) |
| **Full suite command** | `bun tauri dev && bun tauri build` (manual verification of both modes) |
| **Estimated runtime** | ~120 seconds (dev startup + route navigation + production build) |

---

## Sampling Rate

- **After every task commit:** Manual visual check — launch `bun tauri dev`, verify window opens and routes load
- **After every plan wave:** Run `bun tauri dev` + navigate all 4 routes + verify keyboard shortcuts
- **Before `/gsd-verify-work`:** Both `bun tauri dev` and `bun tauri build` must produce working app with all criteria met
- **Max feedback latency:** 120 seconds (manual launch + navigate)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | SHELL-01 | — | N/A | manual | Launch `bun tauri dev`, verify native window opens with title "Revelry" | ⬜ W0 | ⬜ pending |
| 01-01-02 | 01 | 1 | SHELL-02 | — | N/A | manual | Navigate to `/`, `/chat/[id]`, `/characters`, `/settings` | ⬜ W0 | ⬜ pending |
| 01-02-01 | 02 | 1 | SHELL-02 | — | N/A | manual | `bun run build` produces `build/index.html`; all routes work | ⬜ W0 | ⬜ pending |
| 01-02-02 | 02 | 1 | SHELL-03 | — | N/A | manual | `tauri build` produces platform installer; production app runs without errors | ⬜ W0 | ⬜ pending |
| 01-03-01 | 03 | 2 | SHELL-03 | T-1-01 | CSP blocks inline scripts in production | manual | Verify production CSP: no `unsafe-eval` in script-src | ⬜ W0 | ⬜ pending |
| 01-03-02 | 03 | 2 | SHELL-03 | T-1-02 | DevTools disabled in production | manual | Verify `devtools: false` in production window config works | ⬜ W0 | ⬜ pending |
| 01-04-01 | 04 | 2 | SETT-03 | — | N/A | manual | F9 toggles sidebar in Tauri window | ⬜ W0 | ⬜ pending |
| 01-04-02 | 04 | 2 | CHAT-05 | — | N/A | manual | Enter/Ctrl+Enter sends message; Shift+Enter inserts newline | ⬜ W0 | ⬜ pending |
| 01-04-03 | 04 | 2 | SHELL-01 | — | N/A | manual | Window drag via AppHeader `.drag-region` works | ⬜ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] No automated test framework to install — Phase 1 validation is entirely manual
- [ ] Future phases should add Vitest for automated Svelte component testing
- [ ] Future phases should add `cargo test` for Rust backend testing

*Existing infrastructure does NOT cover phase requirements — all verification is manual.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Native desktop window launches | SHELL-01 | Requires Tauri runtime; cannot verify in browser | Run `bun tauri dev`; verify window opens with title "Revelry", size ~1200×800, resizable |
| All SvelteKit routes work in webview | SHELL-02 | Requires Tauri webview; cannot verify in browser | Navigate to `/`, `/chat/[id]`, `/characters`, `/settings` — all render without errors |
| Production build runs correctly | SHELL-03 | Requires `tauri build` output; cannot verify in dev mode | Run `bun tauri build`; install/run the produced binary; verify no console errors |
| DevTools disabled in production | SHELL-03 | Requires production build to verify | In production build, right-click → no "Inspect" option; DevTools don't open |
| Keyboard shortcuts work in Tauri | SETT-03, CHAT-05 | Requires Tauri key event handling | Press F9 → sidebar toggles; Enter → sends message; Shift+Enter → newline |
| CSP blocks unauthorized scripts | SHELL-03 | Requires production CSP enforcement | In production build, verify no CSP violation errors in console |
| Window drag region works | SHELL-01 | Requires native window chrome | Drag from AppHeader area → window moves; drag from buttons → no movement |

---

## Validation Sign-Off

- [ ] All tasks have verification method (manual or automated)
- [ ] Sampling continuity: every task has at least one verification check
- [ ] Wave 0 covers all MISSING references (manual-only acknowledged)
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending

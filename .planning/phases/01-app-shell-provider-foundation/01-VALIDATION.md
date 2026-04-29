---
phase: 1
slug: app-shell-provider-foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-29
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[cfg(test)]` + `cargo test` |
| **Config file** | `Cargo.toml` (built-in) |
| **Quick run command** | `cargo test --lib 2>&1` |
| **Full suite command** | `cargo test 2>&1` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib 2>&1`
- **After every plan wave:** Run `cargo test 2>&1`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | UISL-01, UISL-02 | — | N/A | build | `cargo build 2>&1` | ❌ W0 | ⬜ pending |
| 01-01-02 | 01 | 1 | UISL-03, UISL-04 | — | N/A | build | `cargo build 2>&1` | ❌ W0 | ⬜ pending |
| 01-02-01 | 02 | 1 | PROV-01, PROV-02, PROV-05 | T-01-01 | API key never in DB | unit | `cargo test --lib db::providers 2>&1` | ❌ W0 | ⬜ pending |
| 01-02-02 | 02 | 1 | PROV-01, PROV-02 | T-01-02 | Keyring error handled gracefully | unit | `cargo test --lib secrets 2>&1` | ❌ W0 | ⬜ pending |
| 01-03-01 | 03 | 2 | UISL-05, UISL-06 | — | N/A | build+manual | `cargo build 2>&1` | ❌ W0 | ⬜ pending |
| 01-03-02 | 03 | 2 | PROV-01, PROV-02 | — | N/A | build | `cargo build 2>&1` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/db/mod.rs` — DB Worker module with migration setup (stubs for provider CRUD tests)
- [ ] `src/db/providers.rs` — Provider CRUD query module
- [ ] `src/secrets/mod.rs` — Keyring module stubs
- [ ] `Cargo.toml` — Created with all Phase 1 dependencies

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Window renders with Adwaita styling | UISL-04 | GTK4 requires display server | Run `cargo run`, verify window shows with Adwaita theme |
| Sidebar collapses at 720px breakpoint | UISL-02 | Window resize requires display | Resize window below 720px, verify sidebar goes to overlay mode |
| F9 toggles sidebar | UISL-03 | Keyboard shortcut requires display | Press F9, verify sidebar toggles |
| Provider test shows spinner/success/failure | PROV-02 | Visual feedback requires display | Add OpenAI provider, click Test Connection, verify spinner then result |
| Empty states show correct copy | UISL-04 | Visual text requires display | Launch with no providers, verify "Welcome to Revelry" appears |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending

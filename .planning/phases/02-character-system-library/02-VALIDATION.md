---
phase: 02
slug: character-system-library
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-29
---

# Phase 02 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`#[test]` + `cargo test`) |
| **Config file** | Cargo.toml (already configured) |
| **Quick run command** | `cargo test --lib 2>&1 | tail -20` |
| **Full suite command** | `cargo test 2>&1 | tail -30` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | CHAR-01 | — | N/A | unit | `cargo test --lib domain::character` | ⬜ W0 | ⬜ pending |
| 02-01-02 | 01 | 1 | CHAR-01, CHAR-07 | T-02-01 | Input validation on character fields | unit | `cargo test --lib db::characters` | ⬜ W0 | ⬜ pending |
| 02-02-01 | 02 | 2 | CHAR-02, CHAR-03, CHAR-04, CHAR-05 | T-02-02 | PNG chunk size limits, JSON payload sanitization | unit | `cargo test --lib service::card_io` | ⬜ W0 | ⬜ pending |
| 02-02-02 | 02 | 2 | CHAR-06, CHAR-07 | — | N/A | build | `cargo build` | ⬜ W0 | ⬜ pending |
| 02-03-01 | 03 | 3 | CHAR-01 | — | N/A | build | `cargo build` | ⬜ W0 | ⬜ pending |
| 02-03-02 | 03 | 3 | CHAR-01, CHAR-06 | — | N/A | build+grep | `cargo build && grep -c "CharacterEditor" src/ui/mod.rs` | ⬜ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. Rust `#[test]` is built-in, all dependencies are in Cargo.toml.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Character card appears in grid after creation | CHAR-01 | Visual GTK4 rendering | Create character → see it in CharactersPanel grid |
| Import from PNG shows in library | CHAR-02 | File picker interaction | Import PNG → see character in library |
| Export creates valid PNG with embedded data | CHAR-04 | File system verification | Export → re-import → verify roundtrip |
| Search returns matching characters instantly | CHAR-07 | Perceived latency | Add 5+ characters → search → verify instant results |
| Tab switcher shows Characters/Personas | D-03 | Visual tab presence | See two tabs in CharactersPanel |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (none — existing infra)
- [x] No watch-mode flags
- [x] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter (set after execution)

**Approval:** pending

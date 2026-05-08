# Phase 6: Generation Controls, Regex & Data - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-08
**Phase:** 06-generation-controls-regex-data
**Areas discussed:** Preset parameter scope, Regex script lifecycle, Backup/restore format, Starter presets & defaults

---

## Preset Parameter Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Full GENR-01 only | 6 numeric/sequence params + name. System prompt stays on character card. | ✓ |
| Include system prompt | Presets can optionally override character's system prompt. | |
| Include jb_override | Configurable system prompt prefix that prepends to character prompt. | |

**User's choice:** Full GENR-01 only
**Notes:** Presets are purely about generation behavior, not content. Clean separation of concerns.

| Option | Description | Selected |
|--------|-------------|----------|
| Extend with raw API | Use rig builder for supported params, pass unsupported via provider-specific config. | ✓ |
| Only support rig-native | Only expose temperature and max_tokens. | |
| Fork/wrap rig calls | Custom request layer that wraps rig. | |

**User's choice:** Extend with raw API
**Notes:** Silently skip unsupported params per-provider.

| Option | Description | Selected |
|--------|-------------|----------|
| Match persona pattern | Settings (global) + Character Editor (per-char) + Chat Header (per-conv). | ✓ |
| Settings + Chat only | Global + per-conversation. No per-character. | |
| Character Editor only | Per-character + global default. No mid-conversation switching. | |

**User's choice:** Match persona pattern
**Notes:** Same UX pattern as persona assignment hierarchy — consistent.

---

## Regex Script Lifecycle

| Option | Description | Selected |
|--------|-------------|----------|
| Separate DB table | regex_scripts table with character_id FK. Same pattern as lorebook_entries. | ✓ |
| In extensions JSON | Store in character's extensions.regex_scripts JSON column. | |

**User's choice:** Separate DB table
**Notes:** Import/export serializes to/from extensions.regex_scripts for SillyTavern compatibility.

| Option | Description | Selected |
|--------|-------------|----------|
| Sequential ordered | Scripts run in sort_order sequence (pipe chain). | ✓ |
| Parallel + merge | All scripts run on original output simultaneously. | |

**User's choice:** Sequential ordered
**Notes:** Each script's output becomes next script's input. User controls order.

| Option | Description | Selected |
|--------|-------------|----------|
| Block + warn | Use size_limit() to cap backtracking. Block saving if pattern fails. | ✓ |
| Warn only | Warning for dangerous patterns but allow saving anyway. | |
| Auto-timeout at runtime | Short execution timeout when running scripts. Skip on timeout. | |

**User's choice:** Block + warn
**Notes:** Catch dangerous patterns at design time, not runtime.

| Option | Description | Selected |
|--------|-------------|----------|
| In regex editor | Test panel at bottom of editor. Input field + live before/after display. | ✓ |
| Separate test window | Dedicated test dialog opened from editor. | |
| Chat context test | Test against last AI response in current conversation. | |

**User's choice:** In regex editor
**Notes:** Inline with editing flow — test as you write.

---

## Backup/Restore Format

| Option | Description | Selected |
|--------|-------------|----------|
| Full .dump output | Pure SQL text file, recreates all tables and inserts. | ✓ |
| Filtered SQL dump | Only user-data tables, skip provider configs. | |

**User's choice:** Full .dump output
**Notes:** Most portable — can be loaded by any SQLite tool.

| Option | Description | Selected |
|--------|-------------|----------|
| DB + avatars + settings | Full SQLite file, avatar directory, app_settings JSON. | ✓ |
| DB + avatars only | SQLite file and avatar images. | |
| Full data directory mirror | Entire data directory including logs. | |

**User's choice:** DB + avatars + settings
**Notes:** Everything needed to fully restore app state on a new machine.

| Option | Description | Selected |
|--------|-------------|----------|
| Replace + confirm | Import replaces all data with confirmation dialog. | ✓ |
| Merge (smart) | Detect conflicts, let user choose per-conflict. | |
| Side-by-side | Import into separate DB, selective copy. | |

**User's choice:** Replace + confirm
**Notes:** Destructive but simple and predictable — matches SillyTavern behavior.

| Option | Description | Selected |
|--------|-------------|----------|
| Run migrations | Restore DB, then run standard migration suite. Older auto-upgrades. | ✓ |
| Version gate | Only allow same or older versions. Newer rejected. | |
| Best effort | Try to import regardless, missing columns get defaults. | |

**User's choice:** Run migrations
**Notes:** Older backups auto-upgrade. Newer backups fail with "incompatible version" message.

---

## Starter Presets & Defaults

| Option | Description | Selected |
|--------|-------------|----------|
| SillyTavern-inspired | Creative: temp 1.2. Balanced: temp 0.8. Precise: temp 0.3. | ✓ |
| OpenAI-influenced | Creative: temp 1.0. Balanced: temp 0.7. Precise: temp 0.2. | |
| You decide | Let planner pick sensible defaults. | |

**User's choice:** SillyTavern-inspired
**Notes:** Matches community expectations for roleplay apps.

| Option | Description | Selected |
|--------|-------------|----------|
| Provider defaults | Send no parameter overrides when no preset assigned. | ✓ |
| Balanced preset | Always use Balanced as implicit default. | |
| Minimal safe values | Hardcoded temp 0.8, top_p 0.9, no penalties. | |

**User's choice:** Provider defaults
**Notes:** Respects each provider's own tuning.

| Option | Description | Selected |
|--------|-------------|----------|
| Read-only built-ins | Cannot edit or delete. Users can duplicate as starting point. | ✓ |
| Fully editable | Users can edit or delete built-in presets. | |
| Editable, not deletable | Can modify values but can't delete. | |

**User's choice:** Read-only built-ins
**Notes:** Prevents accidental deletion, ensures new users always have defaults.

---

## the agent's Discretion

- Exact regex_scripts table column types and naming
- Migration numbering (next after M8)
- How to pass unsupported params to rig provider-specific request config
- Backup file naming convention
- ZIP archive internal directory structure
- Preset editor UI layout (sliders vs spinbuttons vs text entry)
- Stop sequences UI (comma-separated text entry vs tag-style list)
- Provider-specific parameter support discovery at runtime
- Default max_tokens per model

## Deferred Ideas

- Vault mode / database encryption (v2 / SECU-01, SECU-02)
- Batch character import/export improvements (v2 / CHAR-A04)
- Per-provider tokenizer dispatch (future polish)
- {{char.field}} template expansion beyond {{char}} (deferred from Phase 5)
- Preset sharing/export between users (future feature)
- Regex script import/export as standalone files (future polish)
- Incremental/differential backups (future enhancement)
- Cloud backup sync (contradicts local-first core value)

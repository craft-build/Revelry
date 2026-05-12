# Phase 3: Editors - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-12
**Phase:** 03-Editors
**Areas discussed:** Editor container pattern, Character editor flow, Avatar handling, Lorebook editor scope

---

## Editor Container Pattern

| Option | Description | Selected |
|--------|-------------|----------|
| Modal dialog (max-w-2xl) | Bits.ui Dialog-based, opens over library page, scrollable content | ✓ |
| Full page | Navigate to route, maximum room, lose context | |
| Slide-over panel | Right-side drawer, middle ground, needs custom component | |

**User's choice:** 1 — Modal dialog (max-w-2xl)
**Notes:** User confirmed modal dialog keeps the lightweight "pop open, fill, close" feel despite having 15+ character fields.

| Option | Description | Selected |
|--------|-------------|----------|
| Separate dialogs | CharacterEditorDialog and PersonaEditorDialog as distinct components | ✓ |
| Shared base shell | Generic shell with specialized content slotted in | |
| You decide | Let Claude pick | |

**User's choice:** 1 — Separate dialogs

| Option | Description | Selected |
|--------|-------------|----------|
| Inside editor dialog | Delete button at bottom alongside Cancel/Save | |
| On card as kebab menu | Three-dot menu with Edit and Delete actions | ✓ |
| Both | Card menu for quick access + inside editor as backup | |

**User's choice:** 2 — Kebab menu on cards + confirmation dialog

---

## Character Editor Flow

| Option | Description | Selected |
|--------|-------------|----------|
| Single scrolling form | All fields visible, fast for power users | |
| Multi-step wizard (4 steps) | Thematic groupings, easier for newcomers | ✓ |
| Hybrid (collapsible sections) | Single screen with accordion groups | |

**User's choice:** 2 — Multi-step wizard

| Option | Description | Selected |
|--------|-------------|----------|
| 3 steps (broad) | Identity, Behavior, Greeting & Examples | |
| 4 steps (granular) | Identity, Personality & Scenario, Prompt Engineering, Greeting & Dialogues | ✓ |
| You decide | Let Claude pick | |

**User's choice:** 2 — 4 steps

| Option | Description | Selected |
|--------|-------------|----------|
| Click any step header | Free jump between steps | |
| Linear only | Must go step by step via Next/Back | |
| Linear for create, free-jump for edit | Different navigation per intent | ✓ |

**User's choice:** 3 — Linear for create, free-jump for edit

| Option | Description | Selected |
|--------|-------------|----------|
| Name only | Bare minimum, everything else defaults to empty | |
| Name + greeting | What's visible when starting a chat | |
| Name + description + greeting | Visible trio: card visible + chat greeting | |
| Everything on last step | Must reach Step 4 to save | |

**User's choice:** 3 — then corrected to include personality (Name + description + personality + greeting)
**Final required fields:** Name, description, personality, and first_mes (greeting)

---

## Avatar Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Tauri asset protocol | Files stored in app data dir, served via asset:// | ✓ |
| Base64 data URL | Embedded in DB, no backend file handling needed | |
| External URL | Paste a CDN link | |

**User's choice:** 1 — Tauri asset protocol

| Option | Description | Selected |
|--------|-------------|----------|
| Native file picker via Tauri | Click avatar → native OS file picker opens | ✓ |
| Drag-and-drop onto webview | Drop desktop file onto avatar area | |
| Both | Support click and drag-and-drop | |

**User's choice:** 1 — Native file picker only

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-cleanup on replace/delete | Delete old file when new one is saved or entity deleted | ✓ |
| Leave orphaned files | No risk of breaking references, but accumulates | |
| Reference-counting | Track references, delete only when count hits zero | |

**User's choice:** 1 — Auto-cleanup on replace/delete (with note: revisit if clone/duplicate is added later)

---

## Lorebook Editor Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Inline tab in character editor | Part of character creation wizard | |
| Separate standalone editor | Named lorebook resources, linked to characters | ✓ |
| Both | Inline for quick management + standalone for bulk editing | |

**User's choice:** 2 — Separate standalone editor
**Notes:** User explicitly noted "this is exactly what I want — also the same lorebook could be attached to multiple characters." This changes the schema from Phase 2's per-character entries to a many-to-many junction model.

| Option | Description | Selected |
|--------|-------------|----------|
| Inside the lorebook editor | Test panel within the standalone editor | ✓ |
| Separate tool/modal | "Test keywords" button somewhere else in app | |
| Inline per-entry | Small test box while editing one entry | |

**User's choice:** 1 — Inside the lorebook editor

---

## Claude's Discretion

- Exact positioning of "Test Keywords" panel (side panel vs. bottom accordion vs. tab)
- Step header navigation UI style (dots vs. numbered tabs vs. stepper bar)
- Avatar upload button visual design (upload icon, overlay, replaceable image)
- Loading state during save (optimistic vs. spinner vs. disabled form)
- Error display inside the wizard (inline per-field, top-level banner, or step-level summary)
- Whether lorebook editor opens as dialog (like character/persona) or as settings-style page

---

## Deferred Ideas

- Drag-and-drop avatar upload (deferred to future enhancement)
- Character import/export (PNG card I/O) — Phase 7
- Lorebook full-text search — Phase 8
- Lorebook context injection in prompts — Phase 9
- Duplicate/clone character or persona (copy button currently a no-op)

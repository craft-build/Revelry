# Phase 4: Personas & Template Resolution - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions captured in CONTEXT.md — this log preserves the analysis.

**Date:** 2026-05-05
**Phase:** 04-personas-template-resolution
**Mode:** discuss (interactive)
**Areas discussed:** 4

## Gray Areas Identified

1. **Persona schema & fields** — Editor layout, required vs optional fields, relationships format
2. **Template variable resolution** — Syntax style, resolution timing, {{char}} expansion
3. **Persona injection format** — Plain text vs XML vs Markdown, how it appears in the prompt
4. **Persona assignment UX** — Where in UI, how hierarchy is surfaced to user

## Discussion Log

### Area 1: Persona Schema & Fields

**Question 1:** How should the persona editor work?
- Options: Single-page compact form / Multi-step wizard
- **Selected:** Multi-step wizard (AdwPreferencesWindow sub-pages) — Basic Info → Background → Personality → Relationships → Review
- **Reasoning:** Better for first-time users, more GNOME-like, aligns with character editor patterns

**Question 2:** Which fields should be required vs optional?
- Options: Name only / Name + personality / All 8 fields
- **Selected:** Name + personality required, rest optional
- **Rationale:** Personality is the minimum viable identity for roleplay injection; lets users create minimal personas and fill out later

**Question 3:** How should the 'relationships' field work?
- Options: Freeform text / Structured list with entries
- **Selected:** Freeform text (like personality field)
- **Rationale:** Simpler to implement, consistent with character field patterns, sufficient for prompt injection

### Area 2: Template Variable Resolution

**Question 1:** How should {{persona.*}} variables be structured?
- Options: Dot-path style / Flat names with underscore
- **Selected:** Dot-path style: {{persona.field_name}} (e.g., {{persona.appearance}})
- **Rationale:** More intuitive, extensible, matches nested field access patterns

**Question 2:** When and where should template variables be resolved?
- Options: Prompt-assembly-time only / UI display + prompt-assembly
- **Selected:** Prompt-assembly-time only (raw values stored, resolved at send)
- **Rationale:** Preserves SillyTavern compatibility, keeps stored data portable, avoids confusion about when resolution happens

**Question 3:** Should we add {{char.name}} as a template variable?
- Options: {{char}} only (as today) / Expand to {{char.field}}
- **Selected:** {{char}} resolves to character name only (as it does today)
- **Rationale:** Avoids recursive resolution complexity; {{char.field}} expansion deferred to Phase 5/6

### Area 3: Persona Injection Format

**Question 1:** How should persona data appear in the AI prompt?
- Options: Plain text / XML / Markdown
- **Selected:** Same formatting as character system prompt (plain text with labeled fields)
- **Rationale:** Consistency with existing prompt assembly; works with any model

**Question 2:** What label should introduce the persona block?
- Options: "User Persona" section header / "You are interacting with {{user}}, who is..."
- **Selected:** "You are interacting with {{user}}, who is..."
- **Rationale:** Roleplay-focused framing, matches SillyTavern conventions, keeps the AI in-character

### Area 4: Persona Assignment UX

**Question 1:** Where should users access and manage personas?
- Options: CharactersPanel tab switcher / Dedicated view
- **Selected:** CharactersPanel tab switcher (Characters + Personas tabs)
- **Rationale:** Reuses existing placeholder, keeps persona management co-located with character management

**Question 2:** How should persona assignment work across the three levels?
- Options: Three separate UI locations / Centralized Persona Manager
- **Selected:** Three separate UI locations matching the hierarchy (Settings, Character Editor, Chat Header)
- **Rationale:** Maps directly to resolution hierarchy, progressive disclosure, standard GNOME pattern

## Decisions Summary

| Decision | Choice |
|----------|--------|
| Editor layout | Multi-step wizard |
| Required fields | Name + personality |
| Relationships format | Freeform text |
| Variable syntax | {{persona.field_name}} |
| Resolution timing | Prompt-assembly-time only |
| {{char}} expansion | Name only (no dot-path) |
| Injection format | Plain text with labeled fields |
| Injection header | "You are interacting with {{user}}, who is..." |
| Management location | CharactersPanel tab switcher |
| Assignment UX | Settings + Character Editor + Chat Header |
| Hierarchy resolution | conversation → character → global → none |

## Scope Creep Redirected

No scope creep detected during discussion. All topics stayed within Phase 4 boundaries.

## Post-Discussion Notes

- Persona editor should reuse CharacterEditorInit enum pattern (New/Edit modes)
- Database migration will be M7 (sequential after existing M6)
- Persona block injection point in PromptEngine is between character system prompt and chat history
- Consider adding a "no persona" option explicitly in assignment dropdowns for clarity

---

*Phase: 04-personas-template-resolution*
*Discussion completed: 2026-05-05*

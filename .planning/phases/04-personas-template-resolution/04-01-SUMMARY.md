---
phase: 04-personas-template-resolution
plan: 01
subsystem: data-foundation
tags: [persona, database, template, prompt]
requires: [PERS-01, PERS-02, PERS-05, PERS-06, CHAR-08]
provides: [Persona domain model, Persona CRUD, Template resolver, Persona block builder]
affects: [src/domain/persona.rs, src/db/personas.rs, src/service/template.rs, src/service/prompt.rs]
tech-stack.added: []
patterns:
  - "Persona struct follows CharacterCardV2 pattern (UUID v4, timestamps, serde)"
  - "Tags stored as JSON array TEXT in DB (same as character alternate_greetings)"
  - "Template resolution via simple string replace (single-pass, no recursion)"
  - "PromptEngine injects persona block between character system prompt and history"
key-files.created:
  - src/domain/persona.rs
  - src/db/personas.rs
  - src/service/template.rs
key-files.modified:
  - src/db/migrations.rs (M007)
  - src/db/models.rs (row_to_persona)
  - src/db/mod.rs (DbInput/DbOutput variants)
  - src/domain/mod.rs
  - src/service/prompt.rs (persona-aware assemble)
  - src/service/mod.rs
  - src/ui/chat/mod.rs (PromptEngine usage)
key-decisions:
  - "Template variables resolved at prompt-assembly-time only (raw in DB)"
  - "Empty persona fields omitted from prompt block (no placeholder text)"
  - "Assignment hierarchy: conv -> char -> global -> none"
requirements-completed: [PERS-01, PERS-02, PERS-05, PERS-06, CHAR-08]
duration: "14 min"
completed: "2026-05-06"
---

# Phase 04 Plan 01: Persona Data Foundation Summary

## Overview

Persona domain model with 8 structured fields, SQLite personas table (Migration 007), full CRUD with assignment resolution, template resolver for `{{user}}`/`{{char}}`/`{{persona.*}}` variables, and persona block builder for prompt injection.

## Tasks Completed

### Task 1: Persona domain model and database layer
- Created `src/domain/persona.rs` with `Persona` struct (id, name, appearance, personality, background, role_description, relationships, speech_style, tags, created_at, updated_at)
- `Persona::new(name, personality)` generates UUID v4, sets timestamps, defaults optional fields
- Added Migration 007: `personas` table + `default_persona_id` on `characters` table
- Created `src/db/personas.rs` with insert, load_all, load_by_id, update, delete
- Added `row_to_persona` to `src/db/models.rs`
- Added `LoadPersonas`, `SavePersona`, `DeletePersona`, `SetGlobalDefaultPersona` to `DbInput`
- Added `PersonasLoaded`, `PersonaSaved`, `PersonaDeleted`, `GlobalDefaultPersonaSet` to `DbOutput`
- Implemented `resolve_persona_for_conversation` with assignment hierarchy (conv -> char -> global -> none)
- **12 tests passing**

### Task 2: Template resolver and persona prompt block builder
- Created `src/service/template.rs` with `resolve_variables()` for all template variable types
- `{{user}}` → persona name or "User", `{{char}}` → character name
- `{{persona.name}}` through `{{persona.speech_style}}` → corresponding field
- Unknown variables left untouched; empty fields resolve to empty string
- Created `build_persona_block()` with "You are interacting with {{user}}, who is:" header
- Empty fields omitted from block
- Updated `PromptEngine::assemble()` and `assemble_initial()` to accept `Option<&Persona>`
- Persona block injected between character system prompt and conversation history
- Final preamble resolves all template variables
- Updated `src/ui/chat/mod.rs` to use `PromptEngine::assemble()` instead of `build_system_prompt()`
- **14 template tests + 14 prompt tests passing**

## Verification

- `cargo test --lib`: **160 passed, 0 failed, 0 regressions**
- `cargo build`: Compiles with no errors

## Deviations from Plan

None - plan executed exactly as written.

## Next Plan

Ready for **04-02**: Persona editor wizard, library list, and assignment UI.

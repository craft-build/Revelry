---
phase: 05-context-engineering-lorebook
plan: 02
subsystem: context-engineering
tags: [prompt-assembly, lorebook-activation, persona-compression, budget-enforcement]
dependency_graph:
  requires: ["05-01"]
  provides: ["PromptEngine::assemble_with_budget", "activate_lorebook_entries", "inject_lorebook_entries", "PersonaCompressionTier"]
  affects: [src/service/prompt.rs, src/service/template.rs, src/service/lorebook.rs]
tech_stack:
  added: [lorebook-activation-engine, persona-compression-tiers]
  patterns: [priority-layer-assembly, oldest-first-truncation, binary-search-truncation]
key_files:
  created:
    - src/service/lorebook.rs
  modified:
    - src/service/prompt.rs
    - src/service/template.rs
    - src/service/mod.rs
decisions:
  - Lorebook budget allocated as 20% of total context window for entry activation
  - Binary search for token-budget truncation (efficient character-level search)
  - Persona compression tier selected greedily (first tier that fits)
  - Post-history instructions dropped entirely if they don't fit (not truncated)
  - Empty keyword strings ignored in matching (prevents false positives)
metrics:
  duration: 3min
  completed: 2026-05-08
  tasks_completed: 2
  tests: 216
  files_created: 1
  files_modified: 3
---

# Phase 5 Plan 02: Budget-Aware Prompt Assembly & Lorebook Activation Summary

Budget-aware prompt assembly with persona compression and lorebook activation engine — the core intelligence layer for context engineering.

## One-Liner

Priority-layer prompt assembly with 4-tier persona compression, oldest-first history truncation, OR keyword lorebook activation with recursive support and per-entry token budgets.

## Completed Tasks

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Budget-aware prompt assembly with persona compression and history truncation | 74a3c36 | src/service/prompt.rs, src/service/template.rs |
| 2 | Lorebook activation engine with keyword matching and recursive activation | ad57242 | src/service/lorebook.rs, src/service/mod.rs |

## What Was Built

### Task 1: Budget-aware Prompt Assembly

- `PromptEngine::assemble_with_budget()` — the main budget-aware entry point following D-04 priority layer order:
  1. **Sticky layer** (system prompt + character card fields) — never truncated
  2. **Lorebook layer** — activated entries injected at their depth positions (20% of total context budget)
  3. **Persona layer** — compressed Full → Trimmed → Minimal → Essential progressively per D-06
  4. **History layer** — oldest messages truncated first per D-05
  5. **Post-history instructions** — placed after history, dropped entirely if no room
- `PersonaCompressionTier` enum (Full, Trimmed, Minimal, Essential) in `src/service/template.rs`
- `build_persona_block_compressed()` for tiered persona block generation
- `select_persona_tier()` auto-selects best compression tier based on remaining budget
- `truncate_history_oldest_first()` for D-05 oldest-first truncation
- `AssembledPrompt` now has `context_utilization`, `truncated_messages`, `active_lorebook_count` fields
- Existing `assemble()` and `assemble_initial()` preserved for backward compatibility

### Task 2: Lorebook Activation Engine

- `activate_lorebook_entries()` — the core lorebook activation function:
  - OR keyword matching (any keyword or secondary_term triggers activation, case-insensitive)
  - Configurable scan depth (last N messages) per D-10
  - Recursive activation (activated entries' content can trigger other entries, up to max depth) per D-12
  - Per-entry token budget enforcement (truncate from end, overflow frees tokens) per D-13
  - Entries without explicit budget share remaining lorebook allocation
  - Returns `Vec<ActivatedEntry>` sorted by `sort_order`
- `inject_lorebook_entries()` — injects activated entries into the prompt:
  - Depth 0 entries: append to preamble (after character definition)
  - Depth > 0 entries: insert system messages at N messages from end
- `ActivatedEntry` struct with `entry`, `injected_content`, `effective_depth`
- `keyword_matches()` helper for OR logic, case-insensitive matching
- `truncate_to_token_budget()` using binary search for efficient truncation

## Test Coverage

- **216 total tests** (up from 182 in 05-01)
- 16 new budget-aware prompt tests (context_utilization, sticky layers, 4 persona tiers, history truncation, truncated count, post-history placement, lorebook activation/deactivation)
- 6 new persona compression template tests (full, trimmed, minimal, essential, empty name)
- 17 new lorebook activation tests (keyword match, no match, OR logic, disabled, scan depth, recursive activation, max depth, token budgets, shared budget, depth insertion, secondary terms, preamble injection, history injection, sort order, keyword helper tests)
- All existing tests preserved and passing

## Decisions Made

1. **Lorebook budget = 20% of total context** — prevents lorebook from overwhelming other layers while giving enough room for world-building
2. **Binary search for token truncation** — efficient character-level search to find longest content prefix that fits budget
3. **Greedy persona tier selection** — first tier that fits remaining budget (no lookahead optimization needed)
4. **Post-history instructions dropped entirely** — not truncated, either fits or doesn't (simplest approach)
5. **Empty keywords ignored in matching** — prevents false positive matches on empty strings
6. **Lorebook entries injected as system messages** — for depth > 0 entries, they become system messages in chat history

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — all functionality fully implemented.

## Threat Flags

No new threat surface beyond what was documented in the plan's threat model. Lorebook content injection is user-authored data (accepted risk per T-05-05). Recursive depth limit prevents DoS per T-05-04. Keyword matching uses simple string contains, not regex (no ReDoS risk per T-05-06).

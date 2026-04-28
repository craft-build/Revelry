# Persona — Project Guide

## What This Is

Persona is a native desktop AI roleplay application built with Tauri 2.0. It provides a GNOME-inspired, distraction-free interface for rich, sustained AI character interactions with full data sovereignty — all data stays local.

## Quick Reference

- **Stack:** Tauri 2.0, React 18, TypeScript 5, Rust, SQLite
- **Design:** GNOME Human Interface Guidelines (Adwaita palette)
- **Core Value:** Users own their data completely — no cloud, no accounts, no telemetry

## Project Files

| File | Purpose |
|------|---------|
| `.planning/PROJECT.md` | Living project context |
| `.planning/REQUIREMENTS.md` | v1/v2 requirements with traceability |
| `.planning/ROADMAP.md` | 4-phase execution roadmap |
| `.planning/STATE.md` | Current project state |
| `.planning/research/SUMMARY.md` | Technical decisions from domain research |
| `.planning/config.json` | Workflow preferences |

## Current Phase

**Phase 1: Foundation**
- Status: Not started
- Goal: Infrastructure, database schema, UI shell
- Requirements: 12 (INFRA, PROV-02, UI shell)

## Critical Constraints

1. **Security:** API keys ONLY in OS keychain (Windows Credential Manager, macOS Keychain)
2. **Performance:** 5-10MB binary, <100MB memory footprint
3. **Privacy:** No telemetry, no cloud sync, no accounts
4. **Compatibility:** CharacterCardV2 full spec support (PNG + JSON)

## Key Decisions

- Tauri over Electron (smaller, faster, native feel)
- SQLite for local storage (single-file portable)
- OpenAI-compatible API (20+ providers supported)
- GNOME minimalist design (content-first, reduced cognitive load)

## GSD Workflow

```
/gsd-discuss-phase 1  → Gather context for Phase 1
/gsd-plan-phase 1     → Create detailed plan
/gsd-execute-phase 1  → Execute all plans in Phase 1
```

## Architecture Notes

- **IPC:** Commands for CRUD, Channels for streaming, Events for notifications
- **State:** Zustand (UI) + TanStack Query (server) + SQLite (persistence)
- **Database:** tauri-plugin-sql with connection pooling
- **Streaming:** Tauri Channels for SSE token streaming

## Pitfalls to Avoid

- SQLite connections are NOT Send — use Mutex or connection pool
- Tauri 2.0 requires explicit capabilities configuration
- Unlisten to Tauri events in useEffect cleanup (memory leaks)
- Paginate large payloads (IPC has size limits)

---
*Generated: 2025-04-27*

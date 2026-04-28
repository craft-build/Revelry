# State: Persona

**Project:** Persona - Desktop AI Roleplay Application  
**Current Milestone:** v1.0.0  
**Last Updated:** 2025-04-27

---

## Project Reference

**Core Value:** Users can engage in uncensored, private AI character roleplay with full data sovereignty — their conversations, characters, and creative content remain entirely on their device.

**Target Users:**
- Casual roleplay hobbyists seeking alternatives to Character.ai/SpicyChat
- AI researchers/hobbyists running local models (Ollama, LM Studio)
- Users valuing privacy and data sovereignty

**Key Constraints:**
- Tech Stack: Tauri 2.0, React 18, TypeScript 5, Rust 1.75+, SQLite
- Design System: GNOME Human Interface Guidelines (Adwaita palette)
- Security: API keys in OS keychain only, never in database or config files
- Performance: Target 5-10MB binary, <100MB memory footprint

---

## Current Position

| Attribute | Value |
|-----------|-------|
| **Phase** | 1 |
| **Phase Name** | Foundation |
| **Status** | Not started |
| **Plan** | None yet |
| **Progress** | 0% |

**Progress Bar:**
```
[░░░░░░░░░░░░░░░░░░] 0% (0/4 phases)
```

---

## Performance Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Phases Complete | 0 | 4 |
| Requirements Delivered | 0 | 47 |
| Lines of Code | 0 | — |
| Test Coverage | 0% | >70% |
| Binary Size | — | 5-10MB |
| Memory Footprint | — | <100MB |

---

## Accumulated Context

### Key Decisions Made

| Decision | Date | Rationale |
|----------|------|-----------|
| Tauri over Electron | 2025-04-27 | 5-10MB vs 150+MB binary, native webview, better performance |
| GNOME design philosophy | 2025-04-27 | Content-first, progressive disclosure, reduced cognitive load |
| CharacterCardV2 format | 2025-04-27 | Maximum interoperability with SillyTavern ecosystem |
| OpenAI-compatible API | 2025-04-27 | Industry standard, supports 20+ providers with one interface |
| SQLite over client-server DB | 2025-04-27 | Zero-config, single-file portability, perfect for desktop |
| Rust backend for sensitive ops | 2025-04-27 | Secure key storage, type safety, native performance |
| Local-first, no account | 2025-04-27 | Data sovereignty, removes adoption barrier |
| 4-phase coarse roadmap | 2025-04-27 | Infrastructure → Core → Advanced → Polish |

### Open Questions

| Question | Status | Next Action |
|----------|--------|-------------|
| GNOME Design Appeal | Open | Validate during Phase 2 UI prototyping |
| Conversation Branching UX | Open | Research during Phase 3 planning |
| Vault Mode Encryption Approach | Open | Technical spike in v2 |
| Lorebook Performance at Scale | Open | Benchmark during Phase 3 |

### Known Blockers

| Blocker | Phase | Mitigation |
|---------|-------|------------|
| None currently | — | — |

### Technical Debt

| Item | Phase Introduced | Resolution Plan |
|------|------------------|-----------------|
| None yet | — | — |

---

## Session Continuity

### Last Action
Created roadmap with 4 phases covering all 47 v1 requirements.

### Next Actions
1. Plan Phase 1: Foundation with specific pitfall mitigations
2. Define character card import/export acceptance criteria
3. Set up Tauri 2.0 project scaffold

### Context Preservation

**Project State Snapshot:**
- Requirements: 47 v1 requirements defined and mapped
- Research: Complete (Tauri 2.0, SQLite, CharacterCardV2)
- Roadmap: 4-phase coarse structure approved
- Phase 1 ready for planning

**Files to Load on Resume:**
- `.planning/PROJECT.md` (core value, constraints)
- `.planning/REQUIREMENTS.md` (all requirements)
- `.planning/ROADMAP.md` (phase structure)
- `.planning/research/SUMMARY.md` (technical decisions)

---

## Phase History

| Phase | Status | Date | Notes |
|-------|--------|------|-------|
| — | — | — | — |

---

## Notes

**Milestone v1.0.0 Definition of Done:**
- [ ] All 4 phases complete
- [ ] 47 v1 requirements delivered
- [ ] Code signed for macOS and Windows
- [ ] CI/CD pipeline operational
- [ ] Basic documentation complete
- [ ] No critical bugs

**Deferred to v2:**
- Group chat (ADV-05)
- Regex scripting (ADV-06)
- Vault Mode encryption (SEC-01/02/03)
- Accessibility audit (PLATFORM-02)
- Internationalization (PLATFORM-03)
- Plugin system (ECO-01/02/03)

---

*This file updates automatically during phase transitions and milestone completions.*

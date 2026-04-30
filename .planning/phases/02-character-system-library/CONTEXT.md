# Phase 2 CONTEXT — Character System & Library

## Prior Decisions (from Phase 1)

| ID | Decision | Rationale |
|----|----------|-----------|
| P1-D-01 | Feature-based modules: app/, providers/, characters/, chat/, settings/ | Clean separation by domain |
| P1-D-02 | Tokio runtime inside Relm4 Worker | Single runtime, no bridge complexity |
| P1-D-03 | Single AppModel + active_view enum | Centralized state, view switching |
| P1-D-04 | First launch shows welcome + setup prompt | Guided onboarding |
| P1-D-13 | rusqlite_migration for schema versioning | Atomic migrations, user_version tracking |
| P1-D-14 | Minimal Phase 1 schema: providers + app_settings tables only | Incremental schema growth |
| P1-D-15 | keyring crate for secrets (not in DB/config) | Security constraint |
| P1-D-16 | UUID v4 for entity IDs | Consistent ID strategy |
| P1-CD | Widget hierarchy, error styling, loading indicators, migration file structure | Claude's discretion |

## Phase 2 Decisions

| ID | Decision | Rationale | Scope Impact |
|----|----------|-----------|--------------|
| D-01 | Unified character editor component that adapts between creation wizard mode (AdwPreferencesWindow) and edit sliding-panel mode (AdwNavigationView) | Shared validation, single source of truth for form fields, less duplication | Medium — one complex component instead of two simpler ones |
| D-02 | Avatar images stored on filesystem in data directory; generated thumbnails in cache directory | Matches spec's "thumbnails cache directory". Avoids bloating SQLite. Efficient for image loading. | Medium — need file management utilities |
| D-03 | Tab switcher (Characters / My Personas) added to CharactersPanel now; personas tab shows empty state | Phase 4 fills in personas panel without restructuring layout. Avoids layout rework. | Low — just a tab container with placeholder |
| D-04 | Import via GtkFileChooserNative only; drag-and-drop deferred | Reduces Phase 2 scope. File picker covers all import needs. DnD is polish. | Low — single import path |

## Claude's Discretion

| ID | Area | Guidance |
|----|------|----------|
| CD-01 | Character domain model | Map CharacterCardV2 spec fields to Rust structs. Use serde for JSON serialization. Separate DB model from domain model (follow Phase 1 pattern in db/models.rs). |
| CD-02 | Database schema | Add `characters` table (see spec schema: id, name, description, personality, scenario, first_mes, mes_example, system_prompt, post_history_instructions, alternate_greetings, tags, extensions, creator, character_version, created_at, updated_at). Add FTS5 virtual table for instant search. Add migration file (003). |
| CD-03 | GtkFlowBox card widget | Each card: AdwAvatar (circular), name, creator, tags (as labels), brief description. Card click opens editor. |
| CD-04 | Avatar thumbnail generation | Use `image` crate to resize to thumbnail size (e.g., 64px). Cache in platform-appropriate cache directory (dirs::cache_dir()). |
| CD-05 | Search/filter | GtkSearchEntry → filter GtkFlowBox via GtkFilter + GtkFilterListModel. FTS5 query behind DbInput/DbOutput. Instant matching (debounce at 150ms). |
| CD-06 | Import validation | Validate CharacterCardV2 JSON schema via serde. Sanitize oversized payloads. Reject malformed PNG chunks. Show error toast on failure. |
| CD-07 | Export file naming | `{character_name}.{ext}` with slugified name. Prompt for location via GtkFileChooserNative. |
| CD-08 | CharactersPanel upgrade | Upgrade from SimpleComponent to full Component with model state (loaded characters, search filter, selected character, edit state). Follow SettingsPanel pattern for DbInput forwarding. |
| CD-09 | Alternate greetings | Store as JSON array in DB. Editor shows editable list with add/remove rows. |
| CD-10 | Tag system | Tags stored as JSON array in characters table. Not a separate tags table — simpler for V1, matches CharacterCardV2 spec format. |

## Canonical References

1. `docs/Revelry_Design_Specification_v2.0.docx` — §7 Character Card System, §7.1 Card Format, §7.3 Card Import/Export, §7.4 Character Creation Wizard, §10.3 Characters View, §11.2 Character Management
2. `.planning/REQUIREMENTS.md` — CHAR-01 through CHAR-07
3. `.planning/ROADMAP.md` — Phase 2 definition
4. `.planning/research/STACK.md` — png, image, base64, serde_json for card handling

## Key Design Spec Directives

- CharacterCardV2 spec is the canonical card format
- PNG cards: embedded JSON in tEXt chunk (key "chara")
- JSON cards: V1/V2 with auto-migration
- Character creation: guided multi-step AdwPreferencesWindow (identity → personality → scenario → advanced)
- Character editing: sliding panel via AdwNavigationView
- Library: GtkFlowBox responsive grid with AdwAvatar, name, creator, tags, description
- Search: GtkSearchEntry + GtkFilterControls, SQLite FTS5 for instant matching
- Tab switcher: Characters / My Personas (personas tab deferred content)
- Keyboard: Ctrl+Shift+N for new character

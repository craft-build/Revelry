# Phase 2 RESEARCH — Character System & Library

**Researched:** 2026-04-29
**Status:** Complete
**Discovery Level:** Level 1 — Quick Verification (single known tech stack, confirming API patterns)

## Standard Stack

All dependencies already in Cargo.toml from Phase 1 scaffold:
- `png = "0.18.1"` — PNG tEXt/iTXt chunk reading/writing for CharacterCardV2
- `image = "0.25.10"` — Avatar thumbnail generation
- `base64 = "0.22.1"` — Base64 encode/decode for PNG-embedded card data
- `serde = "1.0.228"` + `serde_json = "1.0.149"` — JSON serialization
- `rusqlite = "0.39.0"` with `bundled` — SQLite for character storage + FTS5
- `uuid = "1.23.1"` with `v4` — Character IDs
- `chrono = "0.4.44"` — Timestamps
- `gtk4`, `libadwaita`, `relm4` — UI framework (already established)

## Architecture Patterns (from Phase 1)

### Established Patterns to Follow
1. **DbService Worker**: Background-thread Worker processing DbInput/DbOutput messages serially (src/db/mod.rs)
2. **Domain model separation**: src/domain/ structs with serde, separate from DB row functions in src/db/models.rs
3. **Timestamp as TEXT**: NaiveDateTime stored as TEXT string in SQLite, parsed on read via format "%Y-%m-%d %H:%M:%S%.f"
4. **Serde JSON for complex types**: Enums/structs stored as serde_json TEXT in SQLite columns (like ProviderType)
5. **prepare_cached()** for all CRUD queries
6. **Component trait** (not SimpleComponent) for panels needing CommandOutput (established in Plan 03)
7. **Output message delegation**: Panels emit Output messages, AppModel routes to DbService
8. **Feature-based modules**: domain/, db/, service/, ui/

### Key Integration Points
- `src/db/mod.rs`: DbInput/DbOutput enums need new character variants
- `src/app.rs`: AppModel needs character panel integration, editor launch routing
- `src/ui/characters.rs`: Currently a placeholder SimpleComponent — needs full rewrite to Component
- `src/ui/mod.rs`: Module exports need updating for new character files
- `src/db/migrations.rs`: Next migration is #003 for characters table

## CharacterCardV2 Spec Analysis

### Data Fields (from design spec §7.1)
| Field | Type | Storage |
|-------|------|---------|
| name | String | TEXT NOT NULL |
| description | String | TEXT (can be long) |
| personality | String | TEXT |
| scenario | String | TEXT |
| first_mes | String | TEXT |
| mes_example | String | TEXT |
| system_prompt | String | TEXT |
| post_history_instructions | String | TEXT |
| alternate_greetings | Vec<String> | TEXT (JSON array) |
| tags | Vec<String> | TEXT (JSON array) |
| creator | String | TEXT |
| character_version | String | TEXT (default "2.0") |
| avatar_path | Option<String> | TEXT (filesystem path) |
| extensions | serde_json::Value | TEXT (JSON blob) |
| created_at | NaiveDateTime | TEXT |
| updated_at | NaiveDateTime | TEXT |

### PNG Import Format
- CharacterCardV2 data stored in tEXt chunk with key "chara"
- Value is base64-encoded JSON string
- Read using png crate's `Info::tEXt_chunk` iterator
- Write using png::Encoder with custom tEXt chunk

### JSON Import Format
- V2 format: direct match to CharacterCardV2 struct
- V1 format: has different field names (char_name, char_persona, char_greeting, etc.) — needs migration mapping
- V1→V2 migration: map old field names to new, set defaults for missing fields

### FTS5 Search
```sql
CREATE VIRTUAL TABLE characters_fts USING fts5(
    name, description, tags,
    content=characters,
    content_rowid=rowid
);
```
- Triggers to keep FTS in sync on INSERT/UPDATE/DELETE
- Search query: `SELECT rowid FROM characters_fts WHERE characters_fts MATCH ? ORDER BY rank`
- Instant matching with debounce at 150ms (CD-05)

## UI Architecture

### CharactersPanel (rewrite)
- Upgrade from SimpleComponent to Component (needs CommandOutput for background operations)
- GtkFlowBox for responsive grid of character cards
- GtkSearchEntry at top → GtkFilterListModel → GtkFilter → FlowBox filter
- AdwViewSwitcherBar or AdwTabBar for Characters/Personas tabs (D-03)
- "New Character" button (Ctrl+Shift+N)
- Import button → GtkFileChooserNative (D-04)

### Character Card Widget
- AdwAvatar (circular, with character avatar or fallback initials)
- Name label
- Creator label (secondary)
- Tag labels (horizontal flow of small badges)
- Brief description (ellipsized)

### Character Editor (D-01)
- **Creation mode**: AdwPreferencesWindow with multiple pages (Identity → Personality → Scenario → Advanced)
- **Edit mode**: AdwNavigationView with sliding panels
- Shared form fields between both modes
- Avatar selection via GtkFileChooserNative
- Alternate greetings as editable list with add/remove rows

## Security Considerations

- Import validation: sanitize oversized JSON payloads, reject malformed PNG chunks (CD-06)
- PNG chunk size limits to prevent memory exhaustion
- No character data should contain executable paths or system commands
- Character card data is user-created content — no content filtering per project constraints

## Common Pitfalls

1. **PNG crate async**: png crate is synchronous — file I/O must happen on background thread (tokio::task::spawn_blocking or Relm4 Command)
2. **GtkFlowBox selection**: GtkFlowBox uses single-click selection by default, not activation — need connect_selected_children_changed or connect_child_activated
3. **FTS5 triggers**: Must handle INSERT, UPDATE, DELETE to keep FTS index in sync
4. **Image crate thread safety**: image::DynamicImage is not Send — thumbnail generation must complete before sending across threads
5. **Large avatar images**: Must resize before storage to prevent database/filesystem bloat
6. **Relm4 Component vs SimpleComponent**: CharactersPanel needs Component trait for spawn_oneshot_command (learned in Phase 1 Plan 03)
7. **AdwPreferencesWindow**: Not a regular window — must be created with AdwPreferencesWindow::new() and presented with present()
8. **GtkFileChooserNative**: Must be correctly parented and kept alive during the async response — store in model struct

## Validation Architecture

### Dimension Coverage
| Dimension | Validation Method |
|-----------|-------------------|
| D1: Domain Model | Unit tests for CharacterCardV2 serde roundtrip, V1 migration |
| D2: Database | Integration tests for CRUD, FTS5 search accuracy |
| D3: Import/Export | Unit tests for PNG tEXt chunk encode/decode, JSON V1/V2 parsing |
| D4: UI Library | Build verification, grep checks for UI component presence |
| D5: UI Editor | Build verification, grep checks for editor component presence |
| D6: Integration | cargo test (all tests), cargo build (compilation) |
| D7: End-to-End | Manual UAT: create → appears in library → edit → import/export |
| D8: Performance | FTS5 query < 50ms for 1000 characters (benchmark if needed) |

---
*Phase: 02-character-system-library*
*Research completed: 2026-04-29*

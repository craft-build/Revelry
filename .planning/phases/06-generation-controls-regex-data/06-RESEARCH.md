# Phase 6: Generation Controls, Regex & Data — Research

**Researched:** 2026-05-08
**Status:** Complete

## Domain Analysis

### Generation Parameters via rig

**Supported natively by AgentBuilder:**
- `.temperature(f64)` — all providers
- `.max_tokens(u64)` — all providers

**Passed via `.additional_params(serde_json::Value)`:**
- `top_p` — OpenAI (Responses API AdditionalParameters has `top_p: Option<f64>`), Gemini (GenerationConfig has `top_p: Option<f64>`)
- `frequency_penalty` — Not in OpenAI Responses API struct, must pass as raw JSON via additional_params
- `presence_penalty` — Same as frequency_penalty
- `stop_sequences` — Gemini has `stop_sequences: Option<Vec<String>>` in GenerationConfig. OpenAI Responses API does NOT have stop in AdditionalParameters — must pass as raw JSON

**Strategy per D-02:**
1. Use `.temperature()` and `.max_tokens()` on AgentBuilder (natively supported)
2. Use `.additional_params(serde_json::json!({...}))` for top_p, frequency_penalty, presence_penalty, stop_sequences
3. Build the JSON object per-provider based on what each API accepts
4. Silently omit unsupported params (e.g., Anthropic doesn't support frequency/presence_penalty)

**Per-provider parameter matrix:**
| Parameter | OpenAI | Anthropic | Gemini | Ollama | Groq | DeepSeek |
|-----------|--------|-----------|--------|--------|------|----------|
| temperature | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| max_tokens | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| top_p | ✓ (AdditionalParameters) | ✓ | ✓ (GenerationConfig) | ✓ | ✓ | ✓ |
| frequency_penalty | ✓ (raw JSON) | ✗ | ✗ | ✓ | ✓ | ✓ |
| presence_penalty | ✓ (raw JSON) | ✗ | ✗ | ✓ | ✓ | ✓ |
| stop_sequences | ✓ (raw JSON) | ✓ | ✓ (GenerationConfig) | ✓ | ✓ | ✓ |

**Implementation approach:**
- Modify `AiService::stream_chat()` to accept `GenerationParams` struct
- In `run_stream()`, build the agent with `.temperature()` and `.max_tokens()` first
- Then add `.additional_params(json!({...}))` with provider-specific subset
- Each provider match arm builds the appropriate JSON object

### Regex ReDoS Validation

**Approach per D-06:**
```rust
use regex::RegexBuilder;
fn validate_regex_pattern(pattern: &str, flags: &str) -> Result<regex::Regex, String> {
    let mut builder = RegexBuilder::new(pattern);
    builder.size_limit(1_000_000); // 1MB backtracking limit
    // Apply flags: i=case_insensitive, m=multi_line, s=dot_matches_new_line
    if flags.contains('i') { builder.case_insensitive(true); }
    if flags.contains('m') { builder.multi_line(true); }
    if flags.contains('s') { builder.dot_matches_new_line(true); }
    builder.build().map_err(|e| e.to_string())
}
```
- Compilation is synchronous and fast (<10ms for typical patterns)
- If size_limit exceeded → pattern is dangerous, block saving
- If syntax error → show regex crate error message
- g flag = global (apply all matches) — handled at replacement time, not compilation

### Regex Script Execution Pipeline

Per D-05, scripts execute as a pipe chain in sort_order:
```rust
fn apply_regex_scripts(text: &str, scripts: &[RegexScript]) -> String {
    let mut result = text.to_string();
    for script in scripts.iter().filter(|s| s.enabled).sorted_by_key(|s| s.sort_order) {
        if let Ok(re) = validate_regex_pattern(&script.find_pattern, &script.flags) {
            let is_global = script.flags.contains('g');
            if is_global {
                result = re.replace_all(&result, &script.replace_pattern).to_string();
            } else {
                result = re.replace(&result, &script.replace_pattern).to_string();
            }
        }
    }
    result
}
```
- Applied AFTER AI response completes, BEFORE markdown rendering
- Applied in ChatPanel when receiving StreamChunk::Complete

### SQLite Dump Export

Per D-08, SQL dump uses SQLite's `.dump` output:
```rust
fn export_sql_dump(conn: &Connection, path: &Path) -> Result<()> {
    let mut output = File::create(path)?;
    // Iterate all tables
    let tables: Vec<String> = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?
        .query_map([], |r| r.get(0))?
        .filter_map(|t| t.ok())
        .collect();
    writeln!(output, "PRAGMA foreign_keys=OFF;")?;
    writeln!(output, "BEGIN TRANSACTION;")?;
    for table in &tables {
        // DROP TABLE IF EXISTS
        writeln!(output, "DROP TABLE IF EXISTS {};", table)?;
        // Get CREATE TABLE from sqlite_master
        let ddl: String = conn.query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name=?1",
            [table], |r| r.get(0))?;
        writeln!(output, "{}", ddl)?;
        // INSERT statements for each row
        // ... iterate rows, serialize as INSERT INTO
    }
    writeln!(output, "COMMIT;")?;
    Ok(())
}
```

### ZIP Archive Export/Import

Per D-09, ZIP contains DB file + avatars + settings JSON:
```rust
use zip::ZipWriter;
fn export_zip(db_path: &Path, avatar_dir: &Path, settings: &Value, output_path: &Path) -> Result<()> {
    let file = File::create(output_path)?;
    let mut zip = ZipWriter::new(file);
    // Add DB file
    zip.start_file("revelry.db", SimpleFileOptions::default())?;
    let db_bytes = std::fs::read(db_path)?;
    zip.write_all(&db_bytes)?;
    // Add avatars directory
    for entry in std::fs::read_dir(avatar_dir)? {
        let entry = entry?;
        zip.start_file(format!("avatars/{}", entry.file_name().to_string_lossy()), SimpleFileOptions::default())?;
        let mut f = File::open(entry.path())?;
        std::io::copy(&mut f, &mut zip)?;
    }
    // Add settings JSON
    zip.start_file("app_settings.json", SimpleFileOptions::default())?;
    zip.write_all(serde_json::to_string_pretty(settings)?.as_bytes())?;
    zip.finish()?;
    Ok(())
}
```

### Import Flow

Per D-10, D-11:
1. For SQL dump: read file, execute as raw SQL against a fresh connection
2. For ZIP: extract DB file to temp, replace current DB, copy avatars, restore settings
3. After restore: run `run_migrations()` on the restored DB
4. Check schema version: if user_version > current max migration → error "newer version"
5. Full app data reload after import (re-dispatch LoadProviders, LoadCharacters, etc.)

## Architecture Decisions

### GenerationParams Domain Model
```rust
pub struct GenerationPreset {
    pub id: String,
    pub name: String,
    pub is_builtin: bool,  // true for Creative/Balanced/Precise
    pub temperature: f64,
    pub top_p: f64,
    pub max_tokens: i64,   // 0 = no limit
    pub frequency_penalty: f64,
    pub presence_penalty: f64,
    pub stop_sequences: Vec<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
```

### RegexScript Domain Model
```rust
pub struct RegexScript {
    pub id: String,
    pub character_id: String,
    pub name: String,
    pub find_pattern: String,
    pub replace_pattern: String,
    pub flags: String,       // e.g. "gi"
    pub sort_order: i32,
    pub enabled: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
```

### Database Schema (Migration 9)
```sql
CREATE TABLE generation_presets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    is_builtin INTEGER NOT NULL DEFAULT 0,
    temperature REAL NOT NULL DEFAULT 0.8,
    top_p REAL NOT NULL DEFAULT 0.9,
    max_tokens INTEGER NOT NULL DEFAULT 0,
    frequency_penalty REAL NOT NULL DEFAULT 0.0,
    presence_penalty REAL NOT NULL DEFAULT 0.0,
    stop_sequences TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE regex_scripts (
    id TEXT PRIMARY KEY,
    character_id TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    name TEXT NOT NULL DEFAULT '',
    find_pattern TEXT NOT NULL DEFAULT '',
    replace_pattern TEXT NOT NULL DEFAULT '',
    flags TEXT NOT NULL DEFAULT 'gi',
    sort_order INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX idx_regex_scripts_character ON regex_scripts(character_id, sort_order);

ALTER TABLE characters ADD COLUMN default_preset_id TEXT;
ALTER TABLE conversations ADD COLUMN generation_preset_id TEXT;
```

### Built-in Presets Seed Data (D-12)
- Creative: temp=1.2, top_p=0.9, max_tokens=0, penalties=0.0, stop=[]
- Balanced: temp=0.8, top_p=0.9, max_tokens=0, penalties=0.0, stop=[]
- Precise: temp=0.3, top_p=0.75, max_tokens=0, penalties=0.0, stop=[]

## Common Pitfalls

1. **Don't pass frequency/presence_penalty to Anthropic** — these APIs don't support them; silently omit
2. **ReDoS validation must be synchronous** — regex compilation is fast, don't async it
3. **ZIP export must close DB connection** — SQLite file must be fully flushed before copying into ZIP; use `PRAGMA wal_checkpoint(TRUNCATE)` first
4. **Import must check schema version BEFORE replacing** — can't downgrade from a newer schema
5. **Built-in presets must use fixed UUIDs** — so they're stable across import/export, not random on each startup
6. **Regex replacement with $1 syntax** — regex crate uses `$1` (not `\1`) for capture group references; this matches JavaScript/ECMAScript conventions used by SillyTavern
7. **Agent builder `.additional_params()` merges with existing params** — rig merges tools into additional_params; be careful not to overwrite

## Validation Architecture

### Dimension 8: Nyquist Validation
- Generation params: test each preset value round-trips through DB → struct → rig API
- Regex: test ReDoS validation blocks dangerous patterns, allows safe patterns
- Regex: test pipe chain execution with multiple scripts
- Backup: test SQL dump round-trips (export → clear → import → verify data)
- Backup: test ZIP round-trips similarly

---
*Phase: 06-generation-controls-regex-data*
*Research completed: 2026-05-08*

# Domain Pitfalls

**Domain:** Tauri desktop AI roleplay app with SvelteKit frontend — migrating Rust backend from gtk_ref/

## Critical Pitfalls

### Pitfall 1: Synchronous DB Calls in Tauri Commands

**Warning signs:** UI freezes when saving/loading data; Tauri commands take >16ms.

**Prevention:** Never call rusqlite directly from `#[tauri::command]` handlers. Use `tokio::task::spawn_blocking` for individual queries, or a background DB worker thread with mpsc channels (the pattern from `gtk_ref/` `DbService`). All DB operations must be off the main thread.

**Phase:** Phase 2 (Domain Models + DB Layer) — establish this pattern from day one.

### Pitfall 2: Tauri Channel vs Event Confusion

**Warning signs:** Using `emit()`/`listen()` for streaming tokens and experiencing dropped events, ordering issues, or backpressure problems.

**Prevention:** Use `Channel<T>` (Tauri v2 IPC channel) for streaming — it's ordered, typed, and scoped to a single invoke call. Use `emit()`/`listen()` only for broadcast-style unsolicited notifications (e.g., "database migrated"). The streaming pattern is: `invoke('command', { onEvent: Channel })` — verified via Context7 Tauri docs.

**Phase:** Phase 4 (AI Service + Streaming) — get this right when building the streaming path.

### Pitfall 3: SSR Conflicts in Tauri + SvelteKit

**Warning signs:** Hydration errors, `window is not defined` errors, or routing breaks inside Tauri webview.

**Prevention:** Switch from `adapter-auto` to `@sveltejs/adapter-static` with `fallback: 'index.html'`. Ensure `ssr = false` in root `+layout.ts`. The current codebase already has `ssr = false`, but the adapter must also be static for Tauri. Also: Tauri loads from `tauri://` protocol, not `http://localhost` — any absolute URL assumptions will break.

**Phase:** Phase 1 (Tauri Shell Setup) — this is the first thing to get right.

### Pitfall 4: API Key Exposure

**Warning signs:** API keys appearing in SQLite, config files, localStorage, or error messages.

**Prevention:** All API keys go through the `keyring` crate to OS-native storage. Never log keys. Never include keys in error messages. The DB stores provider config without the key — keys are retrieved from keyring at runtime. This is a hard constraint from the design spec and a security invariant from `gtk_ref/`.

**Phase:** Phase 3 (Keyring + Provider Config) — enforce from day one.

## Moderate Pitfalls

### Pitfall 5: rig-core Provider Configuration Drift

**Warning signs:** Provider works in `gtk_ref/` but fails when ported; streaming errors after updating rig-core version.

**Prevention:** Pin rig-core version in `Cargo.toml`. The `gtk_ref/` uses rig-core ^0.36 — verify the same API surface exists in the latest version. Custom OpenAI-compatible endpoints use `rig::providers::openai::Client::from_url()` — verify this API hasn't changed. Run provider connectivity tests after migration.

**Phase:** Phase 4 (AI Service + Streaming) — validate during migration.

### Pitfall 6: Linux Keyring Daemon Absence

**Warning signs:** Keyring operations fail on Linux with "no secret service" or "dbus" errors; app crashes on headless Linux or minimal installs.

**Prevention:** The `keyring` crate requires a running keyring daemon on Linux (gnome-keyring or kwallet). Handle `keyring::Error` gracefully — provide a fallback (e.g., encrypted file storage) or a clear error message. Don't let keyring failure crash the app. Test on a minimal Linux install during cross-platform testing.

**Phase:** Phase 3 (Keyring + Provider Config) — handle the failure path.

### Pitfall 7: Token Counting Mismatch

**Warning signs:** Context utilization bar shows wrong values; prompt truncation happens at wrong points; tiktoken-rs gives different counts than what the model actually uses.

**Prevention:** tiktoken-rs uses the same BPE encoding as OpenAI's tiktoken, but different models use different encodings (cl100k, p50k, o200k). Map each model to its correct encoding. For non-OpenAI models, the heuristic-based estimation from the current frontend (`4 chars ≈ 1 token`) may actually be more practical as a fallback. Document which encodings are used and test against known token counts.

**Phase:** Phase 5 (Prompt Engine + Token Counting) — validate against real token counts.

### Pitfall 8: Cross-Platform File Paths

**Warning signs:** DB file not found on Windows; card import fails on Linux; paths use hard-coded separators.

**Prevention:** Use the `dirs` crate for all platform-specific directories (data, config, cache). Use `std::path::PathBuf` and `Path` — never hard-code `/` or `\`. The `gtk_ref/` already uses `dirs` correctly; preserve this pattern. On Windows, the data directory is `%APPDATA%`, on macOS `~/Library/Application Support/`, on Linux `$XDG_DATA_HOME` or `~/.local/share/`.

**Phase:** Phase 2 (Domain Models + DB Layer) — use `dirs` from the start.

### Pitfall 9: PNG Card Metadata Handling

**Warning signs:** Imported character cards lose data; exported cards don't load in other apps; PNG metadata corrupted.

**Prevention:** The TavernAI V2 spec embeds JSON in PNG tEXt chunks under the key `chara`. Use the `png` crate for proper chunk handling. Validate the JSON structure before accepting. The `gtk_ref/` `card_io.rs` has a working implementation — port and test it with cards from SillyTavern and other compatible apps.

**Phase:** Phase 7 (Card I/O) — test with real character cards from the community.

### Pitfall 10: Streaming Backpressure

**Warning signs:** UI jank during streaming; memory usage grows during long responses; tokens arrive faster than UI can render.

**Prevention:** Tauri's `Channel` handles backpressure by design (it's not unbounded like raw events). However, the frontend must process tokens efficiently — batch DOM updates, avoid re-renders on every single token. The existing mock streaming code already handles this with a typewriter pattern; adapt it for real tokens.

**Phase:** Phase 4 (AI Service + Streaming) — profile and optimize.

## Minor Pitfalls

### Pitfall 11: Tauri Dev vs Production Differences

**Warning signs:** App works in `tauri dev` but fails in built binary; CORS errors in production; assets not found.

**Prevention:** In dev mode, Tauri serves from `http://localhost:5173` (Vite dev server). In production, it serves from `tauri://` protocol. Test both modes. Ensure `tauri.conf.json` `frontendDist` points to the correct build output (`../build` for SvelteKit static adapter). Test the production build before every release.

**Phase:** Phase 1 (Tauri Shell Setup) — establish CI testing early.

### Pitfall 12: Tauri v2 Permissions System

**Warning signs:** Commands silently fail; "permission denied" errors in console; shell/FS operations blocked.

**Prevention:** Tauri v2 has a capabilities/permissions system that's more restrictive than v1. Each command must be allowed in the capability config. Shell access, filesystem access, and HTTP requests all need explicit permissions. Configure capabilities in `src-tauri/capabilities/default.json`. This is the most common surprise when migrating from Tauri v1 patterns.

**Phase:** Phase 1 (Tauri Shell Setup) — set up capabilities as commands are added.

### Pitfall 13: Lorebook Recursive Insertion Loops

**Warning signs:** Prompt assembly hangs or exceeds token budget; lorebook entries reference each other in cycles.

**Prevention:** Limit recursion depth (e.g., max 3 levels). Track visited entries during insertion. Count tokens after each insertion round and stop if budget is exceeded. The `gtk_ref/` `lorebook.rs` has a working bounded implementation — preserve the safeguards.

**Phase:** Phase 9 (Lorebook) — add cycle detection from the start.

### Pitfall 14: Regex Script Execution Safety

**Warning signs:** Regex takes excessive time (ReDoS); malformed patterns crash the service; scripts affect other scripts unexpectedly.

**Prevention:** Use `fancy-regex` or the standard `regex` crate with a time limit. Validate patterns on save (not at runtime). Isolate script execution so one script's output doesn't accidentally match another script's pattern (unless intentional). The `gtk_ref/` `regex.rs` has a working implementation — port the safety measures.

**Phase:** Phase 10 (Regex Scripting) — validate on save, limit execution time.

## Phase-Specific Warnings

| Phase                   | Key Risk                    | Prevention                                            |
| ----------------------- | --------------------------- | ----------------------------------------------------- |
| Phase 1 (Tauri Shell)   | SSR/adapter mismatch        | Use `adapter-static` with `fallback: 'index.html'`    |
| Phase 2 (DB Layer)      | Sync DB on main thread      | `spawn_blocking` or background worker from day one    |
| Phase 3 (Keyring)       | Linux keyring not available | Graceful fallback, clear error messages               |
| Phase 4 (AI Streaming)  | Channel vs Event confusion  | Use `Channel<T>` for streaming, `emit` for broadcasts |
| Phase 5 (Prompt Engine) | Token count mismatch        | Map model→encoding, test against known counts         |
| Phase 7 (Card I/O)      | PNG metadata corruption     | Test with real community character cards              |
| Phase 9 (Lorebook)      | Recursive insertion loops   | Max depth limit, budget check                         |
| Phase 10 (Regex)        | ReDoS, unsafe patterns      | Validate on save, execution time limit                |

## Sources

- Tauri v2 official docs (Context7) — Channel API, capabilities, SvelteKit integration
- rig-core API docs (Context7) — Provider configuration, streaming
- Existing codebase: `.planning/codebase/CONCERNS.md`
- Archival reference: `gtk_ref/` — proven patterns and safety measures in services

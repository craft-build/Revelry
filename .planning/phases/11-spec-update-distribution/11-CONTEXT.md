# Phase 11: Spec Update & Distribution - Context

**Gathered:** 2026-05-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Two deliverables: (1) Convert the design specification from .docx to Markdown with GTK4/GNOME references replaced by SvelteKit/Tauri equivalents and updated to reflect actual implementation, and (2) establish cross-platform builds for macOS, Linux, and Windows with CI-driven distribution to GitHub Releases.

No new application features. This phase is documentation + build infrastructure.

</domain>

<decisions>
## Implementation Decisions

### Design Spec Conversion

- **D-01:** Convert `docs/Revelry_Design_Specification_v2.0.docx` to a multi-file Markdown breakdown in `docs/` (e.g., architecture.md, data-model.md, ui-reference.md, etc.)
- **D-02:** Delete the original .docx after conversion is verified complete — Markdown is the single source of truth going forward
- **D-03:** Reference swap + reality check — replace all GTK4/GNOME/Relm4 references with SvelteKit/Tauri equivalents AND update sections to reflect how features were actually implemented (actual component names, state management approach, IPC patterns, etc.)

### Cross-Platform Build Strategy

- **D-04:** Build verification: local `tauri build` on macOS for DMG; GitLab CI/CD for Linux and Windows builds
- **D-05:** Linux packaging: AppImage only (single-file, no-install, works across distros) — no .deb
- **D-06:** macOS packaging: DMG (Tauri default)
- **D-07:** Windows packaging: MSI or NSIS (Tauri default — agent decides which)

### Distribution & Signing

- **D-08:** No code signing or notarization in v1 — users will see OS warnings on first launch. Can be added later with paid developer accounts
- **D-09:** No auto-update in v1 — deferred to v2 (already listed in v2 requirements)
- **D-10:** Build artifacts published to GitLab Releases via CI pipeline (on tag or manual trigger)

### Agent's Discretion

- Exact Markdown file structure and naming for the multi-file spec breakdown
- Which Markdown sections to split vs. keep together
- Whether to include Mermaid diagrams or ASCII diagrams for architecture sections
- Windows installer format choice (MSI vs NSIS — Tauri supports both)
- CI trigger mechanism (tag-based, manual, or on push to main)
- GitLab runner configuration (shared runners vs. specific runners, Docker executor for Linux/Windows)
- Caching strategy for Rust builds in GitLab CI (cache key per Cargo.lock hash)
- Whether the `docs/Revelry.svg` logo file stays or moves

</decisions>

<canonical_refs>

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design Specification (primary work artifact)

- `docs/Revelry_Design_Specification_v2.0.docx` — Source document to be converted. All GTK4/GNOME/Relm4 references must be replaced. This is the authoritative spec.

### Current Architecture (for reality-check updates)

- `src-tauri/tauri.conf.json` — Current Tauri config (bundle settings, CSP, window config)
- `src-tauri/Cargo.toml` — Rust dependencies and build configuration
- `package.json` — Frontend dependencies and scripts
- `src/lib/stores/app.svelte.ts` — Central AppState (actual state management pattern)
- `src/lib/types/index.ts` — Actual TypeScript type definitions
- `src/app.css` — OKLCH theme tokens, custom utilities, font definitions

### Prior Phase Decisions (carry-forward for spec accuracy)

- `.planning/phases/01-native-desktop-shell/01-CONTEXT.md` — Tauri init, adapter strategy, dev workflow, capabilities scope
- `.planning/phases/02-persistent-data/02-CONTEXT.md` — SQLite schema, migration strategy
- `.planning/phases/04-provider-setup/04-CONTEXT.md` — Keyring, provider CRUD, settings structure
- `.planning/phases/05-live-conversations/05-CONTEXT.md` — Streaming architecture, AI provider integration
- `.planning/phases/07-rich-output-cards/07-CONTEXT.md` — Markdown rendering pipeline, Shiki theme
- `.planning/phases/10-theme-polish/10-CONTEXT.md` — Theme system, light/dark toggle, polish decisions

### Reference Implementation (archival)

- `gtk_ref/` — Original GTK4/Rust implementation. Reference only for understanding what the spec was describing. Not modified.

### Project Planning

- `.planning/ROADMAP.md` — Phase definitions, success criteria, requirements traceability
- `.planning/REQUIREMENTS.md` — DOC-01, DIST-01, DIST-02, DIST-03 requirements
- `.planning/PROJECT.md` — Vision, constraints, key decisions

</canonical_refs>

<code_context>

## Existing Code Insights

### Reusable Assets

- `src-tauri/tauri.conf.json` bundle section — Already has `active: true` and icon set configured (32x32, 128x128, 128x128@2x, .icns, .ico)
- `src-tauri/icons/` — Full icon set already exists (PNG, ICO, ICNS formats for all platforms)
- `docs/Revelry.svg` — App logo in vector format
- `.github/` — May already have CI workflows from prior phases (check for existing patterns)

### Established Patterns

- Tauri v2 with `adapter-static` for SPA build
- Bun as package manager (bun.lock present)
- Vite 8.x as bundler with `@tailwindcss/vite` plugin
- SQLite via `rusqlite` with bundled feature (cross-compilation implications for Windows)

### Integration Points

- `tauri build` is the single command for all platform builds — frontend build runs first via `beforeBuildCommand`
- Bundle config in `tauri.conf.json` controls packaging format per platform
- GitLab CI runners need Rust toolchain + Bun + Node.js for cross-platform builds (Linux/Windows via Docker executors, macOS runner if available)
- `rusqlite` bundled feature means SQLite is statically compiled — no external dependency, simplifies distribution

</code_context>

<specifics>
## Specific Ideas

- Multi-file Markdown is better than monolithic for version control — reviewers can see which sections changed
- The spec should reflect the actual implemented architecture, not an idealized version — downstream readers need accuracy
- GitLab Releases as distribution is the simplest viable approach — no need for app stores or custom update servers in v1

</specifics>

<deferred>
## Deferred Ideas

- Code signing / notarization — requires paid developer accounts (Apple Developer, Windows code signing cert). Can be added later without architecture changes
- Auto-update — already in v2 requirements (v2 Requirements > Autoupdate). Tauri has built-in updater support
- .deb packaging for Linux — AppImage covers the use case; .deb can be added later if demand exists
- App store distribution (Mac App Store, Microsoft Store, Flathub) — separate effort

</deferred>

---

_Phase: 11-spec-update-distribution_
_Context gathered: 2026-05-20_

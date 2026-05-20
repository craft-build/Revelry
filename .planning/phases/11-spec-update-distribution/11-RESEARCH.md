# Phase 11 Research: Spec Update & Distribution

**Researched:** 2026-05-20
**Status:** Complete

---

## Research Questions

### RQ-1: What is the scope of the design spec conversion?

**Answer:** The source document `docs/Revelry_Design_Specification_v2.0.docx` is ~66K characters across 876 paragraphs with 16 major sections. It contains pervasive GTK4/GNOME/Relm4 references:

| Term               | Occurrences |
| ------------------ | ----------- |
| GTK4/gtk           | 35          |
| libadwaita/Adwaita | 32          |
| Relm4/relm         | 44          |
| GNOME              | 48          |
| Pango              | 5           |

**Sections most affected by GTK4/GNOME references:**

1. **Section 4 — "Design Philosophy: GNOME Ethos"** — Entire section frames the design around GNOME HIG. Needs complete rewrite to a general design philosophy (content-first, progressive disclosure, etc.) without GNOME specificity.
2. **Section 5 — "Technology Stack"** — Five subsections (5.1 Framework, 5.3 State Management, 5.5 Markdown Rendering) are GTK4/Relm4/Pango specific. Must become SvelteKit/Tauri equivalents.
3. **Section 6 — "System Architecture"** — Describes Relm4 component hierarchy, GLib main loop, tokio-GLib bridge. Needs rewrite for Tauri IPC + SvelteKit frontend.
4. **Section 10 — "UI/UX Design"** — References AdwOverlaySplitView, GtkFlowBox, GtkSearchEntry, GtkPopover, GtkTextView. Must map to actual Svelte components.
5. **Section 16 — "Appendix B: GTK4/libadwaita Widget Mapping"** — Entire appendix is obsolete; replace with Svelte component mapping or remove.
6. **Section 5.6 — "Technology Summary"** table — GTK4/libadwaita, Relm4 entries need SvelteKit/Tauri replacements.
7. **Competitive Analysis (Section 3)** — "Native Desktop (Rust/GTK4)" differentiator needs reframing.
8. **Section 13.2/13.4** — Security references to GNOME Keyring, GTK4 input sanitization.

**What stays (no GTK4 references):**

- Section 7 — Character Card System (data model, import/export)
- Section 8 — AI Provider Integration (rig framework details remain valid)
- Section 9 — User Persona System (data model, prompt integration)
- Section 12 — Data Model and Storage (SQLite schema)
- Section 13 — Security (mostly, except GNOME Keyring mentions)

**Conversion approach (per D-01):** Multi-file Markdown breakdown. The docx has natural section boundaries (16 sections) that map well to individual Markdown files. Suggested structure:

```
docs/
├── 01-executive-summary.md
├── 02-vision-and-goals.md
├── 03-competitive-analysis.md
├── 04-design-philosophy.md          ← Heavy rewrite (GNOME → general)
├── 05-technology-stack.md            ← Heavy rewrite (GTK4 → SvelteKit/Tauri)
├── 06-system-architecture.md         ← Heavy rewrite (Relm4 → Tauri IPC)
├── 07-character-card-system.md       ← Light update only
├── 08-ai-provider-integration.md     ← Light update only
├── 09-user-persona-system.md         ← Light update only
├── 10-ui-ux-design.md                ← Medium rewrite (widget names → components)
├── 11-feature-specifications.md      ← Medium rewrite
├── 12-data-model-and-storage.md      ← Light update only
├── 13-security-and-privacy.md        ← Light update (keyring refs)
├── 14-development-roadmap.md         ← Update to reflect actual phases
├── 15-rig-provider-matrix.md         ← No change needed
└── 16-component-reference.md         ← Replace Appendix B: widget mapping → Svelte components
```

**Reality-check sources** (per D-03): The spec must be updated to reflect the actual implemented architecture, not just a find-replace. Key source files:

- `src/lib/stores/app.svelte.ts` — Actual state management (Svelte 5 runes, not Relm4)
- `src/lib/types/index.ts` — Actual TypeScript types
- `src-tauri/src/` — Actual Rust backend structure (commands, db, domain, service layers)
- `src/app.css` — Actual theme tokens (OKLCH, not Adwaita)
- `package.json` — Actual frontend dependencies
- `src-tauri/Cargo.toml` — Actual Rust dependencies
- `.planning/phases/*/` — Actual implementation decisions from each phase

---

### RQ-2: What does the Tauri v2 cross-platform build require?

**Answer:** Tauri v2 (currently `tauri 2` in Cargo.toml) provides a unified `tauri build` command that handles frontend build + Rust compilation + platform packaging in one step.

**Current configuration (`tauri.conf.json`):**

- `beforeBuildCommand: "bun run build"` — builds SvelteKit SPA via Vite
- `frontendDist: "../build"` — adapter-static output
- `bundle.active: true` — bundling enabled
- Icons already exist in all required formats (PNG, ICO, ICNS)
- Bundle identifier: `com.revelry.desktop`

**Platform-specific requirements:**

| Platform    | Build Prerequisites                                                                   | Output Format                | Notes                                                                                    |
| ----------- | ------------------------------------------------------------------------------------- | ---------------------------- | ---------------------------------------------------------------------------------------- |
| **macOS**   | Xcode CLI tools, Rust stable                                                          | `.dmg` (default)             | Tauri default. Works on Apple Silicon and Intel via universal binary or separate targets |
| **Linux**   | `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libappindicator3-dev`, `librsvg2-dev`, etc. | `.AppImage` (per D-05)       | Tauri supports AppImage, deb, rpm. AppImage chosen for universal Linux compatibility     |
| **Windows** | MSVC build tools (via Visual Studio Build Tools), WebView2 (bundled by Tauri)         | `.msi` or `.nsis` (per D-07) | WebView2 is included in Windows 11; Tauri can bundle it for Windows 10                   |

**Key consideration: `rusqlite` bundled feature.** The `rusqlite` dependency uses `features = ["bundled"]` which statically compiles SQLite. This simplifies cross-platform distribution — no external SQLite dependency needed. However, it requires a C compiler on each platform (GCC on Linux, MSVC on Windows, Clang via Xcode on macOS).

**Tauri v2 bundle configuration for target platforms:**

```json
{
  "bundle": {
    "active": true,
    "targets": ["dmg", "appimage", "nsis"],  // or "msi"
    "icon": [...],
    "linux": {
      "deb": { "depends": [...] },   // Not needed for AppImage-only
      "appimage": { "bundleMediaFramework": false }
    },
    "windows": {
      "nsis": { ... }  // or "msi": { ... }
    }
  }
}
```

---

### RQ-3: What CI/CD infrastructure is needed for GitLab?

**Answer:** The project uses GitLab (`git@gitlab.com:akarifur/persona.git`) and has **no existing CI configuration**. Phase 11 must create `.gitlab-ci.yml` from scratch.

**Build pipeline requirements (per D-04, D-10):**

1. **macOS build** — Run locally via `tauri build` on developer machine (D-04 says "local `tauri build` on macOS for DMG"). GitLab shared runners don't provide macOS executors unless using self-hosted runners or macOS-specific SaaS runners.

2. **Linux build** — GitLab CI with Docker executor. Needs a Docker image with: Rust toolchain, Bun, Node.js, `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, and other system dependencies. Can use a custom Dockerfile or install deps in CI script.

3. **Windows build** — GitLab CI with Windows runner (shared runners on GitLab.com provide Windows executors). Needs: Rust toolchain (MSVC target), Bun, Node.js, Visual Studio Build Tools.

**CI trigger (per D-10, agent discretion):** Tag-based trigger is recommended — push a `v*` tag to trigger all platform builds and publish to GitLab Releases. Manual trigger option via GitLab CI's `when: manual` for ad-hoc builds.

**Proposed pipeline structure:**

```yaml
stages:
  - build
  - release

build:macos: # Self-hosted runner or manual
build:linux: # Docker executor on shared runner
build:windows: # Windows shared runner

release:
  stage: release
  needs: [build:macos, build:linux, build:windows]
  # Upload artifacts to GitLab Releases
```

**Rust build caching (agent discretion):** Cache `$CI_PROJECT_DIR/target/` keyed by `Cargo.lock` hash. Tauri builds are Rust-heavy and can take 10-20 minutes without caching. With caching, incremental builds drop to 2-5 minutes.

**Key challenge:** macOS builds. GitLab.com shared runners don't offer macOS. Options:

1. Self-hosted macOS runner (requires a Mac always on)
2. Build macOS locally, push DMG to GitLab Release manually
3. Use a third-party CI service (CircleCI, GitHub Actions) for macOS only — but project is on GitLab

Decision D-04 says "local `tauri build` on macOS for DMG" which implies option 2: local macOS build + CI for Linux/Windows.

---

### RQ-4: What sections of the spec need reality-checking vs. the actual implementation?

**Answer:** Significant sections of the spec describe features or architecture that were implemented differently. Key divergences:

**Architecture:**

- **Spec says:** Relm4 Elm Architecture (Model/Message), GTK4 widget tree, GLib main loop
- **Reality:** SvelteKit 2 + Svelte 5 runes (`$state`, `$derived`, `$effect`), Tauri v2 IPC bridge, Vite 8 bundler

**State Management:**

- **Spec says:** Relm4 Model/Message pattern
- **Reality:** Single `AppState` class in `src/lib/stores/app.svelte.ts` with `$state` fields, getters, and mutation methods. Pattern mirrors Elm/Relm4 conceptually but implemented via Svelte 5 runes.

**UI Framework:**

- **Spec says:** GTK4/libadwaita widgets (AdwHeaderBar, AdwOverlaySplitView, GtkFlowBox, etc.)
- **Reality:** bits-ui 2.x headless primitives + Tailwind CSS v4 with OKLCH theme tokens. Custom components in `src/lib/components/`.

**Markdown Rendering:**

- **Spec says:** pulldown-cmark + Pango
- **Reality:** marked v18 + DOMPurify + Shiki v4 (vitesse-dark theme) on frontend. Raw text during streaming, MarkdownRenderer on completion.

**Frontend Package Manager:**

- **Spec says:** Not specified (Rust-only)
- **Reality:** Bun as package manager

**Backend Structure:**

- **Spec says:** Single Relm4 model
- **Reality:** Layered Rust backend: `commands/` (Tauri IPC handlers) → `service/` (business logic) → `db/` (SQLite access) → `domain/` (types). Separate `ai/` module for rig integration, `secrets/` for keyring.

**Features not in spec:**

- Keyring v4 with `use_native_store(true)` init pattern
- Frontend pre-generates message UUID and sort_order for atomic backend save
- ChatContext + load_chat_context pattern for shared DB loading
- BudgetInfo derives Copy for tokio::select! safety
- Conditional rendering: raw text during streaming, MarkdownRenderer on completion

**Features in spec but not implemented:**

- Group Chat (Tier 2) — not in v1 requirements
- Conversation branching UI — type fields exist, no UI yet
- Persona Evolution Tracking (Tier 3)
- Persona Inheritance (Tier 3)
- Dual-Persona Mode (Tier 3)
- Vault Mode (Tier 3) — explicitly excluded

---

### RQ-5: What are the risks and blockers for cross-platform builds?

**Answer:**

1. **Linux system dependency installation.** Tauri on Linux requires many `-dev` packages. The CI Docker image must have all of them, or the build will fail at link time. Mitigation: use an existing Tauri Linux build Docker image as base, or install packages in CI script.

2. **Windows MSVC toolchain.** Rust on Windows targets MSVC by default. The CI runner needs Visual Studio Build Tools installed. GitLab's Windows shared runners typically have these pre-installed.

3. **WebView2 on Windows.** Tauri bundles WebView2 by default for Windows builds. This increases the installer size (~40MB for WebView2 bootstrapper). Can be configured to skip bundling if WebView2 is assumed present (Windows 11).

4. **Cross-compilation not needed.** Since each platform builds on its native runner, there's no cross-compilation complexity. Each runner builds for its own platform.

5. **No code signing (per D-08).** Users will see OS warnings:
   - macOS: "Cannot verify developer" — user must right-click → Open, or System Preferences override
   - Windows: SmartScreen warning on first launch
   - Linux: AppImage may need `chmod +x` before running (standard for AppImages)

6. **rusqlite bundled feature** is actually a strength — no external SQLite needed on any platform.

7. **Icon set already complete.** All icon formats (PNG, ICO, ICNS) exist in `src-tauri/icons/`. No icon generation needed.

---

## Key Findings Summary

### For Plan 11-01 (Spec Conversion)

1. The docx is ~66K chars / 16 sections. ~164 explicit GTK4/GNOME/Relm4 references across at least 5 heavily-affected sections.
2. Conversion is not a find-replace — significant sections need substantive rewrite (architecture, technology stack, design philosophy, UI reference).
3. Multi-file Markdown structure maps cleanly to the 16 docx sections.
4. Reality-check requires reading actual source files from each prior phase to update the spec to match what was built.
5. Delete docx after verification (per D-02).

### For Plan 11-02/11-03 (Cross-Platform Build & Distribution)

1. `tauri.conf.json` needs `targets` field added to `bundle` section specifying `["dmg", "appimage", "nsis"]` (or msi).
2. macOS: local build only (no GitLab macOS runners). Manual DMG upload to GitLab Releases.
3. Linux + Windows: GitLab CI pipeline with appropriate runners.
4. No `.gitlab-ci.yml` exists — must be created from scratch.
5. Rust build caching is important for CI performance.
6. No code signing in v1 — document OS warning workarounds for users.

### Dependencies Between Plans

Plan 11-01 (spec conversion) is independent of Plans 11-02/11-03 (build infrastructure). They can run in parallel if desired, but the CONTEXT suggests sequential: spec first, then build.

### Open Questions for Planning

1. **NSIS vs MSI for Windows?** (Agent discretion per D-07) — NSIS is simpler and more common for desktop apps. MSI is enterprise-friendly. Recommend NSIS for v1.
2. **CI trigger: tag-based or manual?** (Agent discretion) — Tag-based (`v*`) is standard practice. Manual trigger as fallback.
3. **Mermaid diagrams in spec?** (Agent discretion) — Recommended for architecture sections (system architecture, module architecture, thread architecture). GitLab renders Mermaid natively in Markdown.
4. **How to handle macOS DMG in the release pipeline?** — Local build + manual upload to GitLab Release, OR a self-hosted macOS runner. Decision D-04 says local build.

---

_Created: 2026-05-20_

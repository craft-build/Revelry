# Codebase Structure

**Analysis Date:** 2026-05-10

## Directory Layout

```
persona/
├── .gsd/                    # GSD workflow framework (agents, workflows, tools)
├── .maki/                   # Maki permissions config
├── .planning/               # GSD planning artifacts (architecture, structure docs)
├── .svelte-kit/             # SvelteKit generated build cache (gitignored)
├── docs/                    # Design specification documents
├── gtk_ref/                 # Rust/GTK4 reference implementation (production backend)
│   └── src/
│       ├── db/              # SQLite persistence layer
│       ├── domain/          # Domain models
│       ├── secrets/         # OS keyring integration
│       ├── service/         # AI, prompt, token, markdown services
│       └── ui/              # GTK4/Relm4 UI components
├── node_modules/            # NPM dependencies (gitignored)
├── src/                     # SvelteKit UI scaffold (active frontend)
│   ├── lib/
│   │   ├── components/      # Svelte components by domain
│   │   │   ├── characters/  # CharacterCard
│   │   │   ├── chat/        # ChatHeader, MessageBubble, MessageInput, PersonaSwitcher
│   │   │   ├── layout/      # AppHeader, ConversationSidebar
│   │   │   ├── personas/    # PersonaCard
│   │   │   └── ui/          # Headless UI primitives (Avatar, Button, Input, Tag, Tooltip)
│   │   ├── data/            # Mock data for UI development
│   │   ├── stores/          # Svelte 5 rune-based state (AppState)
│   │   ├── types/           # TypeScript type definitions
│   │   └── utils/           # Shared utilities (cn, formatRelativeTime, etc.)
│   └── routes/              # SvelteKit file-based routing
│       ├── chat/[id]/       # Active conversation view
│       ├── characters/      # Character & persona library
│       └── settings/        # Preferences panel
├── static/                  # Static assets (favicon)
├── AGENTS.md                # Agent-facing project documentation
├── README.md                # Project overview, quickstart, spec mapping
├── package.json             # NPM config: SvelteKit, Svelte 5, Bits.ui, Tailwind v4
├── svelte.config.js         # SvelteKit adapter & alias config
├── tsconfig.json            # TypeScript config (strict, bundler resolution)
└── vite.config.ts           # Vite: Tailwind v4 + SvelteKit plugins, port 5173
```

## Directory Purposes

**`src/lib/components/`:**
- Purpose: All Svelte UI components organized by domain
- Contains: `.svelte` component files grouped into subdirectories by feature area
- Key files: `chat/MessageBubble.svelte`, `chat/MessageInput.svelte`, `layout/AppHeader.svelte`, `ui/Avatar.svelte`, `ui/Button.svelte`

**`src/lib/components/ui/`:**
- Purpose: Headless UI primitives — Bits.ui wrappers with Tailwind styling
- Contains: Reusable presentational components with variant APIs
- Key files: `Avatar.svelte`, `Button.svelte`, `Input.svelte`, `Tag.svelte`, `Tooltip.svelte`, `index.ts`

**`src/lib/stores/`:**
- Purpose: Centralized reactive state management
- Contains: Single `AppState` class using Svelte 5 runes
- Key files: `app.svelte.ts`

**`src/lib/types/`:**
- Purpose: TypeScript type definitions mirroring the design spec's SQLite schema
- Contains: Interfaces and type aliases for all domain entities
- Key files: `index.ts`

**`src/lib/data/`:**
- Purpose: Mock data for UI development without a backend
- Contains: Hard-coded characters, personas, conversations, providers, presets
- Key files: `mock.ts`

**`src/lib/utils/`:**
- Purpose: Shared utility functions
- Contains: Class merge, formatting, token estimation, avatar initials
- Key files: `index.ts`

**`src/routes/`:**
- Purpose: SvelteKit file-based routing — each directory is a URL segment
- Contains: `+page.svelte` (page components), `+page.ts` (load functions), `+layout.svelte` (shared layout), `+layout.ts` (layout config)
- Key files: `+layout.svelte`, `+page.svelte`, `chat/[id]/+page.svelte`, `characters/+page.svelte`, `settings/+page.svelte`

**`gtk_ref/src/`:**
- Purpose: Rust/GTK4/libadwaita reference implementation — the production desktop backend
- Contains: Complete Relm4 application with domain models, services, DB, and UI
- Key files: `main.rs`, `app.rs`, `lib.rs`

**`gtk_ref/src/domain/`:**
- Purpose: Domain model definitions (pure data + validation)
- Contains: Rust structs with serde derives
- Key files: `character.rs`, `conversation.rs`, `persona.rs`, `provider.rs`, `generation.rs`, `lorebook.rs`, `regex_script.rs`, `app_settings.rs`

**`gtk_ref/src/db/`:**
- Purpose: SQLite persistence with background-thread worker
- Contains: CRUD modules per entity, migrations, row mappers, DbService worker
- Key files: `mod.rs`, `migrations.rs`, `models.rs`, `characters.rs`, `conversations.rs`, `providers.rs`, `personas.rs`

**`gtk_ref/src/service/`:**
- Purpose: Business logic services — AI inference, prompt assembly, token counting, markdown, regex, card I/O
- Contains: Service structs with async methods
- Key files: `ai.rs`, `prompt.rs`, `token_count.rs`, `markdown.rs`, `regex.rs`, `card_io.rs`, `lorebook.rs`, `template.rs`

**`gtk_ref/src/secrets/`:**
- Purpose: API key storage via OS-native keychain
- Contains: `KeyringStore` with save/load/delete operations
- Key files: `mod.rs`

**`gtk_ref/src/ui/`:**
- Purpose: GTK4/Relm4 UI components
- Contains: Relm4 component modules for chat, characters, settings, editors
- Key files: `mod.rs`, `chat/`, `characters.rs`, `settings/`, `character_card.rs`, `character_editor.rs`, `persona_editor.rs`, `lorebook_editor.rs`

**`docs/`:**
- Purpose: Design specification documents
- Contains: `Revelry_Design_Specification_v2.0.docx`
- Key files: `Revelry_Design_Specification_v2.0.docx`

**`.gsd/`:**
- Purpose: GSD (Get Shit Done) workflow framework
- Contains: Agent definitions, workflow templates, tools, contexts
- Key files: `agents/`, `get-shit-done/`

## Key File Locations

**Entry Points:**
- `vite.config.ts`: Vite dev server configuration (port 5173)
- `src/routes/+layout.svelte`: App shell — state hydration, keyboard shortcuts, layout structure
- `src/routes/+layout.ts`: SSR disabled (`ssr = false`, `prerender = false`)
- `gtk_ref/src/main.rs`: Rust/GTK4 application entry point

**Configuration:**
- `package.json`: Dependencies, scripts, project metadata
- `svelte.config.js`: SvelteKit adapter and `$lib` alias
- `tsconfig.json`: TypeScript strict mode, bundler resolution
- `vite.config.ts`: Vite plugins (Tailwind v4, SvelteKit), dev server port
- `gtk_ref/Cargo.toml`: Rust dependencies (GTK4, Relm4, rig-core, rusqlite, keyring)

**Core Logic:**
- `src/lib/stores/app.svelte.ts`: Central `AppState` — all reactive state, derived getters, action methods
- `src/lib/types/index.ts`: TypeScript interfaces mirroring spec §12.1 SQLite schema
- `src/lib/data/mock.ts`: Mock characters, personas, conversations, providers, presets
- `src/lib/utils/index.ts`: `cn()`, `estimateTokens()`, `formatRelativeTime()`, `avatarInitials()`
- `src/app.css`: Tailwind v4 theme tokens (colors, radii, fonts) + custom utilities
- `gtk_ref/src/app.rs`: Root Relm4 `AppModel` — view routing, DB worker, state management
- `gtk_ref/src/service/ai.rs`: `AiService` — rig-core provider-agnostic streaming
- `gtk_ref/src/service/prompt.rs`: `PromptEngine` — budget-aware prompt assembly
- `gtk_ref/src/db/mod.rs`: `DbService` — background-thread DB worker with `DbInput`/`DbOutput`
- `gtk_ref/src/db/migrations.rs`: Schema migration definitions
- `gtk_ref/src/db/models.rs`: Row→domain type mapper functions

**Testing:**
- `gtk_ref/src/secrets/mod.rs`: Contains `#[cfg(test)]` module with keyring unit tests
- `gtk_ref/src/service/ai.rs`: Contains `mod tests` with provider type and streaming tests
- `gtk_ref/src/service/prompt.rs`: Contains `mod tests` with prompt assembly and budget tests
- `gtk_ref/src/service/provider_test.rs`: Provider connectivity testing
- No SvelteKit test files exist yet (no `*.test.ts` or `*.spec.ts` files)

## Naming Conventions

**Files:**
- Svelte components: PascalCase — `MessageBubble.svelte`, `PersonaSwitcher.svelte`
- UI primitives: PascalCase — `Avatar.svelte`, `Button.svelte`
- Barrel exports: `index.ts` — re-exports all components from a directory (`src/lib/components/ui/index.ts`)
- Route files: SvelteKit convention — `+page.svelte`, `+page.ts`, `+layout.svelte`, `+layout.ts`
- TypeScript types: `index.ts` in types directory
- Store files: camelCase with `.svelte.ts` extension — `app.svelte.ts`
- Utility files: `index.ts` barrel in utils directory
- CSS: `app.css` at `src/` root
- HTML shell: `app.html` at `src/` root
- Rust modules: snake_case — `character_card.rs`, `prompt_engine.rs`, `db_service.rs`
- Rust domain files: snake_case matching struct name — `character.rs`, `conversation.rs`

**Directories:**
- Component groups: lowercase domain name — `chat/`, `layout/`, `characters/`, `personas/`, `ui/`
- Route segments: lowercase matching URL — `chat/`, `characters/`, `settings/`
- Dynamic route params: bracketed — `[id]/`
- Rust modules: lowercase snake_case — `domain/`, `service/`, `db/`, `secrets/`, `ui/`

## Where to Add New Code

**New Feature (UI page):**
- Primary code: `src/routes/[feature-name]/+page.svelte`
- Load function: `src/routes/[feature-name]/+page.ts` (if data loading needed)
- State: Add fields/actions to `src/lib/stores/app.svelte.ts`
- Types: Add interfaces to `src/lib/types/index.ts`

**New Component:**
- Domain component: `src/lib/components/[domain]/ComponentName.svelte`
- UI primitive: `src/lib/components/ui/ComponentName.svelte` + export from `src/lib/components/ui/index.ts`
- Variant component: Use `tailwind-variants` `tv()` in `<script module>`, export variants alongside default

**New Feature (Rust backend):**
- Domain model: `gtk_ref/src/domain/[model_name].rs` + register in `mod.rs`
- DB module: `gtk_ref/src/db/[table_name].rs` + add `DbInput`/`DbOutput` variants in `mod.rs`
- Service: `gtk_ref/src/service/[service_name].rs` + register in `mod.rs`
- UI component: `gtk_ref/src/ui/[component_name].rs` + register in `mod.rs`

**Utilities:**
- Shared helpers (SvelteKit): `src/lib/utils/index.ts`
- Shared helpers (Rust): Add to appropriate service module or create new module in `gtk_ref/src/service/`

**Tests:**
- Rust: Inline `#[cfg(test)] mod tests` in each source file
- SvelteKit: No test infrastructure yet — add `*.test.ts` alongside source files when testing is set up

## Special Directories

**`.svelte-kit/`:**
- Purpose: SvelteKit development server generated files
- Generated: Yes
- Committed: No (gitignored)

**`node_modules/`:**
- Purpose: NPM package dependencies
- Generated: Yes (by `npm install`)
- Committed: No (gitignored)

**`gtk_ref/target/`:**
- Purpose: Rust build artifacts
- Generated: Yes (by `cargo build`)
- Committed: No (gitignored)

**`.gsd/`:**
- Purpose: GSD workflow framework — agents, workflows, tools, contexts
- Generated: Partially (managed by GSD tooling)
- Committed: Yes

**`.planning/`:**
- Purpose: GSD planning artifacts — architecture docs, execution plans
- Generated: Yes (by GSD workflows)
- Committed: Yes (but never edited outside GSD workflows per `.rules`)

**`docs/`:**
- Purpose: Design specification documents
- Generated: No
- Committed: Yes

---

*Structure analysis: 2026-05-10*

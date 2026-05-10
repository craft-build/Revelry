# Revelry — UI

A **SvelteKit + Svelte 5 + Bits.ui** front-end for the Revelry AI roleplay desktop
application. This implements the user-facing layer described in the v2.0 design
specification, but swaps out the original GTK4/Relm4/libadwaita stack for a web
component stack that can be embedded in Tauri (or run directly in the browser
during development).

The original spec's design philosophy — content-first, progressive disclosure,
adaptive layout, native-feeling — is preserved. Only the rendering technology
has changed.

---

## Quick start

```bash
npm install
npm run dev
```

Open <http://localhost:5173>. The app will load with mock characters,
conversations, and personas so you can navigate every view without a backend.

> **Note:** the project assumes Node 20+. Network access is required for the
> initial install (Bits.ui, Tailwind v4, SvelteKit, lucide).

### Other scripts

```bash
npm run build      # production build
npm run preview    # preview the production build
npm run check      # svelte-check + tsc
npm run format     # prettier write
```

---

## Mapping to the design spec

The original spec's GTK4/libadwaita widgets all have direct counterparts in this
build (mirroring Appendix B of the design doc):

| Spec widget                                     | Web equivalent here                               |
| ----------------------------------------------- | ------------------------------------------------- |
| `AdwApplicationWindow`                          | `src/routes/+layout.svelte`                       |
| `AdwHeaderBar` + `AdwViewSwitcher`              | `src/lib/components/layout/AppHeader.svelte`      |
| `AdwOverlaySplitView` (sidebar pattern)         | `ConversationSidebar.svelte` + `+layout.svelte`   |
| `GtkListView` w/ message rows                   | `MessageBubble.svelte` rendered in a flex column  |
| `AdwAvatar`                                     | Bits.ui `Avatar` (`src/lib/components/ui/Avatar`) |
| `GtkPopover` (persona switcher)                 | Bits.ui `Popover` (`PersonaSwitcher.svelte`)      |
| `AdwPreferencesWindow`                          | `src/routes/settings/+page.svelte`                |
| `GtkFlowBox` (character grid)                   | CSS grid in `characters/+page.svelte`             |
| `GtkTextView` + `AdwClamp` (input)              | `MessageInput.svelte` (auto-grow textarea)        |
| `AdwToastOverlay`                               | _(stub — wire via Bits.ui Toast in Phase 2)_      |
| `GtkSearchEntry`                                | `Input` with `Search` icon                        |

The data model in `src/lib/types/index.ts` is a one-to-one TypeScript
translation of the SQLite schema in §12.1 of the spec — so the same shapes flow
through whether the backend is Rust+rusqlite (the spec) or a stubbed mock.

---

## Project structure

```
src/
├── app.css                    # Tailwind v4 theme tokens + base styles
├── app.html                   # HTML shell (Geist + Fraunces fonts)
├── lib/
│   ├── components/
│   │   ├── ui/                # Bits.ui-styled primitives (Avatar, Button…)
│   │   ├── layout/            # AppHeader, ConversationSidebar
│   │   ├── chat/              # ChatHeader, MessageBubble, MessageInput, PersonaSwitcher
│   │   ├── characters/        # CharacterCard
│   │   └── personas/          # PersonaCard
│   ├── stores/
│   │   └── app.svelte.ts      # Svelte 5 rune-based root state (≈ Relm4 AppModel)
│   ├── types/index.ts         # TS translation of the SQLite schema
│   ├── data/mock.ts           # Mock characters/personas/conversations
│   └── utils/index.ts         # cn(), token estimation, date format, initials
└── routes/
    ├── +layout.svelte         # App shell + state hydration + global keys
    ├── +layout.ts             # SSR off (desktop SPA)
    ├── +page.svelte           # Welcome state with suggested characters
    ├── chat/[id]/+page.svelte # Active conversation view
    ├── chat/[id]/+page.ts     # Loads messages for the conversation
    ├── characters/+page.svelte# Library: characters & personas tabs
    └── settings/+page.svelte  # Preferences (providers, generation, etc.)
```

### Why Svelte 5 runes?

The spec uses Relm4's Elm-style Model/Update/Message architecture. Svelte 5
runes (`$state`, `$derived`, `$effect`) give us very nearly the same ergonomics:
a single reactive state object that components subscribe to without manual
plumbing. `src/lib/stores/app.svelte.ts` is the closest analog to the
`AppModel` from §6.2.

### Why Bits.ui?

Bits.ui is a **headless** component library (the Svelte equivalent of Radix /
shadcn). That matters because the GNOME/Adwaita aesthetic the spec calls for is
specific and opinionated — we need full styling control. Bits.ui handles the
hard parts (focus management, keyboard navigation, ARIA) without imposing a
look. Used here for: `Avatar`, `Popover`, `Tooltip`, `Tabs`. Add more as needed
(`Dialog`, `Select`, `Toast`, `Slider`, `Switch` are the obvious next ones).

---

## What's wired vs. stubbed

### Wired

- App shell with header, sidebar, view switcher
- Conversation list with relative timestamps and search
- Active conversation view with message rendering and auto-scroll
- Message input with auto-grow, token estimate, context utilization bar
- **Mock streaming response** — typing in the chat triggers a fake token-by-
  token reply with the typewriter caret animation, so the UI is fully testable
  without a backend
- Persona switcher popover with active-persona indication
- Character & persona library with tab switching, search, and grid layout
- Settings shell with provider list and shortcut reference
- F9 to toggle sidebar (per spec §4.1.5)
- Ctrl+Enter and plain Enter to send

### Stubbed (clearly marked in code)

- **Backend bridge** — `MessageInput.svelte` and `chat/[id]/+page.svelte` fake
  the streaming locally. In production, replace `handleSend` with an IPC call
  to the Rust chat-engine over Tauri's `invoke()` or a similar transport
- **Real markdown rendering** — `MessageBubble.svelte` does naive
  asterisk-italic rendering only. Swap in `marked` / `markdown-it` (or render
  in Rust via `pulldown-cmark` and pass HTML strings) for the production build
- **Token counting** — `utils/index.ts:estimateTokens` uses a 4-char heuristic.
  The real value comes from `tiktoken-rs` via the backend
- **Character & persona editors** — only the cards and library views exist.
  The multi-step wizard from §7.4 still needs to be built (use Bits.ui's
  `Dialog` for the modal shell)
- **Lorebook UI** — entirely Tier-2 in the spec; deferred
- **Theme toggle, branch navigation, regex scripting, FTS search, vault mode**
  — all marked Phase 2/3 in the spec

---

## Embedding in Tauri

This UI is built as an SPA (`ssr = false` in the root layout) so it drops
straight into Tauri:

```bash
npm install -D @tauri-apps/cli
npx tauri init
```

Point Tauri's `tauri.conf.json` `frontendDist` at `build/` (after `npm run
build`) and Tauri serves the SPA in a webview. The Rust backend then exposes
`invoke()` commands that map to the chat-engine, card-service, persona-service,
etc. modules from §6.2 of the spec.

If you'd rather skip Tauri and ship it as a web app, the same code runs against
any HTTP backend that exposes the data shapes in `src/lib/types/index.ts`.

---

## Aesthetic notes

The look departs from strict Adwaita imitation in two intentional ways:

1. **Dark by default.** Most users running a creative roleplay tool at length
   are working at night. The spec's "respect system theme" goal is preserved
   in spirit (the `<html class="dark">` is the default, switchable later via
   `AdwStyleManager`-equivalent logic).

2. **Fraunces for character names.** A subtle editorial flourish — the
   serif treatment lets character names feel like they belong in a story
   rather than a chat app. Body and UI text remain Geist for clarity.

Both choices respect the "content-first, ruthless about visual noise" principle
from §4.1; they just adapt it to a web idiom.

---

## License

Same as the parent Revelry project.

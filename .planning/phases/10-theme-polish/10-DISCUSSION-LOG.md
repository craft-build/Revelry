# Phase 10: Theme & Polish - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-05-19
**Phase:** 10-theme-polish
**Areas discussed:** Light theme aesthetic, Theme toggle UX, Polish scope, Theme switching mechanics

---

## Light Theme Aesthetic

| Option | Description | Selected |
| ---------- | ---------------------------------- | -------- |
| Cool-toned (same hue) | Same 270 hue family as dark, just inverted lightness. Consistent identity. | ✓ |
| Warm/paper-like | Shift to warmer hue (30–60). Creamy backgrounds, book-like. | |
| Neutral/gray | Near-achromatic grays, teal accent pops more. | |

**User's choice:** Cool-toned (same hue)

| Option | Description | Selected |
| ---------- | ---------------------------------- | -------- |
| Bright/white-heavy | oklch(0.97–0.99), high contrast, crisp. | |
| Soft/off-white | oklch(0.94–0.97), slightly muted, easier on eyes. | ✓ |
| You decide | Agent calibrates lightness levels. | |

**User's choice:** Soft/off-white

**Notes:** Want the light mode to feel like "the same app in a different light" — same hue family, same accent, just lightness inverted. Soft surfaces like Notion/Bear, not stark white like Linear/Vercel.

---

## Theme Toggle UX

| Option | Description | Selected |
| ---------- | ---------------------------------- | -------- |
| Three-way toggle | System / Light / Dark. Default System. Standard desktop app pattern. | ✓ |
| Binary toggle only | Light or Dark, no OS detection. | |
| Segmented control | Three options as button group instead of dropdown. | |

**User's choice:** Three-way toggle

| Option | Description | Selected |
| ---------- | ---------------------------------- | -------- |
| Settings page only | In Appearance section. Standard location, stub already exists. | ✓ |
| Header + Settings | Quick toggle in AppHeader plus full control in Settings. | |
| Sidebar + Settings | Toggle at sidebar bottom plus full control in Settings. | |

**User's choice:** Settings page only

| Option | Description | Selected |
| ---------- | ---------------------------------- | -------- |
| Button group | Three adjacent buttons: System \| Light \| Dark. Active highlighted. | ✓ |
| Dropdown | Select element with three options. | |
| You decide | Agent picks best UI pattern. | |

**User's choice:** Button group

---

## Polish Scope

| Option | Description | Selected |
| ---------- | ---------------------------------- | -------- |
| Transitions + animations | Sidebar slide, page cross-fade, message appear. Standard desktop feel. | |
| Theme-only polish | Only theme toggle transition, skip other animation work. | |
| Full visual audit | Transitions, animations, PLUS spacing, hover states, focus rings audit. | ✓ |

**User's choice:** Full visual audit

| Option | Description | Selected |
| ---------- | ---------------------------------- | -------- |
| Subtle/fast (150–200ms) | Quiet — felt, not seen. Matches content-first philosophy. | ✓ |
| Expressive/playful (250–350ms) | More personality, elasticity. Potentially distracting. | |
| You decide | Agent picks durations and easing curves. | |

**User's choice:** Subtle/fast (150–200ms)

**Notes:** Content-first app with prose-heavy UI. Animations support writing experience, not distract from it. Build on existing `transition-colors` pattern.

---

## Theme Switching Mechanics

| Option | Description | Selected |
| ---------- | ---------------------------------- | -------- |
| Class-based token override | `.light` class overrides @theme tokens. Cleanest with Tailwind v4. | ✓ |
| Media query + class hybrid | @media for system, class for manual override. | |

**User's choice:** Class-based token override

| Option | Description | Selected |
| ---------- | ---------------------------------- | -------- |
| Inline script | <script> in app.html reads localStorage before hydration. No flash. | ✓ |
| Tauri-injected | Backend passes theme during init. Adds coupling. | |
| You decide | Agent picks best approach. | |

**User's choice:** Inline script

| Option | Description | Selected |
| ---------- | ---------------------------------- | -------- |
| Vitesse pair | vitesse-dark + vitesse-light. Same family, consistent feel. | ✓ |
| You decide | Agent picks matching light theme. | |

**User's choice:** Vitesse pair

**Notes:** Inline script reads from localStorage (set by app during normal operation) — not the Tauri backend. Backend persistence is for cross-session storage; localStorage is for instant flash prevention.

---

## Agent's Discretion

- Exact OKLCH lightness/chroma values for light tokens
- Easing curves for transitions
- Which components get appear animations vs. instant
- Sidebar slide implementation (CSS width vs. transform)
- Page route transition approach (Svelte `transition:` vs CSS)
- Order of polish audit execution

## Deferred Ideas

None — all discussion stayed within phase scope.

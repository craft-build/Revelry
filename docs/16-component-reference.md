<!-- Revelry Design Specification v2.0 — Section 16 -->
<!-- Last updated: 2026-05-20 -->

# Component Reference

This section documents all Svelte components in the Revelry UI, organized by category. Components follow Svelte 5 conventions: `$props()` for prop declaration, `children?: Snippet` for content projection, and `cn()` for class merging.

## UI Primitives

Components in `src/lib/components/ui/` — headless-style primitives styled with Tailwind CSS:

| Component                   | Purpose                                         | Key Props                         |
| --------------------------- | ----------------------------------------------- | --------------------------------- |
| `Avatar.svelte`             | Character/persona avatar with initials fallback | `src`, `name`, `size`             |
| `AvatarUpload.svelte`       | Avatar image upload with preview                | `onUpload`                        |
| `Button.svelte`             | Variant button with tailwind-variants           | `variant`, `size`, `class`        |
| `Dialog.svelte`             | Modal dialog overlay                            | `open`, `title`, `children`       |
| `Input.svelte`              | Text input with label                           | `value`, `placeholder`, `class`   |
| `KebabMenu.svelte`          | Three-dot context menu                          | `items`                           |
| `KebabMenuItem.svelte`      | Individual menu item                            | `label`, `icon`, `onClick`        |
| `KebabMenuSeparator.svelte` | Menu separator line                             | —                                 |
| `Logo.svelte`               | App logo display                                | `size`                            |
| `Select.svelte`             | Dropdown select                                 | `options`, `value`, `onChange`    |
| `StepperHeader.svelte`      | Wizard step indicator                           | `steps`, `current`                |
| `Tag.svelte`                | Badge/tag display                               | `label`, `color`                  |
| `Textarea.svelte`           | Auto-growing textarea                           | `value`, `placeholder`, `maxRows` |
| `Tooltip.svelte`            | Hover tooltip                                   | `content`, `children`             |

`Button.svelte` exports `buttonVariants` from a module script block, allowing consumers to reference variant styles without importing the component.

## Layout Components

Components in `src/lib/components/layout/`:

- **AppHeader.svelte** — Window header bar with `.drag-region` CSS utility for native window dragging, sidebar toggle button, theme toggle (`ThemeToggle`), and contextual actions. Combines window title area and navigation into a single space-efficient element.
- **ConversationSidebar.svelte** — Left sidebar displaying conversation list with relative timestamps, search filtering, persona indicators, and new conversation button. CSS-based layout with toggle visibility.

## Chat Components

Components in `src/lib/components/chat/`:

- **ChatHeader.svelte** — Active conversation header with character name (Fraunces serif), persona indicator, provider/model selector, and action buttons.
- **MessageBubble.svelte** — Single message display with markdown rendering, streaming text, hover-revealed edit/regenerate/delete/copy actions, branch navigation, and token count.
- **MessageInput.svelte** — Auto-grow textarea with send/stop button, token estimate, and context utilization bar. Enter sends, Ctrl+Enter inserts newline.
- **PersonaSwitcher.svelte** — Dropdown to select active persona for the current conversation.
- **MarkdownRenderer.svelte** — Renders completed AI messages using marked + DOMPurify + Shiki (vitesse-dark theme). During streaming, raw text is displayed instead.
- **BranchBadge.svelte** — Visual badge indicating a message has branched siblings.
- **BranchPopover.svelte** — Popover for navigating between message branches.
- **ConversationLorebookPanel.svelte** — Side panel for managing conversation-scoped lorebook entries.
- **LorebookDetailPanel.svelte** — Detail view/editor for a single lorebook entry.
- **ProviderModelSelector.svelte** — Combined provider and model selection dropdown.
- **GenerationPanel.svelte** — Panel for adjusting generation parameters per-conversation.

## Character Components

Components in `src/lib/components/characters/`:

- **CharacterCard.svelte** — Grid card item for the character library showing avatar, name, creator, tags, and description.
- **CharacterEditorDialog.svelte** — Multi-step wizard dialog for creating and editing characters (Identity → Personality → Scenario → Advanced) with `StepperHeader`.
- **DeleteConfirmationDialog.svelte** — Confirm-before-delete modal for character deletion.
- **ImportErrorDialog.svelte** — Error display for failed character card imports.
- **RegexScriptEditor.svelte** — Editor for regex post-processing scripts attached to characters, with test panel.

## Persona Components

Components in `src/lib/components/personas/`:

- **PersonaCard.svelte** — Grid card item for the persona library showing avatar, name, role, tags, and default badge.
- **PersonaEditorDialog.svelte** — Dialog for creating and editing user personas with tabbed layout (Identity, Appearance/Personality, Background/Relationships, Settings).

## Lorebook Components

Components in `src/lib/components/lorebooks/`:

- **LorebookCard.svelte** — Card display for a lorebook entry showing keywords, content preview, and activation state.
- **LorebookEditor.svelte** — Full lorebook management with keyword testing, entry CRUD, and character/conversation assignment.

## Provider Components

Components in `src/lib/components/providers/`:

- **ProviderEditorDialog.svelte** — Dialog for configuring LLM providers with name, type, base URL, API key (stored via keyring), and model list.
- **ModelEntryRow.svelte** — Row component for a single model entry within a provider configuration.
- **KeyringUnavailableBanner.svelte** — Warning banner displayed when the OS keychain is unavailable for API key storage.

## Settings Components

Components in `src/lib/components/settings/`:

- **ThemeToggle.svelte** — Dark/light theme switcher that toggles the `class="dark"` attribute on `<html>`.
- **PresetEditorDialog.svelte** — Dialog for creating and editing generation parameter presets.

## Component Conventions

All components follow these Svelte 5 patterns:

**Props declaration** — `$props()` with TypeScript interface:

```svelte
<script lang="ts">
  interface Props {
    class?: string;
    children?: Snippet;
  }
  let { class: className, children }: Props = $props();
</script>
```

**Content projection** — `children?: Snippet` + `{@render children?.()}` instead of `<slot />`.

**Class merging** — Every component accepting a `class` prop merges it through `cn()`:

```svelte
<button class={cn(buttonVariants({ variant }), className)}>
```

**Module script for variant exports** — Components that expose variant helpers use a `<script lang="ts" module>` block:

```svelte
<script lang="ts" module>
  import { tv } from 'tailwind-variants';
  export const buttonVariants = tv({ ... });
</script>
```

**Barrel exports** — UI primitives are re-exported from `src/lib/components/ui/index.ts`.

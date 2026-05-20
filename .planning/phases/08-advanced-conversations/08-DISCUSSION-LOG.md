# Phase 8: Advanced Conversations - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-18
**Phase:** 08-advanced-conversations
**Areas discussed:** Branching model, Branch navigation UX, Branch visualization, Search scope & UX

---

## Branching Model

| Option                   | Description                                                                                             | Selected |
| ------------------------ | ------------------------------------------------------------------------------------------------------- | -------- |
| Branch-grouped (gtk_ref) | message_branches table + branch_id on messages. Forks create a branch row; messages belong to branches. | ✓        |
| Tree model (parent_id)   | Each message points to parent. Branching is implicit via multiple children.                             |          |
| Hybrid                   | Both parent_id on messages AND message_branches table.                                                  |          |

**User's choice:** Branch-grouped (Recommended)
**Notes:** Matches gtk_ref reference implementation. Natural grouping, proven in production.

### Fork Trigger

| Option           | Description                                      | Selected |
| ---------------- | ------------------------------------------------ | -------- |
| From any message | Fork from user or AI messages. Standard pattern. | ✓        |
| AI messages only | Fork only on AI responses. More restrictive.     |          |

**User's choice:** From any message

### Shared History on Fork

| Option                  | Description                                                                | Selected |
| ----------------------- | -------------------------------------------------------------------------- | -------- |
| Shared history + branch | Messages up to fork point shared. New branch starts after. No duplication. | ✓        |
| Copy everything         | All messages duplicated with new branch_id. Self-contained branches.       |          |

**User's choice:** Shared history + branch

### Branch Labels

| Option                  | Description                                                                  | Selected |
| ----------------------- | ---------------------------------------------------------------------------- | -------- |
| Auto-labeled            | "Branch 1", "Branch 2", etc. Root is "Main". Rename later.                   | ✓        |
| User-named at fork time | Prompt user to name branch when forking. More descriptive but adds friction. |          |

**User's choice:** Auto-labeled (Recommended)

---

## Branch Navigation UX

| Option                   | Description                                                                              | Selected |
| ------------------------ | ---------------------------------------------------------------------------------------- | -------- |
| Inline badge + popover   | Badge on forked messages ("1/3"). Click opens popover with branch list. Matches gtk_ref. | ✓        |
| Breadcrumb bar           | Strip at top/bottom showing branch path. Click arrows to switch.                         |          |
| Both badge + context bar | Inline badges AND a context bar showing current branch.                                  |          |

**User's choice:** Inline badge + popover (Recommended)

---

## Branch Visualization

| Option                    | Description                                                                        | Selected |
| ------------------------- | ---------------------------------------------------------------------------------- | -------- |
| Flat with fork indicators | Single timeline. Badges at fork points. Switching swaps messages from fork onward. | ✓        |
| Split/timeline view       | Side-by-side panels showing branches simultaneously.                               |          |
| Tree/graph view           | Dendrogram visualization with clickable nodes.                                     |          |

**User's choice:** Flat with fork indicators

---

## Search Scope & UX

### Search Coverage

| Option                                | Description                                                | Selected |
| ------------------------------------- | ---------------------------------------------------------- | -------- |
| Conversations + messages              | FTS5 indexes both conversation titles and message content. | ✓        |
| Messages only                         | FTS5 on message content only. Title search stays LIKE.     |          |
| Conversations + messages + characters | Also index character info. Broadest scope.                 |          |

**User's choice:** Conversations + messages (Recommended)

### Search UI Location

| Option                  | Description                                                  | Selected |
| ----------------------- | ------------------------------------------------------------ | -------- |
| Enhanced sidebar search | Same search box, richer backend results. Minimal UI change.  | ✓        |
| Dedicated search panel  | New panel/modal with filters. More powerful, new UI surface. |          |
| Global search bar       | App header search with dropdown overlay.                     |          |

**User's choice:** Enhanced sidebar search

### Search Results Presentation

| Option                         | Description                                                                            | Selected |
| ------------------------------ | -------------------------------------------------------------------------------------- | -------- |
| Conversation + message preview | Show conversation in sidebar with matching message snippet. Click navigates + scrolls. | ✓        |
| Conversation list only         | Show matching conversations. User scrolls to find message.                             |          |
| Message-level results          | Individual messages as results with conversation context.                              |          |

**User's choice:** Conversation + message preview

---

## Branch Management

### Branch Deletion

| Option                 | Description                                                         | Selected |
| ---------------------- | ------------------------------------------------------------------- | -------- |
| Delete whole branch    | Remove entire branch + all its messages. Other branches unaffected. | ✓        |
| Delete messages only   | Individual message deletion within branches. More granular.         |          |
| Both options available | Delete individual messages OR entire branch.                        |          |

**User's choice:** Delete whole branch

### Fork + Generate

| Option                   | Description                                                                               | Selected |
| ------------------------ | ----------------------------------------------------------------------------------------- | -------- |
| Auto-generate on AI fork | Forking from AI message auto-generates new response on branch. Like branching regenerate. | ✓        |
| Empty branch only        | Forking always creates empty branch. User populates manually.                             |          |

**User's choice:** Auto-generate on AI fork

---

## Agent's Discretion

- Exact FTS5 schema design (unified vs separate tables)
- Snippet/highlight extraction from FTS5 results
- Branch popover component structure
- Branch-aware load_messages implementation
- Branch creation command structure (single vs multi-step)
- Migration numbering and structure
- Last active branch persistence (in-memory vs persisted)

## Deferred Ideas

None — discussion stayed within phase scope.

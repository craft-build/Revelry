# Phase 11: Spec Update & Distribution - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in 11-CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-05-20
**Phase:** 11-spec-update-distribution
**Areas discussed:** Design spec update scope, Cross-platform build targets, Distribution packaging

---

## Design Spec Conversion

| Option                  | Description                                                                 | Selected |
| ----------------------- | --------------------------------------------------------------------------- | -------- |
| Convert to Markdown     | Convert entire docx to Markdown. Better for version control and diffing.    | ✓        |
| Edit docx in-place      | Keep docx format, edit references directly.                                 |          |
| New Markdown companion  | Keep docx as archival, write new Markdown as live version.                  |          |

**User's choice:** Convert to Markdown
**Notes:** Multi-file breakdown (not single file). Delete original docx after verified conversion.

## Update Depth

| Option                      | Description                                                              | Selected |
| --------------------------- | ------------------------------------------------------------------------ | -------- |
| Reference swap only         | Replace GTK4/GNOME names with SvelteKit/Tauri equivalents only.         |          |
| Reference swap + reality check | Replace references AND update to reflect actual implementation.      | ✓        |
| Ground-up rewrite           | Full rewrite to match current architecture exactly.                      |          |

**User's choice:** Reference swap + reality check
**Notes:** Update sections to reflect how features were actually built — actual component names, state patterns, IPC patterns.

## Spec File Structure

| Option                    | Description                                                       | Selected |
| ------------------------- | ----------------------------------------------------------------- | -------- |
| Single comprehensive file | One big Markdown file with all sections.                          |          |
| Multi-file breakdown      | Split into separate .md files by section (architecture, data, UI). | ✓        |

**User's choice:** Multi-file breakdown

## Docx Archival

| Option              | Description                                               | Selected |
| ------------------- | --------------------------------------------------------- | -------- |
| Keep as archival    | Retain docx alongside new Markdown.                       |          |
| Delete after conversion | Remove docx once Markdown is verified.                | ✓        |

**User's choice:** Delete after conversion

## Cross-Platform Build Verification

| Option                       | Description                                                    | Selected |
| ---------------------------- | -------------------------------------------------------------- | -------- |
| Local macOS + GitHub CI      | Local DMG build, CI for Linux/Windows. Standard Tauri approach. | ✓        |
| GitHub CI for all three      | CI handles all platforms. Local build optional.                |          |
| macOS only for now           | Only verify macOS locally. Defer others.                       |          |

**User's choice:** Local macOS + GitHub CI

## Linux Packaging Format

| Option           | Description                                                    | Selected |
| ---------------- | -------------------------------------------------------------- | -------- |
| AppImage only    | Single file, no install, works across distros. Roadmap default. | ✓        |
| AppImage + .deb  | Also produce .deb for Debian/Ubuntu specifically.              |          |

**User's choice:** AppImage only

## Code Signing

| Option              | Description                                                           | Selected |
| ------------------- | --------------------------------------------------------------------- | -------- |
| Unsigned for now    | No signing. OS warnings on first launch. Addable later.               | ✓        |
| macOS signing only  | Apple Developer signing + notarization. Requires paid account.        |          |
| Full code signing   | All platforms signed. Requires multiple paid accounts.                |          |

**User's choice:** Unsigned for now

## Auto-Update

| Option                           | Description                                            | Selected |
| -------------------------------- | ------------------------------------------------------ | -------- |
| Skip for v1                      | Manual download only. Simpler phase. v2 requirement.   | ✓        |
| Tauri updater + GitHub Releases  | Built-in updater with GitHub as update server.         |          |

**User's choice:** Skip for v1

## Publishing Destination

| Option              | Description                                             | Selected |
| ------------------- | ------------------------------------------------------- | -------- |
| GitHub Releases     | Artifacts attached to releases via CI. Standard approach. | ✓        |
| Local builds only   | Just produce artifacts locally. No distribution setup.  |          |

**User's choice:** GitHub Releases

## Agent's Discretion

- Exact Markdown file structure and naming for multi-file spec
- Which sections to split vs keep together
- Diagram format (Mermaid vs ASCII)
- Windows installer format (MSI vs NSIS)
- CI trigger mechanism
- CI runner sizes and Rust build caching
- Logo file placement

## Deferred Ideas

- Code signing / notarization — requires paid developer accounts
- Auto-update — v2 requirement, Tauri has built-in support
- .deb packaging — can add later if needed
- App store distribution — separate effort

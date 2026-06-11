import {
  createHighlighter,
  createJavaScriptRegexEngine,
  bundledThemesInfo,
  type Highlighter,
  type BundledThemeInfo,
  type BundledTheme,
} from "shiki";

let highlighterInstance: Highlighter | null = null;
let highlighterPromise: Promise<Highlighter> | null = null;

const LANGS = [
  "javascript",
  "typescript",
  "python",
  "rust",
  "json",
  "html",
  "css",
  "bash",
  "markdown",
] as const;

const DEFAULT_THEMES = ["vitesse-dark", "vitesse-light"] as const;

export async function getShikiHighlighter(): Promise<Highlighter> {
  if (highlighterInstance) return highlighterInstance;
  if (highlighterPromise) return highlighterPromise;

  highlighterPromise = createHighlighter({
    themes: [...DEFAULT_THEMES],
    langs: [...LANGS],
    engine: createJavaScriptRegexEngine(),
  }).then((hl) => {
    highlighterInstance = hl;
    return hl;
  });

  return highlighterPromise;
}

/** Load a theme on-demand. No-op if already loaded. */
export async function loadShikiTheme(themeId: string): Promise<void> {
  const hl = await getShikiHighlighter();
  if (!hl.getLoadedThemes().includes(themeId)) {
    await hl.loadTheme(themeId as BundledTheme);
  }
}

export interface ThemeColors {
  fg: string;
  bg: string;
  muted: string;
  accent: string;
  border: string;
  /** Surface for code blocks / pre elements */
  surface: string;
}

/** Extract usable colors from a loaded Shiki theme for markdown styling. */
export async function getThemeColors(themeId: string): Promise<ThemeColors> {
  const hl = await getShikiHighlighter();
  const theme = hl.getTheme(themeId);

  // Walk token settings to find good color candidates
  let stringColor = theme.fg;
  let variableColor = theme.fg;
  let keywordColor = theme.fg;
  for (const s of theme.settings) {
    const scopes = Array.isArray(s.scope) ? s.scope : s.scope ? [s.scope] : [];
    if (scopes.some((sc) => typeof sc === "string" && sc.includes("string"))) {
      stringColor = s.settings.foreground ?? stringColor;
    }
    if (
      scopes.some((sc) => typeof sc === "string" && sc.includes("variable"))
    ) {
      variableColor = s.settings.foreground ?? variableColor;
    }
    if (scopes.some((sc) => typeof sc === "string" && sc.includes("keyword"))) {
      keywordColor = s.settings.foreground ?? keywordColor;
    }
  }

  // Use variable color for muted prose text (em, blockquotes) — it's a
  // mid-tone that's readable against the background, unlike comment colors
  // which are intentionally dim in most code themes.
  const muted = variableColor !== theme.fg ? variableColor : keywordColor;

  return {
    fg: theme.fg,
    bg: theme.bg,
    muted,
    accent: stringColor,
    border: keywordColor,
    surface: theme.bg,
  };
}

/** All built-in Shiki themes with dark/light metadata. */
export function getThemeCatalog(): BundledThemeInfo[] {
  return bundledThemesInfo;
}

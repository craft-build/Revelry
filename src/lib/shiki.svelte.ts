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

/** Parse a hex color (#rgb/#rgba/#rrggbb/#rrggbbaa) to [r, g, b, a]. */
function parseHex(hex: string): [number, number, number, number] | null {
  const m = /^#([\da-f]{3,8})$/i.exec(hex.trim());
  if (!m) return null;
  let h = m[1];
  if (h.length === 3 || h.length === 4) {
    h = [...h].map((c) => c + c).join("");
  }
  if (h.length !== 6 && h.length !== 8) return null;
  return [
    parseInt(h.slice(0, 2), 16),
    parseInt(h.slice(2, 4), 16),
    parseInt(h.slice(4, 6), 16),
    h.length === 8 ? parseInt(h.slice(6, 8), 16) / 255 : 1,
  ];
}

/** Linear mix of two hex colors (`t` = weight of `b`). Falls back to `a`. */
function mixHex(a: string, b: string, t: number): string {
  const ca = parseHex(a);
  const cb = parseHex(b);
  if (!ca || !cb) return a;
  const mix = (x: number, y: number) => Math.round(x + (y - x) * t);
  return (
    "#" +
    [mix(ca[0], cb[0]), mix(ca[1], cb[1]), mix(ca[2], cb[2])]
      .map((v) => v.toString(16).padStart(2, "0"))
      .join("")
  );
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

  // Some themes declare a (semi-)transparent background, which would make
  // bubbles invisible — substitute the app's neutral surface for those.
  let bg = theme.bg;
  const parsedBg = parseHex(bg);
  if (parsedBg && parsedBg[3] < 0.5) {
    bg =
      (theme as { type?: "dark" | "light" }).type === "light"
        ? "#ffffff"
        : "#161928";
  }

  return {
    fg: theme.fg,
    bg,
    muted,
    accent: stringColor,
    border: keywordColor,
    // Lift the theme bg slightly toward its foreground so inline code chips,
    // fenced blocks and table headers stay visible now that bubble
    // backgrounds use the theme bg itself.
    surface: mixHex(bg, theme.fg, 0.07),
  };
}

/** All built-in Shiki themes with dark/light metadata. */
export function getThemeCatalog(): BundledThemeInfo[] {
  return bundledThemesInfo;
}

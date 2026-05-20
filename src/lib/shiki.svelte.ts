import { createHighlighter, type Highlighter } from "shiki";

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

const THEMES = ["vitesse-dark", "vitesse-light"] as const;

export async function getShikiHighlighter(): Promise<Highlighter> {
	if (highlighterInstance) return highlighterInstance;
	if (highlighterPromise) return highlighterPromise;

	highlighterPromise = createHighlighter({
		themes: [...THEMES],
		langs: [...LANGS],
	}).then((hl) => {
		highlighterInstance = hl;
		return hl;
	});

	return highlighterPromise;
}

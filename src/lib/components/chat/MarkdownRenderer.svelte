<script lang="ts">
	import { marked } from 'marked';
	import DOMPurify from 'dompurify';
	import { onMount } from 'svelte';
	import { getShikiHighlighter, loadShikiTheme } from '$lib/shiki.svelte';
	import { app } from '$lib/stores/app.svelte';

	interface Props {
		content: string;
	}

	let { content }: Props = $props();

	let renderedHtml = $state<string>('');
	let highlighter = $state<Awaited<ReturnType<typeof getShikiHighlighter>> | null>(null);
	let containerEl: HTMLDivElement | undefined = $state();
	let themeVersion = $state(0);
	// Shared palette from AppState — same source that drives bubble
	// backgrounds, so prose always matches the surface it sits on.
	let themeColors = $derived(app.code_theme_colors);

	// Preload highlighter with common languages
	onMount(async () => {
		try {
			highlighter = await getShikiHighlighter();
		} catch (err) {
			console.warn('Failed to load Shiki highlighter:', err);
		}
	});

	// Re-render markdown when content or theme changes
	$effect(() => {
		const rawContent = content;

		// Eagerly ensure the current code theme is loaded before rendering
		const shikiTheme = app.code_theme;
		let themeReady = highlighter?.getLoadedThemes().includes(shikiTheme) ?? false;

		// Bump reactive counter after async load so the effect re-runs
		const _v = themeVersion;

		if (highlighter && !themeReady) {
			loadShikiTheme(shikiTheme)
				.then(() => {
					themeVersion++;
				})
				.catch((err) => {
					console.warn('Failed to load Shiki theme:', shikiTheme, err);
				});
		}

		const renderer = new marked.Renderer();
		const hl = highlighter;

		// Custom code block renderer with Shiki highlighting
		renderer.code = function ({ text, lang }: { text: string; lang?: string }) {
			if (lang && hl && themeReady) {
				try {
					if (hl.getLoadedLanguages().includes(lang)) {
						const highlighted = hl.codeToHtml(text, { lang, theme: shikiTheme });
						return `<div class="relative"><span class="lang-label">${lang}</span>${highlighted}</div>`;
					}
			} catch {
				// Fall through to plain code block
			}
			}
			// No language, Shiki not loaded, or theme not ready — plain code block
			const langAttr = lang ? ` class="language-${lang}"` : '';
			const langLabel = lang ? `<span class="lang-label">${lang}</span>` : '';
			return `<div class="relative">${langLabel}<pre><code${langAttr}>${escapeHtml(text)}</code></pre></div>`;
		};

		marked.setOptions({
			gfm: true,
			breaks: false,
			renderer
		});

		const rawHtml = marked.parse(rawContent) as string;
		const sanitized = DOMPurify.sanitize(rawHtml, {
			ALLOWED_TAGS: [
				'p',
				'br',
				'strong',
				'em',
				'del',
				'h1',
				'h2',
				'h3',
				'h4',
				'h5',
				'h6',
				'ul',
				'ol',
				'li',
				'a',
				'blockquote',
				'pre',
				'code',
				'hr',
				'table',
				'thead',
				'tbody',
				'tr',
				'th',
				'td',
				'img',
				'input',
				'span',
				'div'
			],
			ALLOWED_ATTR: [
				'href',
				'src',
				'alt',
				'class',
				'type',
				'checked',
				'disabled',
				'data-language',
				'style'
			],
			ALLOW_DATA_ATTR: false
		});

		renderedHtml = sanitized;
	});

	function escapeHtml(text: string): string {
		return text
			.replace(/&/g, '&amp;')
			.replace(/</g, '&lt;')
			.replace(/>/g, '&gt;')
			.replace(/"/g, '&quot;');
	}
</script>

<div
	class="prose-markdown"
	bind:this={containerEl}
	style={themeColors
		? `--md-fg: ${themeColors.fg}; --md-bg: ${themeColors.bg}; --md-muted: ${themeColors.muted}; --md-accent: ${themeColors.accent}; --md-border: ${themeColors.border}; --md-surface: ${themeColors.surface};`
		: ''}
>
	<!-- eslint-disable-next-line svelte/no-at-html-tags -->
	{@html renderedHtml}
</div>

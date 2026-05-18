<script lang="ts">
	import { marked } from 'marked';
	import DOMPurify from 'dompurify';
	import { createHighlighter, type Highlighter } from 'shiki';
	import { onMount } from 'svelte';

	interface Props {
		content: string;
	}

	let { content }: Props = $props();

	let renderedHtml = $state<string>('');
	let highlighter = $state<Highlighter | null>(null);
	let containerEl: HTMLDivElement | undefined = $state();

	// Preload highlighter with common languages
	onMount(async () => {
		try {
			highlighter = await createHighlighter({
				themes: ['vitesse-dark'],
				langs: [
					'javascript',
					'typescript',
					'python',
					'rust',
					'json',
					'html',
					'css',
					'bash',
					'markdown'
				]
			});
		} catch (err) {
			console.warn('Failed to load Shiki highlighter:', err);
		}
	});

	// Re-render markdown when content changes
	$effect(() => {
		const rawContent = content;

		const renderer = new marked.Renderer();
		const hl = highlighter;

		// Custom code block renderer with Shiki highlighting
		renderer.code = function ({ text, lang }: { text: string; lang?: string }) {
			if (lang && hl) {
				try {
					if (
						hl.getLoadedLanguages().includes(lang) &&
						hl.getLoadedThemes().includes('vitesse-dark')
					) {
						const highlighted = hl.codeToHtml(text, { lang, theme: 'vitesse-dark' });
						return `<div class="relative"><span class="lang-label">${lang}</span>${highlighted}</div>`;
					}
				} catch {
					// Fall through to plain code block
				}
			}
			// No language or Shiki not loaded — plain code block
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

<div class="prose-markdown" bind:this={containerEl}>
	<!-- eslint-disable-next-line svelte/no-at-html-tags -->
	{@html renderedHtml}
</div>

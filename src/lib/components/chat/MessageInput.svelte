<script lang="ts">
	import { Send, Square, Paperclip } from '@lucide/svelte';
	import { app } from '$lib/stores/app.svelte';
	import { estimateTokens } from '$lib/utils';

	interface Props {
		onsend?: (text: string) => void;
		oncancel?: () => void;
	}

	let { onsend, oncancel }: Props = $props();

	let text = $state('');
	let textarea: HTMLTextAreaElement | undefined = $state();

	let tokens = $derived(estimateTokens(text));

	// Approximate context utilization across all in-context messages.
	// Real value comes from the backend's chat-engine token budget calc.
	let usedContext = $derived(
		app.messages.reduce((sum, m) => sum + (m.token_count ?? estimateTokens(m.content)), 0) + tokens
	);
	const contextLimit = 8192;
	let contextPct = $derived(Math.min(100, (usedContext / contextLimit) * 100));

	function handleSubmit() {
		const trimmed = text.trim();
		if (!trimmed || app.is_generating) return;
		onsend?.(trimmed);
		text = '';
		// Reset auto-grow
		if (textarea) textarea.style.height = 'auto';
	}

	function handleKey(e: KeyboardEvent) {
		// Ctrl/Cmd+Enter — send
		if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
			e.preventDefault();
			handleSubmit();
		}
		// Plain Enter without shift — send (chat convention)
		if (e.key === 'Enter' && !e.shiftKey && !e.metaKey && !e.ctrlKey) {
			e.preventDefault();
			handleSubmit();
		}
	}

	function autogrow() {
		if (!textarea) return;
		textarea.style.height = 'auto';
		textarea.style.height = `${Math.min(textarea.scrollHeight, 240)}px`;
	}
</script>

<div class="border-t border-(--color-border) bg-(--color-surface) px-6 py-4">
	<!-- AdwClamp equivalent: max width on wide screens -->
	<div class="mx-auto max-w-[80ch]">
		<!-- Context bar -->
		<div class="mb-2 flex items-center justify-between gap-3 px-1 text-[10px] uppercase tracking-wider text-(--color-subtle)">
			<div class="flex items-center gap-2">
				<span>{usedContext.toLocaleString()} / {contextLimit.toLocaleString()} tok</span>
				<div class="h-1 w-32 overflow-hidden rounded-full bg-(--color-surface-elevated)">
					<div
						class="h-full bg-(--color-accent) transition-[width] duration-300"
						style:width="{contextPct}%"
					></div>
				</div>
			</div>
			<div class="flex items-center gap-2">
				<span>Enter to send · Shift+Enter for newline</span>
			</div>
		</div>

		<!-- Input frame -->
		<div
			class="flex items-end gap-2 rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface-input) p-2 focus-within:border-(--color-accent)"
		>
			<button
				type="button"
				class="inline-flex size-8 shrink-0 items-center justify-center rounded-(--radius-sm) text-(--color-subtle) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
				aria-label="Attach"
			>
				<Paperclip class="size-4" />
			</button>

			<textarea
				bind:this={textarea}
				bind:value={text}
				oninput={autogrow}
				onkeydown={handleKey}
				placeholder="Message {app.active_character?.name ?? 'character'}…"
				rows="1"
				class="block flex-1 resize-none bg-transparent px-1 py-1.5 text-[15px] leading-relaxed text-(--color-foreground) placeholder:text-(--color-subtle) focus:outline-none"
			></textarea>

			{#if app.is_generating}
				<button
					type="button"
					onclick={() => oncancel?.()}
					class="inline-flex size-8 shrink-0 items-center justify-center rounded-(--radius-sm) bg-(--color-destructive) text-white hover:bg-(--color-destructive)/90"
					aria-label="Stop generation"
				>
					<Square class="size-3.5" fill="currentColor" />
				</button>
			{:else}
				<button
					type="button"
					onclick={handleSubmit}
					disabled={!text.trim()}
					class="inline-flex size-8 shrink-0 items-center justify-center rounded-(--radius-sm) bg-(--color-accent) text-(--color-accent-fg) hover:bg-(--color-accent)/90 disabled:cursor-not-allowed disabled:opacity-50"
					aria-label="Send"
				>
					<Send class="size-3.5" />
				</button>
			{/if}
		</div>
	</div>
</div>

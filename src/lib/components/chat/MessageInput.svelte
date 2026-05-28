<script lang="ts">
	import { Send, Square } from '@lucide/svelte';
	import { app } from '$lib/stores/app.svelte';
	import LorebookDetailPanel from './LorebookDetailPanel.svelte';

	interface Props {
		onsend?: (text: string) => void;
		oncancel?: () => void;
	}

	let { onsend, oncancel }: Props = $props();

	let text = $state('');
	let textarea: HTMLTextAreaElement | undefined = $state();

	let usedContext = $derived(app.context_budget?.tokensUsed ?? 0);
	let contextLimit = $derived(app.effective_context_limit);
	let contextPct = $derived(
		contextLimit > 0 ? Math.min(100, (usedContext / contextLimit) * 100) : 0,
	);

	/** Returns the semantic color token for the progress bar fill based on utilization. */
	let barColor = $derived.by(() => {
		if (contextPct > 95) return 'bg-(--color-destructive)';
		if (contextPct >= 80) return 'bg-(--color-warning)';
		return 'bg-(--color-success)';
	});

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
						class="h-full {barColor} transition-[width,background-color] duration-300"
						style:width="{contextPct}%"
					></div>
				</div>
			</div>
			<div class="flex items-center gap-2">
				<span>Enter to send · Shift+Enter for newline</span>
				<span class="text-(--color-subtle)/60">· <code class="text-[10px]">/steer</code> to instruct</span>
			</div>
		</div>

		<!-- Lorebook detail panel (expandable) -->
		<LorebookDetailPanel />

		<!-- Input frame -->
		<div
			class="flex items-end gap-2 rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface-input) p-2 focus-within:border-(--color-accent)"
		>
			<textarea
				bind:this={textarea}
				bind:value={text}
				oninput={autogrow}
				onkeydown={handleKey}
				placeholder={`Message ${app.active_character?.name ?? 'character'}…`}
				rows="1"
				disabled={app.is_generating}
				class="block flex-1 resize-none bg-transparent px-1 py-1.5 text-[15px] leading-relaxed text-(--color-foreground) placeholder:text-(--color-subtle) focus:outline-none disabled:opacity-50"
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

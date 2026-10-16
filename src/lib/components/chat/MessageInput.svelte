<script lang="ts">
	import { ArrowUp, Square } from '@lucide/svelte';
	import { app } from '$lib/stores/app.svelte';
	import { Tooltip } from '$lib/components/ui';
	import LorebookDetailPanel from './LorebookDetailPanel.svelte';
	import PersonaSwitcher from './PersonaSwitcher.svelte';
	import ProviderModelSelector from './ProviderModelSelector.svelte';

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

	/** Returns the semantic color token for the meter fill based on utilization. */
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

<!-- Floating composer: an xl-radius elevated card carrying persona + model
     chips and a mini context meter, with the hint line beneath. -->
<div class="px-6 pb-[18px]">
	<!-- AdwClamp equivalent: max width on wide screens -->
	<div class="mx-auto max-w-[80ch]">
		<!-- Lorebook detail panel (expandable) -->
		<div class="px-1.5 pb-2">
			<LorebookDetailPanel />
		</div>

		<div
			class="rounded-(--radius-xl) border border-(--color-border-strong) bg-(--color-surface-elevated) shadow-lg transition-[box-shadow,border-color] duration-150 focus-within:border-(--color-accent)/60 focus-within:shadow-[var(--shadow-lg),var(--shadow-glow)]"
		>
			<textarea
				bind:this={textarea}
				bind:value={text}
				oninput={autogrow}
				onkeydown={handleKey}
				placeholder={`Message ${app.active_character?.name ?? 'character'}…`}
				rows="1"
				disabled={app.is_generating}
				class="block w-full resize-none bg-transparent px-4 pt-3.5 pb-1.5 text-[15px] leading-[1.6] text-(--color-foreground) placeholder:text-(--color-subtle) focus:outline-none disabled:opacity-50"
			></textarea>

			<div class="flex items-center gap-1.5 pt-1.5 pr-2 pb-2 pl-2.5">
				<PersonaSwitcher />
				<ProviderModelSelector />

				<div class="flex-1"></div>

				<!-- Mini context meter -->
				<Tooltip
					text={`${usedContext.toLocaleString()} / ${contextLimit.toLocaleString()} tokens in context`}
				>
					<span class="flex items-center gap-2 px-1.5 font-mono text-[10px] text-(--color-subtle)">
						<span>{(usedContext / 1000).toFixed(1)}k</span>
						<span class="block h-1 w-14 overflow-hidden rounded-full bg-(--color-surface-input)">
							<span
								class="block h-full {barColor} transition-[width,background-color] duration-300"
								style:width="{Math.max(contextPct, 2)}%"
							></span>
						</span>
					</span>
				</Tooltip>

				{#if app.is_generating}
					<button
						type="button"
						onclick={() => oncancel?.()}
						class="inline-flex size-[34px] shrink-0 items-center justify-center rounded-(--radius-md) bg-(--color-destructive) text-white transition-colors hover:bg-(--color-destructive)/90"
						aria-label="Stop generation"
					>
						<Square class="size-3.5" fill="currentColor" />
					</button>
				{:else}
					<button
						type="button"
						onclick={handleSubmit}
						disabled={!text.trim()}
						class="inline-flex size-[34px] shrink-0 items-center justify-center rounded-(--radius-md) bg-(--color-accent) text-(--color-accent-fg) transition-[background-color,transform] duration-150 hover:bg-(--color-accent-hover) active:scale-[0.98] active:bg-(--color-accent-active) disabled:cursor-not-allowed disabled:opacity-45"
						aria-label="Send"
					>
						<ArrowUp class="size-4" />
					</button>
				{/if}
			</div>
		</div>

		<div class="mt-2 text-center text-[10px] tracking-wider text-(--color-subtle) uppercase">
			Enter to send · Shift+Enter for newline ·
			<code class="font-mono text-[10px] normal-case">/steer</code> to instruct
		</div>
	</div>
</div>

<script lang="ts">
	import { app } from '$lib/stores/app.svelte';
	import { ChevronRight, ChevronDown, Check } from '@lucide/svelte';
	import { cn } from '$lib/utils';

	let isOpen = $derived(app.lorebook_detail_open);
	let detailCount = $derived(app.lorebook_details.length);

	function toggle() {
		app.lorebook_detail_open = !app.lorebook_detail_open;
	}

	function truncate(str: string, max: number): string {
		if (!str) return '';
		return str.length > max ? str.slice(0, max) + '...' : str;
	}
</script>

<div class="text-[10px]">
	<!-- Collapsed / toggle header -->
	<button
		type="button"
		onclick={toggle}
		class="flex w-full items-center gap-1 px-1 text-left transition-colors hover:text-(--color-foreground)"
	>
		{#if isOpen}
			<ChevronDown class="size-3 shrink-0" />
		{:else}
			<ChevronRight class="size-3 shrink-0" />
		{/if}
		<span
			class={cn(
				'uppercase tracking-wider',
				detailCount > 0
					? 'text-(--color-accent-vivid)'
					: 'text-(--color-subtle)',
			)}
		>
			{#if detailCount > 0}
				Lorebook: {detailCount} active
			{:else}
				Lorebook: none active
			{/if}
		</span>
	</button>

	<!-- Expanded detail list -->
	{#if isOpen && detailCount > 0}
		<div class="mt-1 max-h-48 space-y-1 overflow-y-auto px-1 pb-2">
			{#each app.lorebook_details as detail (detail.entry_name)}
				<div
					class="flex items-center gap-2 rounded-(--radius-sm) px-2 py-1.5 bg-(--color-surface-elevated)"
				>
					<div class="min-w-0 flex-1">
						<div class="flex items-center gap-1.5">
							<span class="text-sm text-(--color-foreground) truncate">
								{detail.entry_name || 'Untitled entry'}
							</span>
							{#if detail.matched_keys && detail.matched_keys.length > 0}
								<Check class="size-3 shrink-0 text-(--color-success)" />
							{:else}
								<span class="text-[10px] text-(--color-subtle) shrink-0">constant</span>
							{/if}
						</div>
						<p class="text-xs text-(--color-muted) truncate">
							{truncate(detail.content_snippet, 30)}
						</p>
					</div>
					<span
						class="shrink-0 text-[10px] font-mono text-(--color-subtle) bg-(--color-surface) rounded-(--radius-sm) px-1.5 py-0.5"
					>
						{detail.token_cost} tok
					</span>
				</div>
			{/each}
		</div>
	{/if}
</div>

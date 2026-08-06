<script lang="ts">
	import type { Lorebook } from '$lib/types';
	import { KebabMenu, KebabMenuItem, KebabMenuSeparator } from '$lib/components/ui';
	import { BookOpen, Pencil, Trash2 } from '@lucide/svelte';

	interface Props {
		lorebook: Lorebook;
		onedit?: (id: string) => void;
		ondelete?: (id: string) => void;
	}

	let { lorebook, onedit, ondelete }: Props = $props();
</script>

<article
	class="group relative flex flex-col gap-3 rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface) p-4 transition-all hover:border-(--color-border-strong) hover:bg-(--color-surface-elevated) hover:shadow-md"
>
	<!-- Top: icon + name -->
	<div class="flex items-start gap-3">
		<div
			class="flex size-12 shrink-0 items-center justify-center rounded-(--radius-md) bg-(--color-accent)/10 text-(--color-accent-vivid)"
		>
			<BookOpen class="size-6" />
		</div>
		<div class="min-w-0 flex-1 pt-1">
			<h3
				class="truncate font-display text-lg font-semibold leading-tight tracking-tight"
			>
				{lorebook.name}
			</h3>
			{#if lorebook.description}
				<p class="mt-0.5 line-clamp-2 text-sm leading-relaxed text-(--color-muted)">
					{lorebook.description}
				</p>
			{:else}
				<p class="mt-0.5 text-sm italic text-(--color-subtle)">No description</p>
			{/if}
		</div>
	</div>

	<!-- Actions -->
	<div class="mt-auto flex items-center justify-end pt-2">
		<KebabMenu>
			<KebabMenuItem onclick={() => onedit?.(lorebook.id)}>
				<Pencil class="size-3.5" />
				Edit
			</KebabMenuItem>
			<KebabMenuSeparator />
			<KebabMenuItem destructive onclick={() => ondelete?.(lorebook.id)}>
				<Trash2 class="size-3.5" />
				Delete
			</KebabMenuItem>
		</KebabMenu>
	</div>
</article>

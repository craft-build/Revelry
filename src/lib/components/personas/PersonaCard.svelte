<script lang="ts">
	import type { UserPersona } from '$lib/types';
	import { Avatar, Tag, KebabMenu, KebabMenuItem, KebabMenuSeparator } from '$lib/components/ui';
	import { Pencil, Trash2 } from '@lucide/svelte';

	interface Props {
		persona: UserPersona;
		onedit?: (id: string) => void;
		ondelete?: (id: string) => void;
	}

	let { persona, onedit, ondelete }: Props = $props();
</script>

<article
	class="group relative flex flex-col gap-3 rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface) p-4 transition-all hover:border-(--color-border-strong) hover:bg-(--color-surface-elevated) hover:shadow-md"
>
	<div class="flex items-start gap-3">
		<Avatar name={persona.name} src={persona.avatar_path} size="lg" />
		<div class="min-w-0 flex-1 pt-1">
			<h3 class="truncate font-display text-lg font-semibold leading-tight tracking-tight">
				{persona.name}
			</h3>
			{#if persona.role_description}
				<p class="truncate text-[11px] text-(--color-subtle)">{persona.role_description}</p>
			{/if}
		</div>
	</div>

	<p class="line-clamp-3 text-sm leading-relaxed text-(--color-muted)">
		{persona.personality}
	</p>

	{#if persona.tags && persona.tags.length > 0}
		<div class="flex flex-wrap gap-1">
			{#each persona.tags as tag (tag)}
				<Tag>{tag}</Tag>
			{/each}
		</div>
	{/if}

	<div class="mt-auto flex items-center gap-2 pt-2">
		<KebabMenu>
			<KebabMenuItem onclick={() => onedit?.(persona.id)}>
				<Pencil class="size-3.5" />
				Edit
			</KebabMenuItem>
			<KebabMenuSeparator />
			<KebabMenuItem destructive onclick={() => ondelete?.(persona.id)}>
				<Trash2 class="size-3.5" />
				Delete
			</KebabMenuItem>
		</KebabMenu>
	</div>
</article>

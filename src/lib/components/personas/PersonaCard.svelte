<script lang="ts">
	import type { UserPersona } from '$lib/types';
	import { Avatar, Tag } from '$lib/components/ui';
	import { Star, Pencil, Copy } from '@lucide/svelte';

	interface Props {
		persona: UserPersona;
		onedit?: (id: string) => void;
	}

	let { persona, onedit }: Props = $props();
</script>

<article
	class="group relative flex flex-col gap-3 rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface) p-4 transition-all hover:border-(--color-border-strong) hover:bg-(--color-surface-elevated)"
>
	{#if persona.is_default}
		<div
			class="absolute right-3 top-3 inline-flex items-center gap-1 rounded-full bg-(--color-accent-muted)/40 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wider text-(--color-accent)"
		>
			<Star class="size-2.5" fill="currentColor" />
			Default
		</div>
	{/if}

	<div class="flex items-start gap-3">
		<Avatar name={persona.name} src={persona.avatar_url} size="lg" />
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
		<button
			type="button"
			onclick={() => onedit?.(persona.id)}
			class="inline-flex h-8 flex-1 items-center justify-center gap-1.5 rounded-(--radius-md) border border-(--color-border) px-3 text-xs font-medium hover:bg-(--color-surface-hover)"
		>
			<Pencil class="size-3.5" />
			Edit
		</button>
		<button
			type="button"
			class="inline-flex size-8 items-center justify-center rounded-(--radius-md) border border-(--color-border) text-(--color-muted) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
			aria-label="Duplicate persona"
		>
			<Copy class="size-3.5" />
		</button>
	</div>
</article>

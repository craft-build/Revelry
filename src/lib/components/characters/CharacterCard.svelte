<script lang="ts">
	import type { CharacterCard } from '$lib/types';
	import { app } from '$lib/stores/app.svelte';
	import { Avatar, Tag, KebabMenu, KebabMenuItem, KebabMenuSeparator } from '$lib/components/ui';
	import { MessageSquare, Pencil, Trash2, Download, Eraser } from '@lucide/svelte';

	interface Props {
		character: CharacterCard;
		onchat?: (id: string) => void;
		onedit?: (id: string) => void;
		ondelete?: (id: string) => void;
		onexport?: (id: string) => void;
		onforget?: (id: string) => void;
		onrestore?: (id: string) => void;
	}

	let { character, onchat, onedit, ondelete, onexport, onforget, onrestore }: Props =
		$props();

	let memory_relevant = $derived(
		app.embeddings_enabled && app.cross_conversation_memory
	);
	let memories_forgotten = $derived(app.forgotten_memories.has(character.id));
</script>

<article
	class="group relative flex flex-col gap-3 rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface) p-4 transition-all hover:border-(--color-border-strong) hover:bg-(--color-surface-elevated) hover:shadow-md"
>
	<!-- Top: avatar + name -->
	<div class="flex items-start gap-3">
		<Avatar name={character.name} src={character.avatar_path} size="lg" enlargeable />
		<div class="min-w-0 flex-1 pt-1">
			<h3 class="truncate font-display text-lg font-semibold leading-tight tracking-tight">
				{character.name}
			</h3>
			{#if character.creator}
				<p class="text-[11px] text-(--color-subtle)">by {character.creator}</p>
			{/if}
		</div>
	</div>

	<!-- Description -->
	<p class="line-clamp-3 text-sm leading-relaxed text-(--color-muted)">
		{character.description}
	</p>

	<!-- Forgotten-memory indicator -->
	{#if memories_forgotten}
		<p
			class="inline-flex w-fit items-center gap-1.5 rounded-(--radius-sm) bg-(--color-surface-input) px-2 py-1 text-[11px] text-(--color-subtle)"
			title="Their cross-conversation memories were forgotten — nothing before that moment is recalled"
		>
			<Eraser class="size-3" />
			cross-chat memories forgotten
		</p>
	{/if}

	<!-- Tags -->
	{#if character.tags && character.tags.length > 0}
		<div class="flex flex-wrap gap-1">
			{#each character.tags as tag (tag)}
				<Tag>{tag}</Tag>
			{/each}
		</div>
	{/if}

	<!-- Actions -->
	<div class="mt-auto flex items-center gap-2 pt-2">
		<button
			type="button"
			onclick={() => onchat?.(character.id)}
			class="inline-flex h-8 flex-1 items-center justify-center gap-1.5 rounded-(--radius-lg) bg-(--color-accent) px-3 text-xs font-semibold text-(--color-accent-fg) hover:bg-(--color-accent-hover)"
		>
			<MessageSquare class="size-3.5" />
			Start chat
		</button>
		<KebabMenu>
			<KebabMenuItem onclick={() => onedit?.(character.id)}>
				<Pencil class="size-3.5" />
				Edit
			</KebabMenuItem>
			<KebabMenuItem onclick={() => onexport?.(character.id)}>
				<Download class="size-3.5" />
				Export
			</KebabMenuItem>
			{#if memory_relevant && (onforget || onrestore)}
				{#if memories_forgotten}
					<KebabMenuItem onclick={() => onrestore?.(character.id)}>
						<Eraser class="size-3.5" />
						Restore forgotten memories
					</KebabMenuItem>
				{:else if onforget}
					<KebabMenuItem onclick={() => onforget(character.id)}>
						<Eraser class="size-3.5" />
						Forget their cross-chat memories
					</KebabMenuItem>
				{/if}
			{/if}
			<KebabMenuSeparator />
			<KebabMenuItem destructive onclick={() => ondelete?.(character.id)}>
				<Trash2 class="size-3.5" />
				Delete
			</KebabMenuItem>
		</KebabMenu>
	</div>
</article>

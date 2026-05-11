<script lang="ts">
	import type { Message, CharacterCard, UserPersona } from '$lib/types';
	import { Avatar } from '$lib/components/ui';
	import { Pencil, RotateCcw, Copy, Trash2 } from '@lucide/svelte';
	import { cn } from '$lib/utils';

	interface Props {
		message: Message;
		character: CharacterCard | undefined;
		persona: UserPersona | undefined;
	}

	let { message, character, persona }: Props = $props();

	let isUser = $derived(message.role === 'user');

	/**
	 * Naive italic-asterisk rendering — *narration* stays italic,
	 * regular speech stays upright. The real app uses pulldown-cmark
	 * via the backend bridge.
	 */
	function renderInlineMarkdown(text: string): string {
		const escaped = text
			.replace(/&/g, '&amp;')
			.replace(/</g, '&lt;')
			.replace(/>/g, '&gt;');
		return escaped
			.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
			.replace(/\*([^*]+?)\*/g, '<em class="text-[--color-muted]">$1</em>');
	}
</script>

<article
	class={cn(
		'group flex gap-3 px-1 py-1',
		isUser ? 'flex-row-reverse' : 'flex-row'
	)}
>
	<!-- Avatar -->
	<div class="shrink-0 pt-1">
		<Avatar
			name={isUser ? (persona?.name ?? 'You') : (character?.name ?? '?')}
			src={isUser ? persona?.avatar_path : character?.avatar_path}
			size="sm"
		/>
	</div>

	<!-- Bubble + meta -->
	<div class={cn('flex max-w-[70ch] flex-col gap-1', isUser ? 'items-end' : 'items-start')}>
		<!-- Speaker name -->
		<div class="flex items-baseline gap-2 px-1">
			<span
				class={cn(
					'font-display text-sm font-semibold tracking-tight',
					isUser ? 'text-(--color-foreground)/80' : 'text-(--color-foreground)'
				)}
			>
				{isUser ? (persona?.name ?? 'You') : (character?.name ?? 'Character')}
			</span>
			{#if !isUser && message.metadata?.model}
				<span class="text-[10px] uppercase tracking-wider text-(--color-subtle)">
					{message.metadata.model}
				</span>
			{/if}
		</div>

		<!-- Bubble -->
		<div
			class={cn(
				'rounded-(--radius-lg) border px-4 py-3 text-[15px] leading-relaxed',
				isUser
					? 'border-(--color-border) bg-(--color-bubble-user) text-(--color-foreground)'
					: 'border-(--color-border) bg-(--color-bubble-character) text-(--color-foreground)',
				message.is_streaming && 'streaming-caret'
			)}
		>
			<p class="whitespace-pre-wrap break-words">
				<!-- eslint-disable-next-line svelte/no-at-html-tags -->
				{@html renderInlineMarkdown(message.content)}
			</p>
		</div>

		<!-- Actions — hover to reveal -->
		<div
			class={cn(
				'mt-0.5 flex gap-0.5 px-1 opacity-0 transition-opacity group-hover:opacity-100',
				isUser ? 'flex-row-reverse' : 'flex-row'
			)}
		>
			<button
				type="button"
				class="inline-flex size-6 items-center justify-center rounded-(--radius-sm) text-(--color-subtle) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
				aria-label="Copy"
			>
				<Copy class="size-3" />
			</button>
			<button
				type="button"
				class="inline-flex size-6 items-center justify-center rounded-(--radius-sm) text-(--color-subtle) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
				aria-label="Edit"
			>
				<Pencil class="size-3" />
			</button>
			{#if !isUser}
				<button
					type="button"
					class="inline-flex size-6 items-center justify-center rounded-(--radius-sm) text-(--color-subtle) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
					aria-label="Regenerate"
				>
					<RotateCcw class="size-3" />
				</button>
			{/if}
			<button
				type="button"
				class="inline-flex size-6 items-center justify-center rounded-(--radius-sm) text-(--color-subtle) hover:bg-(--color-surface-hover) hover:text-(--color-destructive)"
				aria-label="Delete"
			>
				<Trash2 class="size-3" />
			</button>
		</div>
	</div>
</article>

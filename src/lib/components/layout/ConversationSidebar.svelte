<script lang="ts">
	import { Plus, Search, MessageSquareDashed } from '@lucide/svelte';
	import { app } from '$lib/stores/app.svelte';
	import { Button, Input, Avatar } from '$lib/components/ui';
	import { cn, formatRelativeTime } from '$lib/utils';

	let search = $state('');

	let filtered = $derived(
		(() => {
			const q = search.trim().toLowerCase();
			if (!q) return app.conversations;
			return app.conversations.filter(
				(c) =>
					c.title.toLowerCase().includes(q) ||
					c.last_message_preview.toLowerCase().includes(q)
			);
		})()
	);

	function characterFor(id: string) {
		return app.characters.find((c) => c.id === id);
	}
</script>

<aside
	class={cn(
		'flex h-full w-72 shrink-0 flex-col border-r border-(--color-border) bg-(--color-surface) transition-[width] duration-200',
		!app.sidebar_open && 'w-0 overflow-hidden border-r-0'
	)}
>
	<!-- New chat -->
	<div class="flex shrink-0 items-center gap-2 px-3 py-3">
		<Button variant="primary" size="sm" class="flex-1">
			<Plus class="size-3.5" />
			New chat
		</Button>
	</div>

	<!-- Search -->
	<div class="relative px-3 pb-2">
		<Search
			class="pointer-events-none absolute left-5 top-1/2 size-3.5 -translate-y-1/2 text-(--color-subtle)"
		/>
		<Input bind:value={search} placeholder="Search conversations" class="h-8 pl-8 text-xs" />
	</div>

	<!-- Conversation list -->
	<div class="flex-1 overflow-y-auto px-2 pb-2">
		{#if filtered.length === 0}
			<div class="flex h-full flex-col items-center justify-center gap-2 px-4 text-center">
				<MessageSquareDashed class="size-6 text-(--color-subtle)" />
				<p class="text-xs text-(--color-muted)">
					{search ? 'No conversations match.' : 'No conversations yet.'}
				</p>
			</div>
		{:else}
			<ul class="flex flex-col gap-0.5">
				{#each filtered as conv (conv.id)}
					{@const char = characterFor(conv.character_id)}
					{@const active = conv.id === app.active_conversation_id}
					<li>
						<a
							href={`/chat/${conv.id}`}
							class={cn(
								'group flex gap-2.5 rounded-(--radius-md) px-2 py-2 text-left transition-colors',
								active ? 'bg-(--color-surface-elevated)' : 'hover:bg-(--color-surface-hover)'
							)}
						>
							<Avatar
								name={char?.name ?? '?'}
								src={char?.avatar_path}
								size="sm"
								class="mt-0.5"
							/>
							<div class="min-w-0 flex-1">
								<div class="flex items-baseline justify-between gap-2">
									<span
										class={cn(
											'truncate text-xs font-medium',
											active ? 'text-(--color-foreground)' : 'text-(--color-foreground)/90'
										)}
									>
										{conv.title}
									</span>
									<span class="shrink-0 text-[10px] text-(--color-subtle)">
										{formatRelativeTime(conv.updated_at)}
									</span>
								</div>
								<p class="mt-0.5 line-clamp-2 text-[11px] leading-tight text-(--color-muted)">
									{conv.last_message_preview || '—'}
								</p>
							</div>
						</a>
					</li>
				{/each}
			</ul>
		{/if}
	</div>
</aside>

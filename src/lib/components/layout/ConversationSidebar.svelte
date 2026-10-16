<script lang="ts">
	import { Plus, Search, MessageSquareDashed, Trash2, Pencil } from '@lucide/svelte';
	import { goto } from '$app/navigation';
	import { app } from '$lib/stores/app.svelte';
	import type { SearchResult } from '$lib/types';
	import { Button, Input, Avatar, KebabMenu, KebabMenuItem } from '$lib/components/ui';
	import { DeleteConfirmationDialog } from '$lib/components/characters';
	import { cn, conversationGroup, formatRelativeTime, type ConversationGroup } from '$lib/utils';
	import type { Conversation } from '$lib/types';

	let search = $state('');
	let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null;

	let deleting_id = $state<string | null>(null);
	let deleting_name = $state('');
	let delete_open = $state(false);
	let delete_loading = $state(false);

	// Rename state
	let renaming_id = $state<string | null>(null);
	let renaming_title = $state('');
	let rename_input: HTMLInputElement | undefined = $state();

	// Dual-mode: empty search shows all conversations (no backend call).
	// Non-empty search shows backend FTS5 message results (app.search_results)
	// plus conversations whose titles match (app.searched_conversations).
	let filtered = $derived(
		(() => {
			const q = search.trim();
			if (!q)
				return {
					conversations: app.conversations,
					searchMode: false,
					results: [] as SearchResult[],
					titleConversations: [] as import('$lib/types').Conversation[],
				};
			return {
				conversations: [],
				searchMode: true,
				results: app.search_results,
				titleConversations: app.searched_conversations,
			};
		})(),
	);

	// Group search results by conversation
	let groupedSearchResults = $derived(
		(() => {
			const map = new Map<string, SearchResult[]>();
			for (const result of filtered.results) {
				const existing = map.get(result.conversation_id) ?? [];
				existing.push(result);
				map.set(result.conversation_id, existing);
			}
			return map;
		})(),
	);

	// Debounced backend search (message FTS5 + conversation title matches).
	$effect(() => {
		const q = search.trim();
		if (searchDebounceTimer) clearTimeout(searchDebounceTimer);
		if (!q) {
			app.search_results = [];
			app.searched_conversations = [];
			return;
		}
		searchDebounceTimer = setTimeout(() => {
			app.searchMessagesAction(q);
		}, 300);
		return () => {
			if (searchDebounceTimer) clearTimeout(searchDebounceTimer);
		};
	});

	// Bucket the (already recency-sorted) list into Today / This week / Earlier
	let grouped = $derived(
		(() => {
			const order: ConversationGroup[] = ['Today', 'This week', 'Earlier'];
			const map = new Map<ConversationGroup, Conversation[]>();
			for (const conv of filtered.conversations) {
				const g = conversationGroup(conv.updated_at);
				map.set(g, [...(map.get(g) ?? []), conv]);
			}
			return order.filter((g) => map.has(g)).map((g) => [g, map.get(g)!] as const);
		})(),
	);

	function characterFor(id: string) {
		return app.characters.find((c) => c.id === id);
	}

	function handleDeleteConversation() {
		if (!deleting_id) return;
		delete_loading = true;
		app.deleteConversation(deleting_id)
			.then(() => {
				delete_open = false;
				deleting_id = null;
			})
			.catch(console.error)
			.finally(() => {
				delete_loading = false;
			});
	}

	function startRename(convId: string, currentTitle: string) {
		renaming_id = convId;
		renaming_title = currentTitle;
		// Focus the input after the DOM updates
		setTimeout(() => rename_input?.focus(), 0);
	}

	async function commitRename() {
		if (!renaming_id || !renaming_title.trim()) {
			renaming_id = null;
			return;
		}
		try {
			await app.renameConversation(renaming_id, renaming_title.trim());
		} catch {
			// Store reverts local state and logs on failure.
		}
		renaming_id = null;
	}

	function cancelRename() {
		renaming_id = null;
	}
</script>

<aside
	class={cn(
		'flex h-full shrink-0 flex-col border-r border-(--color-border) bg-(--color-surface)/70 backdrop-blur-[20px] transition-[transform,margin,opacity] duration-200 ease-in-out',
		app.sidebar_open ? 'w-72 translate-x-0' : 'w-72 -translate-x-full -ml-72 border-r-0 opacity-0 pointer-events-none'
	)}
>
	<!-- New chat -->
	<div class="flex shrink-0 items-center gap-2 px-3 py-3">
		<Button variant="primary" size="sm" class="flex-1" onclick={() => goto('/')}>
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
		{#if filtered.searchMode}
			{#if app.is_searching}
				<div class="flex h-full items-center justify-center">
					<p class="text-xs text-(--color-muted)">Searching…</p>
				</div>
			{:else if filtered.results.length === 0 && filtered.titleConversations.length === 0}
				<div class="flex h-full flex-col items-center justify-center gap-2 px-4 text-center">
					<MessageSquareDashed class="size-6 text-(--color-subtle)" />
					<p class="text-xs text-(--color-muted)">No conversations match.</p>
				</div>
			{:else}
				<ul class="flex flex-col gap-0.5">
					{#each groupedSearchResults as [convId, results] (convId)}
						{@const firstResult = results[0]}
						{@const char = characterFor(firstResult.character_id)}
						{@const active = convId === app.active_conversation_id}
						<li class="group/conv relative">
							<button
								type="button"
								onclick={() => {
									app.navigateToSearchResult(firstResult);
									goto('/chat/' + firstResult.conversation_id);
									search = '';
									requestAnimationFrame(() => {
										if (firstResult.message_id) {
											const el = document.getElementById(`message-${firstResult.message_id}`);
											el?.scrollIntoView({ behavior: 'smooth', block: 'center' });
										}
									});
								}}
								class={cn(
									'group flex w-full gap-2.5 rounded-(--radius-md) px-2 py-2 text-left transition-colors',
									active
										? 'bg-(--color-surface-elevated) ring-1 ring-(--color-border) ring-inset'
										: 'hover:bg-(--color-surface-hover)'
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
										<span class={cn('truncate text-xs font-medium', active ? 'text-(--color-foreground)' : 'text-(--color-foreground)/90')}>
											{firstResult.conversation_title}
										</span>
									</div>
									{#each results.slice(0, 1) as r}
										{#if r.message_snippet}
											<p class="mt-0.5 line-clamp-2 text-[11px] leading-tight text-(--color-muted)">
												{@html r.message_snippet}
											</p>
										{/if}
									{/each}
								</div>
							</button>
						</li>
					{/each}
					{#each filtered.titleConversations.filter(
							(tc) => !groupedSearchResults.has(tc.id),
						) as conv (conv.id)}
						{@const char = characterFor(conv.character_id)}
						{@const active = conv.id === app.active_conversation_id}
						<li class="group/conv relative">
							<a
								href={`/chat/${conv.id}`}
								class={cn(
									'group flex gap-2.5 rounded-(--radius-md) px-2 py-2 text-left transition-colors',
									active
										? 'bg-(--color-surface-elevated) ring-1 ring-(--color-border) ring-inset'
										: 'hover:bg-(--color-surface-hover)'
								)}
							>
								<Avatar
									name={char?.name ?? '?'}
									src={char?.avatar_path}
									size="sm"
									class="mt-0.5"
								/>
								<div class="min-w-0 flex-1">
									<span
										class={cn(
											'truncate text-xs font-medium block',
											active ? 'text-(--color-foreground)' : 'text-(--color-foreground)/90'
										)}
									>
										{conv.title}
									</span>
									<p class="mt-0.5 line-clamp-1 text-[11px] leading-tight text-(--color-subtle)">
										Title match
									</p>
								</div>
							</a>
						</li>
					{/each}
				</ul>
			{/if}
		{:else if filtered.conversations.length === 0}
			<div class="flex h-full flex-col items-center justify-center gap-2 px-4 text-center">
				<MessageSquareDashed class="size-6 text-(--color-subtle)" />
				<p class="text-xs text-(--color-muted)">No conversations yet.</p>
			</div>
		{:else}
			{#each grouped as [group, convs] (group)}
			<div class="mt-2 first:mt-0">
			<div class="px-2 py-1 text-[10px] font-medium tracking-wider text-(--color-subtle) uppercase">
				{group}
			</div>
			<ul class="flex flex-col gap-0.5">
				{#each convs as conv (conv.id)}
					{@const char = characterFor(conv.character_id)}
					{@const active = conv.id === app.active_conversation_id}
					<li class="group/conv relative">
						{#if renaming_id === conv.id}
							<div class="flex items-center gap-2 rounded-(--radius-md) px-2 py-2">
								<Avatar
									name={char?.name ?? '?'}
									src={char?.avatar_path}
									size="sm"
									class="mt-0.5"
								/>
								<input
									bind:this={rename_input}
									bind:value={renaming_title}
									onkeydown={(e) => {
										if (e.key === 'Enter') { e.preventDefault(); commitRename(); }
										if (e.key === 'Escape') { e.preventDefault(); cancelRename(); }
									}}
									onblur={commitRename}
									class="min-w-0 flex-1 rounded-(--radius-sm) border border-(--color-accent) bg-(--color-surface-input) px-2 py-0.5 text-xs text-(--color-foreground) focus:outline-none"
								/>
							</div>
						{:else}
							<a
								href={`/chat/${conv.id}`}
								class={cn(
									'group flex gap-2.5 rounded-(--radius-md) px-2 py-2 text-left transition-colors',
									active
										? 'bg-(--color-surface-elevated) ring-1 ring-(--color-border) ring-inset'
										: 'hover:bg-(--color-surface-hover)'
								)}
							>
								<Avatar
									name={char?.name ?? '?'}
									src={char?.avatar_path}
									size="sm"
									ring={active}
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
										<span
											class="shrink-0 text-[10px] text-(--color-subtle) transition-opacity group-hover/conv:opacity-0"
										>
											{formatRelativeTime(conv.updated_at)}
										</span>
									</div>
									<p class="mt-0.5 line-clamp-2 text-[11px] leading-tight text-(--color-muted)">
										{conv.last_message_preview || '—'}
									</p>
								</div>
							</a>
							<div class="absolute right-1 top-1 opacity-0 transition-opacity group-hover/conv:opacity-100">
								<KebabMenu>
									<KebabMenuItem
										onclick={() => startRename(conv.id, conv.title)}
									>
										<Pencil class="size-3.5" />
										Rename
									</KebabMenuItem>
									<KebabMenuItem
										destructive
										onclick={() => {
											deleting_id = conv.id;
											deleting_name = conv.title;
											delete_open = true;
										}}
									>
										<Trash2 class="size-3.5" />
										Delete
									</KebabMenuItem>
								</KebabMenu>
							</div>
						{/if}
					</li>
				{/each}
			</ul>
			</div>
			{/each}
		{/if}
	</div>
	<DeleteConfirmationDialog
		bind:open={delete_open}
		title="Delete conversation"
		description={`Are you sure you want to delete "${deleting_name}"? This action cannot be undone.`}
		confirm_label="Delete"
		onconfirm={handleDeleteConversation}
		loading={delete_loading}
	/>
</aside>

<script lang="ts">
	import { app } from '$lib/stores/app.svelte';
	import { BookOpen } from '@lucide/svelte';
	import { Popover } from 'bits-ui';
	import { cn } from '$lib/utils';

	let open = $state(false);

	// Character-level lorebooks (read-only, always active)
	let characterLorebooks = $derived(app.character_lorebooks);

	// Character lorebook IDs for quick lookup
	let characterLorebookIds = $derived(new Set(characterLorebooks.map((lb) => lb.id)));

	// Available lorebooks that are NOT character-assigned (for conversation assignment)
	let availableForConversation = $derived(
		app.lorebooks.filter((lb) => !characterLorebookIds.has(lb.id)),
	);

	// Conversation-assigned lorebook IDs
	let conversationLorebookIds = $derived(
		new Set(app.conversation_lorebooks.map((lb) => lb.id)),
	);

	let assignedCount = $derived(
		characterLorebooks.length + app.conversation_lorebooks.length,
	);

	let totalLorebooks = $derived(app.lorebooks.length);

	async function handleToggle(lorebookId: string) {
		if (!app.active_conversation_id) return;
		await app.toggleConversationLorebook(app.active_conversation_id, lorebookId);
	}
</script>

<Popover.Root bind:open>
	<Popover.Trigger>
		<button
			type="button"
			class={cn(
				'inline-flex items-center gap-1.5 rounded-(--radius-md) px-2 py-1 text-xs transition-colors',
				app.conversation_lorebooks.length > 0
					? 'bg-(--color-accent)/10 text-(--color-accent-vivid) border border-(--color-accent)/30'
					: 'text-(--color-muted) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)',
			)}
			aria-label="Conversation lorebooks"
		>
			<BookOpen class="size-3.5" />
			Lorebooks
			{#if assignedCount > 0}
				<span class="text-[10px] font-medium">({assignedCount})</span>
			{/if}
		</button>
	</Popover.Trigger>

	<Popover.Portal>
		<Popover.Content
			class="w-80 rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface-elevated) p-1.5 shadow-xl z-50"
			sideOffset={6}
			align="end"
		>
			<!-- Header -->
			<div class="px-2 pb-1.5 pt-1">
				<div class="text-[10px] font-semibold uppercase tracking-wider text-(--color-subtle)">
					Conversation Lorebooks
				</div>
				<p class="mt-0.5 text-[10px] text-(--color-muted)">
					Even budget split across selected lorebooks
				</p>
			</div>

			<div class="max-h-80 overflow-y-auto space-y-0.5 px-0.5">
				{#if totalLorebooks === 0}
					<!-- Empty state -->
					<div class="px-2 py-4 text-center">
						<p class="text-xs text-(--color-subtle)">
							No lorebooks available. Create one in the Characters tab.
						</p>
					</div>
				{:else}
					<!-- Character-level lorebooks (read-only) -->
					{#if characterLorebooks.length > 0}
						<div class="px-2 pt-1 pb-0.5 text-[10px] font-medium uppercase tracking-wider text-(--color-subtle)">
							Character
						</div>
						{#each characterLorebooks as lorebook (lorebook.id)}
							<label
								class="flex items-center gap-2 rounded-(--radius-md) px-2 py-1.5 cursor-default"
							>
								<input
									type="checkbox"
									checked={true}
									disabled={true}
									class="size-3.5 rounded border-(--color-border) bg-(--color-surface-input) accent-(--color-accent)"
								/>
								<span class="min-w-0 flex-1 truncate text-sm text-(--color-foreground)">{lorebook.name}</span>
								<span class="shrink-0 rounded-(--radius-sm) bg-(--color-accent)/15 px-1.5 py-0.5 text-[10px] font-medium text-(--color-accent-vivid)">
									Character
								</span>
							</label>
						{/each}
					{/if}

					<!-- Conversation-level lorebooks (toggleable) -->
					{#if availableForConversation.length > 0}
						<div class="px-2 pt-1.5 pb-0.5 text-[10px] font-medium uppercase tracking-wider text-(--color-subtle)">
							Conversation
						</div>
						{#each availableForConversation as lorebook (lorebook.id)}
							{@const isAssigned = conversationLorebookIds.has(lorebook.id)}
							<button
								type="button"
								class="flex w-full items-center gap-2 rounded-(--radius-md) px-2 py-1.5 text-left transition-colors hover:bg-(--color-surface-hover)"
								onclick={() => handleToggle(lorebook.id)}
							>
								<input
									type="checkbox"
									checked={isAssigned}
									class="size-3.5 rounded border-(--color-border) bg-(--color-surface-input) accent-(--color-accent) pointer-events-none"
									tabindex={-1}
									readonly
								/>
								<span class="min-w-0 flex-1 truncate text-sm text-(--color-foreground)">{lorebook.name}</span>
								{#if lorebook.description}
									<span class="min-w-0 max-w-[100px] truncate text-[10px] text-(--color-muted)">{lorebook.description}</span>
								{/if}
							</button>
						{/each}
					{/if}
				{/if}
			</div>

			<!-- Budget display at bottom -->
			<div class="border-t border-(--color-border) px-2 pt-1.5 pb-1 mt-1 text-[10px] text-(--color-subtle)">
				Budget: {assignedCount} lorebook{assignedCount !== 1 ? 's' : ''} active
			</div>
		</Popover.Content>
	</Popover.Portal>
</Popover.Root>

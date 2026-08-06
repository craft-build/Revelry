<script lang="ts">
	import { app } from '$lib/stores/app.svelte';
	import { Avatar, Tooltip } from '$lib/components/ui';
	import { GitBranch, SlidersHorizontal } from '@lucide/svelte';
	import { Popover } from 'bits-ui';
	import { Check, Trash2 } from '@lucide/svelte';
	import { cn } from '$lib/utils';
	import PersonaSwitcher from './PersonaSwitcher.svelte';
	import ProviderModelSelector from './ProviderModelSelector.svelte';
	import ConversationLorebookPanel from './ConversationLorebookPanel.svelte';

	let providerTrigger: HTMLButtonElement | undefined = $state();
	let branchPopoverOpen = $state(false);

	// When focus_provider_selector is set (from error bubble "Switch provider" link),
	// focus the provider/model selector trigger and reset the flag.
	$effect(() => {
		if (app.focus_provider_selector && providerTrigger) {
			providerTrigger.click();
			app.focus_provider_selector = false;
		}
	});

	let hasBranches = $derived(app.branches.length > 0);

	let deletingBranchId = $state<string | null>(null);

	function getPreviewSnippet(branchId: string | null): string {
		const msg = app.messages.find((m) =>
			branchId ? m.branch_id === branchId : !m.branch_id,
		);
		if (msg) {
			return msg.content.slice(0, 50) + (msg.content.length > 50 ? '...' : '');
		}
		return 'No messages';
	}

	function handleSelect(branchId: string | null) {
		app.switchBranch(branchId);
		branchPopoverOpen = false;
	}

	async function handleDelete(branchId: string) {
		if (deletingBranchId) return;
		deletingBranchId = branchId;
	}

	async function confirmDelete(branchId: string) {
		try {
			await app.deleteBranchAction(branchId);
			deletingBranchId = null;
			branchPopoverOpen = false;
		} catch (err) {
			console.error('Failed to delete branch:', err);
			deletingBranchId = null;
		}
	}

	function cancelDelete() {
		deletingBranchId = null;
	}
</script>

<header
	class="flex h-14 shrink-0 items-center justify-between gap-3 border-b border-(--color-border) bg-(--color-surface) px-4"
>
	<!-- Active character -->
	<div class="flex min-w-0 items-center gap-3">
		{#if app.active_character}
			<Avatar
				name={app.active_character.name}
				src={app.active_character.avatar_path}
				size="md"
			/>
			<div class="min-w-0">
				<h2 class="truncate font-display text-lg font-semibold leading-tight tracking-tight">
					{app.active_character.name}
				</h2>
				<p class="truncate text-[11px] text-(--color-muted)">
					{app.active_conversation?.title ?? 'New conversation'}
				</p>
			</div>
		{/if}
	</div>

	<!-- Right cluster: persona, model, actions -->
	<div class="flex items-center gap-2">
		<PersonaSwitcher />

		<div class="h-5 w-px bg-(--color-border)" aria-hidden="true"></div>

		<ProviderModelSelector />

		<div class="h-5 w-px bg-(--color-border)" aria-hidden="true"></div>

		<ConversationLorebookPanel />

		<!-- Branch switcher — only shown when branches exist -->
		{#if hasBranches}
			<div class="h-5 w-px bg-(--color-border)" aria-hidden="true"></div>

			<Popover.Root bind:open={branchPopoverOpen}>
				<Popover.Trigger>
					<button
						type="button"
						class={cn(
							'inline-flex items-center gap-1.5 rounded-(--radius-md) px-2 py-1 text-xs transition-colors',
							app.active_branch_id
								? 'bg-(--color-accent)/10 text-(--color-accent-vivid) border border-(--color-accent)/30'
								: 'text-(--color-muted) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)',
						)}
						aria-label="Switch branch"
					>
						<GitBranch class="size-3.5" />
						{#if app.active_branch}
							{app.active_branch.label}
						{:else}
							Main
						{/if}
					</button>
				</Popover.Trigger>
				<Popover.Portal>
					<Popover.Content
						class="w-72 rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface-elevated) p-1 shadow-xl z-50"
						sideOffset={4}
						align="end"
					>
						<!-- Header -->
						<div class="px-2 pb-1.5 pt-1 text-[10px] font-semibold uppercase tracking-wider text-(--color-subtle)">
							Branches
						</div>

						<!-- Main branch -->
						<button
							type="button"
							onclick={() => handleSelect(null)}
							class={cn(
								'group flex w-full items-center gap-2 rounded-(--radius-md) px-2 py-1.5 text-left',
								app.active_branch_id === null
									? 'bg-(--color-surface) border-l-2 border-(--color-accent)'
									: 'hover:bg-(--color-surface-hover)',
							)}
						>
							<div class="flex-1 min-w-0">
								<div class="text-sm font-medium">Main</div>
								<div class="text-[11px] text-(--color-muted) truncate">{getPreviewSnippet(null)}</div>
							</div>
							{#if app.active_branch_id === null}
								<Check class="size-3.5 shrink-0 text-(--color-accent-vivid)" />
							{/if}
						</button>

						<!-- Other branches -->
						{#each app.branches as branch (branch.id)}
							{#if deletingBranchId === branch.id}
								<!-- Inline delete confirmation -->
								<div class="rounded-(--radius-md) px-2 py-1.5">
									<p class="text-xs text-(--color-foreground)">Delete this branch?</p>
									<p class="text-[10px] text-(--color-muted)">This cannot be undone.</p>
									<div class="mt-1.5 flex gap-2">
										<button
											type="button"
											onclick={() => confirmDelete(branch.id)}
											class="text-xs font-medium text-(--color-destructive) hover:underline"
										>
											Delete
										</button>
										<button
											type="button"
											onclick={cancelDelete}
											class="text-xs font-medium text-(--color-subtle) hover:text-(--color-foreground)"
										>
											Cancel
										</button>
									</div>
								</div>
							{:else}
								<div
									role="button"
									tabindex="0"
									onclick={() => handleSelect(branch.id)}
									onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleSelect(branch.id); } }}
									class={cn(
										'group flex w-full items-center gap-2 rounded-(--radius-md) px-2 py-1.5 text-left cursor-pointer',
										app.active_branch_id === branch.id
											? 'bg-(--color-surface) border-l-2 border-(--color-accent)'
											: 'hover:bg-(--color-surface-hover)',
									)}
								>
									<div class="flex-1 min-w-0">
										<div class="text-sm font-medium">{branch.label}</div>
										<div class="text-[11px] text-(--color-muted) truncate">{getPreviewSnippet(branch.id)}</div>
									</div>
									{#if app.active_branch_id === branch.id}
										<Check class="size-3.5 shrink-0 text-(--color-accent-vivid)" />
									{:else}
										<button
											type="button"
											onclick={(e) => { e.stopPropagation(); handleDelete(branch.id); }}
											class="size-5 shrink-0 inline-flex items-center justify-center rounded-(--radius-sm) text-(--color-subtle) opacity-0 group-hover:opacity-100 hover:text-(--color-destructive) hover:bg-(--color-surface-hover)"
											aria-label="Delete branch"
										>
											<Trash2 class="size-3" />
										</button>
									{/if}
								</div>
							{/if}
						{/each}
					</Popover.Content>
				</Popover.Portal>
			</Popover.Root>
		{/if}

		<div class="h-5 w-px bg-(--color-border)" aria-hidden="true"></div>

		<Tooltip text="Generation settings">
			<button
				type="button"
				onclick={() => (app.generation_panel_open = !app.generation_panel_open)}
				class={cn(
					'inline-flex size-8 items-center justify-center rounded-(--radius-md) transition-colors',
					app.generation_panel_open
						? 'bg-(--color-accent)/10 text-(--color-accent-vivid)'
						: 'text-(--color-muted) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)',
				)}
				aria-label="Generation settings"
			>
				<SlidersHorizontal class="size-4" />
			</button>
		</Tooltip>
	</div>
</header>

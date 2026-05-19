<script lang="ts">
	import type { Message as MessageType, MessageBranch } from '$lib/types';
	import { Popover } from 'bits-ui';
	import { Check, Trash2 } from '@lucide/svelte';
	import { cn } from '$lib/utils';
	import { app } from '$lib/stores/app.svelte';

	interface Props {
		branches: MessageBranch[];
		activeBranchId: string | null;
		messages: MessageType[];
		open: boolean;
		onopenchange: (open: boolean) => void;
		children?: import('svelte').Snippet;
	}

	let { branches, activeBranchId, messages, open = $bindable(false), onopenchange, children }: Props = $props();

	let deletingBranchId = $state<string | null>(null);

	function getPreviewSnippet(branch: MessageBranch): string {
		const branchMsg = messages.find((m) => m.branch_id === branch.id);
		if (branchMsg) {
			return branchMsg.content.slice(0, 50) + (branchMsg.content.length > 50 ? '...' : '');
		}
		return 'No messages yet';
	}

	function getMainPreview(): string {
		const mainMsg = messages.find((m) => !m.branch_id);
		if (mainMsg) {
			return mainMsg.content.slice(0, 50) + (mainMsg.content.length > 50 ? '...' : '');
		}
		return 'No messages';
	}

	function handleSelect(branchId: string | null) {
		app.switchBranch(branchId);
		onopenchange(false);
	}

	async function handleDelete(branchId: string) {
		if (deletingBranchId) return; // Already confirming
		deletingBranchId = branchId;
	}

	async function confirmDelete(branchId: string) {
		try {
			await app.deleteBranchAction(branchId);
			deletingBranchId = null;
			onopenchange(false);
		} catch (err) {
			console.error('Failed to delete branch:', err);
			deletingBranchId = null;
		}
	}

	function cancelDelete() {
		deletingBranchId = null;
	}
</script>

<Popover.Root bind:open onOpenChange={onopenchange}>
	<Popover.Trigger>
		{@render children?.()}
	</Popover.Trigger>
	<Popover.Portal>
		<Popover.Content
			class="w-72 rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface-elevated) p-1 shadow-xl z-50"
			sideOffset={4}
			align="start"
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
					activeBranchId === null
						? 'bg-(--color-surface) border-l-2 border-(--color-accent)'
						: 'hover:bg-(--color-surface-hover)',
				)}
			>
				<div class="flex-1 min-w-0">
					<div class="text-sm font-medium">Main</div>
					<div class="text-[11px] text-(--color-muted) truncate">{getMainPreview()}</div>
				</div>
				{#if activeBranchId === null}
					<Check class="size-3.5 shrink-0 text-(--color-accent)" />
				{/if}
			</button>

			<!-- Other branches -->
			{#each branches as branch (branch.id)}
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
							activeBranchId === branch.id
								? 'bg-(--color-surface) border-l-2 border-(--color-accent)'
								: 'hover:bg-(--color-surface-hover)',
						)}
					>
						<div class="flex-1 min-w-0">
							<div class="text-sm font-medium">{branch.label}</div>
							<div class="text-[11px] text-(--color-muted) truncate">{getPreviewSnippet(branch)}</div>
						</div>
						{#if activeBranchId === branch.id}
							<Check class="size-3.5 shrink-0 text-(--color-accent)" />
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

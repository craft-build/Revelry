<script lang="ts">
	import { Dialog, Button } from '$lib/components/ui';
	import { Trash2 } from '@lucide/svelte';

	interface Props {
		open?: boolean;
		onopenchange?: (open: boolean) => void;
		title?: string;
		description?: string;
		/** Label for the confirm button */
		confirm_label?: string;
		/** Label for the cancel button */
		cancel_label?: string;
		/** Called when the user confirms deletion */
		onconfirm?: () => void;
		/** Called when the user cancels */
		oncancel?: () => void;
		/** Whether the deletion is in progress */
		loading?: boolean;
	}

	let {
		open = $bindable(false),
		onopenchange,
		title = 'Delete',
		description = 'This action cannot be undone. Are you sure you want to continue?',
		confirm_label = 'Delete',
		cancel_label = 'Cancel',
		onconfirm,
		oncancel,
		loading = false
	}: Props = $props();

	function handleOpenChange(value: boolean) {
		open = value;
		onopenchange?.(value);
	}

	function handleConfirm() {
		onconfirm?.();
	}
</script>

<Dialog {open} onopenchange={handleOpenChange} {title} {description}>
	{#snippet footer()}
		<Button variant="secondary" size="sm" onclick={() => { oncancel?.(); handleOpenChange(false); }} disabled={loading}>
			{cancel_label}
		</Button>
		<Button variant="destructive" size="sm" onclick={handleConfirm} disabled={loading}>
			{#if loading}
				Deleting…
			{:else}
				<Trash2 class="size-4" />
				{confirm_label}
			{/if}
		</Button>
	{/snippet}
</Dialog>

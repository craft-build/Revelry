<script lang="ts">
	import { Dialog } from '$lib/components/ui';
	import { AlertTriangle } from '@lucide/svelte';

	interface Props {
		open: boolean;
		message: string;
	}

	let { open = $bindable(), message }: Props = $props();
</script>

<Dialog bind:open title="Import failed">
	<div class="flex items-start gap-3">
		<div
			class="flex size-8 shrink-0 items-center justify-center rounded-full bg-(--color-destructive)/10"
		>
			<AlertTriangle class="size-4 text-(--color-destructive)" />
		</div>
		<div class="flex-1">
			<p class="text-[14px] leading-relaxed text-(--color-foreground)">
				{message}
			</p>
		</div>
	</div>
	<div class="mt-4 flex justify-end">
		<button
			type="button"
			onclick={() => (open = false)}
			class="rounded-(--radius-sm) px-3 py-1.5 text-xs font-medium text-(--color-subtle) hover:text-(--color-foreground)"
		>
			Close
		</button>
	</div>
</Dialog>

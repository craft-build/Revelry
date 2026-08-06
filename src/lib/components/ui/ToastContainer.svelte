<script lang="ts">
	import { fade, fly } from 'svelte/transition';
	import { X } from '@lucide/svelte';
	import { cn } from '$lib/utils';
	import { toasts } from '$lib/stores/toasts.svelte';

	function dismiss(id: string) {
		toasts.dismiss(id);
	}
</script>

<div
	class="fixed bottom-6 right-6 z-50 flex flex-col gap-2"
	role="status"
	aria-live="polite"
>
	{#each toasts.list as toast (toast.id)}
		<div
			class={cn(
				'flex items-center gap-3 rounded-(--radius-lg) border px-4 py-3 text-sm shadow-lg backdrop-blur-[20px]',
				'border-(--color-border-strong) bg-(--color-surface-elevated) text-(--color-foreground)',
			)}
			transition:fly={{ y: 20, duration: 200 }}
		>
			<span
				class={cn(
					'size-2 shrink-0 rounded-full',
					toast.variant === 'error' ? 'bg-(--color-destructive)' : 'bg-(--color-success)'
				)}
				aria-hidden="true"
			></span>
			<span class="min-w-0 flex-1">{toast.message}</span>
			<button
				type="button"
				onclick={() => dismiss(toast.id)}
				class="shrink-0 text-(--color-subtle) hover:text-(--color-foreground)"
				aria-label="Dismiss"
			>
				<X class="size-3.5" />
			</button>
		</div>
	{/each}
</div>

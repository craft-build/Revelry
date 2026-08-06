<script lang="ts">
	import { Dialog as DialogPrimitive } from 'bits-ui';
	import { X } from '@lucide/svelte';
	import { cn } from '$lib/utils';

	interface Props {
		open?: boolean;
		onopenchange?: (open: boolean) => void;
		title?: string;
		description?: string;
		class?: string;
		children?: import('svelte').Snippet;
		footer?: import('svelte').Snippet;
	}

	let {
		open = $bindable(false),
		onopenchange,
		title,
		description,
		class: className,
		children,
		footer,
	}: Props = $props();

	function handleOpenChange(value: boolean) {
		open = value;
		onopenchange?.(value);
	}
</script>

<DialogPrimitive.Root bind:open onOpenChange={handleOpenChange}>
	<DialogPrimitive.Portal>
		<DialogPrimitive.Overlay
			class="fixed inset-0 z-50 bg-black/60 backdrop-blur-[20px] data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0"
		/>
		<DialogPrimitive.Content
			class={cn(
				'fixed left-1/2 top-1/2 z-50 flex max-h-[90vh] w-full max-w-lg -translate-x-1/2 -translate-y-1/2 flex-col rounded-(--radius-xl) border border-(--color-border-strong) bg-(--color-surface-elevated) p-6 shadow-lg data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[state=closed]:slide-out-to-left-1/2 data-[state=closed]:slide-out-to-top-48% data-[state=open]:slide-in-from-left-1/2 data-[state=open]:slide-in-from-top-48%',
				className
			)}
		>
			{#if title || description}
				<div class="mb-4">
					{#if title}
						<DialogPrimitive.Title
							class="text-lg font-semibold leading-none tracking-tight text-(--color-foreground)"
						>
							{title}
						</DialogPrimitive.Title>
					{/if}
					{#if description}
						<DialogPrimitive.Description class="mt-2 text-sm text-(--color-muted)">
							{description}
						</DialogPrimitive.Description>
					{/if}
				</div>
			{/if}

			<div class="min-h-0 flex-1 overflow-y-auto">
				{@render children?.()}
			</div>

			{#if footer}
				<div class="mt-6 flex items-center justify-end gap-2">
					{@render footer()}
				</div>
			{/if}

			<DialogPrimitive.Close
				class="absolute right-4 top-4 rounded-(--radius-sm) p-1 text-(--color-muted) opacity-70 transition-opacity hover:opacity-100 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-(--color-accent-vivid)"
			>
				<X class="size-4" />
				<span class="sr-only">Close</span>
			</DialogPrimitive.Close>
		</DialogPrimitive.Content>
	</DialogPrimitive.Portal>
</DialogPrimitive.Root>

<script lang="ts">
	import { DropdownMenu } from 'bits-ui';
	import { MoreVertical } from '@lucide/svelte';
	import { cn } from '$lib/utils';

	interface Props {
		class?: string;
		children?: import('svelte').Snippet;
		/** Alignment of the dropdown menu relative to the trigger */
		align?: 'start' | 'end';
	}

	let { class: className, children, align = 'end' }: Props = $props();
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger
		class={cn(
			'inline-flex size-8 items-center justify-center rounded-(--radius-md) text-(--color-muted) hover:bg-(--color-surface-hover) hover:text-(--color-foreground) transition-colors',
			className
		)}
	>
		<MoreVertical class="size-4" />
		<span class="sr-only">Open menu</span>
	</DropdownMenu.Trigger>

	<DropdownMenu.Portal>
		<DropdownMenu.Content
			{align}
			class="z-50 min-w-[8rem] overflow-hidden rounded-(--radius-md) border border-(--color-border-strong) bg-(--color-surface-elevated) p-1 shadow-lg backdrop-blur-[20px] data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2"
		>
			{@render children?.()}
		</DropdownMenu.Content>
	</DropdownMenu.Portal>
</DropdownMenu.Root>

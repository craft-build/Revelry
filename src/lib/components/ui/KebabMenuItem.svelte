<script lang="ts">
	import { DropdownMenu } from 'bits-ui';
	import { cn } from '$lib/utils';

	interface Props {
		class?: string;
		destructive?: boolean;
		onclick?: () => void;
		disabled?: boolean;
		children?: import('svelte').Snippet;
	}

	let { class: className, destructive = false, onclick, disabled = false, children }: Props = $props();
</script>

<DropdownMenu.Item
	class={cn(
		'relative flex w-full cursor-default select-none items-center gap-2 rounded-(--radius-sm) px-2 py-1.5 text-sm outline-none transition-colors focus:bg-(--color-surface-hover) focus:text-(--color-foreground) data-[disabled]:pointer-events-none data-[disabled]:opacity-50 [&_svg]:size-3.5 [&_svg]:shrink-0',
		destructive && 'text-(--color-destructive) focus:text-(--color-destructive)',
		className
	)}
	{onclick}
	{disabled}
>
	{@render children?.()}
</DropdownMenu.Item>

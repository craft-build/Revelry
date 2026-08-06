<script lang="ts">
	import { Tooltip as TooltipPrimitive } from 'bits-ui';
	import type { Snippet } from 'svelte';
	import { cn } from '$lib/utils';

	interface Props {
		text: string;
		side?: 'top' | 'right' | 'bottom' | 'left';
		class?: string;
		children: Snippet;
	}

	let { text, side = 'top', class: className, children }: Props = $props();
</script>

<TooltipPrimitive.Provider>
	<TooltipPrimitive.Root delayDuration={300}>
		<TooltipPrimitive.Trigger>
			{@render children()}
		</TooltipPrimitive.Trigger>
		<TooltipPrimitive.Content
			{side}
			sideOffset={6}
			class={cn(
				'z-50 rounded-(--radius-sm) border border-(--color-border) bg-(--color-surface-elevated) px-2.5 py-1.5 text-xs text-(--color-foreground) shadow-md backdrop-blur-[20px]',
				className
			)}
		>
			{text}
		</TooltipPrimitive.Content>
	</TooltipPrimitive.Root>
</TooltipPrimitive.Provider>

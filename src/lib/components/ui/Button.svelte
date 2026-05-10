<script lang="ts" module>
	import { tv, type VariantProps } from 'tailwind-variants';

	export const buttonVariants = tv({
		base: 'inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-(--radius-md) text-sm font-medium transition-colors focus-visible:outline-2 focus-visible:outline-(--color-accent) focus-visible:outline-offset-2 disabled:pointer-events-none disabled:opacity-50 [&_svg]:size-4 [&_svg]:shrink-0',
		variants: {
			variant: {
				primary:
					'bg-(--color-accent) text-(--color-accent-fg) hover:bg-(--color-accent)/90 active:bg-(--color-accent)/80',
				secondary:
					'bg-(--color-surface-elevated) text-(--color-foreground) hover:bg-(--color-surface-hover) border border-(--color-border)',
				ghost: 'text-(--color-foreground) hover:bg-(--color-surface-hover)',
				outline:
					'border border-(--color-border-strong) bg-transparent text-(--color-foreground) hover:bg-(--color-surface-hover)',
				destructive:
					'bg-(--color-destructive) text-white hover:bg-(--color-destructive)/90 active:bg-(--color-destructive)/80',
				link: 'text-(--color-accent) underline-offset-4 hover:underline'
			},
			size: {
				sm: 'h-8 px-3 text-xs',
				md: 'h-10 px-4',
				lg: 'h-11 px-6 text-base',
				icon: 'size-9 p-0',
				'icon-sm': 'size-7 p-0'
			}
		},
		defaultVariants: {
			variant: 'secondary',
			size: 'md'
		}
	});

	export type ButtonProps = VariantProps<typeof buttonVariants>;
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';
	import { cn } from '$lib/utils';

	interface Props extends HTMLButtonAttributes {
		variant?: ButtonProps['variant'];
		size?: ButtonProps['size'];
		class?: string;
		children?: Snippet;
	}

	let {
		variant = 'secondary',
		size = 'md',
		class: className,
		children,
		...rest
	}: Props = $props();
</script>

<button class={cn(buttonVariants({ variant, size }), className)} {...rest}>
	{@render children?.()}
</button>

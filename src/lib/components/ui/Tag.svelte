<script lang="ts">
	import type { Snippet } from 'svelte';
	import { cn } from '$lib/utils';

	interface Props {
		variant?: 'default' | 'accent' | 'outline';
		class?: string;
		children?: Snippet;
	}

	let { variant = 'default', class: className, children }: Props = $props();

	const variants = {
		default: 'bg-(--color-surface) text-(--color-muted) border-(--color-border)',
		accent: 'bg-(--color-accent) text-(--color-accent-fg) border-transparent',
		outline: 'bg-transparent text-(--color-muted) border-(--color-border-strong)'
	};
</script>

<span
	class={cn(
		'inline-flex h-[26px] items-center rounded-(--radius-md) border px-2.5 text-xs font-medium',
		variants[variant],
		className
	)}
>
	{@render children?.()}
</span>

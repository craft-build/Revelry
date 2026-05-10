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
		default: 'bg-(--color-surface-elevated) text-(--color-muted) border-(--color-border)',
		accent: 'bg-(--color-accent-muted)/30 text-(--color-accent) border-(--color-accent-muted)',
		outline: 'bg-transparent text-(--color-muted) border-(--color-border-strong)'
	};
</script>

<span
	class={cn(
		'inline-flex items-center rounded-full border px-2 py-0.5 text-[10px] font-medium uppercase tracking-wide',
		variants[variant],
		className
	)}
>
	{@render children?.()}
</span>

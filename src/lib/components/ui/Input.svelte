<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';
	import { cn } from '$lib/utils';

	interface Props extends HTMLInputAttributes {
		class?: string;
		value?: string;
	}

	let { class: className, value = $bindable(), ...rest }: Props = $props();
</script>

<input
	bind:value
	class={cn(
		'flex h-10 w-full rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-input) px-3 py-2 text-sm text-(--color-foreground) placeholder:text-(--color-subtle) transition-shadow duration-150 focus-visible:border-(--color-accent) focus-visible:outline-none focus-visible:shadow-(--shadow-glow) disabled:cursor-not-allowed disabled:opacity-50',
		className
	)}
	{...rest}
/>

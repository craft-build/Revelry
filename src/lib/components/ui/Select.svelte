<script lang="ts">
	import type { HTMLSelectAttributes } from 'svelte/elements';
	import { cn } from '$lib/utils';

	interface Props extends HTMLSelectAttributes {
		class?: string;
		value?: string;
		children?: import('svelte').Snippet;
	}

	let { class: className, value = $bindable(), children, ...rest }: Props = $props();
</script>

<select
	bind:value
	class={cn(
		'flex h-10 w-full items-center rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-input) px-3 py-2 text-sm focus-visible:border-(--color-accent) focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-(--color-accent) disabled:cursor-not-allowed disabled:opacity-50',
		className
	)}
	{...rest}
>
	{@render children?.()}
</select>

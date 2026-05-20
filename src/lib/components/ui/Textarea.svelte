<script lang="ts">
	import type { HTMLTextareaAttributes } from 'svelte/elements';
	import { cn } from '$lib/utils';

	interface Props extends HTMLTextareaAttributes {
		class?: string;
		value?: string;
		/** Automatically grow the textarea height to fit content */
		autogrow?: boolean;
	}

	let { class: className, value = $bindable(), autogrow = false, ...rest }: Props = $props();

	let textarea_el: HTMLTextAreaElement | undefined = $state();

	function autoResize() {
		if (!autogrow || !textarea_el) return;
		textarea_el.style.height = 'auto';
		textarea_el.style.height = `${textarea_el.scrollHeight}px`;
	}

	$effect(() => {
		// Re-measure when value changes
		if (value !== undefined) autoResize();
	});
</script>

<textarea
	bind:this={textarea_el}
	bind:value
	class={cn(
		'flex min-h-[80px] w-full rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-input) px-3 py-2 text-sm placeholder:text-(--color-subtle) focus-visible:border-(--color-accent) focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-(--color-accent) disabled:cursor-not-allowed disabled:opacity-50',
		autogrow && 'resize-none max-h-64 overflow-y-auto',
		className
	)}
	oninput={autoResize}
	{...rest}
></textarea>

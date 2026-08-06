<script lang="ts">
	import { cn } from '$lib/utils';

	interface Props {
		class?: string;
		alt?: string;
	}

	let { class: className, alt = 'Revelry' }: Props = $props();
</script>

<img
	src="/logo-squircle.png"
	{alt}
	class={cn('size-8 shrink-0', className)}
	draggable="false"
/>

<script lang="ts">
	import { Avatar as AvatarPrimitive } from 'bits-ui';
	import { cn, avatarInitials } from '$lib/utils';

	interface Props {
		src?: string;
		name: string;
		size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl';
		class?: string;
		/** Tinted ring around avatar — useful for active/selected states */
		ring?: boolean;
	}

	let { src, name, size = 'md', class: className, ring = false }: Props = $props();

	const sizeClasses = {
		xs: 'size-6 text-[10px]',
		sm: 'size-8 text-xs',
		md: 'size-10 text-sm',
		lg: 'size-14 text-base',
		xl: 'size-20 text-xl'
	};
</script>

<AvatarPrimitive.Root
	class={cn(
		'relative inline-flex shrink-0 overflow-hidden rounded-full bg-(--color-surface-elevated) font-medium text-(--color-muted)',
		sizeClasses[size],
		ring && 'ring-2 ring-(--color-accent) ring-offset-2 ring-offset-(--color-background)',
		className
	)}
>
	{#if src}
		<AvatarPrimitive.Image {src} alt={name} class="aspect-square size-full object-cover" />
	{/if}
	<AvatarPrimitive.Fallback
		class="flex size-full items-center justify-center font-display tracking-wider"
	>
		{avatarInitials(name)}
	</AvatarPrimitive.Fallback>
</AvatarPrimitive.Root>

<script lang="ts">
	import { Avatar as AvatarPrimitive } from 'bits-ui';
	import { cn, avatarInitials, resolveAvatarUrl } from '$lib/utils';

	interface Props {
		src?: string;
		name: string;
		size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl';
		class?: string;
		/** Tinted ring around avatar — useful for active/selected states */
		ring?: boolean;
	}

	let { src, name, size = 'md', class: className, ring = false }: Props = $props();

	// Track whether the image failed to load so we fall back to initials
	let img_failed = $state(false);

	// Resolve the raw avatar_path (absolute filesystem path) to a displayable asset:// URL
	let resolved_src = $derived(resolveAvatarUrl(src));

	// Reset failure state when src changes
	$effect(() => {
		void src;
		img_failed = false;
	});

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
	{#if resolved_src && !img_failed}
		<AvatarPrimitive.Image
			src={resolved_src}
			alt={name}
			class="aspect-square size-full object-cover"
			onerror={() => (img_failed = true)}
		/>
	{/if}
	<AvatarPrimitive.Fallback
		class="flex size-full items-center justify-center font-display tracking-wider"
	>
		{avatarInitials(name)}
	</AvatarPrimitive.Fallback>
</AvatarPrimitive.Root>

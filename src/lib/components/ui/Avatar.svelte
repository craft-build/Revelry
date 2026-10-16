<script lang="ts">
	import { Avatar as AvatarPrimitive, Dialog as DialogPrimitive } from 'bits-ui';
	import { X } from '@lucide/svelte';
	import { cn, avatarInitials, avatarTone, resolveAvatarUrl, type AvatarTone } from '$lib/utils';

	interface Props {
		src?: string;
		name: string;
		size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl';
		class?: string;
		/** Tinted ring around avatar — useful for active/selected states */
		ring?: boolean;
		/**
		 * When true, clicking an avatar that successfully loaded an image opens a
		 * dismissable full-size lightbox. Has no effect when there is no image
		 * (the avatar falls back to initials and stays non-interactive).
		 */
		enlargeable?: boolean;
		/**
		 * Fallback fill. Defaults to one of four tail tints hashed from the
		 * name; 'brand' uses the full tail-sweep gradient.
		 */
		tone?: AvatarTone | 'brand';
	}

	let {
		src,
		name,
		size = 'md',
		class: className,
		ring = false,
		enlargeable = false,
		tone
	}: Props = $props();

	// Track whether the image failed to load so we fall back to initials
	let img_failed = $state(false);

	// Resolve the raw avatar_path (absolute filesystem path) to a displayable asset:// URL
	let resolved_src = $derived(resolveAvatarUrl(src));

	// The avatar is only enlargeable when it actually shows an image
	let can_enlarge = $derived(enlargeable && !!resolved_src && !img_failed);

	// Reset failure state when src changes
	$effect(() => {
		void src;
		img_failed = false;
	});

	let fill = $derived(
		tone === 'brand'
			? 'background: var(--gradient-brand)'
			: `background: var(--gradient-avatar-${tone ?? avatarTone(name)})`
	);

	const sizeClasses = {
		xs: 'size-6 text-[10px]',
		sm: 'size-8 text-xs',
		md: 'size-10 text-sm',
		lg: 'size-14 text-base',
		xl: 'size-20 text-xl'
	};
</script>

{#snippet avatarInner()}
	{#if resolved_src && !img_failed}
		<AvatarPrimitive.Image
			src={resolved_src}
			alt={name}
			class="aspect-square size-full object-cover"
			onerror={() => (img_failed = true)}
		/>
	{/if}
	<AvatarPrimitive.Fallback
		class="flex size-full items-center justify-center font-display font-semibold tracking-wider"
	>
		{avatarInitials(name)}
	</AvatarPrimitive.Fallback>
{/snippet}

{#if can_enlarge}
	<DialogPrimitive.Root>
		<DialogPrimitive.Trigger
			class={cn(
				'inline-flex shrink-0 cursor-zoom-in items-center justify-center rounded-full outline-none transition-shadow hover:ring-2 hover:ring-(--color-accent-vivid)/50 focus-visible:ring-2 focus-visible:ring-(--color-accent-vivid) focus-visible:ring-offset-2 focus-visible:ring-offset-(--color-background)',
				className
			)}
			aria-label={`View ${name}'s avatar`}
		>
			<AvatarPrimitive.Root
				class={cn(
					'relative inline-flex shrink-0 overflow-hidden rounded-full font-medium text-white',
					sizeClasses[size],
					ring && 'ring-2 ring-(--color-accent-vivid) ring-offset-2 ring-offset-(--color-background)'
				)}
				style={fill}
			>
				{@render avatarInner()}
			</AvatarPrimitive.Root>
		</DialogPrimitive.Trigger>

		<DialogPrimitive.Portal>
			<DialogPrimitive.Overlay
				class="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:animate-in data-[state=open]:fade-in-0"
			/>
			<DialogPrimitive.Content
				class="fixed left-1/2 top-1/2 z-50 -translate-x-1/2 -translate-y-1/2 outline-none data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95"
			>
				<DialogPrimitive.Title class="sr-only">{name}</DialogPrimitive.Title>
				<DialogPrimitive.Description class="sr-only">
					Full avatar image
				</DialogPrimitive.Description>

				<div class="relative">
					<img
						src={resolved_src}
						alt={name}
						class="max-h-[88vh] max-w-[92vw] rounded-(--radius-xl) object-contain shadow-2xl"
					/>
					<DialogPrimitive.Close
						class="absolute right-2 top-2 inline-flex size-8 items-center justify-center rounded-full bg-black/50 text-white backdrop-blur transition-colors hover:bg-black/70 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-(--color-accent-vivid)"
						aria-label="Close"
					>
						<X class="size-4" />
					</DialogPrimitive.Close>
				</div>
			</DialogPrimitive.Content>
		</DialogPrimitive.Portal>
	</DialogPrimitive.Root>
{:else}
	<AvatarPrimitive.Root
		class={cn(
			'relative inline-flex shrink-0 overflow-hidden rounded-full font-medium text-white',
			sizeClasses[size],
			ring && 'ring-2 ring-(--color-accent-vivid) ring-offset-2 ring-offset-(--color-background)',
			className
		)}
		style={fill}
	>
		{@render avatarInner()}
	</AvatarPrimitive.Root>
{/if}

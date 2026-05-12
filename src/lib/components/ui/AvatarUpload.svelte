<script lang="ts">
	import { open } from '@tauri-apps/plugin-dialog';
	import { Camera } from '@lucide/svelte';
	import Avatar from './Avatar.svelte';
	import { cn } from '$lib/utils';

	interface Props {
		/** Current avatar image source (absolute filesystem path from backend) */
		src?: string;
		/** Display name for initials fallback */
		name: string;
		/** Size of the avatar */
		size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl';
		class?: string;
		/** Called when a new avatar file is selected. Receives the source file path. */
		onselect?: (path: string) => void;
		/** Whether the upload button is disabled */
		disabled?: boolean;
	}

	let { src, name, size = 'lg', class: className, onselect, disabled = false }: Props = $props();

	const sizeClasses = {
		xs: 'size-6',
		sm: 'size-8',
		md: 'size-10',
		lg: 'size-14',
		xl: 'size-20'
	};

	async function handleClick() {
		if (disabled) return;
		const selected = await open({
			multiple: false,
			directory: false,
			filters: [
				{
					name: 'Images',
					extensions: ['png', 'jpg', 'jpeg', 'webp', 'gif']
				}
			]
		});
		if (selected) {
			onselect?.(selected);
		}
	}
</script>

<button
	type="button"
	{disabled}
	onclick={handleClick}
	class={cn(
		'group relative inline-flex shrink-0 overflow-hidden rounded-full',
		sizeClasses[size],
		!disabled && 'cursor-pointer',
		disabled && 'cursor-not-allowed opacity-50',
		className
	)}
>
	<Avatar {src} {name} {size} />

	{#if !disabled}
		<div
			class="absolute inset-0 flex items-center justify-center rounded-full bg-black/0 transition-colors group-hover:bg-black/40"
		>
			<Camera class="size-5 text-white opacity-0 transition-opacity group-hover:opacity-100" />
		</div>
	{/if}
</button>

<script lang="ts">
	import { Input } from '$lib/components/ui';
	import { X } from '@lucide/svelte';
	import { cn } from '$lib/utils';

	interface Props {
		name?: string;
		contextWindowSize?: number | null;
		enabled?: boolean;
		removable?: boolean;
		class?: string;
		disabled?: boolean;
		onremove?: () => void;
		onupdatename?: (value: string) => void;
		onupdatecontextwindow?: (value: number | null) => void;
		onupdateenabled?: (value: boolean) => void;
	}

	let {
		name = '',
		contextWindowSize = null,
		enabled = true,
		removable = true,
		class: className,
		disabled = false,
		onremove,
		onupdatename,
		onupdatecontextwindow,
		onupdateenabled
	}: Props = $props();

	function handleNameChange(e: Event) {
		const target = e.target as HTMLInputElement;
		onupdatename?.(target.value);
	}

	function handleContextWindowChange(e: Event) {
		const target = e.target as HTMLInputElement;
		const val = target.value ? Number(target.value) : null;
		onupdatecontextwindow?.(val);
	}

	function handleEnabledChange() {
		onupdateenabled?.(!enabled);
	}
</script>

<div
	class={cn(
		'flex items-center gap-2 rounded-(--radius-md) border border-(--color-border) bg-(--color-surface) p-2',
		!enabled && 'opacity-50',
		className
	)}
>
	<Input
		value={name}
		onchange={handleNameChange}
		placeholder="e.g. gpt-4o-mini"
		disabled={disabled}
		class="flex-1 h-8 text-sm"
	/>

	<Input
		type="number"
		value={contextWindowSize != null ? String(contextWindowSize) : ''}
		onchange={handleContextWindowChange}
		placeholder="e.g. 128000"
		disabled={disabled}
		class="h-8 w-28 text-sm"
	/>

	<label class="flex items-center gap-1.5 text-xs text-(--color-muted)">
		<input
			type="checkbox"
			checked={enabled}
			onchange={handleEnabledChange}
			disabled={disabled}
			class="size-3.5 rounded border-(--color-border) bg-(--color-surface-input) accent-(--color-accent)"
		/>
		On
	</label>

	<button
		type="button"
		onclick={onremove}
		disabled={disabled || !removable}
		aria-label="Remove model"
		class={cn(
			'inline-flex size-7 items-center justify-center rounded-(--radius-sm) text-(--color-muted) transition-colors hover:bg-(--color-surface-hover) hover:text-(--color-foreground)',
			(!removable || disabled) && 'cursor-not-allowed opacity-30'
		)}
	>
		<X class="size-3.5" />
	</button>
</div>

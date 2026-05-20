<script lang="ts">
	import { app } from '$lib/stores/app.svelte';
	import { cn } from '$lib/utils';
	import { Monitor, Sun, Moon } from '@lucide/svelte';

	type ThemeValue = 'system' | 'light' | 'dark';

	const options: { value: ThemeValue; label: string; icon: typeof Monitor; ariaLabel: string }[] = [
		{ value: 'system', label: 'System', icon: Monitor, ariaLabel: 'Follow system preference' },
		{ value: 'light', label: 'Light', icon: Sun, ariaLabel: 'Light theme' },
		{ value: 'dark', label: 'Dark', icon: Moon, ariaLabel: 'Dark theme' },
	];

	function getAnnotation(theme: ThemeValue): string {
		switch (theme) {
			case 'system':
				return `Following your system preference (${app.resolved_theme})`;
			case 'light':
				return 'Light theme active';
			case 'dark':
				return 'Dark theme active';
		}
	}
</script>

<div
	class="inline-flex rounded-(--radius-lg) border border-(--color-border) bg-(--color-background) p-1 gap-1"
	role="radiogroup"
	aria-label="Theme preference"
>
	{#each options as option (option.value)}
		{@const isActive = app.theme === option.value}
		<button
			type="button"
			role="radio"
			aria-checked={isActive}
			aria-label={option.ariaLabel}
			onclick={() => app.setTheme(option.value)}
			class={cn(
				'inline-flex items-center gap-1.5 rounded-(--radius-md) px-4 py-2 text-sm font-medium transition-all duration-150',
				isActive
					? 'bg-(--color-surface-elevated) text-(--color-foreground) shadow-sm'
					: 'text-(--color-muted) hover:text-(--color-foreground)'
			)}
		>
			<option.icon class="size-4" />
			{option.label}
		</button>
	{/each}
</div>
<p class="text-xs text-(--color-subtle) mt-2">
	{getAnnotation(app.theme)}
</p>

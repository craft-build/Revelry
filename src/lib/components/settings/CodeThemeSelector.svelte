<script lang="ts">
	import { app } from '$lib/stores/app.svelte';
	import { Select } from '$lib/components/ui';
	import { getThemeCatalog } from '$lib/shiki.svelte';

	const catalog = getThemeCatalog();
	const darkThemes = catalog.filter((t) => t.type === 'dark');
	const lightThemes = catalog.filter((t) => t.type === 'light');

	let selectedDark = $state(app.code_theme_dark);
	let selectedLight = $state(app.code_theme_light);

	// Sync local state if AppState changes externally (e.g. backend sync)
	$effect(() => {
		selectedDark = app.code_theme_dark;
		selectedLight = app.code_theme_light;
	});

	function handleDarkChange() {
		app.setCodeThemeDark(selectedDark);
	}

	function handleLightChange() {
		app.setCodeThemeLight(selectedLight);
	}
</script>

<div class="space-y-4">
	<div class="space-y-1">
		<label for="code-theme-dark" class="text-sm font-medium text-(--color-foreground)"
			>Dark mode theme</label
		>
		<p class="text-xs text-(--color-muted)"
			>Markdown colors when the app is in dark mode.</p
		>
		<div class="mt-2">
			<Select id="code-theme-dark" bind:value={selectedDark} onchange={handleDarkChange}>
				{#each darkThemes as theme (theme.id)}
					<option value={theme.id}>{theme.displayName}</option>
				{/each}
			</Select>
		</div>
	</div>
	<div class="space-y-1">
		<label for="code-theme-light" class="text-sm font-medium text-(--color-foreground)"
			>Light mode theme</label
		>
		<p class="text-xs text-(--color-muted)"
			>Markdown colors when the app is in light mode.</p
		>
		<div class="mt-2">
			<Select id="code-theme-light" bind:value={selectedLight} onchange={handleLightChange}>
				{#each lightThemes as theme (theme.id)}
					<option value={theme.id}>{theme.displayName}</option>
				{/each}
			</Select>
		</div>
	</div>
</div>

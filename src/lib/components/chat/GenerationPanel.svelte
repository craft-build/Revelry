<script lang="ts">
	import { app } from '$lib/stores/app.svelte';
	import { cn } from '$lib/utils';
	import { X, ExternalLink, Sun, Moon } from '@lucide/svelte';
	import { Button, Input, Select } from '$lib/components/ui';
	import { goto } from '$app/navigation';
	import { getThemeCatalog } from '$lib/shiki.svelte';

	let localOverride = $state<string>(
		app.active_conversation?.override_max_tokens?.toString() ?? ''
	);

	let isUpdating = $state(false);

	const catalog = getThemeCatalog();
	const darkCodeThemes = catalog.filter((t) => t.type === 'dark');
	const lightCodeThemes = catalog.filter((t) => t.type === 'light');

	let localCodeTheme = $state(
		app.resolved_theme === 'dark' ? app.code_theme_dark : app.code_theme_light
	);

	$effect(() => {
		localCodeTheme = app.resolved_theme === 'dark' ? app.code_theme_dark : app.code_theme_light;
	});

	function handleCodeThemeChange() {
		if (app.resolved_theme === 'dark') {
			app.setCodeThemeDark(localCodeTheme);
		} else {
			app.setCodeThemeLight(localCodeTheme);
		}
	}

	$effect(() => {
		const conv = app.active_conversation;
		localOverride = conv?.override_max_tokens?.toString() ?? '';
	});

	function handlePresetChange(e: Event) {
		const select = e.target as HTMLSelectElement;
		const value = select.value || null;
		isUpdating = true;
		app.updateConversationGeneration(value, app.active_conversation?.override_max_tokens ?? null)
			.catch((err) => console.error('Failed to update preset:', err))
			.finally(() => (isUpdating = false));
	}

	function handleMaxTokensBlur() {
		const parsed = parseInt(localOverride, 10);
		const value = isNaN(parsed) || parsed <= 0 ? null : parsed;
		isUpdating = true;
		app.updateConversationGeneration(
			app.active_conversation?.generation_preset_id ?? null,
			value
		)
			.catch((err) => console.error('Failed to update max tokens:', err))
			.finally(() => (isUpdating = false));
	}

	function handleClearMaxTokens() {
		localOverride = '';
		isUpdating = true;
		app.updateConversationGeneration(
			app.active_conversation?.generation_preset_id ?? null,
			null
		)
			.catch((err) => console.error('Failed to clear max tokens:', err))
			.finally(() => (isUpdating = false));
	}
</script>

<aside
	class="flex w-72 shrink-0 flex-col border-l border-(--color-border) bg-(--color-surface)"
>
	<!-- Header -->
	<div class="flex h-[60px] shrink-0 items-center justify-between border-b border-(--color-border) px-4">
		<h3 class="text-sm font-semibold">Generation Settings</h3>
		<button
			type="button"
			onclick={() => (app.generation_panel_open = false)}
			class="inline-flex size-7 items-center justify-center rounded-(--radius-md) text-(--color-muted) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
			aria-label="Close panel"
		>
			<X class="size-4" />
		</button>
	</div>

	<!-- Content -->
	<div class="flex-1 overflow-y-auto p-4">
		<div class="flex flex-col gap-5">
			<!-- Generation Profile -->
			<div>
				<label
					for="preset-select"
					class="mb-1.5 block text-xs font-medium text-(--color-muted)"
				>
					Generation Profile
				</label>
				<select
					id="preset-select"
					class="w-full rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-input) px-3 py-2 text-sm text-(--color-foreground) outline-none focus:border-(--color-accent)"
					value={app.active_conversation?.generation_preset_id ?? ''}
					onchange={handlePresetChange}
					disabled={isUpdating}
				>
					<option value="">None (use provider defaults)</option>
					{#each app.presets as preset (preset.id)}
						<option value={preset.id}>{preset.name}</option>
					{/each}
				</select>
			</div>

			<!-- Response Length -->
			<div>
				<label
					for="max-tokens-override"
					class="mb-1.5 block text-xs font-medium text-(--color-muted)"
				>
					Response Length
				</label>
				<div class="flex items-center gap-2">
					<Input
						id="max-tokens-override"
						type="number"
						placeholder="Use default"
						value={localOverride}
						oninput={(e) => (localOverride = (e.target as HTMLInputElement).value)}
						onblur={handleMaxTokensBlur}
						class="flex-1 font-mono text-sm"
						disabled={isUpdating}
					/>
					{#if localOverride}
						<Button variant="ghost" size="sm" onclick={handleClearMaxTokens} disabled={isUpdating}>
							<X class="size-3" />
						</Button>
					{/if}
				</div>
				<p class="mt-1 text-[11px] text-(--color-subtle)">
					{#if app.effective_max_tokens > 0}
						Instructs the AI to target ~{app.effective_max_tokens.toLocaleString()} tokens (~{Math.round(app.effective_max_tokens * 0.75).toLocaleString()} words).
					{:else}
						No length instruction set — AI decides response length.
					{/if}
				</p>
			</div>

			<!-- Active preset info -->
			{#if app.active_preset}
				<div class="rounded-(--radius-md) border border-(--color-border) bg-(--color-surface) p-3">
					<div class="mb-2 text-xs font-semibold text-(--color-muted)">
						{app.active_preset.name}
						{#if app.active_preset.is_builtin}
							<span class="ml-1 text-[10px] font-normal uppercase tracking-wider text-(--color-subtle)">built-in</span>
						{/if}
					</div>
					<div class="grid grid-cols-2 gap-x-4 gap-y-1 text-xs">
						<div>
							<span class="text-(--color-subtle)">Temp</span>{' '}
							<span class="font-mono">{app.active_preset.temperature}</span>
						</div>
						<div>
							<span class="text-(--color-subtle)">Top P</span>{' '}
							<span class="font-mono">{app.active_preset.top_p}</span>
						</div>
						<div>
							<span class="text-(--color-subtle)">Freq Pen</span>{' '}
							<span class="font-mono">{app.active_preset.frequency_penalty}</span>
						</div>
						<div>
							<span class="text-(--color-subtle)">Pres Pen</span>{' '}
							<span class="font-mono">{app.active_preset.presence_penalty}</span>
						</div>
					</div>
					{#if app.active_preset.stop_sequences.length > 0}
						<div class="mt-2 text-xs">
							<span class="text-(--color-subtle)">Stop sequences:</span>{' '}
							<span class="font-mono">{app.active_preset.stop_sequences.join(', ')}</span>
						</div>
					{/if}
				</div>
			{/if}

			<!-- Link to settings -->
			<button
				type="button"
				onclick={() => {
					app.generation_panel_open = false;
					goto('/settings?section=generation');
				}}
				class="inline-flex items-center gap-1.5 text-xs text-(--color-muted) hover:text-(--color-foreground)"
			>
				<ExternalLink class="size-3" />
				Manage presets in Settings
			</button>

			<!-- Theme toggle -->
			<div class="border-t border-(--color-border) pt-4">
				<span class="mb-1.5 block text-xs font-medium text-(--color-muted)">Theme</span>
				<button
					type="button"
				onclick={() => {
						app.setTheme(app.resolved_theme === 'dark' ? 'light' : 'dark');
					}}
					class={cn(
						'inline-flex w-full items-center justify-center gap-2 rounded-(--radius-md) border border-(--color-border) px-3 py-2 text-sm font-medium transition-all duration-150',
						'bg-(--color-surface-input) text-(--color-foreground) hover:bg-(--color-surface-hover)'
					)}
				>
					{#if app.resolved_theme === 'dark'}
						<Sun class="size-4" />
						Switch to Light
					{:else}
						<Moon class="size-4" />
						Switch to Dark
					{/if}
				</button>

				<!-- Code theme selector -->
				<div class="pt-3">
					<span class="mb-1.5 block text-xs font-medium text-(--color-muted)">Code Theme</span>
					<p class="mb-1.5 text-[11px] text-(--color-subtle)">
						{#if app.resolved_theme === 'dark'}
							Markdown colors for dark mode.
						{:else}
							Markdown colors for light mode.
						{/if}
					</p>
					<Select
						bind:value={localCodeTheme}
						onchange={handleCodeThemeChange}
						class="w-full text-sm"
					>
						{#if app.resolved_theme === 'dark'}
							{#each darkCodeThemes as theme (theme.id)}
								<option value={theme.id}>{theme.displayName}</option>
							{/each}
						{:else}
							{#each lightCodeThemes as theme (theme.id)}
								<option value={theme.id}>{theme.displayName}</option>
							{/each}
						{/if}
					</Select>
				</div>
			</div>
		</div>
	</div>
</aside>

<script lang="ts">
	import Logo from '$lib/components/ui/Logo.svelte';
	import type { Provider, GenerationPreset } from '$lib/types';
	import { app } from '$lib/stores/app.svelte';
	import { Button, Tag, KebabMenu, KebabMenuItem, KebabMenuSeparator } from '$lib/components/ui';
	import { ProviderEditorDialog, KeyringUnavailableBanner } from '$lib/components/providers';
	import PresetEditorDialog from '$lib/components/settings/PresetEditorDialog.svelte';
	import { DeleteConfirmationDialog } from '$lib/components/characters';
	import ThemeToggle from '$lib/components/settings/ThemeToggle.svelte';
	import CodeThemeSelector from '$lib/components/settings/CodeThemeSelector.svelte';
	import {
		Cpu,
		SlidersHorizontal,
		Palette,
		KeyRound,
		Database,
		Info,
		Plus,
		Check,
		X,
		Trash2
	} from '@lucide/svelte';
	import { page } from '$app/state';

	const sections = [
		{ id: 'providers', label: 'AI Providers', icon: Cpu },
		{ id: 'generation', label: 'Generation', icon: SlidersHorizontal },
		{ id: 'appearance', label: 'Appearance', icon: Palette },
		{ id: 'shortcuts', label: 'Shortcuts', icon: KeyRound },
		{ id: 'data', label: 'Data', icon: Database },
		{ id: 'about', label: 'About', icon: Info }
	];

	let active = $state<string>('providers');
	let editor_open = $state(false);
	let editor_mode = $state<'create' | 'edit'>('create');
	let editor_provider = $state<Provider | undefined>(undefined);
	let delete_open = $state(false);
	let delete_target = $state<string | null>(null);
	let delete_loading = $state(false);

	// Preset editor state
	let preset_editor_open = $state(false);
	let preset_editor_mode = $state<'create' | 'edit' | 'view'>('create');
	let preset_editor_preset = $state<GenerationPreset | undefined>(undefined);
	let preset_delete_open = $state(false);
	let preset_delete_target = $state<string | null>(null);
	let preset_delete_loading = $state(false);

	// Auto-navigate from URL param ?section=generation
	$effect(() => {
		const section = page.url.searchParams.get('section');
		if (section && sections.some((s) => s.id === section)) {
			active = section;
		}
	});
</script>

<svelte:head>
	<title>Settings — Revelry</title>
</svelte:head>

<div class="flex h-full">
	<!-- Settings nav (AdwPreferencesWindow sidebar) -->
	<aside class="w-56 shrink-0 border-r border-(--color-border) bg-(--color-surface)/70 px-3 py-4 backdrop-blur-[20px]">
		<h2 class="mb-3 px-2 font-display text-base font-semibold tracking-[-0.02em]">Preferences</h2>
		<nav class="flex flex-col gap-0.5">
			{#each sections as section (section.id)}
				<button
					type="button"
					onclick={() => (active = section.id)}
					class="inline-flex items-center gap-2.5 rounded-(--radius-md) px-2.5 py-2 text-left text-sm transition-colors {active ===
					section.id
						? 'bg-(--color-surface-elevated) text-(--color-foreground)'
						: 'text-(--color-muted) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)'}"
				>
					<section.icon class="size-4" />
					{section.label}
				</button>
			{/each}
		</nav>
	</aside>

	<!-- Settings content -->
	<div class="flex-1 overflow-y-auto px-10 py-10">
		<div class="mx-auto max-w-3xl">
			{#if active === 'providers'}
				<header class="mb-6">
					<h1 class="font-display text-[26px] font-semibold tracking-[-0.03em]">AI Providers</h1>
					<p class="mt-1 text-sm text-(--color-muted)">
						Configure AI providers and API keys. Keys are stored in your OS keychain.
					</p>
				</header>

				{#if !app.keyring_available}
					<KeyringUnavailableBanner class="mb-4" />
				{/if}

				{#if app.providers.length === 0}
					<div class="flex flex-col items-center justify-center gap-4 py-12 text-center">
						<div class="flex size-12 items-center justify-center rounded-full bg-(--color-surface-elevated) text-(--color-muted)">
							<Cpu class="size-6" />
						</div>
						<div>
							<p class="font-medium text-(--color-foreground)">No providers yet.</p>
							<p class="mt-1 text-sm text-(--color-muted)">
								Add an AI provider to start chatting. API keys are stored securely in your
								OS keychain.
							</p>
						</div>
						<Button
							variant="primary"
							size="sm"
							onclick={() => {
								editor_mode = 'create';
								editor_provider = undefined;
								editor_open = true;
							}}
						>
							<Plus class="size-4" />
							Add provider
						</Button>
					</div>
				{:else}
					<div class="flex flex-col gap-3">
						{#each app.providers as provider (provider.id)}
							<div
								class="rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface)"
							>
								<div class="flex items-center justify-between gap-3 border-b border-(--color-border) px-4 py-3">
									<div>
										<h3 class="font-display text-base font-semibold tracking-tight">
											{provider.name}
										</h3>
										<div class="mt-0.5 flex items-center gap-2 text-[11px] text-(--color-subtle)">
											<Tag>{provider.provider_type}</Tag>
											{#if provider.models.length > 0}
												<span class="font-mono">
													{provider.models.slice(0, 3).map((m) => m.name).join(' · ')}
													{#if provider.models.length > 3}
														<span class="text-(--color-muted)"> +{provider.models.length - 3} more</span>
													{/if}
												</span>
											{:else}
												<span class="font-mono">{provider.model_name}</span>
											{/if}
										</div>
									</div>
									<div class="flex items-center gap-2">
										{#if provider.has_api_key}
											<span class="inline-flex items-center gap-1 text-xs text-(--color-success)">
												<Check class="size-3" />
												Key set
											</span>
										{:else}
											<span class="inline-flex items-center gap-1 text-xs text-(--color-subtle)">
												<X class="size-3" />
												No key
											</span>
										{/if}
										<KebabMenu>
											<KebabMenuItem
												onclick={() => {
													editor_mode = 'edit';
													editor_provider = provider;
													editor_open = true;
												}}
											>
												Edit
											</KebabMenuItem>
											<KebabMenuSeparator />
											<KebabMenuItem
												onclick={() => {
													delete_target = provider.id;
													delete_open = true;
												}}
											>
												Delete
											</KebabMenuItem>
										</KebabMenu>
									</div>
								</div>

								<div class="px-4 py-3">
									<div class="grid grid-cols-3 gap-2 text-xs">
										<div>
											<div class="text-(--color-subtle)">Temperature</div>
											<div class="font-mono">{provider.default_params?.temperature ?? '—'}</div>
										</div>
										<div>
											<div class="text-(--color-subtle)">Top P</div>
											<div class="font-mono">{provider.default_params?.top_p ?? '—'}</div>
										</div>
										<div>
											<div class="text-(--color-subtle)">Max tokens</div>
											<div class="font-mono">{provider.default_params?.max_tokens ?? '—'}</div>
										</div>
									</div>
								</div>
							</div>
						{/each}

						<button
							type="button"
							onclick={() => {
								editor_mode = 'create';
								editor_provider = undefined;
								editor_open = true;
							}}
							class="inline-flex items-center justify-center gap-2 rounded-(--radius-lg) border border-dashed border-(--color-border) px-4 py-4 text-sm text-(--color-muted) hover:border-(--color-border-strong) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
						>
							<Plus class="size-4" />
							Add provider
						</button>
					</div>
				{/if}
			{:else if active === 'generation'}
				<header class="mb-6">
					<h1 class="font-display text-[26px] font-semibold tracking-[-0.03em]">Generation</h1>
					<p class="mt-1 text-sm text-(--color-muted)">
						Default parameters and prompt presets. Presets can be assigned per-character or
						per-conversation.
					</p>
				</header>

				<h2 class="mb-3 font-display text-sm font-semibold uppercase tracking-wider text-(--color-muted)">
					Prompt presets
				</h2>
				<div class="flex flex-col gap-2">
					{#each app.presets as preset (preset.id)}
						<div
							class="flex items-center justify-between gap-3 rounded-(--radius-md) border border-(--color-border) bg-(--color-surface) px-4 py-3"
						>
							<div>
								<div class="font-medium">
									{preset.name}
									{#if preset.is_builtin}
										<span class="ml-1 text-[10px] font-normal uppercase tracking-wider text-(--color-subtle)">built-in</span>
									{/if}
								</div>
								<div class="text-[11px] text-(--color-subtle)">
									temp {preset.temperature} · top_p {preset.top_p} · {preset.max_tokens} tok{preset.stop_sequences.length ? ` · stops: ${preset.stop_sequences.length}` : ''}
								</div>
							</div>
							<div class="flex items-center gap-1">
								{#if preset.is_builtin}
									<Button
										variant="ghost"
										size="sm"
										onclick={() => {
											preset_editor_mode = 'view';
											preset_editor_preset = preset;
											preset_editor_open = true;
										}}
									>
										View
									</Button>
								{:else}
									<Button
										variant="ghost"
										size="sm"
										onclick={() => {
											preset_editor_mode = 'edit';
											preset_editor_preset = preset;
											preset_editor_open = true;
										}}
									>
										Edit
									</Button>
									<button
										type="button"
										onclick={() => {
											preset_delete_target = preset.id;
											preset_delete_open = true;
										}}
										class="inline-flex size-7 items-center justify-center rounded-(--radius-md) text-(--color-subtle) hover:bg-(--color-surface-hover) hover:text-(--color-destructive)"
										aria-label="Delete preset"
									>
										<Trash2 class="size-3.5" />
									</button>
								{/if}
							</div>
						</div>
					{/each}

					<button
						type="button"
						onclick={() => {
							preset_editor_mode = 'create';
							preset_editor_preset = undefined;
							preset_editor_open = true;
						}}
						class="inline-flex items-center justify-center gap-2 rounded-(--radius-md) border border-dashed border-(--color-border) px-4 py-3 text-sm text-(--color-muted) hover:border-(--color-border-strong) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
					>
						<Plus class="size-4" />
						Add preset
					</button>
				</div>
			{:else if active === 'appearance'}
				<header class="mb-6">
					<h1 class="font-display text-[26px] font-semibold tracking-[-0.03em]">Appearance</h1>
					<p class="mt-1 text-sm text-(--color-muted)">
						Theme, density, and reading comfort.
					</p>
				</header>
				<div class="rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface) p-4">
					<h2 class="font-display text-base font-semibold tracking-tight text-(--color-foreground)">Theme</h2>
					<p class="text-sm text-(--color-muted) mt-1">Choose how Revelry looks. System follows your OS preference.</p>
					<div class="mt-4">
						<ThemeToggle />
					</div>
				</div>
				<div class="mt-4 rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface) p-4">
					<h2 class="font-display text-base font-semibold tracking-tight text-(--color-foreground)">Markdown Theme</h2>
					<p class="text-sm text-(--color-muted) mt-1">Color scheme for rendered markdown in message bubbles.</p>
					<div class="mt-4">
						<CodeThemeSelector />
					</div>
				</div>
			{:else if active === 'shortcuts'}
				<header class="mb-6">
					<h1 class="font-display text-[26px] font-semibold tracking-[-0.03em]">Keyboard shortcuts</h1>
				</header>
				<div class="overflow-hidden rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface)">
					<table class="w-full text-sm">
						<tbody>
							{#each [['New chat', 'Ctrl+N'], ['New character', 'Ctrl+Shift+N'], ['Toggle sidebar', 'F9'], ['Send message', 'Enter'], ['Newline in input', 'Shift+Enter'], ['Cancel generation', 'Esc']] as row}
								<tr class="border-t border-(--color-border) first:border-t-0">
									<td class="px-4 py-2.5">{row[0]}</td>
									<td class="px-4 py-2.5 text-right">
										<kbd class="rounded-(--radius-sm) border border-(--color-border) bg-(--color-surface-elevated) px-2 py-0.5 font-mono text-[11px]">
											{row[1]}
										</kbd>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{:else if active === 'data'}
				<header class="mb-6">
					<h1 class="font-display text-[26px] font-semibold tracking-[-0.03em]">Data</h1>
					<p class="mt-1 text-sm text-(--color-muted)">
						Backup, restore, and export all of your data.
					</p>
				</header>
				<p class="text-sm text-(--color-muted)">Coming soon — export, backup, and data management will be available when the backend supports these operations.</p>
			{:else if active === 'about'}
				<header class="mb-6">
					<h1 class="font-display text-[26px] font-semibold tracking-[-0.03em]">About</h1>
				</header>
				<div
					class="relative overflow-hidden rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface) p-6"
				>
					<!-- Tail-sweep hairline — the one brand moment on this page -->
					<div
						class="absolute inset-x-0 top-0 h-[3px]"
						style="background: var(--gradient-brand)"
						aria-hidden="true"
					></div>
					<div class="flex items-center gap-3.5">
						<Logo class="size-14" alt="" />
						<div>
							<div class="font-display text-[22px] font-semibold tracking-[-0.02em]">Revelry</div>
							<div class="font-mono text-xs text-(--color-subtle)">v{__APP_VERSION__}</div>
						</div>
					</div>
					<p class="mt-4 text-sm text-(--color-muted)">
						A private, local-first roleplay app. Bring your own key for any of 15 providers —
						keys stay in your OS keychain, and conversations, characters and lorebooks never
						leave your machine.
					</p>
				</div>
			{/if}
		</div>
	</div>
</div>

<ProviderEditorDialog
	bind:open={editor_open}
	mode={editor_mode}
	provider={editor_provider}
/>

<DeleteConfirmationDialog
	bind:open={delete_open}
	title={delete_target ? `Delete ${app.providers.find((p) => p.id === delete_target)?.name ?? 'provider'}?` : 'Delete provider?'}
	description="This will permanently remove this provider, its models, and the stored API key. Conversations using this provider will lose their model reference. This cannot be undone."
	confirm_label="Delete"
	loading={delete_loading}
	onconfirm={async () => {
		if (delete_target) {
			delete_loading = true;
			try {
				await app.deleteProvider(delete_target);
				delete_open = false;
			} catch (err) {
				console.error('Failed to delete provider:', err);
			} finally {
				delete_loading = false;
				delete_target = null;
			}
		}
	}}
/>

<PresetEditorDialog
	bind:open={preset_editor_open}
	mode={preset_editor_mode}
	preset={preset_editor_preset}
/>

<DeleteConfirmationDialog
	bind:open={preset_delete_open}
	title={preset_delete_target ? `Delete ${app.presets.find((p) => p.id === preset_delete_target)?.name ?? 'preset'}?` : 'Delete preset?'}
	description="This will permanently remove this preset. Conversations using this preset will fall back to provider defaults. This cannot be undone."
	confirm_label="Delete"
	loading={preset_delete_loading}
	onconfirm={async () => {
		if (preset_delete_target) {
			preset_delete_loading = true;
			try {
				await app.deletePreset(preset_delete_target);
				preset_delete_open = false;
			} catch (err) {
				console.error('Failed to delete preset:', err);
			} finally {
				preset_delete_loading = false;
				preset_delete_target = null;
			}
		}
	}}
/>

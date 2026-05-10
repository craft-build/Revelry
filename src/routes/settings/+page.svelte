<script lang="ts">
	import { app } from '$lib/stores/app.svelte';
	import { Button, Input, Tag } from '$lib/components/ui';
	import {
		Cpu,
		SlidersHorizontal,
		Palette,
		KeyRound,
		Database,
		Info,
		Plus,
		Eye,
		EyeOff
	} from '@lucide/svelte';

	const sections = [
		{ id: 'providers', label: 'AI Providers', icon: Cpu },
		{ id: 'generation', label: 'Generation', icon: SlidersHorizontal },
		{ id: 'appearance', label: 'Appearance', icon: Palette },
		{ id: 'shortcuts', label: 'Shortcuts', icon: KeyRound },
		{ id: 'data', label: 'Data', icon: Database },
		{ id: 'about', label: 'About', icon: Info }
	];

	let active = $state<string>('providers');
	let revealKey = $state<Record<string, boolean>>({});
</script>

<svelte:head>
	<title>Settings — Revelry</title>
</svelte:head>

<div class="flex h-full">
	<!-- Settings nav (AdwPreferencesWindow sidebar) -->
	<aside class="w-56 shrink-0 border-r border-(--color-border) bg-(--color-surface) px-3 py-4">
		<h2 class="mb-3 px-2 font-display text-base font-semibold tracking-tight">Preferences</h2>
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
					<h1 class="font-display text-2xl font-semibold tracking-tight">AI Providers</h1>
					<p class="mt-1 text-sm text-(--color-muted)">
						Configure providers via the rig framework. API keys are stored in your OS keychain
						and never written to disk in plaintext.
					</p>
				</header>

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
										<span class="font-mono">{provider.model}</span>
									</div>
								</div>
								<Button variant="ghost" size="sm">Edit</Button>
							</div>

							<div class="flex flex-col gap-2.5 px-4 py-3">
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

								<div class="flex items-center gap-2">
									<Input
										type={revealKey[provider.id] ? 'text' : 'password'}
										value="sk-…example…not-stored"
										readonly
										class="h-8 flex-1 font-mono text-xs"
									/>
									<button
										type="button"
										onclick={() =>
											(revealKey = {
												...revealKey,
												[provider.id]: !revealKey[provider.id]
											})}
										class="inline-flex size-8 items-center justify-center rounded-(--radius-md) border border-(--color-border) text-(--color-muted) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
										aria-label="Toggle key visibility"
									>
										{#if revealKey[provider.id]}
											<EyeOff class="size-3.5" />
										{:else}
											<Eye class="size-3.5" />
										{/if}
									</button>
								</div>
							</div>
						</div>
					{/each}

					<button
						type="button"
						class="inline-flex items-center justify-center gap-2 rounded-(--radius-lg) border border-dashed border-(--color-border) px-4 py-4 text-sm text-(--color-muted) hover:border-(--color-border-strong) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
					>
						<Plus class="size-4" />
						Add provider
					</button>
				</div>
			{:else if active === 'generation'}
				<header class="mb-6">
					<h1 class="font-display text-2xl font-semibold tracking-tight">Generation</h1>
					<p class="mt-1 text-sm text-(--color-muted)">
						Default parameters and prompt presets. Presets can be assigned per-character or
						per-conversation.
					</p>
				</header>

				<h2 class="mb-3 font-display text-sm font-semibold uppercase tracking-wider text-(--color-muted)">
					Prompt presets
				</h2>
				<div class="flex flex-col gap-2">
					{#each app.prompt_presets as preset (preset.id)}
						<div
							class="flex items-center justify-between gap-3 rounded-(--radius-md) border border-(--color-border) bg-(--color-surface) px-4 py-3"
						>
							<div>
								<div class="font-medium">{preset.name}</div>
								<div class="text-[11px] text-(--color-subtle)">
									temp {preset.temperature} · top_p {preset.top_p} · {preset.max_tokens} tok
								</div>
							</div>
							<Button variant="ghost" size="sm">Edit</Button>
						</div>
					{/each}
				</div>
			{:else if active === 'appearance'}
				<header class="mb-6">
					<h1 class="font-display text-2xl font-semibold tracking-tight">Appearance</h1>
					<p class="mt-1 text-sm text-(--color-muted)">
						Theme, density, and reading comfort.
					</p>
				</header>
				<div class="rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface) p-4 text-sm text-(--color-muted)">
					Theme controls coming soon. The current build is dark-only.
				</div>
			{:else if active === 'shortcuts'}
				<header class="mb-6">
					<h1 class="font-display text-2xl font-semibold tracking-tight">Keyboard shortcuts</h1>
				</header>
				<div class="overflow-hidden rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface)">
					<table class="w-full text-sm">
						<tbody>
							{#each [['New chat', 'Ctrl+N'], ['New character', 'Ctrl+Shift+N'], ['Toggle sidebar', 'F9'], ['Search', 'Ctrl+F'], ['Open settings', 'Ctrl+,'], ['Send message', 'Enter'], ['Newline in input', 'Shift+Enter'], ['Cancel generation', 'Esc']] as row}
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
					<h1 class="font-display text-2xl font-semibold tracking-tight">Data</h1>
					<p class="mt-1 text-sm text-(--color-muted)">
						Backup, restore, and export all of your data.
					</p>
				</header>
				<div class="flex flex-col gap-2">
					<Button variant="secondary">Export everything to ZIP</Button>
					<Button variant="secondary">Backup database</Button>
					<Button variant="destructive">Clear conversation history</Button>
				</div>
			{:else if active === 'about'}
				<header class="mb-6">
					<h1 class="font-display text-2xl font-semibold tracking-tight">About</h1>
				</header>
				<div class="rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface) p-6">
					<div class="flex items-center gap-3">
						<div
							class="grid size-12 place-items-center rounded-(--radius-lg) bg-gradient-to-br from-(--color-accent) to-(--color-accent-muted)"
						></div>
						<div>
							<div class="font-display text-xl font-semibold tracking-tight">Revelry</div>
							<div class="text-xs text-(--color-subtle)">v0.1.0 · UI scaffold</div>
						</div>
					</div>
					<p class="mt-4 text-sm text-(--color-muted)">
						SvelteKit + Svelte 5 + Bits.ui front-end for the Revelry roleplay desktop
						application. The backend is built in Rust on the rig framework.
					</p>
				</div>
			{/if}
		</div>
	</div>
</div>

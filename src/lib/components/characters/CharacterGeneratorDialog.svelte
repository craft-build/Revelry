<script lang="ts">
	import type { GeneratedCharacterData } from '$lib/types';
	import { app } from '$lib/stores/app.svelte';
	import { Dialog, Button, Textarea, Tag } from '$lib/components/ui';
	import { untrack } from 'svelte';
	import { Popover } from 'bits-ui';
	import { Sparkles, AlertCircle, Loader2, RefreshCw, Pencil, Check, Settings } from '@lucide/svelte';
	import { cn } from '$lib/utils';

	interface Props {
		open?: boolean;
		onopenchange?: (open: boolean) => void;
		oneditdetails?: (data: GeneratedCharacterData) => void;
	}

	let {
		open = $bindable(false),
		onopenchange,
		oneditdetails
	}: Props = $props();

	let idea = $state('');
	let selected_provider_id = $state('');
	let selected_model_name = $state('');
	let popover_open = $state(false);
	let phase = $state<'idle' | 'generating' | 'preview' | 'error'>('idle');

	$effect(() => {
		if (open) {
			untrack(() => {
				idea = '';
				phase = 'idle';
				app.clearGeneration();
				const first = app.available_generation_providers[0];
				if (first) {
					selected_provider_id = first.id;
					const firstEnabled = first.models.find((m) => m.enabled);
					selected_model_name = firstEnabled?.name ?? first.model_name ?? '';
				} else {
					selected_provider_id = '';
					selected_model_name = '';
				}
			});
		}
	});

	// Sync phase with app state
	$effect(() => {
		if (app.is_generating_character) {
			phase = 'generating';
		} else if (app.generated_character_data) {
			phase = 'preview';
		} else if (app.generation_error) {
			phase = 'error';
		} else if (phase !== 'idle') {
			phase = 'idle';
		}
	});

	let selected_provider = $derived(
		selected_provider_id
			? app.available_generation_providers.find((p) => p.id === selected_provider_id)
			: undefined,
	);

	let display_label = $derived(
		selected_provider && selected_model_name
			? selected_model_name
			: selected_provider?.model_name ?? 'No model',
	);

	const availableProviders = $derived(app.available_generation_providers);

	function pickModel(providerId: string, modelName: string) {
		selected_provider_id = providerId;
		selected_model_name = modelName;
		popover_open = false;
	}

	function handleGenerate() {
		if (!idea.trim() || !selected_provider_id || !selected_model_name || app.is_generating_character) return;
		app.generateCharacter(idea, selected_provider_id, selected_model_name);
	}

	function handleRegenerate() {
		app.clearGeneration();
		phase = 'idle';
		handleGenerate();
	}

	function handleEditDetails() {
		if (app.generated_character_data && oneditdetails) {
			oneditdetails(app.generated_character_data);
			open = false;
			onopenchange?.(false);
		}
	}

	function handleClose() {
		app.clearGeneration();
	}

	function handleOpenChange(value: boolean) {
		open = value;
		onopenchange?.(value);
		if (!value) handleClose();
	}
</script>

<Dialog
	bind:open
	onopenchange={handleOpenChange}
	title={phase === 'preview' ? 'Generated Character' : 'Generate Character'}
	class="max-w-xl"
>
	{#if phase === 'idle' || phase === 'generating' || phase === 'error'}
		<div class="space-y-4">
			<div>
				<label for="gen-idea" class="mb-1.5 block text-sm font-medium text-(--color-foreground)">
					Describe your character idea
				</label>
				<Textarea
					id="gen-idea"
					bind:value={idea}
					placeholder="e.g., A cynical detective who solves crimes in a steampunk city..."
					autogrow
					disabled={phase === 'generating'}
				/>
			</div>

			<div>
				<label class="mb-1.5 block text-sm font-medium text-(--color-foreground)">
					Provider / model
				</label>
				{#if availableProviders.length === 0}
					<p class="text-sm text-(--color-muted)">
						No enabled providers with API keys configured.
						<a href="/settings" class="inline-flex items-center gap-1 text-(--color-accent) hover:underline">
							<Settings class="size-3" />
							Add a provider
						</a>
					</p>
				{:else}
					<Popover.Root bind:open={popover_open}>
						<Popover.Trigger
							class="inline-flex h-9 w-full items-center gap-1.5 rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-elevated) px-3 text-sm hover:bg-(--color-surface-hover) disabled:cursor-not-allowed disabled:opacity-50"
							disabled={phase === 'generating'}
						>
							<Sparkles class="size-3.5 text-(--color-accent)" />
							<span class="font-mono text-xs">{display_label}</span>
						</Popover.Trigger>

						<Popover.Content
							side="bottom"
							align="start"
							sideOffset={6}
							class="z-50 w-72 rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface-elevated) p-1.5 shadow-xl"
						>
							<div
								class="px-2 pb-1.5 pt-1 text-[10px] font-semibold uppercase tracking-wider text-(--color-subtle)"
							>
								Provider / model
							</div>

							<ul class="flex max-h-64 flex-col gap-0.5 overflow-y-auto">
								{#each availableProviders as provider (provider.id)}
									{@const provider_active = provider.id === selected_provider_id}
									<li>
										<div
											class="px-2 py-1 text-[10px] font-semibold uppercase tracking-wider text-(--color-subtle)"
										>
											{provider.name}
										</div>
										<ul class="flex flex-col gap-0.5">
											{#each provider.models.filter((m) => m.enabled) as model (model.name)}
												{@const active = provider_active && model.name === selected_model_name}
												<li>
													<button
														type="button"
														onclick={() => pickModel(provider.id, model.name)}
														class={cn(
															'flex w-full items-center gap-2.5 rounded-(--radius-md) px-2 py-1.5 text-left transition-colors',
															active
																? 'bg-(--color-surface)'
																: 'hover:bg-(--color-surface-hover)',
														)}
													>
														<span class="truncate font-mono text-[11px]">{model.name}</span>
														{#if active}
															<Check class="ml-auto size-3.5 shrink-0 text-(--color-accent)" />
														{/if}
													</button>
												</li>
											{/each}
										</ul>
									</li>
								{/each}
							</ul>

							<div class="my-1 h-px bg-(--color-border)"></div>

							<a
								href="/settings"
								class="flex items-center gap-2.5 rounded-(--radius-md) px-2 py-1.5 text-xs text-(--color-muted) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
							>
								<Settings class="size-3.5" />
								Manage providers
							</a>
						</Popover.Content>
					</Popover.Root>
				{/if}
			</div>

			<Button
				variant="primary"
				disabled={!idea.trim() || !selected_provider_id || !selected_model_name || app.is_generating_character || availableProviders.length === 0}
				onclick={handleGenerate}
			>
				{#if phase === 'generating'}
					<Loader2 class="size-4 animate-spin" />
					Generating…
				{:else}
					<Sparkles class="size-4" />
					Generate
				{/if}
			</Button>

			{#if phase === 'generating'}
				<div class="rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-elevated) p-3">
					<pre class="whitespace-pre-wrap text-xs font-mono text-(--color-muted) streaming-caret">{app.generated_character_preview}</pre>
				</div>
				<Button variant="secondary" size="sm" onclick={() => { app.clearGeneration(); }}>
					Stop
				</Button>
			{/if}

			{#if phase === 'error' || app.generation_error}
				<div class="flex items-start gap-2 rounded-(--radius-md) border border-(--color-destructive)/30 bg-(--color-destructive)/10 p-3">
					<AlertCircle class="size-4 shrink-0 text-(--color-destructive)" />
					<p class="text-sm text-(--color-destructive)">{app.generation_error}</p>
				</div>
			{/if}
		</div>
	{:else if phase === 'preview'}
		{#if app.generated_character_data}
			{@const data = app.generated_character_data}
			<div class="space-y-3">
			<div>
				<span class="text-xs font-medium uppercase tracking-wider text-(--color-subtle)">Name</span>
				<p class="mt-0.5 text-sm font-medium text-(--color-foreground)">{data.name}</p>
			</div>

			<div>
				<span class="text-xs font-medium uppercase tracking-wider text-(--color-subtle)">Description</span>
				<p class="mt-0.5 line-clamp-3 text-sm text-(--color-muted)">{data.description}</p>
			</div>

			<div>
				<span class="text-xs font-medium uppercase tracking-wider text-(--color-subtle)">Personality</span>
				<p class="mt-0.5 line-clamp-3 text-sm text-(--color-muted)">{data.personality}</p>
			</div>

			<div>
				<span class="text-xs font-medium uppercase tracking-wider text-(--color-subtle)">Scenario</span>
				<p class="mt-0.5 line-clamp-2 text-sm text-(--color-muted)">{data.scenario}</p>
			</div>

			<div>
				<span class="text-xs font-medium uppercase tracking-wider text-(--color-subtle)">First Message</span>
				<p class="mt-0.5 line-clamp-3 text-sm text-(--color-muted)">{data.first_mes}</p>
			</div>

			{#if data.tags.length > 0}
				<div>
					<span class="text-xs font-medium uppercase tracking-wider text-(--color-subtle)">Tags</span>
					<div class="mt-1 flex flex-wrap gap-1.5">
						{#each data.tags as tag}
							<Tag variant="outline">{tag}</Tag>
						{/each}
					</div>
				</div>
			{/if}

			<div class="flex items-center gap-2 pt-2">
				<Button variant="ghost" size="sm" onclick={handleRegenerate} disabled={app.is_generating_character}>
					<RefreshCw class="size-3.5" />
					Regenerate
				</Button>
				<Button variant="primary" size="sm" onclick={handleEditDetails}>
					<Pencil class="size-3.5" />
					Edit Details
				</Button>
				<Button variant="secondary" size="sm" onclick={() => { open = false; handleOpenChange(false); }}>
					Cancel
				</Button>
			</div>
		</div>
	{/if}
	{/if}
</Dialog>

<script lang="ts">
	import { app } from '$lib/stores/app.svelte';
	import { Button, Tag } from '$lib/components/ui';
	import { LOCAL_EMBEDDING_MODELS, type EmbeddingConfig, type Provider } from '$lib/types';
	import { Brain, Check, Cloud, Download, HardDrive, RefreshCw, Sparkles } from '@lucide/svelte';

	type Source = 'none' | 'remote' | 'local';

	/** Provider types the backend can drive for embeddings (matches build_remote_embedder). */
	const EMBEDDING_PROVIDER_TYPES = new Set([
		'openai',
		'custom_openai_compatible',
		'azure_openai',
		'ollama',
		'gemini',
		'cohere',
		'groq',
		'together',
		'xai',
		'perplexity',
		'huggingface',
		'deepseek',
		'openrouter'
	]);

	const eligible_providers = $derived(
		app.providers.filter(
			(p) => EMBEDDING_PROVIDER_TYPES.has(p.provider_type) && p.enabled && (p.has_api_key || p.provider_type === 'ollama')
		)
	);

	// Draft state — applied on Save / Test.
	let source = $state<Source>('none');
	let provider_id = $state('');
	let model = $state('');
	let local_model = $state(LOCAL_EMBEDDING_MODELS[0].key);

	let dirty = $state(false);
	let saving = $state(false);
	let testing = $state(false);
	let rebuilding = $state(false);
	let test_result = $state<{ dims: number; latency_ms: number } | null>(null);
	let error = $state('');

	// Hydrate the draft from the saved config once.
	$effect(() => {
		const config = app.embedding_config;
		if (config.source === 'remote') {
			source = 'remote';
			provider_id = config.providerId;
			model = config.model;
		} else if (config.source === 'local') {
			source = 'local';
			local_model = config.model;
		} else {
			source = 'none';
		}
		dirty = false;
		test_result = null;
		error = '';
	});

	// Poll status lightly while mounted so rebuild progress shows up.
	$effect(() => {
		const timer = setInterval(() => {
			if (rebuilding || app.embedding_status?.staleCount || dirty) {
				app.refreshEmbeddingStatus();
			}
		}, 5000);
		return () => clearInterval(timer);
	});

	const selected_provider = $derived<Provider | undefined>(
		app.providers.find((p) => p.id === provider_id)
	);

	/** Model suggestions from the provider's list, filtered toward embedders. */
	const model_suggestions = $derived.by(() => {
		const models = selected_provider?.models.map((m) => m.name) ?? [];
		if (models.length === 0) return [];
		const embedders = models.filter((m) => /embed|nomic|bge|voyage/i.test(m));
		return embedders.length > 0 ? embedders : models;
	});

	function mark_dirty() {
		dirty = true;
		test_result = null;
		error = '';
	}

	function select_source(next: Source) {
		if (next === source) return;
		source = next;
		mark_dirty();
	}

	function draft_config(): EmbeddingConfig {
		if (source === 'remote') {
			return { source: 'remote', providerId: provider_id, model: model.trim() };
		}
		if (source === 'local') {
			return { source: 'local', model: local_model };
		}
		return { source: 'none' };
	}

	async function save(): Promise<boolean> {
		error = '';
		const config = draft_config();
		if (config.source === 'remote' && (!config.providerId || !config.model)) {
			error = 'Pick a provider and enter an embedding model.';
			return false;
		}
		saving = true;
		try {
			await app.saveEmbeddingConfig(config);
			dirty = false;
			return true;
		} catch (err) {
			error = err instanceof Error ? err.message : String(err);
			return false;
		} finally {
			saving = false;
		}
	}

	async function test() {
		error = '';
		testing = true;
		test_result = null;
		try {
			if (dirty || source !== app.embedding_config.source) {
				const ok = await save();
				if (!ok) return;
			}
			const result = await app.testEmbeddingConnection();
			test_result = { dims: result.dims, latency_ms: result.latencyMs };
		} catch (err) {
			error = err instanceof Error ? err.message : String(err);
		} finally {
			testing = false;
		}
	}

	async function rebuild() {
		rebuilding = true;
		try {
			await app.rebuildEmbeddingIndexAction();
		} catch (err) {
			error = err instanceof Error ? err.message : String(err);
		} finally {
			rebuilding = false;
		}
	}

	const total_indexed = $derived(
		(app.embedding_status?.counts ?? []).reduce((sum, c) => sum + c.count, 0)
	);
</script>

<div class="flex flex-col gap-4">
	<!-- Source selection -->
	<div class="rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface) p-4">
		<h2 class="font-display text-base font-semibold tracking-tight text-(--color-foreground)">
			Embedding source
		</h2>
		<p class="mt-1 text-sm text-(--color-muted)">
			Embeddings power semantic lorebook activation and story memory recall. Everything works
			without them — enabling this adds semantic matching on top.
		</p>

		<div
			class="mt-4 inline-flex items-center gap-1 rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-input) p-0.5"
			role="tablist"
			aria-label="Embedding source"
		>
			<button
				type="button"
				role="tab"
				aria-selected={source === 'none'}
				onclick={() => select_source('none')}
				class="inline-flex items-center gap-1.5 rounded-(--radius-sm) px-3 py-1.5 text-xs font-medium transition-colors {source ===
				'none'
					? 'bg-(--color-surface-elevated) text-(--color-foreground) shadow-(--shadow-sm) ring-1 ring-(--color-border) ring-inset'
					: 'text-(--color-muted) hover:text-(--color-foreground)'}"
			>
				<Sparkles class="size-3.5" />
				Off
			</button>
			<button
				type="button"
				role="tab"
				aria-selected={source === 'remote'}
				onclick={() => select_source('remote')}
				class="inline-flex items-center gap-1.5 rounded-(--radius-sm) px-3 py-1.5 text-xs font-medium transition-colors {source ===
				'remote'
					? 'bg-(--color-surface-elevated) text-(--color-foreground) shadow-(--shadow-sm) ring-1 ring-(--color-border) ring-inset'
					: 'text-(--color-muted) hover:text-(--color-foreground)'}"
			>
				<Cloud class="size-3.5" />
				Provider
			</button>
			<button
				type="button"
				role="tab"
				aria-selected={source === 'local'}
				onclick={() => select_source('local')}
				class="inline-flex items-center gap-1.5 rounded-(--radius-sm) px-3 py-1.5 text-xs font-medium transition-colors {source ===
				'local'
					? 'bg-(--color-surface-elevated) text-(--color-foreground) shadow-(--shadow-sm) ring-1 ring-(--color-border) ring-inset'
					: 'text-(--color-muted) hover:text-(--color-foreground)'}"
			>
				<HardDrive class="size-3.5" />
				Local model
			</button>
		</div>

		{#if source === 'remote'}
			<div class="mt-4 flex flex-col gap-3">
				{#if eligible_providers.length === 0}
					<p class="text-sm text-(--color-muted)">
						No eligible providers yet. Add an OpenAI-compatible provider (for example a custom
						provider pointing at Synthetic) with an API key first.
					</p>
				{:else}
					<label class="flex flex-col gap-1.5 text-sm">
						<span class="text-(--color-muted)">Provider</span>
						<select
							class="rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-input) px-3 py-2 text-sm outline-none focus:border-(--color-accent) focus:shadow-(--shadow-glow)"
							bind:value={provider_id}
							onchange={mark_dirty}
						>
							<option value="" disabled>Select a provider…</option>
							{#each eligible_providers as provider (provider.id)}
								<option value={provider.id}>{provider.name}</option>
							{/each}
						</select>
					</label>
					<label class="flex flex-col gap-1.5 text-sm">
						<span class="text-(--color-muted)">Embedding model</span>
						<input
							list="embedding-model-suggestions"
							placeholder="e.g. nomic-embed-text-v1.5"
							bind:value={model}
							oninput={mark_dirty}
							class="w-full rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-input) px-3 py-2 font-mono text-sm outline-none placeholder:font-sans placeholder:text-(--color-subtle) focus:border-(--color-accent) focus:shadow-(--shadow-glow)"
						/>
						<datalist id="embedding-model-suggestions">
							{#each model_suggestions as suggestion (suggestion)}
								<option value={suggestion}></option>
							{/each}
						</datalist>
						<span class="text-xs text-(--color-subtle)">
							Uses this provider's API key. Any OpenAI-compatible /embeddings endpoint works.
						</span>
					</label>
					<p class="text-xs text-(--color-subtle)">
						Message and lore text is sent to this provider to compute embeddings.
					</p>
				{/if}
			</div>
		{:else if source === 'local'}
			<div class="mt-4 flex flex-col gap-3">
				<label class="flex flex-col gap-1.5 text-sm">
					<span class="text-(--color-muted)">Local model</span>
					<select
						class="rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-input) px-3 py-2 text-sm outline-none focus:border-(--color-accent) focus:shadow-(--shadow-glow)"
						bind:value={local_model}
						onchange={mark_dirty}
					>
						{#each LOCAL_EMBEDDING_MODELS as choice (choice.key)}
							<option value={choice.key}>{choice.label}</option>
						{/each}
					</select>
				</label>
				<p class="inline-flex items-start gap-1.5 text-xs text-(--color-subtle)">
					<Download class="mt-0.5 size-3.5 shrink-0" />
					Downloads on first use and runs entirely on this machine — nothing leaves your device.
				</p>
			</div>
		{/if}

		{#if error}
			<p class="mt-3 text-sm text-(--color-destructive)">{error}</p>
		{/if}
		{#if test_result}
			<p class="mt-3 inline-flex items-center gap-1.5 text-sm text-(--color-success)">
				<Check class="size-3.5" />
				<span class="font-mono">{test_result.dims} dims</span>
				<span class="text-(--color-subtle)">· {test_result.latency_ms} ms</span>
			</p>
		{/if}

		<div class="mt-4 flex items-center gap-2">
			<Button variant="primary" size="sm" disabled={!dirty || saving} onclick={() => save()}>
				{saving ? 'Saving…' : 'Save'}
			</Button>
			<Button
				variant="secondary"
				size="sm"
				disabled={testing || (source === 'remote' && (!provider_id || !model.trim())) || source === 'none'}
				onclick={() => test()}
			>
				{testing ? 'Testing…' : 'Test'}
			</Button>
		</div>
	</div>

	<!-- Index status -->
	<div class="rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface) p-4">
		<div class="flex items-center justify-between gap-3">
			<h2 class="font-display text-base font-semibold tracking-tight text-(--color-foreground)">
				Semantic index
			</h2>
			<Button variant="ghost" size="sm" disabled={rebuilding} onclick={() => rebuild()}>
				<RefreshCw class="size-3.5 {rebuilding ? 'animate-spin' : ''}" />
				{rebuilding ? 'Rebuilding…' : 'Rebuild'}
			</Button>
		</div>

		{#if app.embedding_status?.configured}
			<div class="mt-3 flex flex-wrap items-center gap-2 text-xs">
				<Tag>{app.embedding_status.source === 'local' ? 'local' : 'provider'}</Tag>
				<span class="font-mono text-(--color-muted)">{app.embedding_status.modelKey}</span>
			</div>
			<p class="mt-2 text-sm text-(--color-muted)">
				{total_indexed.toLocaleString()} items indexed
				{#if app.embedding_status.staleCount > 0}
					· <span class="text-(--color-warning)">
						{app.embedding_status.staleCount.toLocaleString()} stale — rebuild to refresh
					</span>
				{/if}
			</p>
			<p class="mt-1 text-xs text-(--color-subtle)">
				Messages and lorebook entries are embedded automatically as you chat and edit.
			</p>
		{:else}
			<p class="mt-3 text-sm text-(--color-muted)">
				<span class="inline-flex items-center gap-1.5"
					><Brain class="size-3.5" aria-hidden="true" />Off</span
				>
				— the story summary memory still works using your chat provider.
			</p>
		{/if}
	</div>
</div>

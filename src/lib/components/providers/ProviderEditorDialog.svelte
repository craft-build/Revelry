<script lang="ts">
	import type { Provider, ProviderType, ProviderModelEntry } from '$lib/types';
	import { app } from '$lib/stores/app.svelte';
	import {
		Dialog,
		Button,
		Input,
		Select
	} from '$lib/components/ui';
	import { ModelEntryRow } from '$lib/components/providers';
	import { DeleteConfirmationDialog } from '$lib/components/characters';
	import { AlertCircle, Plus, Eye, EyeOff, Loader2 } from '@lucide/svelte';
	import { cn } from '$lib/utils';

	interface Props {
		open?: boolean;
		onopenchange?: (open: boolean) => void;
		mode?: 'create' | 'edit';
		provider?: Provider;
		keyringAvailable?: boolean;
	}

	let {
		open = $bindable(false),
		onopenchange,
		mode = 'create',
		provider,
		keyringAvailable = app.keyring_available
	}: Props = $props();

	// ── Form state ────────────────────────────────────────────────────────
	let form_name = $state('');
	let form_provider_type = $state<string>('');
	let form_base_url = $state('');
	let form_api_key = $state('');
	let form_api_key_visible = $state(false);
	let form_models = $state<ProviderModelEntry[]>([]);
	let form_temperature = $state('');
	let form_top_p = $state('');
	let form_max_tokens = $state('');
	let is_saving = $state(false);
	let backend_error = $state('');
	let discard_open = $state(false);
	let prev_provider_type = $state<string>('');
	let type_change_confirm_open = $state(false);

	// ── Validation errors ──────────────────────────────────────────────────
	let name_error = $state('');
	let type_error = $state('');
	let base_url_error = $state('');
	let api_key_error = $state('');
	let model_errors = $state<Record<number, string>>({});
	let temperature_error = $state('');
	let top_p_error = $state('');
	let max_tokens_error = $state('');

	// ── Dirty tracking ────────────────────────────────────────────────────
	let initial_snapshot = $state('');

	function snapshot(): string {
		return JSON.stringify({
			form_name,
			form_provider_type,
			form_base_url,
			form_api_key,
			form_models,
			form_temperature,
			form_top_p,
			form_max_tokens
		});
	}

	let is_dirty = $derived(snapshot() !== initial_snapshot);

	// ── Provider type defaults ──────────────────────────────────────────────
	const type_defaults: Record<string, { base_url: string | null; model: string }> = {
		openai: { base_url: 'https://api.openai.com/v1', model: 'gpt-4o-mini' },
		anthropic: { base_url: 'https://api.anthropic.com/v1', model: 'claude-sonnet-4-20250514' },
		gemini: { base_url: 'https://generativelanguage.googleapis.com/v1beta', model: 'gemini-2.0-flash' },
		cohere: { base_url: 'https://api.cohere.com/v2', model: 'command-r-plus' },
		mistral: { base_url: 'https://api.mistral.ai/v1', model: 'mistral-small-latest' },
		deepseek: { base_url: 'https://api.deepseek.com/v1', model: 'deepseek-chat' },
		groq: { base_url: 'https://api.groq.com/openai/v1', model: 'llama-3.3-70b-versatile' },
		azure_openai: { base_url: null, model: 'gpt-4o-mini' },
		openrouter: { base_url: 'https://openrouter.ai/api/v1', model: 'openai/gpt-4o-mini' },
		together: { base_url: 'https://api.together.xyz/v1', model: 'meta-llama/Meta-Llama-3.1-8B-Instruct-Turbo' },
		huggingface: { base_url: 'https://api-inference.huggingface.co/v1', model: 'meta-llama/Llama-3.1-8B-Instruct' },
		perplexity: { base_url: 'https://api.perplexity.ai', model: 'llama-3.1-sonar-small-128k-online' },
		xai: { base_url: 'https://api.x.ai/v1', model: 'grok-2-mini' },
		ollama: { base_url: 'http://localhost:11434', model: 'llama3.2' },
		custom_openai_compatible: { base_url: null, model: '' }
	};

	function isOllamaType(): boolean {
		return form_provider_type === 'ollama';
	}

	// ── Initialize form ───────────────────────────────────────────────────
	let was_open = $state(false);

	$effect(() => {
		const just_opened = open && !was_open;
		if (just_opened) {
			if (mode === 'edit' && provider) {
				form_name = provider.name;
				form_provider_type = provider.provider_type;
				prev_provider_type = provider.provider_type;
				form_base_url = provider.base_url ?? '';
				form_api_key = '';
				form_models = provider.models.map((m) => ({ ...m }));
				form_temperature = provider.default_params?.temperature?.toString() ?? '';
				form_top_p = provider.default_params?.top_p?.toString() ?? '';
				form_max_tokens = provider.default_params?.max_tokens?.toString() ?? '';
			} else {
				form_name = '';
				form_provider_type = '';
				prev_provider_type = '';
				form_base_url = '';
				form_api_key = '';
				form_models = [];
				form_temperature = '';
				form_top_p = '';
				form_max_tokens = '';
			}
			// Clear all errors
			name_error = '';
			type_error = '';
			base_url_error = '';
			api_key_error = '';
			model_errors = {};
			temperature_error = '';
			top_p_error = '';
			max_tokens_error = '';
			backend_error = '';
			initial_snapshot = snapshot();
		}
		was_open = open;
	});

	// ── Provider type change handler ──────────────────────────────────────
	function handleTypeChange() {
		const defaults = type_defaults[form_provider_type];
		if (!defaults) {
			type_error = '';
			return;
		}

		// Check if base_url or models were modified from the previous type's defaults
		const prevDefaults = type_defaults[prev_provider_type];
		const base_url_modified = prevDefaults && form_base_url !== (prevDefaults.base_url ?? '');
		const models_modified =
			prevDefaults &&
			(form_models.length !== 1 ||
				form_models[0]?.name !== prevDefaults.model ||
				form_models[0]?.context_window_size !== null ||
				form_models[0]?.enabled !== true);

		if (prev_provider_type && (base_url_modified || models_modified)) {
			// Show confirmation before resetting
			type_change_confirm_open = true;
			return;
		}

		// Not dirty — proceed with reset
		applyTypeDefaults(defaults);
		prev_provider_type = form_provider_type;
		type_error = '';
	}

	function applyTypeDefaults(defaults: { base_url: string | null; model: string }) {
		form_base_url = defaults.base_url ?? '';
		form_models = [{ name: defaults.model, context_window_size: null, enabled: true }];
	}

	function handleTypeChangeConfirm() {
		type_change_confirm_open = false;
		const defaults = type_defaults[form_provider_type];
		if (defaults) {
			applyTypeDefaults(defaults);
		}
		prev_provider_type = form_provider_type;
		type_error = '';
	}

	function handleTypeChangeCancel() {
		type_change_confirm_open = false;
		// Revert to previous type
		form_provider_type = prev_provider_type;
	}

	// ── Dialog close handling ─────────────────────────────────────────────
	function handleOpenChange(value: boolean) {
		if (!value && is_dirty) {
			discard_open = true;
			return;
		}
		open = value;
		onopenchange?.(value);
	}

	function handleDiscardConfirm() {
		discard_open = false;
		open = false;
		onopenchange?.(false);
	}

	// ── Model list management ─────────────────────────────────────────────
	function addModel() {
		form_models = [...form_models, { name: '', context_window_size: null, enabled: true }];
	}

	function removeModel(index: number) {
		if (form_models.length <= 1) return;
		form_models = form_models.filter((_, i) => i !== index);
		// Clear error for removed index
		const newErrors: Record<number, string> = {};
		Object.entries(model_errors).forEach(([k, v]) => {
			const ki = Number(k);
			if (ki < index) newErrors[ki] = v;
			else if (ki > index) newErrors[ki - 1] = v;
		});
		model_errors = newErrors;
	}

	function updateModelName(index: number, value: string) {
		form_models = form_models.map((m, i) => (i === index ? { ...m, name: value } : m));
		if (model_errors[index]) {
			model_errors = { ...model_errors, [index]: '' };
		}
	}

	function updateModelContextWindow(index: number, value: number | null) {
		form_models = form_models.map((m, i) =>
			i === index ? { ...m, context_window_size: value } : m
		);
	}

	function updateModelEnabled(index: number, value: boolean) {
		form_models = form_models.map((m, i) => (i === index ? { ...m, enabled: value } : m));
	}

	// ── Validation ────────────────────────────────────────────────────────
	function validate(): boolean {
		let valid = true;

		// Name
		if (!form_name.trim()) {
			name_error = 'Name is required.';
			valid = false;
		} else {
			name_error = '';
		}

		// Provider type (create only)
		if (mode === 'create' && !form_provider_type) {
			type_error = 'Provider type is required.';
			valid = false;
		} else {
			type_error = '';
		}

		// Base URL required for custom/azure
		if (
			(form_provider_type === 'custom_openai_compatible' || form_provider_type === 'azure_openai') &&
			!form_base_url.trim()
		) {
			base_url_error = 'Base URL is required for this provider type.';
			valid = false;
		} else {
			base_url_error = '';
		}

		// API key required for non-Ollama in create mode when keyring available
		if (mode === 'create' && !isOllamaType() && keyringAvailable && !form_api_key.trim()) {
			api_key_error = 'API key is required.';
			valid = false;
		} else {
			api_key_error = '';
		}

		// Models
		const newModelErrors: Record<number, string> = {};
		form_models.forEach((m, i) => {
			if (!m.name.trim()) {
				newModelErrors[i] = 'Model name is required.';
				valid = false;
			} else if (m.context_window_size != null && (Number.isNaN(m.context_window_size) || m.context_window_size <= 0)) {
				newModelErrors[i] = 'Context size must be a positive number.';
				valid = false;
			}
		});
		model_errors = newModelErrors;

		// At least one model
		if (form_models.length === 0) {
			valid = false;
		}

		// Generation params
		if (form_temperature) {
			const val = Number(form_temperature);
			if (isNaN(val) || val < 0 || val > 2) {
				temperature_error = 'Temperature must be between 0 and 2.';
				valid = false;
			} else {
				temperature_error = '';
			}
		} else {
			temperature_error = '';
		}

		if (form_top_p) {
			const val = Number(form_top_p);
			if (isNaN(val) || val < 0 || val > 1) {
				top_p_error = 'Top P must be between 0 and 1.';
				valid = false;
			} else {
				top_p_error = '';
			}
		} else {
			top_p_error = '';
		}

		if (form_max_tokens) {
			const val = Number(form_max_tokens);
			if (isNaN(val) || val < 1 || !Number.isInteger(val)) {
				max_tokens_error = 'Max tokens must be a positive number.';
				valid = false;
			} else {
				max_tokens_error = '';
			}
		} else {
			max_tokens_error = '';
		}

		return valid;
	}

	// ── Save handler ──────────────────────────────────────────────────────
	async function handleSave() {
		if (is_saving) return;
		if (!validate()) return;

		is_saving = true;
		backend_error = '';

		try {
			const defaultParams = {
				temperature: form_temperature ? Number(form_temperature) : undefined,
				top_p: form_top_p ? Number(form_top_p) : undefined,
				max_tokens: form_max_tokens ? Number(form_max_tokens) : undefined
			};
			const hasParams = defaultParams.temperature !== undefined || defaultParams.top_p !== undefined || defaultParams.max_tokens !== undefined;

			if (mode === 'edit' && provider) {
				const updated: Provider = {
					...provider,
					name: form_name.trim(),
					base_url: form_base_url.trim() || undefined,
					model_name: form_models.find((m) => m.enabled)?.name ?? form_models[0]?.name ?? provider.model_name,
					models: form_models,
					default_params: hasParams ? defaultParams : undefined,
					updated_at: new Date().toISOString()
				};
				await app.updateProvider(updated, form_api_key || undefined);
			} else {
				await app.createProvider({
					name: form_name.trim(),
					provider_type: form_provider_type as ProviderType,
					base_url: form_base_url.trim() || undefined,
					model_name: form_models.find((m) => m.enabled)?.name ?? form_models[0]?.name ?? '',
					models: form_models,
					default_params: hasParams ? defaultParams : undefined,
					enabled: true,
					api_key: form_api_key || undefined
				});
			}
			open = false;
			onopenchange?.(false);
		} catch (err) {
			backend_error =
				err instanceof Error ? err.message : "Couldn't save provider. Something went wrong.";
		} finally {
			is_saving = false;
		}
	}
</script>

<Dialog
	{open}
	onopenchange={handleOpenChange}
	title={mode === 'edit' ? `Edit ${provider?.name ?? 'provider'}` : 'New provider'}
	description={mode === 'edit' ? 'Update provider configuration.' : 'Configure a new AI provider.'}
	class="max-w-lg"
>
	<div class="max-h-[60vh] space-y-5 overflow-y-auto pr-1">
		<!-- Error banner -->
		{#if backend_error}
			<div class="flex items-center gap-2 rounded-(--radius-md) border border-(--color-destructive)/30 bg-(--color-destructive)/10 px-3 py-2 text-sm text-(--color-destructive)">
				<AlertCircle class="size-4 shrink-0" />
				{backend_error}
			</div>
		{/if}

		<!-- ── Identity section ──────────────────────────────────────────── -->
		<div class="space-y-3">
			<!-- Name -->
			<div class="space-y-1">
				<label for="provider-name" class="text-xs text-(--color-subtle)">Name</label>
				<Input
					id="provider-name"
					bind:value={form_name}
					placeholder="e.g. My OpenAI"
					disabled={is_saving}
					class={name_error ? 'border-(--color-destructive)' : ''}
				/>
				{#if name_error}
					<p class="text-xs text-(--color-destructive)">{name_error}</p>
				{/if}
			</div>

			<!-- Provider Type -->
			<div class="space-y-1">
				<label for="provider-type" class="text-xs text-(--color-subtle)">Type</label>
				<Select
					id="provider-type"
					bind:value={form_provider_type}
					onchange={handleTypeChange}
					disabled={is_saving || mode === 'edit'}
					class={type_error ? 'border-(--color-destructive)' : ''}
				>
					<option value="" disabled>Select provider type…</option>
					<optgroup label="Cloud">
						<option value="openai">OpenAI</option>
						<option value="anthropic">Anthropic</option>
						<option value="gemini">Gemini</option>
						<option value="cohere">Cohere</option>
						<option value="mistral">Mistral</option>
						<option value="deepseek">DeepSeek</option>
						<option value="groq">Groq</option>
						<option value="azure_openai">Azure OpenAI</option>
						<option value="huggingface">HuggingFace</option>
						<option value="perplexity">Perplexity</option>
						<option value="xai">xAI</option>
					</optgroup>
					<optgroup label="Local">
						<option value="ollama">Ollama</option>
					</optgroup>
					<optgroup label="Gateway">
						<option value="openrouter">OpenRouter</option>
						<option value="together">Together</option>
					</optgroup>
					<optgroup label="Custom">
						<option value="custom_openai_compatible">OpenAI-compatible</option>
					</optgroup>
				</Select>
				{#if type_error}
					<p class="text-xs text-(--color-destructive)">{type_error}</p>
				{/if}
			</div>

			<!-- Base URL -->
			<div class="space-y-1">
				<label for="provider-base-url" class="text-xs text-(--color-subtle)">Base URL</label>
				<Input
					id="provider-base-url"
					bind:value={form_base_url}
					placeholder="https://api.example.com/v1"
					disabled={is_saving}
					class={base_url_error ? 'border-(--color-destructive)' : ''}
				/>
				{#if base_url_error}
					<p class="text-xs text-(--color-destructive)">{base_url_error}</p>
				{/if}
			</div>
		</div>

		<!-- ── API Key section ────────────────────────────────────────────── -->
		{#if !isOllamaType()}
			<div class="space-y-1">
				<label for="provider-api-key" class="text-xs text-(--color-subtle)">API Key</label>
				<div class="flex items-center gap-2">
					<Input
						id="provider-api-key"
						type={form_api_key_visible ? 'text' : 'password'}
						bind:value={form_api_key}
						placeholder={mode === 'edit' && provider?.has_api_key ? 'Key set •••••••' : keyringAvailable ? 'Enter your API key…' : 'Keychain unavailable'}
						disabled={is_saving || !keyringAvailable}
						class={cn(
							'flex-1',
							api_key_error ? 'border-(--color-destructive)' : ''
						)}
					/>
					<button
						type="button"
						onclick={() => (form_api_key_visible = !form_api_key_visible)}
						class="inline-flex size-10 items-center justify-center rounded-(--radius-md) border border-(--color-border) text-(--color-muted) transition-colors hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
						aria-label={form_api_key_visible ? 'Hide API key' : 'Show API key'}
					>
						{#if form_api_key_visible}
							<EyeOff class="size-4" />
						{:else}
							<Eye class="size-4" />
						{/if}
					</button>
				</div>
				{#if api_key_error}
					<p class="text-xs text-(--color-destructive)">{api_key_error}</p>
				{/if}
				{#if !keyringAvailable}
					<p class="text-xs text-(--color-subtle)">API keys cannot be stored without an OS keychain. You can still configure Ollama (local) providers.</p>
				{/if}
			</div>
		{/if}

		<!-- ── Models section ─────────────────────────────────────────────── -->
		<div class="space-y-2">
			<span class="text-xs font-semibold text-(--color-subtle)">Models</span>
			<div class="flex flex-col gap-2">
				{#each form_models as model, i (i)}
					<div class="relative">
						<ModelEntryRow
							name={model.name}
							contextWindowSize={model.context_window_size}
							enabled={model.enabled}
							removable={form_models.length > 1}
							disabled={is_saving}
							onremove={() => removeModel(i)}
							onupdatename={(v) => updateModelName(i, v)}
							onupdatecontextwindow={(v) => updateModelContextWindow(i, v)}
							onupdateenabled={(v) => updateModelEnabled(i, v)}
						/>
						{#if model_errors[i]}
							<p class="mt-0.5 text-xs text-(--color-destructive)">{model_errors[i]}</p>
						{/if}
					</div>
				{/each}
			</div>
			{#if form_models.length > 0}
				<Button variant="secondary" size="sm" onclick={addModel} disabled={is_saving}>
					<Plus class="size-3.5" />
					Add model
				</Button>
			{:else}
				<div class="rounded-(--radius-md) border border-dashed border-(--color-border) px-4 py-5 text-center">
					<p class="text-sm font-medium text-(--color-foreground)">No models added.</p>
					<p class="mt-1 text-xs text-(--color-muted)">Add at least one model that this provider supports. The default model is pre-filled.</p>
					<Button variant="secondary" size="sm" class="mt-3" onclick={addModel} disabled={is_saving}>
						<Plus class="size-3.5" />
						Add model
					</Button>
				</div>
			{/if}
		</div>

		<!-- ── Generation Parameters section ──────────────────────────────── -->
		<div class="space-y-2">
			<span class="text-xs font-semibold text-(--color-subtle)">Generation parameters</span>
			<div class="grid grid-cols-3 gap-x-3 gap-y-4">
				<div>
					<label for="provider-temperature" class="mb-1.5 block text-xs text-(--color-subtle)">Temperature</label>
					<Input
id="provider-temperature"
							type="number"
							step="0.1"
							min="0"
							max="2"
							bind:value={form_temperature}
						placeholder="—"
						disabled={is_saving}
						class={temperature_error ? 'border-(--color-destructive)' : ''}
					/>
					{#if temperature_error}
						<p class="mt-0.5 text-xs text-(--color-destructive)">{temperature_error}</p>
					{/if}
				</div>
				<div>
					<label for="provider-top-p" class="mb-1.5 block text-xs text-(--color-subtle)">Top P</label>
					<Input
id="provider-top-p"
							type="number"
							step="0.05"
							min="0"
							max="1"
							bind:value={form_top_p}
						placeholder="—"
						disabled={is_saving}
						class={top_p_error ? 'border-(--color-destructive)' : ''}
					/>
					{#if top_p_error}
						<p class="mt-0.5 text-xs text-(--color-destructive)">{top_p_error}</p>
					{/if}
				</div>
				<div>
					<label for="provider-max-tokens" class="mb-1.5 block text-xs text-(--color-subtle)">Max tokens</label>
					<Input
id="provider-max-tokens"
							type="number"
							step="1"
							min="1"
							bind:value={form_max_tokens}
						placeholder="—"
						disabled={is_saving}
						class={max_tokens_error ? 'border-(--color-destructive)' : ''}
					/>
					{#if max_tokens_error}
						<p class="mt-0.5 text-xs text-(--color-destructive)">{max_tokens_error}</p>
					{/if}
				</div>
			</div>
		</div>
	</div>

	{#snippet footer()}
		<div class="flex items-center gap-2">
			<Button variant="secondary" size="sm" onclick={() => handleOpenChange(false)} disabled={is_saving}>
				Cancel
			</Button>
			<Button variant="primary" size="sm" onclick={handleSave} disabled={is_saving}>
				{#if is_saving}
					<Loader2 class="size-4 animate-spin" />
					Saving…
				{:else}
					{mode === 'edit' ? 'Save changes' : 'Save provider'}
				{/if}
			</Button>
		</div>
	{/snippet}
</Dialog>

<DeleteConfirmationDialog
	bind:open={discard_open}
	title="Discard changes?"
	description="You have unsaved changes. Closing this dialog will discard them."
	confirm_label="Discard"
	cancel_label="Keep editing"
	onconfirm={handleDiscardConfirm}
/>

<DeleteConfirmationDialog
	bind:open={type_change_confirm_open}
	title="Change provider type?"
	description="Changing the provider type will reset the base URL and models to the new type's defaults. Continue?"
	confirm_label="Change type"
	cancel_label="Keep current"
	onconfirm={handleTypeChangeConfirm}
	oncancel={handleTypeChangeCancel}
/>

<script lang="ts">
	import type { GenerationPreset } from '$lib/types';
	import { app } from '$lib/stores/app.svelte';
	import { Dialog, Button, Input } from '$lib/components/ui';
	import { Loader2, AlertCircle } from '@lucide/svelte';

	interface Props {
		open?: boolean;
		onopenchange?: (open: boolean) => void;
		mode?: 'create' | 'edit' | 'view';
		preset?: GenerationPreset;
	}

	let {
		open = $bindable(false),
		onopenchange,
		mode = 'create',
		preset,
	}: Props = $props();

	let form_name = $state('');
	let form_temperature = $state('');
	let form_top_p = $state('');
	let form_max_tokens = $state('');
	let form_frequency_penalty = $state('');
	let form_presence_penalty = $state('');
	let form_stop_sequences = $state('');
	let is_saving = $state(false);
	let backend_error = $state('');

	let is_readonly = $derived(mode === 'view');

	// ── Validation errors ──────────────────────────────────────────────────
	let name_error = $state('');
	let temperature_error = $state('');
	let top_p_error = $state('');
	let max_tokens_error = $state('');
	let freq_penalty_error = $state('');
	let pres_penalty_error = $state('');

	// ── Initialize form ───────────────────────────────────────────────────
	let was_open = $state(false);

	$effect(() => {
		const just_opened = open && !was_open;
		if (just_opened) {
			if ((mode === 'edit' || mode === 'view') && preset) {
				form_name = preset.name;
				form_temperature = preset.temperature.toString();
				form_top_p = preset.top_p.toString();
				form_max_tokens = preset.max_tokens.toString();
				form_frequency_penalty = preset.frequency_penalty.toString();
				form_presence_penalty = preset.presence_penalty.toString();
				form_stop_sequences = preset.stop_sequences.join(', ');
			} else {
				form_name = '';
				form_temperature = '0.8';
				form_top_p = '0.9';
				form_max_tokens = '0';
				form_frequency_penalty = '0';
				form_presence_penalty = '0';
				form_stop_sequences = '';
			}
			// Clear all errors
			name_error = '';
			temperature_error = '';
			top_p_error = '';
			max_tokens_error = '';
			freq_penalty_error = '';
			pres_penalty_error = '';
			backend_error = '';
		}
		was_open = open;
	});

	// ── Validation ────────────────────────────────────────────────────────
	function validate(): boolean {
		let valid = true;

		if (!form_name.trim()) {
			name_error = 'Name is required';
			valid = false;
		} else {
			name_error = '';
		}

		const temp = parseFloat(form_temperature);
		if (isNaN(temp) || temp < 0 || temp > 5) {
			temperature_error = 'Must be 0–5';
			valid = false;
		} else {
			temperature_error = '';
		}

		const topP = parseFloat(form_top_p);
		if (isNaN(topP) || topP < 0 || topP > 1) {
			top_p_error = 'Must be 0–1';
			valid = false;
		} else {
			top_p_error = '';
		}

		const maxTok = parseInt(form_max_tokens, 10);
		if (isNaN(maxTok) || maxTok < 0) {
			max_tokens_error = 'Must be a non-negative integer';
			valid = false;
		} else {
			max_tokens_error = '';
		}

		const freqPen = parseFloat(form_frequency_penalty);
		if (isNaN(freqPen) || freqPen < 0 || freqPen > 2) {
			freq_penalty_error = 'Must be 0–2';
			valid = false;
		} else {
			freq_penalty_error = '';
		}

		const presPen = parseFloat(form_presence_penalty);
		if (isNaN(presPen) || presPen < 0 || presPen > 2) {
			pres_penalty_error = 'Must be 0–2';
			valid = false;
		} else {
			pres_penalty_error = '';
		}

		return valid;
	}

	// ── Save ───────────────────────────────────────────────────────────────
	async function handleSave() {
		if (!validate()) return;

		is_saving = true;
		backend_error = '';

		const stops = form_stop_sequences
			.split(',')
			.map((s) => s.trim())
			.filter(Boolean);

		const presetData: GenerationPreset = {
			id: mode === 'edit' && preset ? preset.id : crypto.randomUUID(),
			name: form_name.trim(),
			is_builtin: false,
			temperature: parseFloat(form_temperature),
			top_p: parseFloat(form_top_p),
			max_tokens: parseInt(form_max_tokens, 10),
			frequency_penalty: parseFloat(form_frequency_penalty),
			presence_penalty: parseFloat(form_presence_penalty),
			stop_sequences: stops,
			created_at: mode === 'edit' && preset ? preset.created_at : new Date().toISOString(),
			updated_at: new Date().toISOString(),
		};

		try {
			await app.savePreset(presetData);
			open = false;
		} catch (err: any) {
			backend_error = err?.toString() ?? 'Failed to save preset';
		} finally {
			is_saving = false;
		}
	}

	function handleDuplicate() {
		// Switch to create mode with current values
		mode = 'create';
		preset = undefined;
		form_name = form_name ? `${form_name} (copy)` : '';
		is_readonly = false;
	}
</script>

<Dialog
	{open}
	{onopenchange}
	title={mode === 'view' ? 'Preset Details' : mode === 'edit' ? 'Edit Preset' : 'Create Preset'}
	description={mode === 'view'
		? 'Viewing built-in preset parameters.'
		: mode === 'edit'
			? 'Modify generation parameters for this preset.'
			: 'Define generation parameters for a new preset.'}
>
	{#snippet children()}
		<div class="flex flex-col gap-4">
			<!-- Name -->
			<div>
				<label for="preset-name" class="mb-1 block text-xs font-medium text-(--color-muted)">Name</label>
				<Input
					id="preset-name"
					placeholder="My Preset"
					value={form_name}
					oninput={(e) => (form_name = (e.target as HTMLInputElement).value)}
					disabled={is_readonly}
					class={name_error ? 'border-(--color-destructive)' : ''}
				/>
				{#if name_error}
					<p class="mt-1 text-xs text-(--color-destructive)">{name_error}</p>
				{/if}
			</div>

			<!-- Temperature -->
			<div>
				<label for="preset-temp" class="mb-1 block text-xs font-medium text-(--color-muted)">
					Temperature <span class="text-(--color-subtle)">(0–5)</span>
				</label>
				<Input
					id="preset-temp"
					type="number"
					step="0.01"
					min="0"
					max="5"
					value={form_temperature}
					oninput={(e) => (form_temperature = (e.target as HTMLInputElement).value)}
					disabled={is_readonly}
					class={temperature_error ? 'border-(--color-destructive)' : ''}
				/>
				{#if temperature_error}
					<p class="mt-1 text-xs text-(--color-destructive)">{temperature_error}</p>
				{/if}
			</div>

			<!-- Top P -->
			<div>
				<label for="preset-topp" class="mb-1 block text-xs font-medium text-(--color-muted)">
					Top P <span class="text-(--color-subtle)">(0–1)</span>
				</label>
				<Input
					id="preset-topp"
					type="number"
					step="0.01"
					min="0"
					max="1"
					value={form_top_p}
					oninput={(e) => (form_top_p = (e.target as HTMLInputElement).value)}
					disabled={is_readonly}
					class={top_p_error ? 'border-(--color-destructive)' : ''}
				/>
				{#if top_p_error}
					<p class="mt-1 text-xs text-(--color-destructive)">{top_p_error}</p>
				{/if}
			</div>

			<!-- Response Length -->
			<div>
				<label for="preset-maxtok" class="mb-1 block text-xs font-medium text-(--color-muted)">
					Response Length <span class="text-(--color-subtle)">(0 = default)</span>
				</label>
				<Input
					id="preset-maxtok"
					type="number"
					step="1"
					min="0"
					value={form_max_tokens}
					oninput={(e) => (form_max_tokens = (e.target as HTMLInputElement).value)}
					disabled={is_readonly}
					class={max_tokens_error ? 'border-(--color-destructive)' : ''}
				/>
				{#if max_tokens_error}
					<p class="mt-1 text-xs text-(--color-destructive)">{max_tokens_error}</p>
				{/if}
			</div>

			<!-- Frequency & Presence Penalty -->
			<div class="grid grid-cols-2 gap-3">
				<div>
					<label for="preset-freqpen" class="mb-1 block text-xs font-medium text-(--color-muted)">
						Freq Penalty <span class="text-(--color-subtle)">(0–2)</span>
					</label>
					<Input
						id="preset-freqpen"
						type="number"
						step="0.01"
						min="0"
						max="2"
						value={form_frequency_penalty}
						oninput={(e) => (form_frequency_penalty = (e.target as HTMLInputElement).value)}
						disabled={is_readonly}
						class={freq_penalty_error ? 'border-(--color-destructive)' : ''}
					/>
					{#if freq_penalty_error}
						<p class="mt-1 text-xs text-(--color-destructive)">{freq_penalty_error}</p>
					{/if}
				</div>
				<div>
					<label for="preset-prespen" class="mb-1 block text-xs font-medium text-(--color-muted)">
						Pres Penalty <span class="text-(--color-subtle)">(0–2)</span>
					</label>
					<Input
						id="preset-prespen"
						type="number"
						step="0.01"
						min="0"
						max="2"
						value={form_presence_penalty}
						oninput={(e) => (form_presence_penalty = (e.target as HTMLInputElement).value)}
						disabled={is_readonly}
						class={pres_penalty_error ? 'border-(--color-destructive)' : ''}
					/>
					{#if pres_penalty_error}
						<p class="mt-1 text-xs text-(--color-destructive)">{pres_penalty_error}</p>
					{/if}
				</div>
			</div>

			<!-- Stop Sequences -->
			<div>
				<label for="preset-stops" class="mb-1 block text-xs font-medium text-(--color-muted)">
					Stop Sequences <span class="text-(--color-subtle)">(comma-separated, optional)</span>
				</label>
				<Input
					id="preset-stops"
					placeholder="e.g. \\n, END"
					value={form_stop_sequences}
					oninput={(e) => (form_stop_sequences = (e.target as HTMLInputElement).value)}
					disabled={is_readonly}
				/>
			</div>

			<!-- Backend error -->
			{#if backend_error}
				<div class="flex items-start gap-2 rounded-(--radius-md) border border-(--color-destructive)/30 bg-(--color-destructive)/5 px-3 py-2 text-sm text-(--color-destructive)">
					<AlertCircle class="mt-0.5 size-4 shrink-0" />
					{backend_error}
				</div>
			{/if}
		</div>
	{/snippet}

	{#snippet footer()}
		<div class="flex items-center justify-end gap-2">
			{#if is_readonly}
				<Button variant="secondary" onclick={handleDuplicate}>Duplicate as custom</Button>
				<Button variant="ghost" onclick={() => (open = false)}>Close</Button>
			{:else}
				<Button variant="ghost" onclick={() => (open = false)} disabled={is_saving}>Cancel</Button>
				<Button variant="primary" onclick={handleSave} disabled={is_saving}>
					{#if is_saving}
						<Loader2 class="size-4 animate-spin" />
					{/if}
					{mode === 'edit' ? 'Save Changes' : 'Create Preset'}
				</Button>
			{/if}
		</div>
	{/snippet}
</Dialog>

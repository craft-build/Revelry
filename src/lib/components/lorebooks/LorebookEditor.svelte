<script lang="ts">
	import type {
		Lorebook,
		LorebookEntry,
		LorebookPosition,
		LorebookActivation,
		LorebookActivationMode,
		KeywordTestResult,
		SemanticTestResult
	} from '$lib/types';
	import { app } from '$lib/stores/app.svelte';
	import { cn } from '$lib/utils';
	import {
		getLorebookEntries as apiGetLorebookEntries,
		saveLorebookEntry as apiSaveLorebookEntry,
		deleteLorebookEntry as apiDeleteLorebookEntry,
		testLorebookKeywords as apiTestLorebookKeywords,
		testSemanticActivation as apiTestSemanticActivation
	} from '$lib/data/api';
	import {
		Dialog,
		Button,
		Input,
		Textarea,
		Select
	} from '$lib/components/ui';
	import {
		AlertCircle,
		Plus,
		Trash2,
		Play,
		ChevronDown,
		ChevronRight,
		Check,
		X as XIcon
	} from '@lucide/svelte';

	interface Props {
		open?: boolean;
		onopenchange?: (open: boolean) => void;
		/** 'create' for new lorebooks, 'edit' for existing */
		mode?: 'create' | 'edit';
		/** Lorebook to edit (required for edit mode) */
		lorebook?: Lorebook;
	}

	let {
		open = $bindable(false),
		onopenchange,
		mode = 'create',
		lorebook
	}: Props = $props();

	// ── Lorebook-level form state ────────────────────────────────────────
	let form_name = $state('');
	let form_description = $state('');
	let name_error = $state('');
	let backend_error = $state('');
	let is_saving = $state(false);

	// ── Entry state ──────────────────────────────────────────────────────
	let entries = $state<LorebookEntry[]>([]);
	let selected_entry_id = $state<string | null>(null);
	let entries_loading = $state(false);

	// ── Entry form state ─────────────────────────────────────────────────
	let entry_keys = $state('');
	let entry_secondary_keys = $state('');
	let entry_content = $state('');
	let entry_position = $state<LorebookPosition>('before_char');
	let entry_depth_str = $state('4');
	let entry_activation = $state<LorebookActivation>('always');
	let entry_activation_mode = $state<LorebookActivationMode>('keyword');
	let entry_semantic_threshold = $state(0.6);
	let entry_enabled = $state(true);
	let entry_sort_order = $state(0);
	let entry_token_budget = $state<number | null>(null);
	let entry_keys_error = $state('');
	let entry_content_error = $state('');
	let is_saving_entry = $state(false);

	// ── Keyword test state ───────────────────────────────────────────────
	let test_panel_open = $state(false);
	let test_sample_text = $state('');
	let test_loading = $state(false);
	let test_results = $state<KeywordTestResult[]>([]);
	let test_error = $state('');
	// 'keyword' | 'semantic' — semantic requires embeddings to be configured
	let test_kind = $state<'keyword' | 'semantic'>('keyword');
	let semantic_loading = $state(false);
	let semantic_results = $state<SemanticTestResult[]>([]);

	// ── Derived ────────────────────────────────────────────────────────────
	let selected_entry = $derived(entries.find((e) => e.id === selected_entry_id) ?? null);

	function handleOpenChange(value: boolean) {
		open = value;
		onopenchange?.(value);
	}

	// ── Load entries when dialog opens ────────────────────────────────────
	// Only re-initialize when the dialog *opens* (open transitions to true),
	// not when the lorebook prop mutates mid-dialog.
	let was_open = $state(false);

	$effect(() => {
		const just_opened = open && !was_open;
		if (just_opened) {
			form_name = '';
			form_description = '';
			name_error = '';
			backend_error = '';
			entries = [];
			selected_entry_id = null;
			entry_keys = '';
			entry_secondary_keys = '';
			entry_content = '';
			entry_position = 'before_char';
			entry_depth_str = '4';
			entry_activation = 'always';
			entry_activation_mode = 'keyword';
			entry_semantic_threshold = 0.6;
			entry_keys_error = '';
			entry_content_error = '';
			test_results = [];
			semantic_results = [];
			test_error = '';
			test_sample_text = '';
			test_panel_open = false;

			if (mode === 'edit' && lorebook) {
				form_name = lorebook.name;
				form_description = lorebook.description;
				loadEntries(lorebook.id);
			}
		}
		was_open = open;
	});

	async function loadEntries(lorebookId: string) {
		entries_loading = true;
		try {
			const loaded = await apiGetLorebookEntries(lorebookId);
			entries = loaded;
			// Auto-select first entry if available
			if (loaded.length > 0 && !selected_entry_id) {
				selectEntry(loaded[0].id);
			}
		} catch (err) {
			console.error('Failed to load lorebook entries:', err);
		} finally {
			entries_loading = false;
		}
	}

	function selectEntry(id: string) {
		selected_entry_id = id;
		const entry = entries.find((e) => e.id === id);
		if (entry) {
			entry_keys = entry.keys.join(', ');
			entry_secondary_keys = (entry.secondary_keys ?? []).join(', ');
			entry_content = entry.content;
			entry_position = entry.position;
			entry_depth_str = String(entry.depth);
			entry_activation = entry.activation_state;
			entry_activation_mode = entry.activation_mode ?? 'keyword';
			entry_semantic_threshold = entry.semantic_threshold ?? 0.6;
			entry_enabled = entry.enabled;
			entry_sort_order = entry.sort_order;
			entry_token_budget = entry.token_budget ?? null;
			entry_keys_error = '';
			entry_content_error = '';
		}
	}

	// ── Lorebook save (internal — returns ID, does NOT close dialog) ────
	async function saveLorebookGetId(): Promise<string | null> {
		if (!form_name.trim()) {
			name_error = 'Name is required.';
			return null;
		}
		name_error = '';

		try {
			let saved: Lorebook;
			if (mode === 'edit' && lorebook) {
				saved = await app.updateLorebook({
					...lorebook,
					name: form_name.trim(),
					description: form_description.trim()
				});
			} else {
				saved = await app.createLorebook(form_name.trim());
				if (form_description.trim()) {
					saved = await app.updateLorebook({
						...saved,
						description: form_description.trim()
					});
				}
			}
			return saved.id;
		} catch (err) {
			backend_error =
				err instanceof Error ? err.message : 'Failed to save lorebook.';
			return null;
		}
	}

	// ── Lorebook save (from Save button — closes dialog on success) ──────
	async function handleSaveLorebook() {
		is_saving = true;
		backend_error = '';

		try {
			const id = await saveLorebookGetId();

			if (id !== null) {
				open = false;
				onopenchange?.(false);
			}
		} finally {
			is_saving = false;
		}
	}

	// ── Entry save ────────────────────────────────────────────────────────
	async function handleSaveEntry() {
		if (is_saving_entry) return;

		// Validate — keywords are only mandatory for pure keyword matching.
		const keys = entry_keys
			.split(',')
			.map((k) => k.trim())
			.filter((k) => k.length > 0);
		const needs_keys =
			entry_activation_mode === 'keyword' && entry_activation !== 'always';
		if (keys.length === 0 && needs_keys) {
			entry_keys_error = 'At least one keyword is required.';
			return;
		}
		if (!entry_content.trim()) {
			entry_content_error = 'Content is required.';
			return;
		}
		entry_keys_error = '';
		entry_content_error = '';

		is_saving_entry = true;

		try {
			// Ensure lorebook exists first
			let lorebookId: string;
			if (mode === 'edit' && lorebook) {
				lorebookId = lorebook.id;
			} else {
				// Create lorebook on-the-fly if needed
				const id = await saveLorebookGetId();
				if (!id) return;
				lorebookId = id;
				// Switch to edit mode now that the lorebook exists
				mode = 'edit';
				lorebook = app.lorebooks.find((l) => l.id === id);
			}

			const secondaryKeys = entry_secondary_keys
				.split(',')
				.map((k) => k.trim())
				.filter((k) => k.length > 0);

			const entry: LorebookEntry = {
				id: selected_entry_id ?? crypto.randomUUID(),
				lorebook_id: lorebookId,
				keys,
				secondary_keys: secondaryKeys.length > 0 ? secondaryKeys : undefined,
				content: entry_content.trim(),
				position: entry_position,
				depth: parseInt(entry_depth_str, 10) || 4,
				activation_state: entry_activation,
				activation_mode: entry_activation_mode,
				semantic_threshold: entry_semantic_threshold,
				sort_order: entry_sort_order,
				enabled: entry_enabled,
				token_budget: entry_token_budget,
				created_at: selected_entry?.created_at ?? new Date().toISOString(),
				updated_at: new Date().toISOString()
			};

			const saved = await apiSaveLorebookEntry(entry);

			// Update local entries list
			const existingIdx = entries.findIndex((e) => e.id === saved.id);
			if (existingIdx >= 0) {
				entries[existingIdx] = saved;
			} else {
				entries = [...entries, saved];
			}
			selected_entry_id = saved.id;
		} catch (err) {
			backend_error =
				err instanceof Error ? err.message : 'Failed to save entry.';
		} finally {
			is_saving_entry = false;
		}
	}

	// ── Add entry ─────────────────────────────────────────────────────────
	async function handleAddEntry() {
		// Ensure lorebook exists first
		let lorebookId: string;
		if (mode === 'edit' && lorebook) {
			lorebookId = lorebook.id;
		} else {
			const id = await saveLorebookGetId();
			if (!id) return;
			lorebookId = id;
			mode = 'edit';
			lorebook = app.lorebooks.find((l) => l.id === id);
		}

		// Create a new empty entry
		const newEntry: LorebookEntry = {
			id: crypto.randomUUID(),
			lorebook_id: lorebookId,
			keys: [],
			secondary_keys: undefined,
			content: '',
			position: 'before_char',
			depth: 4,
			activation_state: 'always',
			activation_mode: 'keyword',
			semantic_threshold: 0.6,
			sort_order: 0,
			enabled: true,
			created_at: new Date().toISOString(),
			updated_at: new Date().toISOString()
		};

		// Save to backend
		try {
			const saved = await apiSaveLorebookEntry(newEntry);
			entries = [...entries, saved];
			selectEntry(saved.id);
		} catch (err) {
			backend_error =
				err instanceof Error ? err.message : 'Failed to create entry.';
		}
	}

	// ── Delete entry ─────────────────────────────────────────────────────
	async function handleDeleteEntry(id: string) {
		try {
			await apiDeleteLorebookEntry(id);
			entries = entries.filter((e) => e.id !== id);
			if (selected_entry_id === id) {
				selected_entry_id = entries.length > 0 ? entries[0].id : null;
				if (selected_entry_id) {
					selectEntry(selected_entry_id);
				} else {
					resetEntryForm();
				}
			}
		} catch (err) {
			backend_error =
				err instanceof Error ? err.message : 'Failed to delete entry.';
		}
	}

	function resetEntryForm() {
		entry_keys = '';
		entry_secondary_keys = '';
		entry_content = '';
		entry_position = 'before_char';
		entry_depth_str = '4';
		entry_activation = 'always';
		entry_activation_mode = 'keyword';
		entry_semantic_threshold = 0.6;
		entry_enabled = true;
		entry_sort_order = 0;
		entry_token_budget = null;
		entry_keys_error = '';
		entry_content_error = '';
	}

	// ── Keyword testing ──────────────────────────────────────────────────
	async function handleRunTest() {
		if (test_kind === 'semantic') {
			await handleRunSemanticTest();
			return;
		}
		if (is_saving || test_loading) return;
		if (entries.length === 0) {
			test_error = 'No entries to test.';
			return;
		}
		if (!test_sample_text.trim()) {
			test_error = 'Enter sample text to test against.';
			return;
		}
		test_error = '';
		test_loading = true;
		test_results = [];

		try {
			// Test each entry individually
			const results: KeywordTestResult[] = [];
			for (const entry of entries) {
				// `always` entries activate unconditionally (no keys needed); skip
				// only keyword-gated entries that have no primary keys to test.
				if (entry.activation_state !== 'always' && entry.keys.length === 0)
					continue;
				const result = await apiTestLorebookKeywords(
					entry.keys,
					entry.secondary_keys ?? [],
					test_sample_text.trim(),
					entry.activation_state
				);
				results.push(result);
			}
			test_results = results;
		} catch (err) {
			test_error =
				err instanceof Error ? err.message : 'Keyword test failed.';
		} finally {
			test_loading = false;
		}
	}

	// ── Semantic testing ─────────────────────────────────────────────────
	async function handleRunSemanticTest() {
		if (semantic_loading) return;
		if (!lorebook) {
			test_error = 'Save the lorebook first — the semantic test reads saved entries.';
			return;
		}
		if (!test_sample_text.trim()) {
			test_error = 'Enter sample text to test against.';
			return;
		}
		test_error = '';
		semantic_loading = true;
		semantic_results = [];

		try {
			semantic_results = await apiTestSemanticActivation(
				lorebook.id,
				test_sample_text.trim()
			);
		} catch (err) {
			test_error =
				err instanceof Error ? err.message : 'Semantic test failed.';
		} finally {
			semantic_loading = false;
		}
	}

	function format_similarity(value: number | null | undefined): string {
		if (value == null) return '—';
		return value.toFixed(2);
	}

	// ── Helpers ──────────────────────────────────────────────────────────
	function truncate(str: string, max: number): string {
		if (!str) return '';
		return str.length > max ? str.slice(0, max) + '…' : str;
	}

	function entryHasMatch(entryId: string): boolean {
		return test_results.some((r) => r.entry_id === entryId && r.matched_keys.length > 0);
	}

	function entryMatchedKeys(entryId: string): string[] {
		const result = test_results.find((r) => r.entry_id === entryId);
		return result?.matched_keys ?? [];
	}
</script>

<Dialog
	{open}
	onopenchange={handleOpenChange}
	title={mode === 'edit' ? 'Edit lorebook' : 'New lorebook'}
	description={mode === 'edit' ? 'Manage entries and test keyword activations.' : 'Create a new lorebook with entries.'}
	class="max-w-3xl"
>
	<div class="max-h-[65vh] space-y-4 overflow-y-auto pr-1">
		<!-- Error banner -->
		{#if backend_error}
			<div class="flex items-center gap-2 rounded-(--radius-md) border border-(--color-destructive)/30 bg-(--color-destructive)/10 px-3 py-2 text-sm text-(--color-destructive)">
				<AlertCircle class="size-4 shrink-0" />
				{backend_error}
			</div>
		{/if}

		<!-- Lorebook name + description -->
		<div class="space-y-3">
			<div class="space-y-1">
				<label for="lorebook-name" class="text-sm font-medium text-(--color-foreground)">Name *</label>
				<Input
					id="lorebook-name"
					bind:value={form_name}
					placeholder="Lorebook name"
					disabled={is_saving}
					class={name_error ? 'border-(--color-destructive)' : ''}
				/>
				{#if name_error}
					<p class="text-xs text-(--color-destructive)">{name_error}</p>
				{/if}
			</div>
			<div class="space-y-1">
				<label for="lorebook-description" class="text-sm font-medium text-(--color-foreground)">Description</label>
				<Input
					id="lorebook-description"
					bind:value={form_description}
					placeholder="Short description of this lorebook"
					disabled={is_saving}
				/>
			</div>
		</div>

		<!-- Two-column: entry list + entry form -->
		<div class="flex gap-4 border-t border-(--color-border) pt-4">
			<!-- Left column: Entry list -->
			<div class="flex w-2/5 flex-col gap-2">
				<div class="flex items-center justify-between">
					<h3 class="text-sm font-medium text-(--color-foreground)">Entries</h3>
					<Button variant="ghost" size="icon-sm" onclick={handleAddEntry} disabled={is_saving || entries_loading}>
						<Plus class="size-4" />
					</Button>
				</div>

				<div class="flex flex-col gap-1 overflow-y-auto" style="max-height: 320px;">
					{#if entries_loading}
						<p class="py-4 text-center text-sm text-(--color-subtle)">Loading…</p>
					{:else if entries.length === 0}
						<p class="py-4 text-center text-sm text-(--color-subtle)">No entries yet.</p>
					{:else}
						{#each entries as entry (entry.id)}
							{@const has_match = entryHasMatch(entry.id)}
							{@const matched = entryMatchedKeys(entry.id)}
							<div
								role="button"
								tabindex="0"
								onclick={() => selectEntry(entry.id)}
								onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') selectEntry(entry.id); }}
								class="group/entry relative flex items-start gap-2 rounded-(--radius-md) border px-3 py-2 text-left transition-colors cursor-pointer {selected_entry_id === entry.id ? 'border-(--color-accent) bg-(--color-accent)/5' : 'border-(--color-border) hover:border-(--color-border-strong)'} {test_results.length > 0 && !has_match ? 'opacity-40' : ''} {!entry.enabled ? 'opacity-50' : ''}"
							>
								<div class="min-w-0 flex-1">
									<p class="truncate text-sm font-medium text-(--color-foreground)">
										{entry.keys.length > 0 ? truncate(entry.keys[0], 18) : 'Empty entry'}
										{#if !entry.enabled}
											<span class="ml-1.5 text-[10px] font-normal text-(--color-subtle)">Disabled</span>
										{/if}
									</p>
									<p class="truncate text-xs text-(--color-muted)">
										{truncate(entry.content, 60)}
									</p>
									{#if test_results.length > 0 && has_match}
										<div class="mt-1 flex flex-wrap gap-1">
											{#each matched as key (key)}
												<span class="inline-flex items-center rounded-(--radius-sm) bg-(--color-accent)/15 px-1.5 py-0.5 text-[10px] font-medium text-(--color-accent-vivid)">
													{key}
												</span>
											{/each}
										</div>
									{/if}
								</div>
								<button
									type="button"
									onclick={(e) => { e.stopPropagation(); handleDeleteEntry(entry.id); }}
									class="mt-0.5 shrink-0 rounded-(--radius-sm) p-1 text-(--color-subtle) opacity-0 transition-opacity hover:bg-(--color-destructive)/10 hover:text-(--color-destructive) group-hover/entry:opacity-100"
									aria-label="Delete entry"
								>
									<Trash2 class="size-3.5" />
								</button>
							</div>
					{/each}
					{/if}
				</div>
			</div>

			<!-- Right column: Entry form -->
			<div class="w-3/5 space-y-3 border-l border-(--color-border) pl-4" class:opacity-50={!entry_enabled} class:pointer-events-none={!entry_enabled}>
				{#if selected_entry}
					<h3 class="text-sm font-medium text-(--color-foreground)">Entry details</h3>

					<!-- Enabled toggle -->
					<div class="flex items-center justify-between mb-1">
						<label for="entry-enabled" class="text-xs text-(--color-muted) uppercase tracking-wider">Enabled</label>
						<button
							id="entry-enabled"
							type="button"
							class={cn(
								'relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors',
								entry_enabled ? 'bg-(--color-accent)' : 'bg-(--color-surface-hover)'
							)}
							onclick={() => (entry_enabled = !entry_enabled)}
							aria-label="Toggle entry enabled"
						>
							<span class={cn(
								'pointer-events-none inline-block size-4 rounded-full bg-white shadow ring-0 transition-transform',
								entry_enabled ? 'translate-x-4' : 'translate-x-0'
							)}></span>
						</button>
					</div>

					<!-- Sort order -->
					<div class="space-y-1">
						<label for="entry-sort-order" class="text-xs text-(--color-muted) uppercase tracking-wider">Sort Order</label>
						<input
							id="entry-sort-order"
							type="number"
							bind:value={entry_sort_order}
							class="w-20 rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-input) px-2 py-1 text-sm"
							min="0"
						/>
					</div>

					<!-- Token budget -->
					<div class="space-y-1">
						<label for="entry-token-budget" class="text-xs text-(--color-muted) uppercase tracking-wider">Token Budget</label>
						<div class="flex items-center gap-2">
							<input
								id="entry-token-budget"
								type="number"
								bind:value={entry_token_budget}
								class="w-24 rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-input) px-2 py-1 text-sm"
								min="0"
								placeholder="\u221E"
							/>
							<span class="text-xs text-(--color-subtle)">Leave empty for shared budget</span>
						</div>
					</div>

					<!-- Keywords -->
					<div class="space-y-1">
						<label for="entry-keywords" class="text-sm font-medium text-(--color-foreground)">Keywords *</label>
						<Input
							id="entry-keywords"
							bind:value={entry_keys}
							placeholder="comma-separated keywords"
							disabled={is_saving_entry}
							class={entry_keys_error ? 'border-(--color-destructive)' : ''}
						/>
						{#if entry_keys_error}
							<p class="text-xs text-(--color-destructive)">{entry_keys_error}</p>
						{/if}
					</div>

					<!-- Secondary keys -->
					<div class="space-y-1">
						<label for="entry-secondary-keys" class="text-sm font-medium text-(--color-foreground)">Secondary keys</label>
						<Input
							id="entry-secondary-keys"
							bind:value={entry_secondary_keys}
							placeholder="optional secondary keywords"
							disabled={is_saving_entry}
						/>
					</div>

					<!-- Content -->
					<div class="space-y-1">
						<label for="entry-content" class="text-sm font-medium text-(--color-foreground)">Content *</label>
						<Textarea
							id="entry-content"
							bind:value={entry_content}
							placeholder="Entry content injected into context"
							rows={4}
							disabled={is_saving_entry}
							class={entry_content_error ? 'border-(--color-destructive)' : ''}
						/>
						{#if entry_content_error}
							<p class="text-xs text-(--color-destructive)">{entry_content_error}</p>
						{/if}
					</div>

					<!-- Position + Depth row -->
					<div class="flex gap-3">
						<div class="flex-1 space-y-1">
							<label for="entry-position" class="text-sm font-medium text-(--color-foreground)">Position</label>
							<Select id="entry-position" bind:value={entry_position} disabled={is_saving_entry}>
								<option value="before_char">Before character</option>
								<option value="after_char">After character</option>
								<option value="at_depth">At depth</option>
							</Select>
						</div>
						<div class="w-24 space-y-1">
							<label for="entry-depth" class="text-sm font-medium text-(--color-foreground)">Depth</label>
							<Input
								id="entry-depth"
								type="number"
								bind:value={entry_depth_str}
								min={0}
								max={999}
								disabled={is_saving_entry || entry_position !== 'at_depth'}
								class={entry_position !== 'at_depth' ? 'opacity-50' : ''}
							/>
						</div>
					</div>

					<!-- Activation -->
					<div class="space-y-1">
						<label for="entry-activation" class="text-sm font-medium text-(--color-foreground)">Activation</label>
					<Select id="entry-activation" bind:value={entry_activation} disabled={is_saving_entry}>
							<option value="always">Always active</option>
							<option value="character_only">Character only</option>
							<option value="user_only">User only</option>
						</Select>
					</div>

					<!-- Matching mode -->
					<div class="space-y-1">
						<label for="entry-activation-mode" class="text-sm font-medium text-(--color-foreground)">Matching</label>
						<Select
							id="entry-activation-mode"
							bind:value={entry_activation_mode}
							disabled={is_saving_entry || !app.embeddings_enabled}
						>
							<option value="keyword">Keywords</option>
							<option value="semantic">Semantic (embeddings)</option>
							<option value="hybrid">Hybrid (either)</option>
						</Select>
						{#if !app.embeddings_enabled}
							<p class="text-xs text-(--color-subtle)">
								Enable embeddings in Settings → Memory &amp; embeddings to use semantic matching.
							</p>
						{:else if entry_activation_mode !== 'keyword'}
							<div class="flex items-center gap-2 pt-1">
								<input
									id="entry-semantic-threshold"
									type="range"
									min="0.3"
									max="0.9"
									step="0.05"
									bind:value={entry_semantic_threshold}
									disabled={is_saving_entry}
									class="h-1.5 flex-1 cursor-pointer accent-(--color-accent)"
								/>
								<span class="w-9 text-right font-mono text-xs text-(--color-muted)">
									{entry_semantic_threshold.toFixed(2)}
								</span>
							</div>
							<p class="text-xs text-(--color-subtle)">
								Similarity threshold — the entry activates when the recent chat is at least this close.
							</p>
						{/if}
					</div>

					<!-- Save entry button -->
					<Button
						variant="primary"
						size="sm"
						onclick={handleSaveEntry}
						disabled={is_saving_entry}
						class="w-full"
					>
						{#if is_saving_entry}
							Saving…
						{:else}
							<Check class="size-3.5" />
							Save entry
						{/if}
					</Button>
				{:else}
					<div class="flex h-full items-center justify-center py-12 text-center">
						<p class="text-sm text-(--color-subtle)">
							{entries.length > 0 ? 'Select an entry to edit' : 'Add an entry to get started'}
						</p>
					</div>
				{/if}
			</div>
		</div>

		<!-- Test Keywords panel (collapsible) -->
		<div class="border-t border-(--color-border) pt-3">
			<button
				type="button"
				onclick={() => (test_panel_open = !test_panel_open)}
				class="flex w-full items-center gap-2 text-sm font-medium text-(--color-foreground) transition-colors hover:text-(--color-accent-vivid)"
			>
				{#if test_panel_open}
					<ChevronDown class="size-4" />
				{:else}
					<ChevronRight class="size-4" />
				{/if}
				Test keywords
			</button>

			{#if test_panel_open}
				<div class="mt-3 space-y-3">
					<!-- Test kind toggle (keywords / semantic) -->
					<div class="flex items-center justify-between gap-2">
						<div
							class="inline-flex items-center gap-1 rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-input) p-0.5"
							role="tablist"
							aria-label="Test kind"
						>
							<button
								type="button"
								role="tab"
								aria-selected={test_kind === 'keyword'}
								onclick={() => (test_kind = 'keyword')}
								class="rounded-(--radius-sm) px-2.5 py-1 text-xs font-medium transition-colors {test_kind ===
								'keyword'
									? 'bg-(--color-surface-elevated) text-(--color-foreground) shadow-(--shadow-sm) ring-1 ring-(--color-border) ring-inset'
									: 'text-(--color-muted) hover:text-(--color-foreground)'}"
							>
								Keywords
							</button>
							<button
								type="button"
								role="tab"
								aria-selected={test_kind === 'semantic'}
								onclick={() => (test_kind = 'semantic')}
								disabled={!app.embeddings_enabled}
								title={app.embeddings_enabled ? '' : 'Enable embeddings in Settings first'}
								class="rounded-(--radius-sm) px-2.5 py-1 text-xs font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-40 {test_kind ===
								'semantic'
									? 'bg-(--color-surface-elevated) text-(--color-foreground) shadow-(--shadow-sm) ring-1 ring-(--color-border) ring-inset'
									: 'text-(--color-muted) hover:text-(--color-foreground)'}"
							>
								Semantic
							</button>
						</div>
						<span class="text-[11px] text-(--color-subtle)">
							{test_kind === 'semantic'
								? 'Similarity of each entry against the sample text.'
								: 'Which keywords match the sample text.'}
						</span>
					</div>

					<div class="space-y-1">
						<label for="test-sample-text" class="text-sm font-medium text-(--color-foreground)">Sample text</label>
						<Textarea
							id="test-sample-text"
							bind:value={test_sample_text}
							placeholder="Paste or type sample chat text to test which entries would activate…"
							rows={3}
							disabled={test_loading || semantic_loading}
						/>
					</div>

					{#if test_error}
						<p class="text-xs text-(--color-destructive)">{test_error}</p>
					{/if}

					<Button
						variant="secondary"
						size="sm"
						onclick={handleRunTest}
						disabled={test_loading || semantic_loading || entries.length === 0}
					>
						<Play class="size-3.5" />
						{test_loading || semantic_loading ? 'Running…' : 'Run test'}
					</Button>

					{#if test_kind === 'keyword' && test_results.length > 0}
						<div class="space-y-2 rounded-(--radius-md) border border-(--color-border) p-3">
							<h4 class="text-xs font-semibold uppercase tracking-wider text-(--color-subtle)">Results</h4>
							{#each entries as entry (entry.id)}
								{@const result = test_results.find((r) => r.entry_id === entry.id)}
								{#if result}
									<div
										class="flex items-start gap-2 rounded-(--radius-sm) px-2 py-1.5 {result.matched_keys.length > 0 ? 'bg-(--color-accent)/5' : 'bg-(--color-surface-elevated) opacity-50'}"
									>
										{#if result.matched_keys.length > 0}
											<Check class="mt-0.5 size-3.5 shrink-0 text-(--color-accent-vivid)" />
										{:else}
											<XIcon class="mt-0.5 size-3.5 shrink-0 text-(--color-subtle)" />
										{/if}
										<div class="min-w-0 flex-1">
											<p class="truncate text-sm font-medium text-(--color-foreground)">
												{entry.keys.length > 0 ? entry.keys.join(', ') : 'Empty'}
											</p>
											{#if result.matched_keys.length > 0}
												<div class="mt-1 flex flex-wrap gap-1">
													{#each result.matched_keys as key (key)}
														<span class="inline-flex items-center rounded-(--radius-sm) bg-(--color-accent)/15 px-1.5 py-0.5 text-[10px] font-medium text-(--color-accent-vivid)">
															{key}
														</span>
													{/each}
												</div>
											{:else}
												<p class="text-xs text-(--color-subtle)">No matches</p>
											{/if}
										</div>
									</div>
								{/if}
							{/each}
						</div>
					{:else if test_kind === 'semantic' && semantic_results.length > 0}
						<div class="space-y-2 rounded-(--radius-md) border border-(--color-border) p-3">
							<h4 class="text-xs font-semibold uppercase tracking-wider text-(--color-subtle)">Similarity</h4>
							{#each semantic_results as result (result.entryId)}
								{@const entry = entries.find((e) => e.id === result.entryId)}
								{#if entry}
									<div
										class="flex items-center gap-2 rounded-(--radius-sm) px-2 py-1.5 {result.activated ? 'bg-(--color-accent)/5' : 'bg-(--color-surface-elevated) opacity-50'}"
									>
										{#if result.activated}
											<Check class="size-3.5 shrink-0 text-(--color-accent-vivid)" />
										{:else}
											<XIcon class="size-3.5 shrink-0 text-(--color-subtle)" />
										{/if}
										<div class="min-w-0 flex-1">
											<p class="truncate text-sm font-medium text-(--color-foreground)">
												{entry.keys.length > 0 ? truncate(entry.keys.join(', '), 32) : truncate(entry.content, 32)}
											</p>
											<div class="mt-1 flex items-center gap-2">
												<div class="h-1 flex-1 overflow-hidden rounded-full bg-(--color-surface-input)">
													<div
														class="h-full rounded-full bg-(--color-accent)"
														style="width: {Math.max(0, Math.min(1, result.similarity ?? 0)) * 100}%"
													></div>
												</div>
												<span class="w-24 shrink-0 text-right font-mono text-[10px] text-(--color-muted)">
													{format_similarity(result.similarity)} / {result.threshold.toFixed(2)}
												</span>
												{#if result.reason}
													<span class="shrink-0 rounded-(--radius-sm) bg-(--color-accent)/15 px-1.5 py-0.5 text-[10px] font-medium text-(--color-accent-vivid)">
														{result.reason}
													</span>
												{/if}
											</div>
										</div>
									</div>
								{/if}
							{/each}
						</div>
					{/if}
				</div>
			{/if}
		</div>
	</div>

	{#snippet footer()}
		<div class="flex items-center gap-2">
			<Button variant="secondary" size="sm" onclick={() => handleOpenChange(false)} disabled={is_saving}>
				Close
			</Button>
			<Button variant="primary" size="sm" onclick={handleSaveLorebook} disabled={is_saving}>
				{is_saving ? 'Saving…' : mode === 'edit' ? 'Save lorebook' : 'Create lorebook'}
			</Button>
		</div>
	{/snippet}
</Dialog>

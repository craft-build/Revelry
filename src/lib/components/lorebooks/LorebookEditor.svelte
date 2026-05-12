<script lang="ts">
	import type { Lorebook, LorebookEntry, LorebookPosition, LorebookActivation, KeywordTestResult } from '$lib/types';
	import { app } from '$lib/stores/app.svelte';
	import {
		getLorebookEntries as apiGetLorebookEntries,
		saveLorebookEntry as apiSaveLorebookEntry,
		deleteLorebookEntry as apiDeleteLorebookEntry,
		testLorebookKeywords as apiTestLorebookKeywords
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
	let entry_keys_error = $state('');
	let entry_content_error = $state('');
	let is_saving_entry = $state(false);

	// ── Keyword test state ───────────────────────────────────────────────
	let test_panel_open = $state(false);
	let test_sample_text = $state('');
	let test_loading = $state(false);
	let test_results = $state<KeywordTestResult[]>([]);
	let test_error = $state('');

	// ── Derived ────────────────────────────────────────────────────────────
	let selected_entry = $derived(entries.find((e) => e.id === selected_entry_id) ?? null);

	function handleOpenChange(value: boolean) {
		open = value;
		onopenchange?.(value);
	}

	// ── Load entries when dialog opens ────────────────────────────────────
	$effect(() => {
		if (open) {
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
			entry_keys_error = '';
			entry_content_error = '';
			test_results = [];
			test_error = '';
			test_sample_text = '';
			test_panel_open = false;

			if (mode === 'edit' && lorebook) {
				form_name = lorebook.name;
				form_description = lorebook.description;
				loadEntries(lorebook.id);
			}
		}
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
			entry_keys_error = '';
			entry_content_error = '';
		}
	}

	// ── Lorebook save ─────────────────────────────────────────────────────
	async function handleSaveLorebook(): Promise<string | null> {
		if (!form_name.trim()) {
			name_error = 'Name is required.';
			return null;
		}
		name_error = '';

		is_saving = true;
		backend_error = '';

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
		} finally {
			is_saving = false;
		}
	}

	// ── Entry save ────────────────────────────────────────────────────────
	async function handleSaveEntry() {
		if (is_saving_entry) return;

		// Validate
		const keys = entry_keys
			.split(',')
			.map((k) => k.trim())
			.filter((k) => k.length > 0);
		if (keys.length === 0) {
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
				const id = await handleSaveLorebook();
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
			const id = await handleSaveLorebook();
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
		entry_keys_error = '';
		entry_content_error = '';
	}

	// ── Keyword testing ──────────────────────────────────────────────────
	async function handleRunTest() {
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
				if (entry.keys.length === 0) continue;
				const result = await apiTestLorebookKeywords(
					entry.keys,
					entry.secondary_keys ?? [],
					test_sample_text.trim()
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
				<label class="text-sm font-medium text-(--color-foreground)">Name *</label>
				<Input
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
				<label class="text-sm font-medium text-(--color-foreground)">Description</label>
				<Input
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
								class="group/entry relative flex items-start gap-2 rounded-(--radius-md) border px-3 py-2 text-left transition-colors cursor-pointer {selected_entry_id === entry.id ? 'border-(--color-accent) bg-(--color-accent)/5' : 'border-(--color-border) hover:border-(--color-border-strong)'} {test_results.length > 0 && !has_match ? 'opacity-40' : ''}"
							>
								<div class="min-w-0 flex-1">
									<p class="truncate text-sm font-medium text-(--color-foreground)">
										{entry.keys.length > 0 ? truncate(entry.keys[0], 18) : 'Empty entry'}
									</p>
									<p class="truncate text-xs text-(--color-muted)">
										{truncate(entry.content, 60)}
									</p>
									{#if test_results.length > 0 && has_match}
										<div class="mt-1 flex flex-wrap gap-1">
											{#each matched as key (key)}
												<span class="inline-flex items-center rounded-(--radius-sm) bg-(--color-accent)/15 px-1.5 py-0.5 text-[10px] font-medium text-(--color-accent)">
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
			<div class="w-3/5 space-y-3 border-l border-(--color-border) pl-4">
				{#if selected_entry}
					<h3 class="text-sm font-medium text-(--color-foreground)">Entry details</h3>

					<!-- Keywords -->
					<div class="space-y-1">
						<label class="text-sm font-medium text-(--color-foreground)">Keywords *</label>
						<Input
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
						<label class="text-sm font-medium text-(--color-foreground)">Secondary keys</label>
						<Input
							bind:value={entry_secondary_keys}
							placeholder="optional secondary keywords"
							disabled={is_saving_entry}
						/>
					</div>

					<!-- Content -->
					<div class="space-y-1">
						<label class="text-sm font-medium text-(--color-foreground)">Content *</label>
						<Textarea
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
							<label class="text-sm font-medium text-(--color-foreground)">Position</label>
							<Select bind:value={entry_position} disabled={is_saving_entry}>
								<option value="before_char">Before character</option>
								<option value="after_char">After character</option>
								<option value="at_depth">At depth</option>
							</Select>
						</div>
						<div class="w-24 space-y-1">
							<label class="text-sm font-medium text-(--color-foreground)">Depth</label>
							<Input
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
						<label class="text-sm font-medium text-(--color-foreground)">Activation</label>
						<Select bind:value={entry_activation} disabled={is_saving_entry}>
							<option value="always">Always active</option>
							<option value="character_only">Character only</option>
							<option value="user_only">User only</option>
						</Select>
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
				class="flex w-full items-center gap-2 text-sm font-medium text-(--color-foreground) transition-colors hover:text-(--color-accent)"
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
					<div class="space-y-1">
						<label class="text-sm font-medium text-(--color-foreground)">Sample text</label>
						<Textarea
							bind:value={test_sample_text}
							placeholder="Paste or type sample chat text to test which entries would activate…"
							rows={3}
							disabled={test_loading}
						/>
					</div>

					{#if test_error}
						<p class="text-xs text-(--color-destructive)">{test_error}</p>
					{/if}

					<Button
						variant="secondary"
						size="sm"
						onclick={handleRunTest}
						disabled={test_loading || entries.length === 0}
					>
						<Play class="size-3.5" />
						{test_loading ? 'Running…' : 'Run test'}
					</Button>

					{#if test_results.length > 0}
						<div class="space-y-2 rounded-(--radius-md) border border-(--color-border) p-3">
							<h4 class="text-xs font-semibold uppercase tracking-wider text-(--color-subtle)">Results</h4>
							{#each entries as entry (entry.id)}
								{@const result = test_results.find((r) => r.entry_id === entry.id)}
								{#if result}
									<div
										class="flex items-start gap-2 rounded-(--radius-sm) px-2 py-1.5 {result.matched_keys.length > 0 ? 'bg-(--color-accent)/5' : 'bg-(--color-surface-elevated) opacity-50'}"
									>
										{#if result.matched_keys.length > 0}
											<Check class="mt-0.5 size-3.5 shrink-0 text-(--color-accent)" />
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
														<span class="inline-flex items-center rounded-(--radius-sm) bg-(--color-accent)/15 px-1.5 py-0.5 text-[10px] font-medium text-(--color-accent)">
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

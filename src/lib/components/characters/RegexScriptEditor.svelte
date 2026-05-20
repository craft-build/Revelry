<script lang="ts">
	import type { RegexScript, RegexTestResult } from '$lib/types';
	import { app } from '$lib/stores/app.svelte';
	import { Button, Input, Textarea } from '$lib/components/ui';
	import {
		Plus,
		Trash2,
		Play,
		ChevronDown,
		ChevronRight,
		Check,
		AlertCircle,
	} from '@lucide/svelte';
	import { cn } from '$lib/utils';

	interface Props {
		characterId: string;
		createMode?: boolean;
	}

	let { characterId, createMode = false }: Props = $props();

	// ── State ────────────────────────────────────────────────────────────
	let scripts = $state<RegexScript[]>([]);
	let selectedScriptId = $state<string | null>(null);
	let is_saving = $state(false);
	let is_loading = $state(false);

	// ── Form state ───────────────────────────────────────────────────────
	let form_name = $state('');
	let form_find_pattern = $state('');
	let form_replace_pattern = $state('');
	let form_flags = $state('');
	let form_enabled = $state(true);
	let pattern_error = $state<string | null>(null);

	// ── Test state ───────────────────────────────────────────────────────
	let test_panel_open = $state(false);
	let test_input = $state('');
	let test_result = $state<RegexTestResult | null>(null);
	let test_error = $state<string | null>(null);
	let test_loading = $state(false);

	// ── Delete with undo ─────────────────────────────────────────────────
	let deleted_script: RegexScript | null = $state(null);
	let undo_timer: ReturnType<typeof setTimeout> | null = null;

	// ── Derived ──────────────────────────────────────────────────────────
	let selected_script = $derived(
		scripts.find((s) => s.id === selectedScriptId) ?? null,
	);

	// ── Load on mount ────────────────────────────────────────────────────
	$effect(() => {
		if (!createMode && characterId) {
			loadScripts();
		}
	});

	async function loadScripts() {
		is_loading = true;
		try {
			scripts = await app.loadRegexScripts(characterId).then(() => {
				return app.regex_scripts.get(characterId) ?? [];
			});
		} catch (err) {
			console.error('Failed to load regex scripts:', err);
			scripts = [];
		} finally {
			is_loading = false;
		}
	}

	function selectScript(id: string) {
		selectedScriptId = id;
		const script = scripts.find((s) => s.id === id);
		if (script) {
			form_name = script.name;
			form_find_pattern = script.find_pattern;
			form_replace_pattern = script.replace_pattern;
			form_flags = script.flags;
			form_enabled = script.enabled;
			pattern_error = null;
		}
	}

	function resetForm() {
		form_name = '';
		form_find_pattern = '';
		form_replace_pattern = '';
		form_flags = '';
		form_enabled = true;
		pattern_error = null;
		selectedScriptId = null;
	}

	// ── Add script ───────────────────────────────────────────────────────
	function handleAdd() {
		if (!characterId) return;
		const newScript: RegexScript = {
			id: crypto.randomUUID(),
			character_id: characterId,
			name: 'New script',
			find_pattern: '',
			replace_pattern: '',
			flags: 'gi',
			sort_order: scripts.length,
			enabled: true,
			created_at: new Date().toISOString(),
			updated_at: new Date().toISOString(),
		};
		scripts = [...scripts, newScript];
		selectScript(newScript.id);
	}

	// ── Save script ──────────────────────────────────────────────────────
	async function handleSave() {
		if (is_saving || !selectedScriptId) return;
		pattern_error = null;

		const script: RegexScript = {
			id: selectedScriptId!,
			character_id: characterId,
			name: form_name.trim() || 'Untitled script',
			find_pattern: form_find_pattern,
			replace_pattern: form_replace_pattern,
			flags: form_flags,
			sort_order: selected_script?.sort_order ?? scripts.length,
			enabled: form_enabled,
			created_at: selected_script?.created_at ?? new Date().toISOString(),
			updated_at: new Date().toISOString(),
		};

		is_saving = true;
		try {
			const saved = await app.saveRegexScript(script);
			const idx = scripts.findIndex((s) => s.id === saved.id);
			if (idx >= 0) {
				scripts[idx] = saved;
			} else {
				scripts = [...scripts, saved];
			}
			selectedScriptId = saved.id;
		} catch (err) {
			const msg = err instanceof Error ? err.message : String(err);
			pattern_error = msg;
		} finally {
			is_saving = false;
		}
	}

	// ── Delete script (with undo toast) ──────────────────────────────────
	function handleDelete(id: string) {
		const script = scripts.find((s) => s.id === id);
		if (!script) return;

		// Clear any previous undo timer
		if (undo_timer) clearTimeout(undo_timer);

		deleted_script = script;
		scripts = scripts.filter((s) => s.id !== id);
		if (selectedScriptId === id) {
			resetForm();
		}

		// Auto-confirm delete after 5 seconds
		undo_timer = setTimeout(() => {
			confirmDelete();
		}, 5000);
	}

	async function confirmDelete() {
		if (!deleted_script) return;
		try {
			await app.deleteRegexScript(deleted_script.id, characterId);
		} catch (err) {
			console.error('Failed to delete regex script:', err);
		}
		deleted_script = null;
		if (undo_timer) {
			clearTimeout(undo_timer);
			undo_timer = null;
		}
	}

	function handleUndo() {
		if (!deleted_script) return;
		scripts = [...scripts, deleted_script];
		selectScript(deleted_script.id);
		deleted_script = null;
		if (undo_timer) {
			clearTimeout(undo_timer);
			undo_timer = null;
		}
	}

	// ── Test pattern ─────────────────────────────────────────────────────
	async function handleRunTest() {
		if (test_loading) return;
		if (!form_find_pattern.trim()) {
			test_error = 'Enter a find pattern to test.';
			return;
		}
		if (!test_input.trim()) {
			test_error = 'Enter sample AI text to test against.';
			return;
		}

		test_error = null;
		test_result = null;
		test_loading = true;

		try {
			test_result = await app.testRegexPattern(
				form_find_pattern,
				form_replace_pattern,
				form_flags,
				test_input.trim(),
			);
		} catch (err) {
			test_error = err instanceof Error ? err.message : 'Test failed.';
		} finally {
			test_loading = false;
		}
	}

	// ── Helpers ──────────────────────────────────────────────────────────
	function truncate(str: string, max: number): string {
		if (!str) return '';
		return str.length > max ? str.slice(0, max) + '...' : str;
	}
</script>

{#if createMode}
	<div class="flex h-full items-center justify-center py-12 text-center">
		<p class="text-sm text-(--color-subtle)">
			Regex scripts can be added after the character is created.
		</p>
	</div>
{:else}
	<div class="space-y-4">
		<!-- Undo toast -->
		{#if deleted_script}
			<div class="flex items-center justify-between rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-elevated) px-3 py-2">
				<span class="text-xs text-(--color-foreground)">Deleted "{deleted_script.name}"</span>
				<button
					type="button"
					onclick={handleUndo}
					class="text-xs font-medium text-(--color-accent) hover:underline"
				>
					Undo
				</button>
			</div>
		{/if}

		<!-- Two-column layout -->
		<div class="flex gap-4">
			<!-- Left column: Script list -->
			<div class="flex w-2/5 flex-col gap-2">
				<div class="flex items-center justify-between">
					<h3 class="text-sm font-medium text-(--color-foreground)">Scripts</h3>
					<Button variant="ghost" size="icon-sm" onclick={handleAdd} disabled={is_saving || is_loading}>
						<Plus class="size-4" />
					</Button>
				</div>

				<div class="flex flex-col gap-1 overflow-y-auto" style="max-height: 280px;">
					{#if is_loading}
						<p class="py-4 text-center text-sm text-(--color-subtle)">Loading...</p>
					{:else if scripts.length === 0}
						<p class="py-4 text-center text-sm text-(--color-subtle)">
							No regex scripts. Add one to transform AI output.
						</p>
					{:else}
						{#each scripts as script (script.id)}
							<div
								role="button"
								tabindex="0"
								onclick={() => selectScript(script.id)}
								onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') selectScript(script.id); }}
								class={cn(
									'group/script relative flex items-start gap-2 rounded-(--radius-md) border px-3 py-2 text-left transition-colors cursor-pointer',
									selectedScriptId === script.id
										? 'border-(--color-accent) bg-(--color-accent)/5'
										: 'border-(--color-border) hover:border-(--color-border-strong)',
								)}
							>
								<div class="min-w-0 flex-1">
									<div class="flex items-center gap-1.5">
										<span
											class={cn(
												'size-2 shrink-0 rounded-full',
												script.enabled ? 'bg-(--color-success)' : 'bg-(--color-subtle)',
											)}
										></span>
										<p class="truncate text-sm font-medium text-(--color-foreground)">
											{script.name}
										</p>
									</div>
									<p class="truncate text-xs text-(--color-muted) font-mono mt-0.5">
										{truncate(script.find_pattern, 40)}
									</p>
								</div>
								<button
									type="button"
									onclick={(e) => { e.stopPropagation(); handleDelete(script.id); }}
									class="mt-0.5 shrink-0 rounded-(--radius-sm) p-1 text-(--color-subtle) opacity-0 transition-opacity hover:bg-(--color-destructive)/10 hover:text-(--color-destructive) group-hover/script:opacity-100"
									aria-label="Delete script"
								>
									<Trash2 class="size-3.5" />
								</button>
							</div>
						{/each}
					{/if}
				</div>
			</div>

			<!-- Right column: Script form -->
			<div class="w-3/5 space-y-3 border-l border-(--color-border) pl-4">
				{#if selectedScriptId}
					<h3 class="text-sm font-medium text-(--color-foreground)">Script details</h3>

					{#if pattern_error}
						<div class="flex items-center gap-2 rounded-(--radius-md) border border-(--color-destructive)/30 bg-(--color-destructive)/10 px-3 py-2 text-sm text-(--color-destructive)">
							<AlertCircle class="size-4 shrink-0" />
							{pattern_error}
						</div>
					{/if}

					<!-- Name -->
					<div class="space-y-1">
						<label class="text-sm font-medium text-(--color-foreground)">Name</label>
						<Input
							bind:value={form_name}
							placeholder="Script name"
							disabled={is_saving}
						/>
					</div>

					<!-- Find pattern -->
					<div class="space-y-1">
						<label class="text-sm font-medium text-(--color-foreground)">Find pattern</label>
						<Input
							bind:value={form_find_pattern}
							placeholder="Regex pattern (e.g. \\*\\_([^_]+)\\_\\*)"
							disabled={is_saving}
							class={pattern_error ? 'border-(--color-destructive) font-mono text-[13px]' : 'font-mono text-[13px]'}
						/>
					</div>

					<!-- Replace pattern -->
					<div class="space-y-1">
						<label class="text-sm font-medium text-(--color-foreground)">Replace pattern</label>
						<Input
							bind:value={form_replace_pattern}
							placeholder="Replacement string (e.g. **$1**)"
							disabled={is_saving}
							class="font-mono text-[13px]"
						/>
					</div>

					<!-- Flags + Enabled row -->
					<div class="flex items-end gap-3">
						<div class="w-32 space-y-1">
							<label class="text-sm font-medium text-(--color-foreground)">Flags</label>
							<Input
								bind:value={form_flags}
								placeholder="gi"
								disabled={is_saving}
								class="font-mono text-[13px]"
							/>
						</div>
						<label class="flex items-center gap-2 pb-1">
							<input
								type="checkbox"
								bind:checked={form_enabled}
								class="size-4 rounded border-(--color-border) bg-(--color-surface-input) accent-(--color-accent)"
							/>
							<span class="text-sm text-(--color-foreground)">Enabled</span>
						</label>
					</div>

					<!-- Save button -->
					<Button
						variant="primary"
						size="sm"
						onclick={handleSave}
						disabled={is_saving}
						class="w-full"
					>
						{#if is_saving}
							Saving...
						{:else}
							<Check class="size-3.5" />
							Save script
						{/if}
					</Button>
				{:else}
					<div class="flex h-full items-center justify-center py-12 text-center">
						<p class="text-sm text-(--color-subtle)">
							{scripts.length > 0 ? 'Select a script to edit' : 'Add a script to get started'}
						</p>
					</div>
				{/if}
			</div>
		</div>

		<!-- Test panel (collapsible) -->
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
				Test pattern
			</button>

			{#if test_panel_open}
				<div class="mt-3 space-y-3">
					<div class="space-y-1">
						<label class="text-sm font-medium text-(--color-foreground)">Sample AI text</label>
						<Textarea
							bind:value={test_input}
							placeholder="Paste or type sample AI output to test the regex transformation..."
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
						disabled={test_loading || !form_find_pattern.trim()}
					>
						<Play class="size-3.5" />
						{test_loading ? 'Running...' : 'Run test'}
					</Button>

					{#if test_result}
						<div class="space-y-2 rounded-(--radius-md) border border-(--color-border) p-3">
							<h4 class="text-xs font-semibold uppercase tracking-wider text-(--color-subtle)">Result</h4>
							<div class="rounded-(--radius-md) border border-(--color-border) bg-(--color-surface) p-3 text-sm text-(--color-foreground) whitespace-pre-wrap">
								{test_result.output}
							</div>
							<p class={cn(
								'text-xs',
								test_result.match_count > 0 ? 'text-(--color-accent)' : 'text-(--color-subtle)',
							)}>
								{test_result.match_count > 0
									? `${test_result.match_count} match${test_result.match_count !== 1 ? 'es' : ''} found`
									: 'No matches'}
							</p>
						</div>
					{/if}
				</div>
			{/if}
		</div>
	</div>
{/if}

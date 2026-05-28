<script lang="ts">
	import { Tabs } from 'bits-ui';
	import { Plus, Search, Upload, BookOpen } from '@lucide/svelte';
	import { app } from '$lib/stores/app.svelte';
	import { Input, Button } from '$lib/components/ui';
	import CharacterCard from '$lib/components/characters/CharacterCard.svelte';
	import PersonaCard from '$lib/components/personas/PersonaCard.svelte';
	import { LorebookCard, LorebookEditor } from '$lib/components/lorebooks';
	import { CharacterEditorDialog, DeleteConfirmationDialog, ImportErrorDialog } from '$lib/components/characters';
	import { PersonaEditorDialog } from '$lib/components/personas';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import type { CharacterCard as CharacterCardType, UserPersona, Lorebook } from '$lib/types';

	// Tab state synced to query string so the persona switcher's
	// "Manage personas" link can deep-link.
	let tab = $state<string>(page.url.searchParams.get('tab') ?? 'characters');

	$effect(() => {
		if (page.url.searchParams.get('tab') === tab) return;
		const url = new URL(page.url);
		url.searchParams.set('tab', tab);
		goto(url, { replaceState: true, noScroll: true, keepFocus: true });
	});

	let searchValue = $state('');
	$effect(() => {
		app.character_search_query = searchValue;
	});

	// ── Editor state ────────────────────────────────────────────────────
	let editor_open = $state(false);
	let editor_mode = $state<'create' | 'edit'>('create');
	let editing_character = $state<CharacterCardType | undefined>(undefined);

	// ── Delete state ───────────────────────────────────────────────────
	let delete_open = $state(false);
	let deleting_id = $state<string | null>(null);
	let deleting_name = $state('');
	let deleting_entity_type = $state<'character' | 'persona' | 'lorebook'>('character');
	let is_deleting = $state(false);

	// ── Import state ─────────────────────────────────────────────────────
	let import_error_open = $state(false);
	let import_error_message = $state('');
	let is_importing = $state(false);

	// ── Persona editor state ────────────────────────────────────────────
	let persona_editor_open = $state(false);
	let persona_editor_mode = $state<'create' | 'edit'>('create');
	let editing_persona = $state<UserPersona | undefined>(undefined);

	function openCreateEditor() {
		editor_mode = 'create';
		editing_character = undefined;
		editor_open = true;
	}

	async function handleImport() {
		if (is_importing) return;
		is_importing = true;
		try {
			const importedCharacter = await app.importCharacterFromFile();
			// Open the character editor in "create" mode, pre-filled with imported data
			editor_mode = 'create';
			editing_character = importedCharacter;
			editor_open = true;
		} catch (err) {
			// User cancelled file dialog — not an error
			if (err instanceof Error && err.message === 'cancelled') {
				return;
			}
			// Show import error dialog
			import_error_message =
				err instanceof Error
					? err.message
					: typeof err === 'string'
						? err
						: 'Could not read the character card.';
			import_error_open = true;
		} finally {
			is_importing = false;
		}
	}

	async function handleExport(id: string) {
		try {
			await app.exportCharacter(id);
		} catch (err) {
			console.error('Failed to export character:', err);
		}
	}

	function openEditEditor(id: string) {
		const char = app.characters.find((c) => c.id === id);
		if (char) {
			editor_mode = 'edit';
			editing_character = char;
			editor_open = true;
		}
	}

	function openDeleteDialog(id: string) {
		const char = app.characters.find((c) => c.id === id);
		if (char) {
			deleting_id = id;
			deleting_name = char.name;
			deleting_entity_type = 'character';
			delete_open = true;
		}
	}

	function openPersonaCreateEditor() {
		persona_editor_mode = 'create';
		editing_persona = undefined;
		persona_editor_open = true;
	}

	function openPersonaEditEditor(id: string) {
		const persona = app.personas.find((p) => p.id === id);
		if (persona) {
			persona_editor_mode = 'edit';
			editing_persona = persona;
			persona_editor_open = true;
		}
	}

	function openPersonaDeleteDialog(id: string) {
		const persona = app.personas.find((p) => p.id === id);
		if (persona) {
			deleting_id = id;
			deleting_name = persona.name;
			deleting_entity_type = 'persona';
			delete_open = true;
		}
	}

	// ── Lorebook editor state ─────────────────────────────────────────────
	let lorebook_editor_open = $state(false);
	let lorebook_editor_mode = $state<'create' | 'edit'>('create');
	let editing_lorebook = $state<Lorebook | undefined>(undefined);

	function openLorebookCreateEditor() {
		lorebook_editor_mode = 'create';
		editing_lorebook = undefined;
		lorebook_editor_open = true;
	}

	function openLorebookEditEditor(id: string) {
		const lorebook = app.lorebooks.find((l) => l.id === id);
		if (lorebook) {
			lorebook_editor_mode = 'edit';
			editing_lorebook = lorebook;
			lorebook_editor_open = true;
		}
	}

	function openLorebookDeleteDialog(id: string) {
		const lorebook = app.lorebooks.find((l) => l.id === id);
		if (lorebook) {
			deleting_id = id;
			deleting_name = lorebook.name;
			deleting_entity_type = 'lorebook';
			delete_open = true;
		}
	}

	async function handleDelete() {
		if (!deleting_id) return;
		is_deleting = true;
		try {
			if (deleting_entity_type === 'persona') {
				await app.deletePersona(deleting_id);
			} else if (deleting_entity_type === 'lorebook') {
				await app.deleteLorebook(deleting_id);
			} else {
				await app.deleteCharacter(deleting_id);
			}
			delete_open = false;
			deleting_id = null;
		} catch (err) {
			console.error(`Failed to delete ${deleting_entity_type}:`, err);
		} finally {
			is_deleting = false;
		}
	}
</script>

<svelte:head>
	<title>Library — Revelry</title>
</svelte:head>

<div class="flex h-full flex-col">
	<!-- Section header -->
	<div class="border-b border-(--color-border) bg-(--color-surface) px-8 py-6">
		<div class="mx-auto flex max-w-7xl items-end justify-between gap-4">
			<div>
				<h1 class="font-display text-3xl font-semibold tracking-tight">Library</h1>
				<p class="mt-1 text-sm text-(--color-muted)">
					Your characters and personas. Cards are stored locally — they never leave the machine
					unless you export them.
				</p>
			</div>

			<div class="flex items-center gap-2">
				<Button variant="secondary" size="sm" onclick={handleImport} disabled={is_importing}>
					{#if is_importing}
						<span class="size-3.5 animate-spin rounded-full border-2 border-(--color-subtle) border-t-transparent"></span>
						Importing…
					{:else}
						<Upload class="size-3.5" />
						Import card
					{/if}
				</Button>
				<Button variant="primary" size="sm" onclick={() => tab === 'personas' ? openPersonaCreateEditor() : tab === 'lorebooks' ? openLorebookCreateEditor() : openCreateEditor()}>
					<Plus class="size-3.5" />
					New {tab === 'personas' ? 'persona' : tab === 'lorebooks' ? 'lorebook' : 'character'}
				</Button>
			</div>
		</div>
	</div>

	<!-- Content -->
	<div class="flex-1 overflow-y-auto">
		<div class="mx-auto max-w-7xl px-8 py-6">
			<Tabs.Root bind:value={tab}>
				<!-- Tab strip + search row -->
				<div class="mb-6 flex items-center justify-between gap-4">
					<Tabs.List
						class="inline-flex h-10 items-center gap-1 rounded-(--radius-md) bg-(--color-surface-elevated) p-1"
					>
						<Tabs.Trigger
							value="characters"
							class="inline-flex h-8 items-center gap-2 rounded-(--radius-sm) px-4 text-sm font-medium text-(--color-muted) transition-colors hover:text-(--color-foreground) data-[state=active]:bg-(--color-surface) data-[state=active]:text-(--color-foreground) data-[state=active]:shadow-sm"
						>
							Characters
							<span class="text-[11px] text-(--color-subtle)">
								{app.characters.length}
							</span>
						</Tabs.Trigger>
						<Tabs.Trigger
							value="personas"
							class="inline-flex h-8 items-center gap-2 rounded-(--radius-sm) px-4 text-sm font-medium text-(--color-muted) transition-colors hover:text-(--color-foreground) data-[state=active]:bg-(--color-surface) data-[state=active]:text-(--color-foreground) data-[state=active]:shadow-sm"
						>
							My personas
							<span class="text-[11px] text-(--color-subtle)">
								{app.personas.length}
							</span>
						</Tabs.Trigger>
						<Tabs.Trigger
							value="lorebooks"
							class="inline-flex h-8 items-center gap-2 rounded-(--radius-sm) px-4 text-sm font-medium text-(--color-muted) transition-colors hover:text-(--color-foreground) data-[state=active]:bg-(--color-surface) data-[state=active]:text-(--color-foreground) data-[state=active]:shadow-sm"
						>
							Lorebooks
							<span class="text-[11px] text-(--color-subtle)">
								{app.lorebooks.length}
							</span>
						</Tabs.Trigger>
					</Tabs.List>

					<div class="flex items-center gap-2">
						<div class="relative">
							<Search
								class="pointer-events-none absolute left-3 top-1/2 size-3.5 -translate-y-1/2 text-(--color-subtle)"
							/>
							<Input
								bind:value={searchValue}
								placeholder="Search {tab}…"
								class="h-9 w-72 pl-9"
							/>
						</div>
					</div>
				</div>

				<!-- Characters tab -->
				<Tabs.Content value="characters">
					{#if app.characters.length === 0}
						<div
							class="flex flex-col items-center justify-center gap-3 rounded-(--radius-lg) border border-dashed border-(--color-border) py-20 text-center"
						>
							<p class="text-sm text-(--color-muted)">No characters yet.</p>
							<Button variant="secondary" size="sm" onclick={openCreateEditor}>
								<Plus class="size-3.5" />
								New character
							</Button>
						</div>
					{:else if app.filtered_characters.length === 0}
						<div
							class="flex flex-col items-center justify-center gap-3 rounded-(--radius-lg) border border-dashed border-(--color-border) py-20 text-center"
						>
							<p class="text-sm text-(--color-muted)">No characters match.</p>
							<p class="text-xs text-(--color-subtle)">Try a different search term.</p>
						</div>
					{:else}
						<div
							class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4"
						>
							{#each app.filtered_characters as character (character.id)}
								<CharacterCard {character} onedit={openEditEditor} ondelete={openDeleteDialog} onexport={handleExport} />
							{/each}
						</div>
					{/if}
				</Tabs.Content>

				<!-- Personas tab -->
				<Tabs.Content value="personas">
					{#if app.personas.length === 0}
						<div
							class="flex flex-col items-center justify-center gap-3 rounded-(--radius-lg) border border-dashed border-(--color-border) py-20 text-center"
						>
							<p class="text-sm text-(--color-muted)">No personas yet.</p>
							<Button variant="secondary" size="sm" onclick={openPersonaCreateEditor}>
								<Plus class="size-3.5" />
								New persona
							</Button>
						</div>
					{:else if app.filtered_personas.length === 0}
						<div
							class="flex flex-col items-center justify-center gap-3 rounded-(--radius-lg) border border-dashed border-(--color-border) py-20 text-center"
						>
							<p class="text-sm text-(--color-muted)">No personas match.</p>
							<p class="text-xs text-(--color-subtle)">Try a different search term.</p>
						</div>
					{:else}
						<div
							class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4"
						>
							{#each app.filtered_personas as persona (persona.id)}
								<PersonaCard {persona} onedit={openPersonaEditEditor} ondelete={openPersonaDeleteDialog} />
							{/each}
						</div>
					{/if}
				</Tabs.Content>

				<!-- Lorebooks tab -->
				<Tabs.Content value="lorebooks">
					{#if app.lorebooks.length === 0}
						<div
							class="flex flex-col items-center justify-center gap-3 rounded-(--radius-lg) border border-dashed border-(--color-border) py-20 text-center"
						>
							<BookOpen class="size-8 text-(--color-subtle)" />
							<p class="text-sm text-(--color-muted)">No lorebooks yet.</p>
							<Button variant="secondary" size="sm" onclick={openLorebookCreateEditor}>
								<Plus class="size-3.5" />
								New lorebook
							</Button>
						</div>
					{:else if app.filtered_lorebooks.length === 0}
						<div
							class="flex flex-col items-center justify-center gap-3 rounded-(--radius-lg) border border-dashed border-(--color-border) py-20 text-center"
						>
							<BookOpen class="size-8 text-(--color-subtle)" />
							<p class="text-sm text-(--color-muted)">No lorebooks match.</p>
							<p class="text-xs text-(--color-subtle)">Try a different search term.</p>
						</div>
					{:else}
						<div
							class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4"
						>
							{#each app.filtered_lorebooks as lorebook (lorebook.id)}
								<LorebookCard {lorebook} onedit={openLorebookEditEditor} ondelete={openLorebookDeleteDialog} />
						{/each}
						</div>
					{/if}
				</Tabs.Content>
			</Tabs.Root>
		</div>
	</div>
</div>

<!-- Character editor dialog -->
<CharacterEditorDialog
	bind:open={editor_open}
	mode={editor_mode}
	character={editing_character}
/>

<!-- Persona editor dialog -->
<PersonaEditorDialog
	bind:open={persona_editor_open}
	mode={persona_editor_mode}
	persona={editing_persona}
/>

<!-- Lorebook editor dialog -->
<LorebookEditor
	bind:open={lorebook_editor_open}
	mode={lorebook_editor_mode}
	lorebook={editing_lorebook}
/>

<!-- Import error dialog -->
<ImportErrorDialog
	bind:open={import_error_open}
	message={import_error_message}
/>

<!-- Delete confirmation dialog -->
<DeleteConfirmationDialog
	bind:open={delete_open}
	title={deleting_entity_type === 'persona' ? 'Delete persona' : deleting_entity_type === 'lorebook' ? 'Delete lorebook' : 'Delete character'}
	description={`Are you sure you want to delete "${deleting_name}"? This action cannot be undone.`}
	confirm_label={deleting_entity_type === 'persona' ? 'Delete persona' : deleting_entity_type === 'lorebook' ? 'Delete lorebook' : 'Delete character'}
	onconfirm={handleDelete}
	loading={is_deleting}
/>

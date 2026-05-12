<script lang="ts">
	import { Tabs } from 'bits-ui';
	import { Plus, Search, Upload, Filter } from '@lucide/svelte';
	import { app } from '$lib/stores/app.svelte';
	import { Input, Button, Tooltip } from '$lib/components/ui';
	import CharacterCard from '$lib/components/characters/CharacterCard.svelte';
	import PersonaCard from '$lib/components/personas/PersonaCard.svelte';
	import { CharacterEditorDialog, DeleteConfirmationDialog } from '$lib/components/characters';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import type { CharacterCard as CharacterCardType } from '$lib/types';

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
	let is_deleting = $state(false);

	function openCreateEditor() {
		editor_mode = 'create';
		editing_character = undefined;
		editor_open = true;
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
			delete_open = true;
		}
	}

	async function handleDelete() {
		if (!deleting_id) return;
		is_deleting = true;
		try {
			await app.deleteCharacter(deleting_id);
			delete_open = false;
			deleting_id = null;
		} catch (err) {
			console.error('Failed to delete character:', err);
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
				<Button variant="secondary" size="sm">
					<Upload class="size-3.5" />
					Import card
				</Button>
				<Button variant="primary" size="sm" onclick={openCreateEditor}>
					<Plus class="size-3.5" />
					New {tab === 'personas' ? 'persona' : 'character'}
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
						<Tooltip text="Filter by tag">
							<button
								type="button"
								class="inline-flex size-9 items-center justify-center rounded-(--radius-md) border border-(--color-border) text-(--color-muted) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
								aria-label="Filter"
							>
								<Filter class="size-4" />
							</button>
						</Tooltip>
					</div>
				</div>

				<!-- Characters tab -->
				<Tabs.Content value="characters">
					{#if app.filtered_characters.length === 0}
						<div
							class="flex flex-col items-center justify-center gap-3 rounded-(--radius-lg) border border-dashed border-(--color-border) py-20 text-center"
						>
							<p class="text-sm text-(--color-muted)">No characters match.</p>
						</div>
					{:else}
						<div
							class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4"
						>
							{#each app.filtered_characters as character (character.id)}
								<CharacterCard {character} onedit={openEditEditor} ondelete={openDeleteDialog} />
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
						</div>
					{:else}
						<div
							class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4"
						>
							{#each app.personas as persona (persona.id)}
								<PersonaCard {persona} />
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

<!-- Delete confirmation dialog -->
<DeleteConfirmationDialog
	bind:open={delete_open}
	title="Delete character"
	description={`Are you sure you want to delete "${deleting_name}"? This action cannot be undone.`}
	confirm_label="Delete character"
	onconfirm={handleDelete}
	loading={is_deleting}
/>

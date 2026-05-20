<script lang="ts">
	import type { CharacterCard, Lorebook } from '$lib/types';
	import { app } from '$lib/stores/app.svelte';
	import {
		Dialog,
		Button,
		Input,
		Textarea,
		AvatarUpload,
		StepperHeader
	} from '$lib/components/ui';
	import { DeleteConfirmationDialog } from '$lib/components/characters';
	import RegexScriptEditor from './RegexScriptEditor.svelte';
	import {
		ChevronLeft,
		ChevronRight,
		AlertCircle,
		BookOpen,
		X,
		Plus
	} from '@lucide/svelte';

	interface Props {
		open?: boolean;
		onopenchange?: (open: boolean) => void;
		/** 'create' for new characters, 'edit' for existing */
		mode?: 'create' | 'edit';
		/** Character to edit (required for edit mode) */
		character?: CharacterCard;
	}

	let {
		open = $bindable(false),
		onopenchange,
		mode = 'create',
		character
	}: Props = $props();

	// ── Form state ────────────────────────────────────────────────────────
	let pending_id = $state<string | null>(null);
	let current_step = $state(0);
	let form_name = $state('');
	let form_avatar_path = $state<string | undefined>(undefined);
	let form_description = $state('');
	let form_personality = $state('');
	let form_scenario = $state('');
	let form_first_mes = $state('');
	let form_mes_example = $state('');
	let form_system_prompt = $state('');
	let form_post_history_instructions = $state('');
	let form_creator_notes = $state('');
	let form_creator = $state('');
	let form_lorebooks = $state<Lorebook[]>([]);
	let lorebook_search = $state('');
	let is_saving = $state(false);
	let name_error = $state('');
	let backend_error = $state('');
	let discard_open = $state(false);
	let initial_snapshot = $state('');
	let lorebooks_loading = $state(false);

	const steps = [
		{ label: 'Profile', state: 'upcoming' as const },
		{ label: 'Prompting', state: 'upcoming' as const },
		{ label: 'Lorebooks', state: 'upcoming' as const },
		{ label: 'Regex', state: 'upcoming' as const }
	];

	// ── Derived step states ──────────────────────────────────────────────
	let computed_steps = $derived(
		steps.map((s, i) => ({
			...s,
			state:
				i < current_step
					? ('completed' as const)
					: i === current_step
						? ('current' as const)
						: ('upcoming' as const)
		}))
	);

	let can_advance = $derived.by(() => {
		if (current_step === 0) return form_name.trim().length > 0;
		return true;
	});

	let is_last_step = $derived(current_step === steps.length - 1);

	// ── Lorebook helpers ──────────────────────────────────────────────────
	let available_lorebooks = $derived.by(() => {
		const assigned_ids = new Set(form_lorebooks.map((l) => l.id));
		let pool = app.lorebooks.filter((l) => !assigned_ids.has(l.id));
		if (lorebook_search.trim()) {
			const q = lorebook_search.toLowerCase();
			pool = pool.filter(
				(l) =>
					l.name.toLowerCase().includes(q) ||
					l.description.toLowerCase().includes(q)
			);
		}
		return pool;
	});

	async function loadCharacterLorebooks() {
		if (!pending_id || mode !== 'edit') {
			form_lorebooks = [];
			return;
		}
		lorebooks_loading = true;
		try {
			form_lorebooks = await app.getCharacterLorebooks(pending_id);
		} catch (err) {
			console.error('Failed to load character lorebooks:', err);
			form_lorebooks = [];
		} finally {
			lorebooks_loading = false;
		}
	}

	async function addLorebook(lorebook: Lorebook) {
		if (!pending_id) return;
		try {
			await app.assignLorebookToCharacter(
				pending_id,
				lorebook.id,
				form_lorebooks.length
			);
			form_lorebooks = [...form_lorebooks, lorebook];
		} catch (err) {
			backend_error =
				err instanceof Error ? err.message : 'Failed to assign lorebook.';
		}
	}

	async function removeLorebook(lorebookId: string) {
		if (!pending_id) return;
		try {
			await app.removeLorebookFromCharacter(pending_id, lorebookId);
			form_lorebooks = form_lorebooks.filter((l) => l.id !== lorebookId);
		} catch (err) {
			backend_error =
				err instanceof Error ? err.message : 'Failed to remove lorebook.';
		}
	}

	// ── Dirty tracking ────────────────────────────────────────────────────
	function snapshot(): string {
		return JSON.stringify({
			form_name,
			form_avatar_path,
			form_description,
			form_personality,
			form_scenario,
			form_first_mes,
			form_mes_example,
			form_system_prompt,
			form_post_history_instructions,
			form_creator_notes,
			form_creator,
			form_lorebooks: form_lorebooks.map((l) => l.id)
		});
	}

	let is_dirty = $derived(snapshot() !== initial_snapshot);

	// ── Initialize form from character ────────────────────────────────────
	// Only re-initialize when the dialog *opens* (open transitions to true),
	// not when the character prop mutates mid-dialog (e.g. after a save).
	let was_open = $state(false);

	$effect(() => {
		const just_opened = open && !was_open;
		if (just_opened) {
			if (mode === 'edit' && character) {
				pending_id = character.id;
				form_name = character.name;
				form_avatar_path = character.avatar_path;
				form_description = character.description;
				form_personality = character.personality;
				form_scenario = character.scenario;
				form_first_mes = character.first_mes;
				form_mes_example = character.mes_example;
				form_system_prompt = character.system_prompt;
				form_post_history_instructions = character.post_history_instructions;
				form_creator_notes = character.creator_notes;
				form_creator = character.creator;
				loadCharacterLorebooks();
			} else {
				pending_id = crypto.randomUUID();
				form_name = '';
				form_avatar_path = undefined;
				form_description = '';
				form_personality = '';
				form_scenario = '';
				form_first_mes = '';
				form_mes_example = '';
				form_system_prompt = '';
				form_post_history_instructions = '';
				form_creator_notes = '';
				form_creator = '';
				form_lorebooks = [];
			}
			current_step = 0;
			lorebook_search = '';
			name_error = '';
			backend_error = '';
			initial_snapshot = snapshot();
		}
		was_open = open;
	});

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

	function validateName() {
		if (!form_name.trim()) {
			name_error = 'Name is required';
			return false;
		}
		name_error = '';
		return true;
	}

	function nextStep() {
		if (current_step === 0 && !validateName()) return;
		if (current_step < steps.length - 1) {
			current_step++;
		}
	}

	function prevStep() {
		if (current_step > 0) {
			current_step--;
		}
	}

	async function handleSave() {
		if (is_saving) return;
		if (!validateName()) {
			current_step = 0;
			return;
		}

		is_saving = true;
		try {
			if (mode === 'edit' && character) {
				await app.updateCharacter({
					...character,
					name: form_name,
					avatar_path: form_avatar_path,
					description: form_description,
					personality: form_personality,
					scenario: form_scenario,
					first_mes: form_first_mes,
					mes_example: form_mes_example,
					system_prompt: form_system_prompt,
					post_history_instructions: form_post_history_instructions,
					creator_notes: form_creator_notes,
					creator: form_creator
				});
			} else {
				await app.createCharacter({
					id: pending_id!,
					name: form_name,
					avatar_path: form_avatar_path,
					description: form_description,
					personality: form_personality,
					scenario: form_scenario,
					first_mes: form_first_mes,
					mes_example: form_mes_example,
					system_prompt: form_system_prompt,
					post_history_instructions: form_post_history_instructions,
					creator_notes: form_creator_notes,
					creator: form_creator,
					character_version: '2.0',
					tags: [],
					alternate_greetings: [],
					lorebook_scan_depth: 2,
					lorebook_recursive_depth: 2
				});
			}
			open = false;
			onopenchange?.(false);
		} catch (err) {
			backend_error =
				err instanceof Error ? err.message : 'Failed to save character.';
		} finally {
			is_saving = false;
		}
	}

	async function handleAvatarSelect(sourcePath: string) {
		try {
			const avatarPath = await app.uploadAvatar(pending_id!, sourcePath);
			form_avatar_path = avatarPath;
		} catch (err) {
			backend_error =
				err instanceof Error ? err.message : 'Failed to upload avatar.';
		}
	}
</script>

<Dialog {open} onopenchange={handleOpenChange} class="max-w-2xl">
	<StepperHeader
		steps={computed_steps}
		class="mb-6"
		onstepclick={mode === 'edit' ? (i: number) => { current_step = i; } : undefined}
	/>

	{#if backend_error}
		<div class="mb-4 flex items-center gap-2 rounded-(--radius-md) border border-(--color-destructive)/30 bg-(--color-destructive)/10 px-3 py-2 text-sm text-(--color-destructive)">
			<AlertCircle class="size-4 shrink-0" />
			{backend_error}
		</div>
	{/if}

	<!-- Step 1: Profile (identity + personality) -->
	{#if current_step === 0}
		<div class="space-y-4">
			<div class="flex items-start gap-4">
				<AvatarUpload
					src={form_avatar_path}
					name={form_name || 'New'}
					size="xl"
					onselect={handleAvatarSelect}
				/>
				<div class="flex-1 space-y-1">
					<label class="text-sm font-medium text-(--color-foreground)">Name *</label>
					<Input
						bind:value={form_name}
						placeholder="Character name"
						class={name_error ? 'border-(--color-destructive)' : ''}
					/>
					{#if name_error}
						<p class="text-xs text-(--color-destructive)">{name_error}</p>
					{/if}
				</div>
			</div>

			<div class="space-y-1">
				<label class="text-sm font-medium text-(--color-foreground)">Creator</label>
				<Input bind:value={form_creator} placeholder="Who made this character" />
			</div>

			<div class="space-y-1">
				<label class="text-sm font-medium text-(--color-foreground)">Description</label>
				<Textarea bind:value={form_description} placeholder="Brief description of the character" autogrow />
			</div>

			<div class="space-y-1">
				<label class="text-sm font-medium text-(--color-foreground)">Personality</label>
				<Textarea bind:value={form_personality} placeholder="Character's personality traits and behaviors" autogrow rows={3} />
			</div>

			<div class="space-y-1">
				<label class="text-sm font-medium text-(--color-foreground)">Scenario</label>
				<Textarea bind:value={form_scenario} placeholder="Setting and context for the roleplay" autogrow rows={2} />
			</div>
		</div>

	<!-- Step 2: Prompting (prompt config + greeting) -->
	{:else if current_step === 1}
		<div class="space-y-4">
			<div class="space-y-1">
				<label class="text-sm font-medium text-(--color-foreground)">System Prompt</label>
				<Textarea bind:value={form_system_prompt} placeholder="System-level instructions for the AI" autogrow rows={3} />
			</div>

			<div class="space-y-1">
				<label class="text-sm font-medium text-(--color-foreground)">Post-History Instructions</label>
				<Textarea bind:value={form_post_history_instructions} placeholder="Instructions appended after the chat history" autogrow rows={2} />
			</div>

			<div class="space-y-1">
				<label class="text-sm font-medium text-(--color-foreground)">First Message</label>
				<Textarea bind:value={form_first_mes} placeholder="The character's opening message" autogrow rows={3} />
			</div>

			<div class="space-y-1">
				<label class="text-sm font-medium text-(--color-foreground)">Example Messages</label>
				<Textarea bind:value={form_mes_example} placeholder="Example dialogue between user and character" autogrow rows={3} />
			</div>

			<div class="space-y-1">
				<label class="text-sm font-medium text-(--color-foreground)">Creator Notes</label>
				<Textarea bind:value={form_creator_notes} placeholder="Notes from the creator (not used in prompts)" autogrow rows={2} />
			</div>
		</div>

	<!-- Step 3: Lorebooks -->
	{:else if current_step === 2}
		<div class="space-y-4">
			{#if mode === 'create'}
				<p class="text-sm text-(--color-muted)">
					Lorebooks can be attached after the character is created. Save this character first, then edit it to assign lorebooks.
				</p>
			{:else}
				<!-- Assigned lorebooks -->
				<div class="space-y-2">
					<label class="text-sm font-medium text-(--color-foreground)">Attached Lorebooks</label>
					{#if lorebooks_loading}
						<p class="text-sm text-(--color-muted)">Loading…</p>
					{:else if form_lorebooks.length === 0}
						<p class="text-sm text-(--color-muted)">No lorebooks attached. Add one from the list below.</p>
					{:else}
						<div class="space-y-1.5">
							{#each form_lorebooks as lorebook (lorebook.id)}
								<div class="flex items-center gap-2 overflow-hidden rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-elevated) px-3 py-2">
									<BookOpen class="size-4 shrink-0 text-(--color-muted)" />
									<span class="min-w-0 shrink truncate text-sm font-medium">{lorebook.name}</span>
									{#if lorebook.description}
										<span class="min-w-0 flex-1 truncate text-xs text-(--color-muted)">{lorebook.description}</span>
									{/if}
									<button
										type="button"
										class="shrink-0 rounded-(--radius-sm) p-1 text-(--color-muted) hover:bg-(--color-surface-hover) hover:text-(--color-destructive)"
										onclick={() => removeLorebook(lorebook.id)}
										aria-label="Remove lorebook"
									>
										<X class="size-3.5" />
									</button>
								</div>
							{/each}
						</div>
					{/if}
				</div>

				<!-- Add lorebook -->
				<div class="space-y-2">
					<label class="text-sm font-medium text-(--color-foreground)">Add Lorebook</label>
					<Input
						bind:value={lorebook_search}
						placeholder="Search lorebooks…"
					/>
					{#if available_lorebooks.length === 0}
						<p class="text-xs text-(--color-muted)">
							{#if lorebook_search.trim()}
								No matching lorebooks found.
							{:else}
								All existing lorebooks are already attached.
							{/if}
						</p>
					{:else}
						<div class="max-h-48 space-y-1.5 overflow-y-auto">
							{#each available_lorebooks as lorebook (lorebook.id)}
								<button
									type="button"
									class="flex w-full items-center gap-2 overflow-hidden rounded-(--radius-md) border border-(--color-border) bg-(--color-surface) px-3 py-2 text-left transition-colors hover:bg-(--color-surface-hover)"
									onclick={() => addLorebook(lorebook)}
								>
									<Plus class="size-4 shrink-0 text-(--color-accent)" />
									<span class="min-w-0 shrink truncate text-sm">{lorebook.name}</span>
									{#if lorebook.description}
										<span class="min-w-0 flex-1 truncate text-xs text-(--color-muted)">{lorebook.description}</span>
									{/if}
								</button>
							{/each}
						</div>
					{/if}
				</div>
				{/if}
		</div>

	<!-- Step 4: Regex -->
	{:else if current_step === 3}
		<RegexScriptEditor
			characterId={pending_id ?? character?.id ?? ''}
			createMode={mode === 'create'}
		/>
	{/if}

	{#snippet footer()}
		<div class="flex w-full items-center justify-between">
			<div>
				{#if current_step > 0}
					<Button variant="ghost" size="sm" onclick={prevStep}>
						<ChevronLeft class="size-4" />
						Back
					</Button>
				{/if}
			</div>
			<div class="flex items-center gap-2">
				<Button variant="secondary" size="sm" onclick={() => handleOpenChange(false)}>
					Cancel
				</Button>
				{#if is_last_step}
					<Button variant="primary" size="sm" onclick={handleSave} disabled={is_saving}>
						{is_saving ? 'Saving…' : mode === 'edit' ? 'Save changes' : 'Create character'}
					</Button>
				{:else}
					<Button variant="primary" size="sm" onclick={nextStep} disabled={!can_advance}>
						Next
						<ChevronRight class="size-4" />
					</Button>
				{/if}
			</div>
		</div>
	{/snippet}
</Dialog>

<DeleteConfirmationDialog
	bind:open={discard_open}
	title="Discard changes?"
	description="You have unsaved changes. Are you sure you want to close?"
	confirm_label="Discard"
	onconfirm={handleDiscardConfirm}
/>

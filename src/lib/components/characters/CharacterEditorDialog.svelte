<script lang="ts">
	import type { CharacterCard } from '$lib/types';
	import { app } from '$lib/stores/app.svelte';
	import { untrack } from 'svelte';
	import {
		Dialog,
		Button,
		Input,
		Textarea,
		AvatarUpload,
		StepperHeader
	} from '$lib/components/ui';
	import { ChevronLeft, ChevronRight } from '@lucide/svelte';

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
	let is_saving = $state(false);
	let name_error = $state('');

	const steps = [
		{ label: 'Identity', state: 'upcoming' as const },
		{ label: 'Personality', state: 'upcoming' as const },
		{ label: 'Prompt', state: 'upcoming' as const },
		{ label: 'Greeting', state: 'upcoming' as const }
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
			}
			current_step = 0;
			name_error = '';
		}
		was_open = open;
	});

	function handleOpenChange(value: boolean) {
		open = value;
		onopenchange?.(value);
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
				const created = await app.createCharacter(form_name, pending_id!);
				if (created) {
					await app.updateCharacter({
						...created,
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
				}
			}
			handleOpenChange(false);
		} catch (err) {
			console.error('Failed to save character:', err);
		} finally {
			is_saving = false;
		}
	}

	async function handleAvatarSelect(sourcePath: string) {
		try {
			const avatarPath = await app.uploadAvatar(pending_id!, sourcePath);
			form_avatar_path = avatarPath;
		} catch (err) {
			console.error('Failed to upload avatar:', err);
		}
	}
</script>

<Dialog {open} onopenchange={handleOpenChange} class="max-w-2xl">
	<StepperHeader steps={computed_steps} class="mb-6" />

	<!-- Step 1: Identity -->
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
		</div>

	<!-- Step 2: Personality -->
	{:else if current_step === 1}
		<div class="space-y-4">
			<div class="space-y-1">
				<label class="text-sm font-medium text-(--color-foreground)">Personality</label>
				<Textarea bind:value={form_personality} placeholder="Character's personality traits and behaviors" autogrow rows={4} />
			</div>

			<div class="space-y-1">
				<label class="text-sm font-medium text-(--color-foreground)">Scenario</label>
				<Textarea bind:value={form_scenario} placeholder="Setting and context for the roleplay" autogrow rows={3} />
			</div>
		</div>

	<!-- Step 3: Prompt -->
	{:else if current_step === 2}
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
				<label class="text-sm font-medium text-(--color-foreground)">Creator Notes</label>
				<Textarea bind:value={form_creator_notes} placeholder="Notes from the creator (not used in prompts)" autogrow rows={2} />
			</div>
		</div>

	<!-- Step 4: Greeting -->
	{:else if current_step === 3}
		<div class="space-y-4">
			<div class="space-y-1">
				<label class="text-sm font-medium text-(--color-foreground)">First Message</label>
				<Textarea bind:value={form_first_mes} placeholder="The character's opening message" autogrow rows={4} />
			</div>

			<div class="space-y-1">
				<label class="text-sm font-medium text-(--color-foreground)">Example Messages</label>
				<Textarea bind:value={form_mes_example} placeholder="Example dialogue between user and character" autogrow rows={4} />
			</div>
		</div>
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

<script lang="ts">
	import type { UserPersona } from '$lib/types';
	import { app } from '$lib/stores/app.svelte';
	import {
		Dialog,
		Button,
		Input,
		Textarea,
		AvatarUpload
	} from '$lib/components/ui';
	import { AlertCircle } from '@lucide/svelte';
	import { DeleteConfirmationDialog } from '$lib/components/characters';

	interface Props {
		open?: boolean;
		onopenchange?: (open: boolean) => void;
		/** 'create' for new personas, 'edit' for existing */
		mode?: 'create' | 'edit';
		/** Persona to edit (required for edit mode) */
		persona?: UserPersona;
	}

	let {
		open = $bindable(false),
		onopenchange,
		mode = 'create',
		persona
	}: Props = $props();

	// ── Form state ────────────────────────────────────────────────────────
	let pending_id = $state<string | null>(null);
	let form_name = $state('');
	let form_avatar_path = $state<string | undefined>(undefined);
	let form_appearance = $state('');
	let form_personality = $state('');
	let form_background = $state('');
	let form_role_description = $state('');
	let form_relationships = $state('');
	let form_speech_style = $state('');
	let form_tags = $state('');
	let is_saving = $state(false);
	let name_error = $state('');
	let backend_error = $state('');
	let discard_open = $state(false);

	// ── Dirty tracking ────────────────────────────────────────────────────
	let initial_snapshot = $state('');

	function snapshot(): string {
		return JSON.stringify({
			form_name,
			form_avatar_path,
			form_appearance,
			form_personality,
			form_background,
			form_role_description,
			form_relationships,
			form_speech_style,
			form_tags
		});
	}

	let is_dirty = $derived(snapshot() !== initial_snapshot);

	// ── Initialize form from persona ────────────────────────────────────────
	// Only re-initialize when the dialog *opens* (open transitions to true),
	// not when the persona prop mutates mid-dialog (e.g. after a save).
	let was_open = $state(false);

	$effect(() => {
		const just_opened = open && !was_open;
		if (just_opened) {
			if (mode === 'edit' && persona) {
				pending_id = persona.id;
				form_name = persona.name;
				form_avatar_path = persona.avatar_path;
				form_appearance = persona.appearance;
				form_personality = persona.personality;
				form_background = persona.background;
				form_role_description = persona.role_description;
				form_relationships = persona.relationships;
				form_speech_style = persona.speech_style;
				form_tags = persona.tags.join(', ');
			} else {
				pending_id = crypto.randomUUID();
				form_name = '';
				form_avatar_path = undefined;
				form_appearance = '';
				form_personality = '';
				form_background = '';
				form_role_description = '';
				form_relationships = '';
				form_speech_style = '';
				form_tags = '';
			}
			name_error = '';
			backend_error = '';
			// Capture initial snapshot after populating.
			// Reading form fields here doesn't re-trigger the effect
			// because the effect only runs on open transitions.
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

	function validateName(): boolean {
		if (!form_name.trim()) {
			name_error = 'Name is required.';
			return false;
		}
		name_error = '';
		return true;
	}

	function parseTags(tagsStr: string): string[] {
		return tagsStr
			.split(',')
			.map((t) => t.trim())
			.filter((t) => t.length > 0);
	}

	async function handleSave() {
		if (is_saving) return;
		if (!validateName()) return;

		is_saving = true;
		backend_error = '';

		try {
			if (mode === 'edit' && persona) {
				await app.updatePersona({
					...persona,
					name: form_name,
					avatar_path: form_avatar_path,
					appearance: form_appearance,
					personality: form_personality,
					background: form_background,
					role_description: form_role_description,
					relationships: form_relationships,
					speech_style: form_speech_style,
					tags: parseTags(form_tags)
				});
			} else {
				await app.createPersona({
					id: pending_id!,
					name: form_name,
					avatar_path: form_avatar_path,
					appearance: form_appearance,
					personality: form_personality,
					background: form_background,
					role_description: form_role_description,
					relationships: form_relationships,
					speech_style: form_speech_style,
					tags: parseTags(form_tags)
				});
			}
		open = false;
		onopenchange?.(false);
	} catch (err) {
		backend_error =
			err instanceof Error ? err.message : 'Failed to save persona.';
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

<Dialog
	{open}
	onopenchange={handleOpenChange}
	title={mode === 'edit' ? 'Edit persona' : 'New persona'}
	description={mode === 'edit' ? 'Update your persona details.' : 'Create a new persona for roleplay.'}
	class="max-w-2xl"
>
	<div class="max-h-[60vh] space-y-4 overflow-y-auto pr-1">
		<!-- Error banner -->
		{#if backend_error}
			<div class="flex items-center gap-2 rounded-(--radius-md) border border-(--color-destructive)/30 bg-(--color-destructive)/10 px-3 py-2 text-sm text-(--color-destructive)">
				<AlertCircle class="size-4 shrink-0" />
				{backend_error}
			</div>
		{/if}

		<!-- Avatar + Name row -->
		<div class="flex items-start gap-4">
			<AvatarUpload
				src={form_avatar_path}
				name={form_name || 'New'}
				size="xl"
				onselect={handleAvatarSelect}
				disabled={is_saving}
			/>
			<div class="flex-1 space-y-1">
				<label class="text-sm font-medium text-(--color-foreground)">Name *</label>
				<Input
					bind:value={form_name}
					placeholder="Persona name"
					disabled={is_saving}
					class={name_error ? 'border-(--color-destructive)' : ''}
				/>
				{#if name_error}
					<p class="text-xs text-(--color-destructive)">{name_error}</p>
				{/if}
			</div>
		</div>

		<!-- Appearance -->
		<div class="space-y-1">
			<label class="text-sm font-medium text-(--color-foreground)">Appearance</label>
			<Textarea
				bind:value={form_appearance}
				placeholder="Physical description of the persona"
				autogrow
				disabled={is_saving}
			/>
		</div>

		<!-- Personality -->
		<div class="space-y-1">
			<label class="text-sm font-medium text-(--color-foreground)">Personality</label>
			<Textarea
				bind:value={form_personality}
				placeholder="Personality traits and behaviors"
				autogrow
				rows={3}
				disabled={is_saving}
			/>
		</div>

		<!-- Background -->
		<div class="space-y-1">
			<label class="text-sm font-medium text-(--color-foreground)">Background</label>
			<Textarea
				bind:value={form_background}
				placeholder="Backstory and history"
				autogrow
				disabled={is_saving}
			/>
		</div>

		<!-- Role description -->
		<div class="space-y-1">
			<label class="text-sm font-medium text-(--color-foreground)">Role description</label>
			<Input
				bind:value={form_role_description}
				placeholder="e.g. Mage, Knight, Scholar"
				disabled={is_saving}
			/>
		</div>

		<!-- Relationships -->
		<div class="space-y-1">
			<label class="text-sm font-medium text-(--color-foreground)">Relationships</label>
			<Textarea
				bind:value={form_relationships}
				placeholder="Key relationships with other characters"
				autogrow
				disabled={is_saving}
			/>
		</div>

		<!-- Speech style -->
		<div class="space-y-1">
			<label class="text-sm font-medium text-(--color-foreground)">Speech style</label>
			<Textarea
				bind:value={form_speech_style}
				placeholder="How the persona talks — vocabulary, tone, mannerisms"
				autogrow
				disabled={is_saving}
			/>
		</div>

		<!-- Tags -->
		<div class="space-y-1">
			<label class="text-sm font-medium text-(--color-foreground)">Tags</label>
			<Input
				bind:value={form_tags}
				placeholder="Comma-separated tags"
				disabled={is_saving}
			/>
		</div>
	</div>

	{#snippet footer()}
		<div class="flex items-center gap-2">
			<Button variant="secondary" size="sm" onclick={() => handleOpenChange(false)} disabled={is_saving}>
				Cancel
			</Button>
			<Button variant="primary" size="sm" onclick={handleSave} disabled={is_saving}>
				{is_saving ? 'Saving…' : mode === 'edit' ? 'Save changes' : 'Create persona'}
			</Button>
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

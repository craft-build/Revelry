<script lang="ts">
	import { app } from '$lib/stores/app.svelte';
	import { Button, Avatar, Dialog } from '$lib/components/ui';
	import { Brain, Eraser } from '@lucide/svelte';
	import { goto } from '$app/navigation';

	let open = $derived(app.memory_prompt !== null);
	let busy = $state(false);

	async function createConversation(options: {
		fresh: boolean;
		forget?: boolean;
	}): Promise<void> {
		const prompt = app.memory_prompt;
		if (!prompt || busy) return;
		busy = true;
		try {
			if (options.forget) {
				await app.setCharacterMemoryForgotten(prompt.character.id, true);
			}
			const conversation = await app.startConversation(prompt.character.id);
			if (options.fresh) {
				await app.setConversationMemoryEnabled(conversation.id, false);
			}
			app.dismissMemoryPrompt();
			await goto('/chat/' + conversation.id);
		} catch (err) {
			console.error('Failed to start conversation:', err);
			app.dismissMemoryPrompt();
		} finally {
			busy = false;
		}
	}
</script>

<Dialog
	{open}
	onopenchange={(value) => {
		if (!value) app.dismissMemoryPrompt();
	}}
	title="Carry over memories?"
	class="max-w-sm"
>
	{#if app.memory_prompt}
		{@const character = app.memory_prompt.character}
		<div class="flex items-center gap-3">
			<Avatar name={character.name} src={character.avatar_path} size="md" />
			<p class="truncate text-sm text-(--color-muted)">
				{character.name} · {app.memory_prompt.other_count}
				{app.memory_prompt.other_count === 1 ? 'other chat' : 'other chats'} with you
			</p>
		</div>

		<p class="mt-4 text-sm leading-relaxed text-(--color-muted)">
			Cross-conversation memory is on. {character.name} can recall moments from your other
			conversations together, matched by meaning. Start this chat connected, or fresh.
		</p>

		<div class="mt-5 flex flex-col gap-2">
			<Button variant="primary" disabled={busy} onclick={() => createConversation({ fresh: false })}>
				<Brain class="size-4" />
				Let {character.name} remember
			</Button>
			<Button
				variant="secondary"
				disabled={busy}
				onclick={() => createConversation({ fresh: true })}
			>
				Start fresh
			</Button>
			<button
				type="button"
				disabled={busy}
				onclick={() => createConversation({ fresh: true, forget: true })}
				class="mt-1 inline-flex items-center justify-center gap-1.5 rounded-(--radius-lg) px-3 py-2 text-xs text-(--color-destructive) transition-colors hover:bg-(--color-destructive)/10 disabled:opacity-50"
			>
				<Eraser class="size-3.5" />
				Start fresh and forget everything they remember
			</button>
			<button
				type="button"
				disabled={busy}
				onclick={() => app.dismissMemoryPrompt()}
				class="mt-0.5 rounded-(--radius-lg) px-3 py-1.5 text-xs text-(--color-subtle) transition-colors hover:text-(--color-foreground) disabled:opacity-50"
			>
				Cancel
			</button>
		</div>
	{/if}
</Dialog>

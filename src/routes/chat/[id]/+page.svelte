<script lang="ts">
	import { tick } from 'svelte';
	import { app } from '$lib/stores/app.svelte';
	import type { MessageBranch } from '$lib/types';
	import { getMessages } from '$lib/data/api';
	import ChatHeader from '$lib/components/chat/ChatHeader.svelte';
	import MessageBubble from '$lib/components/chat/MessageBubble.svelte';
	import MessageInput from '$lib/components/chat/MessageInput.svelte';

	interface Props {
		data: { id: string };
	}

	let { data }: Props = $props();

	let scrollEl: HTMLDivElement | undefined = $state();

	// Hydrate the active conversation when the route id changes.
	$effect(() => {
		getMessages(data.id).then((msgs) => {
			app.openConversation(data.id, msgs);
		}).catch((err) => {
			console.error('Failed to load messages:', err);
			app.openConversation(data.id, []);
		});
	});

	// Auto-scroll to bottom when messages change
	$effect(() => {
		// Touch the length so the effect re-runs on append/edit
		void app.messages.length;
		void app.messages.at(-1)?.content;
		tick().then(() => {
			scrollEl?.scrollTo({ top: scrollEl.scrollHeight, behavior: 'smooth' });
		});
	});

	function handleSend(text: string) {
		app.startChat(text);
	}

	function handleCancel() {
		app.stopGeneration();
	}

	let lastAiMessageId = $derived(
		app.messages
			.filter((m) => m.role === 'assistant' && m.metadata?.stop_reason !== 'error')
			.at(-1)?.id,
	);

	// Map parent_message_id -> branches for passing to MessageBubble
	let branchesByParentId = $derived(
		(() => {
			const map = new Map<string, MessageBranch[]>();
			for (const branch of app.branches) {
				const existing = map.get(branch.parent_message_id) ?? [];
				existing.push(branch);
				map.set(branch.parent_message_id, existing);
			}
			return map;
		})(),
	);
</script>

<svelte:head>
	<title>{app.active_character?.name ?? 'Chat'} — Revelry</title>
</svelte:head>

<ChatHeader />

<!-- Message list -->
<div bind:this={scrollEl} class="flex-1 overflow-y-auto px-6 py-6">
	<div class="mx-auto flex max-w-[80ch] flex-col gap-5">
		{#if app.messages.length === 0}
			<div class="py-20 text-center text-sm text-(--color-muted)">
				No messages yet. Send something to begin.
			</div>
		{:else}
			{#each app.messages as message (message.id)}
				<div id="message-{message.id}" data-message-id={message.id}>
					<MessageBubble
						{message}
						character={app.active_character}
						persona={app.active_persona}
						isLastAiMessage={message.id === lastAiMessageId}
						branchesAtMessage={branchesByParentId.get(message.id) ?? []}
					/>
				</div>
			{/each}
		{/if}
	</div>
</div>

<MessageInput onsend={handleSend} oncancel={handleCancel} />

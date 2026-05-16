<script lang="ts">
	import { tick } from 'svelte';
	import { app } from '$lib/stores/app.svelte';
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
				<MessageBubble
					{message}
					character={app.active_character}
					persona={app.active_persona}
				/>
			{/each}
		{/if}
	</div>
</div>

<MessageInput onsend={handleSend} oncancel={handleCancel} />

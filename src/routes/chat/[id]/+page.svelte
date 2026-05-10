<script lang="ts">
	import { tick } from 'svelte';
	import { app } from '$lib/stores/app.svelte';
	import { getMessages } from '$lib/data/mock';
	import ChatHeader from '$lib/components/chat/ChatHeader.svelte';
	import MessageBubble from '$lib/components/chat/MessageBubble.svelte';
	import MessageInput from '$lib/components/chat/MessageInput.svelte';
	import type { Message } from '$lib/types';

	interface Props {
		data: { id: string };
	}

	let { data }: Props = $props();

	let scrollEl: HTMLDivElement | undefined = $state();

	// Hydrate the active conversation when the route id changes.
	// In production this would dispatch a LoadConversation message that
	// queries the SQLite db for messages along the active branch.
	$effect(() => {
		app.openConversation(data.id, getMessages(data.id));
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

	/**
	 * Mock send handler — in the real app this dispatches a SendMessage
	 * message that the chat-engine module turns into a rig streaming
	 * completion. Here we fake the streaming so the UI is testable.
	 */
	function handleSend(text: string) {
		if (!app.active_conversation_id) return;

		const userMsg: Message = {
			id: `msg_${Date.now()}_u`,
			conversation_id: app.active_conversation_id,
			parent_id: app.messages.at(-1)?.id ?? null,
			branch_index: 0,
			role: 'user',
			content: text,
			created_at: new Date().toISOString()
		};
		app.appendMessage(userMsg);

		// Begin a streaming response
		const aiId = `msg_${Date.now()}_a`;
		const aiMsg: Message = {
			id: aiId,
			conversation_id: app.active_conversation_id,
			parent_id: userMsg.id,
			branch_index: 0,
			role: 'character',
			content: '',
			is_streaming: true,
			metadata: { model: 'claude-opus-4-7' },
			created_at: new Date().toISOString()
		};
		app.appendMessage(aiMsg);
		app.is_generating = true;

		// Simulated token stream
		const sample =
			'A measured pause. *She turns from the kettle, finally meeting your eye, and something in her expression has softened by a fraction.* You may sit, if you wish. The chair by the fire — yes, that one. Mind the loose joint in the back leg.';
		const chunks = sample.match(/.{1,4}/g) ?? [sample];
		let acc = '';
		let i = 0;
		const streamTick = () => {
			if (i >= chunks.length) {
				app.updateStreamingMessage(aiId, acc, true);
				return;
			}
			acc += chunks[i++];
			app.updateStreamingMessage(aiId, acc);
			setTimeout(streamTick, 30);
		};
		setTimeout(streamTick, 200);
	}

	function handleCancel() {
		const last = app.messages.at(-1);
		if (last?.is_streaming) {
			app.updateStreamingMessage(last.id, last.content, true);
		}
		app.is_generating = false;
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

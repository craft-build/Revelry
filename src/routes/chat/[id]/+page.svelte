<script lang="ts">
	import { tick, untrack } from 'svelte';
	import { app } from '$lib/stores/app.svelte';
	import type { MessageBranch } from '$lib/types';
	import { getMessages } from '$lib/data/api';
	import ChatHeader from '$lib/components/chat/ChatHeader.svelte';
	import MessageBubble from '$lib/components/chat/MessageBubble.svelte';
	import MessageInput from '$lib/components/chat/MessageInput.svelte';
	import GenerationPanel from '$lib/components/chat/GenerationPanel.svelte';

	interface Props {
		data: { id: string };
	}

	let { data }: Props = $props();

	let scrollEl: HTMLDivElement | undefined = $state();

	// Hydrate the active conversation when the route id changes.
	// Read branch state untracked so branch switches don't re-fetch
	// (switchBranch already updates messages directly).
	$effect(() => {
		const convId = data.id;
		const currentBranchId = untrack(() =>
			app.active_conversation_id === convId ? app.active_branch_id : null,
		);
		getMessages(convId, currentBranchId).then((msgs) => {
			app.openConversation(convId, msgs, currentBranchId);
		}).catch((err) => {
			console.error('Failed to load messages:', err);
			app.openConversation(convId, [], null);
		});
	});

	// Auto-scroll to bottom only when a new message is appended or
	// the last message's streaming content grows — never on edits,
	// deletes, or branch switches in the middle of the conversation.
	let prevLen = $state(0);
	let prevLastContent = $state('');
	let shouldAutoScroll = $state(false);

	$effect(() => {
		const len = app.messages.length;
		const lastContent = app.messages.at(-1)?.content ?? '';

		// Only flag auto-scroll when a message was appended OR the last
		// message content grew (streaming). Edits/deletes mid-conversation
		// are ignored.
		if (len > prevLen) {
			shouldAutoScroll = true;
		} else if (len === prevLen && lastContent.length > prevLastContent.length) {
			shouldAutoScroll = true;
		}

		prevLen = len;
		prevLastContent = lastContent;
	});

	$effect(() => {
		if (!shouldAutoScroll) return;
		shouldAutoScroll = false;
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

	// Client-side branch filter — safety net to hide messages that don't belong
	// to the active branch. The backend should already return the correct subset,
	// but this catches any leaked main-branch messages after a fork point.
	let visibleMessages = $derived.by(() => {
		const branchId = app.active_branch_id;
		if (!branchId) return app.messages;

		// When on a non-Main branch, show:
		// 1. Messages with branch_id matching the active branch
		// 2. Messages with no branch_id that are BEFORE any branch-specific message
		//    (i.e., shared history up to the fork point)
		const branchMessages = app.messages.filter((m) => m.branch_id === branchId);
		if (branchMessages.length === 0) return app.messages;

		const forkSortOrder = Math.min(...branchMessages.map((m) => m.sort_order));
		return app.messages.filter(
			(m) => m.branch_id === branchId || (!m.branch_id && m.sort_order < forkSortOrder),
		);
	});

	let lastAiMessageId = $derived(
		visibleMessages
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

<!-- Chat column + full-height Generation panel -->
<div class="flex min-h-0 flex-1">
	<div class="flex min-w-0 flex-1 flex-col">
		<ChatHeader />

		<!-- Screen reader announcements for streaming state -->
		<div class="sr-only" aria-live="polite">
			{#if app.is_generating}
				{app.active_character?.name ?? 'Character'} is typing…
			{:else if app.messages.at(-1)?.role === 'assistant' && !app.messages.at(-1)?.is_streaming}
				Response complete
			{/if}
		</div>

		<!-- Message list -->
		<div class="flex min-h-0 flex-1">
			<div bind:this={scrollEl} class="flex-1 overflow-y-auto px-6 py-6">
				<div class="mx-auto flex max-w-[80ch] flex-col gap-5">
					{#if visibleMessages.length === 0}
						<div class="py-20 text-center text-sm text-(--color-muted)">
							No messages yet. Send something to begin.
						</div>
					{:else}
						{#each visibleMessages as message (message.id)}
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
		</div>

		<MessageInput onsend={handleSend} oncancel={handleCancel} />
	</div>
	{#if app.generation_panel_open}
		<GenerationPanel />
	{/if}
</div>

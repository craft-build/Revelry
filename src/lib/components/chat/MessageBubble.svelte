<script lang="ts">
	import type { Message, CharacterCard, UserPersona, MessageBranch } from '$lib/types';
	import { Avatar } from '$lib/components/ui';
	import { Pencil, RotateCcw, Copy, Trash2, AlertTriangle, Loader2, GitBranch, Compass } from '@lucide/svelte';
	import { cn, formatRelativeTime } from '$lib/utils';
	import { app } from '$lib/stores/app.svelte';
	import { MarkdownRenderer } from '$lib/components/chat';
	import { fade } from 'svelte/transition';
	import BranchBadge from './BranchBadge.svelte';
	import BranchPopover from './BranchPopover.svelte';

	interface Props {
		message: Message;
		character: CharacterCard | undefined;
		persona: UserPersona | undefined;
		isLastAiMessage?: boolean;
		branchesAtMessage?: MessageBranch[];
	}

	let { message, character, persona, isLastAiMessage = false, branchesAtMessage = [] }: Props = $props();

	let branchPopoverOpen = $state(false);

	let isUser = $derived(message.role === 'user');
	let isError = $derived(message.metadata?.stop_reason === 'error');
	let isCancelled = $derived(message.metadata?.stop_reason === 'cancelled');
	let isLength = $derived(message.metadata?.stop_reason === 'length');
	let showStopReasonBadge = $derived(isCancelled || isLength);

	// Edit mode state
	let isEditing = $state<boolean>(false);
	let editContent = $state<string>('');
	let editTextarea: HTMLTextAreaElement | undefined = $state();
	let isSaving = $state<boolean>(false);

	function getErrorDisplay(errorType?: string, errorContent?: string): {
		heading: string;
		body: string;
		showRetry: boolean;
		showSwitchProvider: boolean;
	} {
		switch (errorType) {
			case 'context_length_exceeded':
				return {
					heading: 'Context length exceeded',
					body: 'The conversation is too long for this model\'s context window.',
					showRetry: false,
					showSwitchProvider: true,
				};
			case 'authentication_failed':
				return {
					heading: 'Authentication failed',
					body: 'Check that your API key is correct in Settings.',
					showRetry: true,
					showSwitchProvider: true,
				};
			case 'network_error':
				return {
					heading: 'Network error',
					body: 'Check your internet connection and try again.',
					showRetry: true,
					showSwitchProvider: false,
				};
			case 'rate_limited':
				return {
					heading: 'Rate limited',
					body: 'You\'ve hit the provider\'s usage limit. Wait a moment and try again.',
					showRetry: true,
					showSwitchProvider: true,
				};
			case 'no_provider_configured':
				return {
					heading: 'No provider configured',
					body: 'Select a provider and model from the header.',
					showRetry: false,
					showSwitchProvider: true,
				};
			case 'no_api_key':
				return {
					heading: 'No API key set',
					body: 'Add your key in Settings.',
					showRetry: true,
					showSwitchProvider: true,
				};
			default:
				return {
					heading: 'Generation failed',
					body: errorContent ?? 'The provider returned an error.',
					showRetry: true,
					showSwitchProvider: true,
				};
		}
	}



	function handleSaveResend() {
		if (editContent.trim() === '' || isSaving) return;
		if (editContent === message.content) {
			isEditing = false;
			return;
		}
		isSaving = true;
		app.editResend(message.id, editContent.trim()).finally(() => {
			isEditing = false;
			isSaving = false;
		});
	}

	function handleCancelEdit() {
		isEditing = false;
		editContent = '';
	}

	function handleEditKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape' && !isSaving) {
			e.preventDefault();
			handleCancelEdit();
		}
		if ((e.metaKey || e.ctrlKey) && e.key === 'Enter' && !isSaving) {
			e.preventDefault();
			handleSaveResend();
		}
	}

	function autogrow(el: HTMLTextAreaElement | undefined) {
		if (!el) return;
		el.style.height = 'auto';
		el.style.height = `${Math.min(el.scrollHeight, 240)}px`;
	}
</script>

<article
	transition:fade={{ duration: 150 }}
	class={cn(
		'group flex gap-3 px-1 py-1',
		isError ? 'flex-row' : isUser ? 'flex-row-reverse' : 'flex-row'
	)}
>
	<!-- Avatar — hidden for error messages -->
	{#if !isError}
		<div class="shrink-0 pt-1">
			<Avatar
				name={isUser ? (persona?.name ?? 'You') : (character?.name ?? '?')}
				src={isUser ? persona?.avatar_path : character?.avatar_path}
				size="sm"
				enlargeable
			/>
		</div>
	{/if}

	<!-- Bubble + meta -->
	<div class={cn('flex max-w-[70ch] flex-col gap-1', isError ? 'items-start w-full' : isUser ? 'items-end' : 'items-start')}>
		<!-- Speaker name -->
		<div class="flex items-baseline gap-2 px-1">
			{#if isError}
				<span class="text-xs font-semibold text-(--color-destructive)">
					Generation failed
				</span>
			{:else}
				<span
					class={cn(
						'font-display text-sm font-semibold tracking-tight',
						isUser ? 'text-(--color-foreground)/80' : 'text-(--color-foreground)'
					)}
				>
					{isUser ? (persona?.name ?? 'You') : (character?.name ?? 'Character')}
				</span>
				{#if !isUser && message.metadata?.model}
					<span class="text-[10px] uppercase tracking-wider text-(--color-subtle)">
						{message.metadata.model}
					</span>
				{/if}
			{/if}
		</div>

		<!-- Branch badge (when branches exist at this fork point) -->
		{#if branchesAtMessage.length > 0}
			<div class={cn('flex items-center gap-1 px-1', isUser ? 'flex-row-reverse' : 'flex-row')}>
				<BranchPopover
					branches={branchesAtMessage}
					activeBranchId={app.active_branch_id}
					messages={app.messages}
					bind:open={branchPopoverOpen}
					onopenchange={(v) => branchPopoverOpen = v}
				>
					<BranchBadge
						branches={branchesAtMessage}
						activeBranchId={app.active_branch_id}
						parentMessageId={message.id}
					/>
				</BranchPopover>
			</div>
		{/if}

		<!-- Bubble -->
		{#if isError}
			{@const errorInfo = getErrorDisplay(message.metadata?.error_type as string, message.content)}
			<div
				class="rounded-(--radius-lg) border border-(--color-destructive)/20 bg-(--color-destructive)/10 px-4 py-3 text-[14px] leading-relaxed"
			>
				<div class="flex items-start gap-2">
					<AlertTriangle class="mt-0.5 size-4 shrink-0 text-(--color-destructive)" />
					<div class="flex flex-col gap-1.5">
						<span class="font-medium text-(--color-destructive)">{errorInfo.heading}</span>
						<span class="text-(--color-destructive)/80">{errorInfo.body}</span>
						<div class="mt-1 flex items-center gap-2">
							{#if errorInfo.showRetry}
								<button
									type="button"
									onclick={() => app.regenerate()}
									class="text-xs text-(--color-accent-vivid) hover:underline"
								>
									Retry
								</button>
							{/if}
							{#if errorInfo.showRetry && errorInfo.showSwitchProvider}
								<span class="text-xs text-(--color-subtle)">·</span>
							{/if}
							{#if errorInfo.showSwitchProvider}
								<button
									type="button"
									onclick={() => { app.focus_provider_selector = true; }}
									class="text-xs text-(--color-accent-vivid) hover:underline"
								>
									Switch provider
								</button>
							{/if}
						</div>
					</div>
				</div>
			</div>
		{:else if isEditing}
			<!-- Edit mode: textarea replaces bubble content -->
			<div
				class="w-full rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface-input) p-3"
			>
				<textarea
					bind:this={editTextarea}
					bind:value={editContent}
					onkeydown={handleEditKeydown}
					oninput={() => autogrow(editTextarea)}
					disabled={isSaving}
					rows="1"
					class="w-full resize-none bg-transparent text-[14px] leading-relaxed text-(--color-foreground) focus:outline-none disabled:opacity-50"
				></textarea>
				<div class="mt-2 flex items-center gap-2">
					<button
						type="button"
						onclick={handleSaveResend}
						disabled={isSaving}
						class="inline-flex items-center gap-1.5 rounded-(--radius-sm) bg-(--color-accent) px-3 py-1 text-xs font-medium text-(--color-accent-fg) hover:bg-(--color-accent)/90 disabled:opacity-50"
					>
						{#if isSaving}
							<Loader2 class="size-3 animate-spin" />
							Saving…
						{:else}
							Save &amp; resend
						{/if}
					</button>
					<button
						type="button"
						onclick={handleCancelEdit}
						disabled={isSaving}
						class="inline-flex items-center gap-1.5 rounded-(--radius-sm) px-3 py-1 text-xs font-medium text-(--color-subtle) hover:text-(--color-foreground) disabled:opacity-50"
					>
						Discard changes
					</button>
				</div>
			</div>
		{:else}
			<div
				class={cn(
					'rounded-(--radius-lg) border px-4 py-3 text-[15px] leading-relaxed',
					isUser
						? 'border-transparent bg-(--color-bubble-user) text-(--color-accent-fg)'
						: 'border-(--color-border) bg-(--color-bubble-character) text-(--color-foreground)',
					message.is_streaming && message.content && 'streaming-caret'
				)}
			>
				{#if message.is_streaming}
					{#if !message.content}
						<div class="thinking-dots" aria-label="Thinking">
							<span></span><span></span><span></span>
						</div>
					{:else}
						<p class="whitespace-pre-wrap break-words">{message.content}</p>
					{/if}
				{:else}
					<MarkdownRenderer content={message.content} />
				{/if}

				<!-- Steering instruction block -->
				{#if isUser && message.steering}
					<div class="mt-2.5 flex gap-2 rounded-(--radius-md) border-l-2 border-(--color-accent-muted) bg-(--color-surface) px-3 py-2">
						<Compass class="mt-0.5 size-3.5 shrink-0 text-(--color-muted)" />
						<div class="flex flex-col gap-0.5 min-w-0">
							<span class="text-[9px] font-semibold uppercase tracking-widest text-(--color-subtle)">Steering</span>
							<span class="text-[13px] italic leading-relaxed text-(--color-muted)">{message.steering}</span>
						</div>
					</div>
				{/if}

				<!-- Stop reason badges -->
				{#if showStopReasonBadge}
					<div class="mt-2">
						{#if isCancelled}
							<span class="bg-(--color-warning)/20 text-(--color-warning) text-[10px] font-medium rounded-(--radius-sm) px-1.5 py-0.5">
								Stopped
							</span>
						{:else if isLength}
							<span class="bg-(--color-warning)/20 text-(--color-warning) text-[10px] font-medium rounded-(--radius-sm) px-1.5 py-0.5">
								Max tokens
							</span>
						{/if}
					</div>
				{/if}
			</div>
		{/if}

		<!-- Actions — hover to reveal (conditional visibility) -->
		{#if !isError && !isEditing}
			<div
				class={cn(
					'mt-0.5 flex gap-0.5 px-1 opacity-0 transition-opacity group-hover:opacity-100 focus-within:opacity-100',
					isUser ? 'flex-row-reverse' : 'flex-row'
				)}
			>
				<!-- Copy: always visible -->
				<button
					type="button"
					class="inline-flex size-6 items-center justify-center rounded-(--radius-sm) text-(--color-subtle) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
					aria-label="Copy"
					onclick={() => navigator.clipboard.writeText(message.content)}
				>
					<Copy class="size-3" />
				</button>
				<!-- Branch from here: on all messages -->
				<button
					type="button"
					onclick={() => app.forkFromMessage(message.id, !isUser)}
					class="inline-flex size-6 items-center justify-center rounded-(--radius-sm) text-(--color-subtle) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
					aria-label="Branch from here"
				>
					<GitBranch class="size-3" />
				</button>
				<!-- Edit: only on user messages -->
				{#if isUser}
					<button
						type="button"
						onclick={() => { editContent = message.content; isEditing = true; }}
						class="inline-flex size-6 items-center justify-center rounded-(--radius-sm) text-(--color-subtle) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
						aria-label="Edit"
					>
						<Pencil class="size-3" />
					</button>
				{/if}
				<!-- Regenerate: only on last AI message -->
				{#if !isUser && isLastAiMessage}
					<button
						type="button"
						onclick={() => app.regenerate()}
						class="inline-flex size-6 items-center justify-center rounded-(--radius-sm) text-(--color-subtle) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
						aria-label="Regenerate"
					>
						<RotateCcw class="size-3" />
					</button>
				{/if}
				<!-- Delete: on user and AI messages, NOT on error messages -->
				<button
					type="button"
					onclick={() => app.deleteMessage(message.id)}
					class="inline-flex size-6 items-center justify-center rounded-(--radius-sm) text-(--color-subtle) hover:bg-(--color-surface-hover) hover:text-(--color-destructive)"
					aria-label="Delete"
				>
					<Trash2 class="size-3" />
				</button>
			</div>
		{/if}

		<!-- Timestamp -->
		<span
			class="mt-0.5 px-1 text-[10px] text-(--color-subtle) opacity-0 transition-opacity group-hover:opacity-100"
			title={message.timestamp ? new Date(message.timestamp).toLocaleString() : ''}
		>
			{message.timestamp ? formatRelativeTime(message.timestamp) : ''}
		</span>
	</div>
</article>

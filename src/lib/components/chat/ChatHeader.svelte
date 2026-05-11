<script lang="ts">
	import { app } from '$lib/stores/app.svelte';
	import { Avatar, Tooltip } from '$lib/components/ui';
	import { Sparkles, Clock, MoreHorizontal } from '@lucide/svelte';
	import PersonaSwitcher from './PersonaSwitcher.svelte';
</script>

<header
	class="flex h-14 shrink-0 items-center justify-between gap-3 border-b border-(--color-border) bg-(--color-surface) px-4"
>
	<!-- Active character -->
	<div class="flex min-w-0 items-center gap-3">
		{#if app.active_character}
			<Avatar
				name={app.active_character.name}
				src={app.active_character.avatar_path}
				size="md"
			/>
			<div class="min-w-0">
				<h2 class="truncate font-display text-lg font-semibold leading-tight tracking-tight">
					{app.active_character.name}
				</h2>
				<p class="truncate text-[11px] text-(--color-muted)">
					{app.active_conversation?.title ?? 'New conversation'}
				</p>
			</div>
		{/if}
	</div>

	<!-- Right cluster: persona, model, actions -->
	<div class="flex items-center gap-2">
		<PersonaSwitcher />

		<div class="h-5 w-px bg-(--color-border)" aria-hidden="true"></div>

		<Tooltip text="Model: claude-opus-4-7">
			<button
				type="button"
				class="inline-flex h-8 items-center gap-1.5 rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-elevated) px-2.5 text-xs font-medium hover:bg-(--color-surface-hover)"
			>
				<Sparkles class="size-3 text-(--color-accent)" />
				<span class="font-mono text-[11px]">opus-4.7</span>
			</button>
		</Tooltip>

		<Tooltip text="Branch history">
			<button
				type="button"
				class="inline-flex size-8 items-center justify-center rounded-(--radius-md) text-(--color-muted) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
				aria-label="Branch history"
			>
				<Clock class="size-4" />
			</button>
		</Tooltip>

		<Tooltip text="More">
			<button
				type="button"
				class="inline-flex size-8 items-center justify-center rounded-(--radius-md) text-(--color-muted) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
				aria-label="More options"
			>
				<MoreHorizontal class="size-4" />
			</button>
		</Tooltip>
	</div>
</header>

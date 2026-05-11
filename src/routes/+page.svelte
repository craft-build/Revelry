<script lang="ts">
	import { Sparkles, ArrowRight } from '@lucide/svelte';
	import { app } from '$lib/stores/app.svelte';
	import { Avatar } from '$lib/components/ui';

	// Surface a few characters as quick starts
	let suggested = $derived(app.characters.slice(0, 3));
</script>

<svelte:head>
	<title>Revelry — Chat</title>
</svelte:head>

<div class="flex flex-1 items-center justify-center px-8">
	<div class="flex w-full max-w-2xl flex-col items-center gap-8 text-center">
		<!-- Brand mark -->
		<div class="relative">
			<div
				class="absolute inset-0 -z-10 size-20 -translate-x-2 -translate-y-2 rounded-full bg-(--color-accent)/10 blur-2xl"
				aria-hidden="true"
			></div>
			<div
				class="grid size-16 place-items-center rounded-(--radius-xl) bg-gradient-to-br from-(--color-accent) to-(--color-accent-muted) text-(--color-accent-fg)"
			>
				<Sparkles class="size-7" />
			</div>
		</div>

		<div class="space-y-3">
			<h1 class="font-display text-4xl font-semibold tracking-tight">
				Bring a character to life.
			</h1>
			<p class="text-base text-(--color-muted)">
				Pick a character to begin a conversation, or import a CharacterCardV2 from your library.
			</p>
		</div>

		<!-- Suggested characters -->
		<div class="grid w-full grid-cols-1 gap-2 sm:grid-cols-3">
			{#each suggested as char (char.id)}
				<a
					href="/chat/conv_01"
					class="group flex flex-col gap-2 rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface) p-4 text-left transition-all hover:-translate-y-0.5 hover:border-(--color-border-strong) hover:bg-(--color-surface-elevated)"
				>
					<Avatar name={char.name} src={char.avatar_path} size="md" />
					<div class="space-y-1">
						<h3
							class="truncate font-display text-base font-semibold leading-tight tracking-tight"
						>
							{char.name}
						</h3>
						<p class="line-clamp-2 text-xs text-(--color-muted)">
							{char.description}
						</p>
					</div>
					<div class="mt-auto flex items-center gap-1 pt-1 text-[11px] font-medium text-(--color-accent)">
						<span>Begin</span>
						<ArrowRight
							class="size-3 transition-transform group-hover:translate-x-0.5"
						/>
					</div>
				</a>
			{/each}
		</div>

		<a
			href="/characters"
			class="text-xs text-(--color-muted) underline-offset-4 hover:text-(--color-foreground) hover:underline"
		>
			Browse all characters →
		</a>
	</div>
</div>

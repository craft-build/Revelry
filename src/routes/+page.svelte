<script lang="ts">
	import { ArrowRight } from '@lucide/svelte';
	import { goto } from '$app/navigation';
	import { app } from '$lib/stores/app.svelte';
	import { Avatar } from '$lib/components/ui';
	import Logo from '$lib/components/ui/Logo.svelte';

	// Surface most recently updated characters as quick starts
	let suggested = $derived(
		[...app.characters]
			.sort((a, b) => b.updated_at.localeCompare(a.updated_at))
			.slice(0, 3),
	);
</script>

<svelte:head>
	<title>Revelry — Chat</title>
</svelte:head>

<div class="flex flex-1 items-center justify-center px-8">
	<div class="flex w-full max-w-2xl flex-col items-center gap-8 text-center">
		<!-- Brand mark -->
		<div class="relative size-24">
			<!-- Night glow behind the mark: blue key light + ember rim -->
			<div
				class="absolute -inset-6 rounded-full blur-[24px]"
				style="background: radial-gradient(circle at 35% 35%, rgba(51, 101, 252, 0.45), transparent 60%), radial-gradient(circle at 70% 75%, rgba(251, 120, 0, 0.35), transparent 60%)"
				aria-hidden="true"
			></div>
			<Logo class="relative size-24" />
		</div>

		<div class="space-y-3">
			<h1 class="font-display text-[40px] leading-[1.1] font-semibold tracking-[-0.03em]">
				Bring a character to <span class="text-gradient-brand">life.</span>
			</h1>
			<p class="text-base text-(--color-muted)">
				Pick a character to begin a conversation, or import a CharacterCardV2 from your library.
			</p>
		</div>

		<!-- Suggested characters -->
		<div class="grid w-full grid-cols-1 gap-2.5 sm:grid-cols-3">
			{#each suggested as char (char.id)}
			<button
				type="button"
				onclick={async () => {
					const conv = await app.startConversation(char.id);
					goto('/chat/' + conv.id);
				}}
				class="group flex flex-col gap-2.5 rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface) p-4 text-left transition-all duration-150 hover:-translate-y-0.5 hover:border-(--color-border-strong) hover:bg-(--color-surface-elevated) hover:shadow-md"
			>
					<Avatar name={char.name} src={char.avatar_path} size="md" />
					<div class="space-y-1">
						<h3
							class="truncate font-display text-base leading-tight font-semibold tracking-[-0.02em]"
						>
							{char.name}
						</h3>
						<p class="line-clamp-2 text-xs leading-normal text-(--color-muted)">
							{char.description}
						</p>
					</div>
					<div class="mt-auto flex items-center gap-1 pt-1 text-[11px] font-semibold text-(--color-accent-vivid)">
						<span>Begin</span>
						<ArrowRight
							class="size-3 transition-transform group-hover:translate-x-0.5"
						/>
					</div>
				</button>
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

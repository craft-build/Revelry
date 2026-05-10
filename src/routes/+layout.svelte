<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { app } from '$lib/stores/app.svelte';
	import {
		mockCharacters,
		mockPersonas,
		mockConversations,
		mockProviders,
		mockPresets
	} from '$lib/data/mock';
	import AppHeader from '$lib/components/layout/AppHeader.svelte';
	import ConversationSidebar from '$lib/components/layout/ConversationSidebar.svelte';
	import { page } from '$app/state';

	let { children } = $props();

	onMount(() => {
		app.hydrate({
			characters: mockCharacters,
			personas: mockPersonas,
			conversations: mockConversations,
			providers: mockProviders,
			prompt_presets: mockPresets
		});

		// F9 toggles sidebar (per spec §4.1.5)
		const onkey = (e: KeyboardEvent) => {
			if (e.key === 'F9') {
				e.preventDefault();
				app.toggleSidebar();
			}
		};
		window.addEventListener('keydown', onkey);
		return () => window.removeEventListener('keydown', onkey);
	});

	// Show conversation sidebar only on chat routes (Adwaita-style: sidebar
	// belongs to the active view, not the application chrome)
	let showConvSidebar = $derived(
		page.url.pathname === '/' || page.url.pathname.startsWith('/chat')
	);
</script>

<div class="surface-grain flex h-dvh flex-col bg-(--color-background) text-(--color-foreground)">
	<AppHeader />

	<div class="flex min-h-0 flex-1">
		{#if showConvSidebar}
			<ConversationSidebar />
		{/if}

		<main class="flex min-w-0 flex-1 flex-col">
			{@render children?.()}
		</main>
	</div>
</div>

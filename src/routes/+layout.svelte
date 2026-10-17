<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { app } from '$lib/stores/app.svelte';
	import AppHeader from '$lib/components/layout/AppHeader.svelte';
	import ConversationSidebar from '$lib/components/layout/ConversationSidebar.svelte';
	import MemoryPromptDialog from '$lib/components/chat/MemoryPromptDialog.svelte';
	import { ToastContainer } from '$lib/components/ui';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { getShikiHighlighter } from '$lib/shiki.svelte';
	import { AlertTriangle, RefreshCw } from '@lucide/svelte';

	let { children } = $props();

	onMount(() => {
		app.hydrate();
		app.checkKeyring();

		// F9 toggles sidebar (per spec §4.1.5)
		const onkey = (e: KeyboardEvent) => {
			if (e.key === 'F9') {
				e.preventDefault();
				app.toggleSidebar();
			}
			// Ctrl/Cmd+N — new chat
			if ((e.metaKey || e.ctrlKey) && e.key === 'n' && !e.shiftKey) {
				e.preventDefault();
				goto('/');
			}
			// Ctrl/Cmd+Shift+N — new character
			if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === 'N') {
				e.preventDefault();
				goto('/characters');
			}
			// Escape — cancel generation
			if (e.key === 'Escape' && app.is_generating) {
				e.preventDefault();
				app.stopGeneration();
			}
		};
		window.addEventListener('keydown', onkey);

		// Preload Shiki syntax highlighting with both themes
		getShikiHighlighter().catch((err: unknown) => {
			console.warn('Shiki preload failed (non-critical):', err);
		});

		// Listen for OS theme preference changes
		const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
		const onMediaChange = () => {
			if (app.theme === 'system') {
				document.documentElement.className = app.resolved_theme;
			}
		};
		mediaQuery.addEventListener('change', onMediaChange);

		return () => {
			window.removeEventListener('keydown', onkey);
			mediaQuery.removeEventListener('change', onMediaChange);
		};
	});

	// Show conversation sidebar only on chat routes (Adwaita-style: sidebar
	// belongs to the active view, not the application chrome)
	let showConvSidebar = $derived(
		page.url.pathname === '/' || page.url.pathname.startsWith('/chat')
	);

	function retryInit() {
		app.hydrate();
	}
</script>

<div class="surface-grain flex h-dvh flex-col bg-(--color-background) text-(--color-foreground) transition-colors duration-150">
	{#if app.is_loading}
		<!-- Skeleton loading state -->
		<div class="flex h-dvh flex-col">
			<!-- Header skeleton -->
			<div class="flex h-14 items-center gap-3 border-b border-(--color-border) px-4">
				<div class="size-8 animate-pulse rounded-(--radius-md) bg-(--color-surface-elevated)"></div>
				<div class="h-4 w-32 animate-pulse rounded bg-(--color-surface-elevated)"></div>
				<div class="ml-auto h-8 w-48 animate-pulse rounded-(--radius-md) bg-(--color-surface-elevated)"></div>
			</div>
			<!-- Body skeleton -->
			<div class="flex min-h-0 flex-1">
				<!-- Sidebar skeleton -->
				<div class="w-72 shrink-0 border-r border-(--color-border) p-3">
					<div class="mb-3 h-8 animate-pulse rounded-(--radius-md) bg-(--color-surface-elevated)"></div>
					{#each Array(5) as _, i}
						<div class="mb-2 rounded-(--radius-md) bg-(--color-surface-elevated) p-3">
							<div class="mb-2 h-3 w-3/4 animate-pulse rounded bg-(--color-border)"></div>
							<div class="h-2 w-1/2 animate-pulse rounded bg-(--color-border)"></div>
						</div>
					{/each}
				</div>
				<!-- Main skeleton -->
				<div class="flex flex-1 items-center justify-center">
					<div class="flex flex-col items-center gap-3">
						<div class="size-12 animate-pulse rounded-(--radius-xl) bg-(--color-surface-elevated)"></div>
						<div class="h-4 w-48 animate-pulse rounded bg-(--color-surface-elevated)"></div>
						<div class="h-3 w-32 animate-pulse rounded bg-(--color-surface-elevated)"></div>
					</div>
				</div>
			</div>
		</div>
	{:else if app.init_error}
		<!-- DB error overlay -->
		<div class="flex h-dvh flex-col items-center justify-center gap-6 px-8">
			<div
				class="flex size-16 items-center justify-center rounded-full bg-(--color-destructive)/10 text-(--color-destructive)"
			>
				<AlertTriangle class="size-8" />
			</div>
			<div class="max-w-md text-center">
				<h1 class="font-display text-xl font-semibold tracking-tight">
					Unable to load data
				</h1>
				<p class="mt-2 text-sm text-(--color-muted)">
					The application could not connect to its local database. This may happen if another
					instance is already running or the database file is corrupted.
				</p>
				<p class="mt-2 font-mono text-xs text-(--color-subtle)">
					{app.init_error}
				</p>
			</div>
			<button
				type="button"
				onclick={retryInit}
				class="inline-flex items-center gap-2 rounded-(--radius-md) bg-(--color-accent) px-4 py-2 text-sm font-medium text-(--color-accent-fg) hover:bg-(--color-accent)/90"
			>
				<RefreshCw class="size-4" />
				Try again
			</button>
		</div>
	{:else}
		<AppHeader />

		<div class="flex min-h-0 flex-1">
			{#if showConvSidebar}
				<ConversationSidebar />
			{/if}

			<main class="flex min-w-0 flex-1 flex-col">
				{@render children?.()}
			</main>
		</div>
		<ToastContainer />
		<MemoryPromptDialog />
	{/if}
</div>

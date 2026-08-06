<script lang="ts">
	import { page } from '$app/state';
	import { MessageSquare, Users, Settings, PanelLeft } from '@lucide/svelte';
	import { app } from '$lib/stores/app.svelte';
	import { cn } from '$lib/utils';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import Logo from '$lib/components/ui/Logo.svelte';

	const views = [
		{ href: '/', label: 'Chat', icon: MessageSquare, match: (p: string) => p === '/' || p.startsWith('/chat') },
		{ href: '/characters', label: 'Library', icon: Users, match: (p: string) => p.startsWith('/characters') },
		{ href: '/settings', label: 'Settings', icon: Settings, match: (p: string) => p.startsWith('/settings') }
	];
</script>

<header
	data-tauri-drag-region
	class="drag-region relative flex h-12 shrink-0 items-center gap-2 border-b border-(--color-border) bg-(--color-surface) py-0 pr-3 pl-[6rem]"
>
	<!-- Sidebar toggle -->
	<div class="no-drag">
		<Tooltip text="Toggle sidebar (F9)">
			<button
				type="button"
				class="inline-flex size-8 items-center justify-center rounded-(--radius-sm) text-(--color-muted) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
				onclick={() => app.toggleSidebar()}
				aria-label="Toggle sidebar"
			>
				<PanelLeft class="size-4" />
			</button>
		</Tooltip>
	</div>

	<!-- App identity -->
	<div class="flex items-center gap-2 px-1">
		<Logo alt="" />
		<span class="font-display text-sm font-semibold tracking-wide">Revelry</span>
	</div>

	<!-- View switcher — centered pill segment, per the design system's TopNav -->
	<nav class="no-drag absolute left-1/2 flex -translate-x-1/2 items-center gap-1">
		{#each views as view (view.href)}
			{@const active = view.match(page.url.pathname)}
			<a
				href={view.href}
				class={cn(
					'inline-flex h-8 items-center gap-1.5 rounded-(--radius-md) px-4 text-sm font-medium transition-colors',
					active
						? 'bg-(--color-surface-elevated) text-(--color-foreground)'
						: 'text-(--color-subtle) hover:text-(--color-foreground)'
				)}
			>
				<view.icon class="size-3.5" />
				{view.label}
			</a>
		{/each}
	</nav>

	<!-- Right cluster — placeholder for future actions -->
	<div class="no-drag w-[76px]"></div>
</header>

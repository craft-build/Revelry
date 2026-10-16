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
	class="drag-region relative z-10 flex h-12 shrink-0 items-center gap-2 border-b border-(--color-border) bg-(--color-surface)/80 py-0 pr-3 pl-[6rem] backdrop-blur-[20px]"
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
		<Logo alt="" class="size-[26px]" />
		<span class="font-display text-[15px] font-semibold tracking-[0.01em]">Revelry</span>
	</div>

	<!-- View switcher — centered segmented tabs track -->
	<nav
		class="no-drag absolute left-1/2 inline-flex -translate-x-1/2 items-center gap-0.5 rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-input) p-[3px]"
	>
		{#each views as view (view.href)}
			{@const active = view.match(page.url.pathname)}
			<a
				href={view.href}
				aria-current={active ? 'page' : undefined}
				class={cn(
					'inline-flex h-[30px] items-center gap-2 rounded-[11px] px-3.5 text-[13px] font-medium whitespace-nowrap transition-colors duration-150',
					active
						? 'bg-(--color-surface-elevated) text-(--color-foreground) shadow-sm ring-1 ring-(--color-border) ring-inset'
						: 'text-(--color-muted) hover:text-(--color-foreground)'
				)}
			>
				<view.icon class="size-4" />
				{view.label}
			</a>
		{/each}
	</nav>

	<!-- Right cluster — placeholder for future actions -->
	<div class="no-drag w-[76px]"></div>
</header>

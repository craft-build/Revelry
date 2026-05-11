<script lang="ts">
	import { Popover } from 'bits-ui';
	import { Check, Plus, UserCog } from '@lucide/svelte';
	import { app } from '$lib/stores/app.svelte';
	import { Avatar } from '$lib/components/ui';
	import { cn } from '$lib/utils';

	let open = $state(false);

	function pick(id: string) {
		app.switchPersona(id);
		open = false;
	}
</script>

<Popover.Root bind:open>
	<Popover.Trigger
		class="inline-flex h-8 items-center gap-2 rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-elevated) pl-1 pr-3 text-xs font-medium hover:bg-(--color-surface-hover)"
	>
		<Avatar
			name={app.active_persona?.name ?? 'No persona'}
			src={app.active_persona?.avatar_path}
			size="xs"
		/>
		<span class="font-display tracking-tight">
			{app.active_persona?.name ?? 'No persona'}
		</span>
	</Popover.Trigger>

	<Popover.Content
		side="bottom"
		align="end"
		sideOffset={6}
		class="z-50 w-72 rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface-elevated) p-1.5 shadow-xl"
	>
		<div class="px-2 pb-1.5 pt-1 text-[10px] font-semibold uppercase tracking-wider text-(--color-subtle)">
			Active persona
		</div>

		<ul class="flex flex-col gap-0.5">
			{#each app.personas as persona (persona.id)}
				{@const active = persona.id === app.active_persona?.id}
				<li>
					<button
						type="button"
						onclick={() => pick(persona.id)}
						class={cn(
							'flex w-full items-center gap-2.5 rounded-(--radius-md) px-2 py-1.5 text-left transition-colors',
							active ? 'bg-(--color-surface)' : 'hover:bg-(--color-surface-hover)'
						)}
					>
						<Avatar name={persona.name} src={persona.avatar_path} size="sm" />
						<div class="min-w-0 flex-1">
							<div class="flex items-baseline gap-1.5">
								<span class="truncate text-sm font-medium">{persona.name}</span>

							</div>
							{#if persona.role_description}
								<p class="truncate text-[11px] text-(--color-muted)">
									{persona.role_description}
								</p>
							{/if}
						</div>
						{#if active}
							<Check class="size-3.5 shrink-0 text-(--color-accent)" />
						{/if}
					</button>
				</li>
			{/each}
		</ul>

		<div class="my-1 h-px bg-(--color-border)"></div>

		<a
			href="/characters?tab=personas"
			class="flex items-center gap-2.5 rounded-(--radius-md) px-2 py-1.5 text-xs text-(--color-muted) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
		>
			<Plus class="size-3.5" />
			Create new persona
		</a>
		<a
			href="/characters?tab=personas"
			class="flex items-center gap-2.5 rounded-(--radius-md) px-2 py-1.5 text-xs text-(--color-muted) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
		>
			<UserCog class="size-3.5" />
			Manage personas
		</a>
	</Popover.Content>
</Popover.Root>

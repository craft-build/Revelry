<script lang="ts">
	import { Popover } from 'bits-ui';
	import { Check, Sparkles, Settings } from '@lucide/svelte';
	import { app } from '$lib/stores/app.svelte';
	import { cn } from '$lib/utils';

	let open = $state(false);

	let active_provider = $derived(
		app.active_conversation?.provider_id
			? app.providers.find((p) => p.id === app.active_conversation?.provider_id)
			: undefined,
	);

	let active_model_name = $derived(app.active_conversation?.model_name ?? '');

	let display_label = $derived(
		active_provider && active_model_name
			? active_model_name
			: active_provider?.model_name ?? 'No model',
	);

	let enabled_providers = $derived(app.providers.filter((p) => p.enabled));

	function pick(providerId: string, modelName: string) {
		app.updateConversationProvider(providerId, modelName);
		open = false;
	}
</script>

<Popover.Root bind:open>
	<Popover.Trigger
		class="inline-flex h-8 items-center gap-1.5 rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-elevated) px-2.5 text-xs font-medium hover:bg-(--color-surface-hover) disabled:cursor-not-allowed disabled:opacity-50"
		disabled={!app.active_conversation}
	>
		<Sparkles class="size-3 text-(--color-accent)" />
		<span class="font-mono text-[11px]">{display_label}</span>
	</Popover.Trigger>

	<Popover.Content
		side="bottom"
		align="end"
		sideOffset={6}
		class="z-50 w-72 rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface-elevated) p-1.5 shadow-xl"
	>
		<div
			class="px-2 pb-1.5 pt-1 text-[10px] font-semibold uppercase tracking-wider text-(--color-subtle)"
		>
			Provider / model
		</div>

		{#if enabled_providers.length === 0}
			<div class="px-2 py-3 text-center text-xs text-(--color-muted)">
				<p>No providers configured.</p>
				<a
					href="/settings"
					class="mt-1.5 inline-flex items-center gap-1 text-(--color-accent) hover:underline"
				>
					<Settings class="size-3" />
					Add a provider
				</a>
			</div>
		{:else}
			<ul class="flex flex-col gap-0.5 max-h-64 overflow-y-auto">
				{#each enabled_providers as provider (provider.id)}
					{@const provider_active = provider.id ===
						app.active_conversation?.provider_id}
					<li>
						<div
							class="px-2 py-1 text-[10px] font-semibold uppercase tracking-wider text-(--color-subtle)"
						>
							{provider.name}
						</div>
						<ul class="flex flex-col gap-0.5">
							{#each provider.models.filter((m) => m.enabled) as model (model.name)}
								{@const active = provider_active && model.name === active_model_name}
								<li>
									<button
										type="button"
										onclick={() => pick(provider.id, model.name)}
										class={cn(
											'flex w-full items-center gap-2.5 rounded-(--radius-md) px-2 py-1.5 text-left transition-colors',
											active
												? 'bg-(--color-surface)'
												: 'hover:bg-(--color-surface-hover)',
										)}
									>
										<span class="truncate font-mono text-[11px]">{model.name}</span>
										{#if active}
											<Check class="ml-auto size-3.5 shrink-0 text-(--color-accent)" />
										{/if}
									</button>
								</li>
							{/each}
						</ul>
					</li>
				{/each}
			</ul>
		{/if}

		<div class="my-1 h-px bg-(--color-border)"></div>

		<a
			href="/settings"
			class="flex items-center gap-2.5 rounded-(--radius-md) px-2 py-1.5 text-xs text-(--color-muted) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
		>
			<Settings class="size-3.5" />
			Manage providers
		</a>
	</Popover.Content>
</Popover.Root>

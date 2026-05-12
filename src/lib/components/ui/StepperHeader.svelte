<script lang="ts">
	import { Check } from '@lucide/svelte';
	import { cn } from '$lib/utils';

	type StepState = 'completed' | 'current' | 'locked' | 'upcoming';

	interface Step {
		label: string;
		state: StepState;
	}

	interface Props {
		steps: Step[];
		class?: string;
		onstepclick?: (index: number) => void;
	}

	let { steps, class: className, onstepclick }: Props = $props();
</script>

<nav class={cn('flex items-center gap-1', className)} aria-label="Progress steps">
	{#each steps as step, i}
		<div class="flex items-center gap-2">
			<!-- Step indicator circle -->
			<button
				type="button"
				disabled={!onstepclick || step.state !== 'completed'}
				onclick={onstepclick && step.state === 'completed' ? () => onstepclick(i) : undefined}
				class={cn(
					'flex size-7 shrink-0 items-center justify-center rounded-full text-xs font-medium transition-colors',
					onstepclick && step.state === 'completed' && 'cursor-pointer hover:opacity-80',
					(!onstepclick || step.state !== 'completed') && 'cursor-default',
					step.state === 'completed' &&
						'bg-(--color-accent) text-(--color-accent-fg)',
					step.state === 'current' &&
						'bg-(--color-accent)/15 text-(--color-accent) ring-1 ring-(--color-accent)',
					(step.state === 'locked' || step.state === 'upcoming') &&
						'bg-(--color-surface-elevated) text-(--color-muted)',
					(step.state === 'locked') && 'opacity-50'
				)}
			>
				{#if step.state === 'completed'}
					<Check class="size-4" />
				{:else}
					{i + 1}
				{/if}
			</button>
			<span
				class={cn(
					'text-sm font-medium whitespace-nowrap',
					step.state === 'current' && 'text-(--color-foreground)',
					step.state === 'completed' && 'text-(--color-foreground)',
					(step.state === 'upcoming' || step.state === 'locked') && 'text-(--color-muted)',
					step.state === 'locked' && 'opacity-50'
				)}
			>
				{step.label}
			</span>

			<!-- Connector line between steps -->
			{#if i < steps.length - 1}
				<div
					class={cn(
						'h-px w-8',
						step.state === 'completed'
							? 'bg-(--color-accent)'
							: 'bg-(--color-border)'
					)}
				></div>
			{/if}
		</div>
	{/each}
</nav>

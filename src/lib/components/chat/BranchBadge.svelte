<script lang="ts">
	import type { MessageBranch } from '$lib/types';
	import { ChevronLeft, ChevronRight } from '@lucide/svelte';
	import { cn } from '$lib/utils';
	import { app } from '$lib/stores/app.svelte';

	interface Props {
		branches: MessageBranch[];
		activeBranchId: string | null;
		parentMessageId: string;
		class?: string;
		onclick?: () => void;
	}

	let { branches, activeBranchId, parentMessageId, class: className, onclick }: Props = $props();

	// Calculate position: 1-based index of active branch among all branches at this fork point
	// "Main" is included as position 1 when branch_id is null
	let totalBranches = $derived(branches.length + 1); // +1 for Main
	let activeIndex = $derived(
		activeBranchId
			? branches.findIndex((b) => b.id === activeBranchId) + 2 // +2 because Main=1, branches start at 2
			: 1, // Main is position 1
	);

	let label = $derived(`${activeIndex}/${totalBranches}`);

	function handlePrev(e: MouseEvent) {
		e.stopPropagation();
		app.cycleBranchAtForkPoint(parentMessageId, 'prev');
	}

	function handleNext(e: MouseEvent) {
		e.stopPropagation();
		app.cycleBranchAtForkPoint(parentMessageId, 'next');
	}
</script>

<div
	class={cn(
		'inline-flex items-center gap-0.5 rounded-(--radius-sm)',
		'border border-(--color-accent)/30 bg-(--color-accent)/10',
		'text-(--color-accent) transition-colors',
		className,
	)}
>
	<button
		type="button"
		onclick={handlePrev}
		class="inline-flex size-5 items-center justify-center rounded-l-(--radius-sm) hover:bg-(--color-accent)/20 transition-colors cursor-pointer"
		aria-label="Previous branch"
	>
		<ChevronLeft class="size-3" />
	</button>
	<button
		type="button"
		class="text-[10px] font-medium px-1 py-0.5 hover:bg-(--color-accent)/20 transition-colors cursor-pointer"
		aria-label="View branches ({activeIndex} of {totalBranches})"
		{onclick}
	>
		{label}
	</button>
	<button
		type="button"
		onclick={handleNext}
		class="inline-flex size-5 items-center justify-center rounded-r-(--radius-sm) hover:bg-(--color-accent)/20 transition-colors cursor-pointer"
		aria-label="Next branch"
	>
		<ChevronRight class="size-3" />
	</button>
</div>

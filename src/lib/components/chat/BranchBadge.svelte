<script lang="ts">
	import type { MessageBranch } from '$lib/types';
	import { cn } from '$lib/utils';

	interface Props {
		branches: MessageBranch[];
		activeBranchId: string | null;
		class?: string;
		onclick?: () => void;
	}

	let { branches, activeBranchId, class: className, onclick }: Props = $props();

	// Calculate position: 1-based index of active branch among all branches at this fork point
	// "Main" is included as position 1 when branch_id is null
	let totalBranches = $derived(branches.length + 1); // +1 for Main
	let activeIndex = $derived(
		activeBranchId
			? branches.findIndex((b) => b.id === activeBranchId) + 2 // +2 because Main=1, branches start at 2
			: 1, // Main is position 1
	);

	let label = $derived(`${activeIndex}/${totalBranches}`);
</script>

<button
	type="button"
	class={cn(
		'text-[10px] font-medium px-1.5 py-0.5 rounded-(--radius-sm)',
		'border border-(--color-accent)/30 text-(--color-accent) bg-(--color-accent)/10',
		'hover:bg-(--color-accent)/20 transition-colors cursor-pointer',
		className,
	)}
	aria-label="View branches ({activeIndex} of {totalBranches})"
	{onclick}
>
	{label}
</button>

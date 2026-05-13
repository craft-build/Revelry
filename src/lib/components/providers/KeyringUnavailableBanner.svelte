<script lang="ts">
	import { cn } from '$lib/utils';
	import { AlertTriangle } from '@lucide/svelte';

	interface Props {
		class?: string;
	}

	let { class: className }: Props = $props();
</script>

<div
	class={cn(
		'rounded-(--radius-lg) border border-(--color-warning)/20 bg-(--color-warning)/10 px-4 py-3',
		className
	)}
>
	<div class="flex items-start gap-2.5">
		<AlertTriangle class="size-4 shrink-0 text-(--color-warning) mt-0.5" />
		<div>
			<p class="text-sm font-semibold text-(--color-foreground)">
				OS keychain unavailable
			</p>
			<p class="mt-0.5 text-sm text-(--color-muted)">
				API keys cannot be stored. You can still configure Ollama (local) providers. To fix
				this, ensure a keyring service is running (e.g., GNOME Keyring on Linux).
			</p>
		</div>
	</div>
</div>

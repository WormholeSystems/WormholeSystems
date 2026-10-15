<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children, actions }: { children: Snippet; actions?: Snippet } = $props();
</script>

<div
	class="@container/panel-header flex h-9 shrink-0 items-center justify-between gap-2 border-b border-border/50 bg-muted/30 px-3"
>
	<h3
		class="flex min-w-0 items-center gap-1 overflow-hidden font-mono whitespace-nowrap text-[10px] tracking-wider text-muted-foreground uppercase"
	>
		{@render children()}
	</h3>
	{#if actions}
		<div class="flex shrink-0 items-center gap-2">
			{@render actions()}
		</div>
	{/if}
</div>

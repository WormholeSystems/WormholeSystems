<script lang="ts">
	// The statics of a wormhole system, pinned to the node's bottom-right corner.
	import type { Static } from '$lib/api/types/Static';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import StaticDetails from '$lib/components/StaticDetails.svelte';
	import { destClassMeta } from '$lib/map/classes';

	let { statics }: { statics: Static[] } = $props();
</script>

<div class="flex items-center justify-end gap-1.5 text-[10px]">
	{#each statics as st (st.code)}
		{@const dest = destClassMeta(st.dest_class)}
		<Tooltip.Root delayDuration={700}>
			<Tooltip.Trigger
				class="flex font-medium"
				data-testid="static-badge"
				style="color: var(--color-{dest.token})"
			>
				{dest.short}
			</Tooltip.Trigger>
			<Tooltip.Content class="p-0" side="bottom">
				<StaticDetails static={st} />
			</Tooltip.Content>
		</Tooltip.Root>
	{/each}
</div>

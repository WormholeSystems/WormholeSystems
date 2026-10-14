<script lang="ts">
	// The pilots-present footer, with the full roster in its tooltip.
	import type { MapCharacter } from '$lib/api/types/MapCharacter';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import EveImage from '$lib/components/EveImage.svelte';

	let { pilots }: { pilots: MapCharacter[] } = $props();
</script>

<Tooltip.Root delayDuration={700}>
	<Tooltip.Trigger
		class="mt-0.5 flex h-[18px] items-center gap-1.5 border-t border-border pt-0.5 text-[10px]"
		data-testid="pilots-row"
	>
		<span class="size-1 animate-pulse rounded-full bg-green-500"></span>
		<span class="truncate">{pilots[0].name}</span>
		{#if pilots.length > 1}
			<span class="shrink-0 text-muted-foreground">and {pilots.length - 1} more</span>
		{/if}
	</Tooltip.Trigger>
	<Tooltip.Content class="p-2" side="bottom">
		<div class="flex max-h-64 flex-col gap-1 overflow-auto">
			{#each pilots as p (p.character_id)}
				<div class="flex items-center gap-2 text-[11px]">
					<EveImage kind="character" id={p.character_id} class="size-5 rounded-full" />
					{p.name}
					<span class="text-muted-foreground">[{p.corporation_ticker}]</span>
					{#if p.ship_type}
						<span class="ml-auto text-muted-foreground">{p.ship_type}</span>
					{/if}
				</div>
			{/each}
		</div>
	</Tooltip.Content>
</Tooltip.Root>

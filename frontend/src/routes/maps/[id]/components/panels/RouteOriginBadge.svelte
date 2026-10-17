<script lang="ts">
	// Cards that count jumps all measure from one origin, which is the route's From when
	// one is set and otherwise wherever the pilot is. Without saying so, a stale From makes
	// every number on the page quietly wrong, so each of those cards names it.
	import type { MapState } from '../../state/map-state.svelte';
	import * as Tooltip from '$lib/components/ui/tooltip';

	let { map }: { map: MapState } = $props();

	const origin = $derived(map.routeOrigin);
	const name = $derived(origin === null ? null : (map.systemInfo(origin)?.name ?? null));
	const pinned = $derived(map.route.fromId !== null);
</script>

{#if name}
	<Tooltip.Root>
		<Tooltip.Trigger class="truncate text-[10px] text-muted-foreground" data-testid="route-origin">
			from <span class={pinned ? 'text-amber-400' : ''}>{name}</span>
		</Tooltip.Trigger>
		<Tooltip.Content>
			{pinned
				? `Jumps are counted from ${name}, set as the route origin. Clear it to go back to where you are.`
				: `Jumps are counted from ${name}, where you are now.`}
		</Tooltip.Content>
	</Tooltip.Root>
{/if}

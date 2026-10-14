<script lang="ts">
	// Where the fleet is forming up, and how far it is from the staging system. A map holds
	// up to two rally points (a form-up spot and the fight, say), each with its own badge. A
	// rally is called for everyone at once, so the count is the map's own: home to rally,
	// the same number for whoever is looking at it.
	import FlagIcon from '@lucide/svelte/icons/flag';
	import * as Popover from '$lib/components/ui/popover';
	import ClassBadge from '$lib/components/ClassBadge.svelte';
	import RouteList from '../panels/RouteList.svelte';
	import { findRoute } from '$lib/routing/algorithm';
	import { solarSystemId } from '$lib/map/system';
	import type { MapSystemView } from '$lib/api/types/MapSystemView';
	import type { MapState } from '../../state/map-state.svelte';

	let { map }: { map: MapState } = $props();

	type RallySystem = Extract<MapSystemView, { kind: 'system' }>;

	const homeId = $derived(
		map.systems.all
			.filter((s) => s.is_home)
			.map(solarSystemId)
			.find((id) => id !== null) ?? null,
	);

	const rallies = $derived(
		map.systems.all
			.filter((s): s is RallySystem => s.is_rally && s.kind === 'system')
			.map((rally) => {
				const graph = map.route.graph;
				const result =
					!graph || homeId === null || homeId === rally.solar_system_id
						? null
						: findRoute(
								graph,
								homeId,
								rally.solar_system_id,
								map.routingSettings,
								map.route.avoided,
							);
				return { rally, result };
			}),
	);
</script>

{#if rallies.length > 0}
	<!-- Out of the way of the zoom and placement controls, which own the bottom corners. The
	     press is stopped here like the other overlays do: the canvas captures the pointer to
	     pan, and would swallow the click before the popover ever saw it. -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="absolute top-3 right-3 flex flex-col items-end gap-2"
		onpointerdown={(ev) => ev.stopPropagation()}
	>
		{#each rallies as { rally, result } (rally.id)}
			<div
				class="flex items-center gap-3 border border-pink-500/40 bg-card px-3 py-2"
				data-testid="rally-badge"
			>
				<div class="flex flex-col gap-0.5">
					<span class="text-[10px] tracking-wider text-pink-500/80 uppercase">Rally point</span>
					<div class="flex items-center gap-1.5 text-sm">
						<ClassBadge
							classId={rally.wormhole_class_id}
							security={rally.security_status}
							class="font-medium"
						/>
						{#if rally.alias}
							<span class="font-medium">{rally.alias}</span>
							<span class="text-muted-foreground">{rally.name}</span>
						{:else}
							<span class="font-medium">{rally.name}</span>
						{/if}
						<span class="text-xs text-muted-foreground">{rally.region}</span>
					</div>
				</div>

				{#if result}
					<Popover.Root>
						<Popover.Trigger
							class="flex h-8 items-center gap-1.5 border border-border px-2.5 font-mono text-sm hover:bg-accent"
							data-testid="rally-jumps"
						>
							<FlagIcon class="size-3" />
							{result.jumps}j
						</Popover.Trigger>
						<Popover.Content class="w-80 gap-0 p-0" align="end">
							<RouteList steps={map.route.withSignatures(result.route)} />
						</Popover.Content>
					</Popover.Root>
				{/if}
			</div>
		{/each}
	</div>
{/if}

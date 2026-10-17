<script lang="ts">
	// Who is where: every pilot sharing their location on this map, and how far away. Pilots
	// who can act sort first; the ones who cannot (docked, podded, in a scanning frigate) sink
	// without being hidden. Distance is measured from the same origin as the watchlist and
	// Find, so every number on the page agrees about where "here" is.
	import type { MapCharacter } from '$lib/api/types/MapCharacter';
	import ClassBadge from '$lib/components/ClassBadge.svelte';
	import EveImage from '$lib/components/EveImage.svelte';
	import MapPanel from '$lib/components/map-panel/MapPanel.svelte';
	import MapPanelContent from '$lib/components/map-panel/MapPanelContent.svelte';
	import MapPanelHeader from '$lib/components/map-panel/MapPanelHeader.svelte';
	import SystemMenu from '../system-menu/SystemMenu.svelte';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { jumpTone } from '$lib/routing/algorithm';
	import { isIdle, isScanner, orderPilots } from '$lib/characters/order';
	import { cn } from '$lib/utils';
	import { clearHover, hoverSystem } from '../../state/map-hover';
	import { routeBatch } from '../../state/route-batch.svelte';
	import type { MapState } from '../../state/map-state.svelte';
	import RoutePopover from './RoutePopover.svelte';
	import RouteOriginBadge from './RouteOriginBadge.svelte';
	import { solarSystemId as solarSystemIdOf } from '$lib/map/system';
	import Hint from '$lib/components/Hint.svelte';

	let { map }: { map: MapState } = $props();

	const pilots = $derived(map.characters.all);
	const sorted = $derived(orderPilots(pilots));

	function place(solarSystemId: number | null) {
		if (solarSystemId === null) return null;
		const info = map.systemInfo(solarSystemId);
		if (!info) return null;
		const placed = map.systems.all.find((s) => solarSystemIdOf(s) === solarSystemId);
		return {
			id: placed?.id ?? null,
			alias: placed?.alias ?? null,
			name: info.name,
			region: info.region,
			classId: info.wormhole_class_id,
			security: info.security,
			info,
		};
	}

	// One search from the origin covers every pilot, rather than one per row.
	// svelte-ignore state_referenced_locally -- the map instance is stable for this mount.
	const batch = routeBatch(map, () =>
		pilots.map((p) => p.solar_system_id).filter((id): id is number => id !== null),
	);
	const routes = $derived(batch.routes);

	/** Highlight the pilot's node and draw their route while the row is hovered. */
	function hover(pilot: MapCharacter, on: boolean) {
		if (!on) {
			clearHover(map);
			return;
		}
		const route = pilot.solar_system_id === null ? undefined : routes.get(pilot.solar_system_id);
		hoverSystem(map, pilot.solar_system_id, route);
	}
</script>

<MapPanel testid="characters-card">
	<MapPanelHeader>
		<span class="inline-flex items-center gap-2">
			<span class="size-1.5 animate-pulse rounded-full bg-green-500"></span>
			Pilots
			<span class="font-mono text-amber-400">{pilots.length}</span>
		</span>
		{#snippet actions()}
			<RouteOriginBadge {map} />
		{/snippet}
	</MapPanelHeader>
	<MapPanelContent>
		{#if pilots.length === 0}
			<p class="px-3 py-4 text-xs text-muted-foreground" data-testid="pilots-empty">
				No pilots online. Anyone who turns on Share location for this map shows up here.
			</p>
		{:else}
			<div class="flex flex-col">
				{#each sorted as pilot (pilot.character_id)}
					{@const where = place(pilot.solar_system_id)}
					{@const route =
						pilot.solar_system_id === null ? undefined : routes.get(pilot.solar_system_id)}
					{#snippet row()}
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div
							class={cn(
								'flex items-center gap-2 border-b border-border/30 px-3 py-1 text-xs last:border-b-0 hover:bg-muted/30',
								isIdle(pilot) && 'opacity-50',
							)}
							data-testid="pilot-row"
							data-pilot={pilot.name}
							onmouseenter={() => hover(pilot, true)}
							onmouseleave={() => hover(pilot, false)}
						>
							<EveImage kind="character" id={pilot.character_id} class="size-5 shrink-0 rounded" />

							<span class="flex min-w-0 shrink-0 basis-28 items-center gap-1">
								<Hint text={pilot.name} clippedOnly focusable={false}>
									{#snippet child({ props })}
										<span
											class={cn('truncate', pilot.is_mine && 'font-medium text-foreground')}
											{...props}>{pilot.name}</span
										>
									{/snippet}
								</Hint>
								{#if pilot.is_docked}
									<Tooltip.Root>
										<Tooltip.Trigger class="shrink-0 text-[10px] text-muted-foreground">
											(D)
										</Tooltip.Trigger>
										<Tooltip.Content>Docked</Tooltip.Content>
									</Tooltip.Root>
								{:else if isScanner(pilot)}
									<Tooltip.Root>
										<Tooltip.Trigger class="shrink-0 text-[10px] text-amber-400"
											>(S)</Tooltip.Trigger
										>
										<Tooltip.Content>Scanner</Tooltip.Content>
									</Tooltip.Root>
								{/if}
							</span>

							<span class="flex min-w-0 flex-1 items-center gap-1.5">
								{#if pilot.ship_type_id !== null}
									<EveImage kind="type" id={pilot.ship_type_id} class="size-4 shrink-0" />
								{/if}
								<Tooltip.Root>
									<Tooltip.Trigger
										class="min-w-0 truncate font-mono text-[10px] text-muted-foreground"
									>
										{pilot.ship_type ?? 'Unknown ship'}
									</Tooltip.Trigger>
									<Tooltip.Content>{pilot.ship_name ?? 'Unnamed'}</Tooltip.Content>
								</Tooltip.Root>
							</span>

							<span class="flex min-w-0 flex-1 items-center gap-1.5">
								{#if where}
									<ClassBadge
										classId={where.classId}
										security={where.security}
										class="shrink-0 text-[10px]"
									/>
									<Hint text={`${where.name} · ${where.region}`}>
										{#snippet child({ props })}
											<button
												class="min-w-0 truncate text-left hover:text-foreground"
												{...props}
												disabled={where.id === null}
												onclick={() => where.id !== null && (map.activeId = where.id)}
											>
												{#if where.alias}<span class="font-medium">{where.alias}</span> ·
												{/if}{where.name}
											</button>
										{/snippet}
									</Hint>
								{:else}
									<span class="text-muted-foreground/60">Unknown</span>
								{/if}
							</span>

							<span class="shrink-0 text-right">
								{#if route}
									<RoutePopover {map} steps={route.route}>
										<span
											class={cn('cursor-pointer font-medium tabular-nums', jumpTone(route.jumps))}
											data-testid="pilot-jumps"
										>
											{route.jumps}j
										</span>
									</RoutePopover>
								{:else}
									<!-- `nowrap`: a lone hyphen pair wraps and makes the row a line taller. -->
									<span class="text-[10px] whitespace-nowrap text-muted-foreground/60">--</span>
								{/if}
							</span>
						</div>
					{/snippet}

					{#if where?.info}
						<SystemMenu system={where.info}>{@render row()}</SystemMenu>
					{:else}
						{@render row()}
					{/if}
				{/each}
			</div>
		{/if}
	</MapPanelContent>
</MapPanel>

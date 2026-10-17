<script lang="ts">
	// Cmd+K over one map: jump to a placed system, or add one that is not on the map yet.
	// Also the map's only system search, so add/connect/assign from the canvas open this with
	// an anchor for where the result should land.
	// Matching happens server-side (name, alias, occupier, and notes for members), so the
	// Command's own filtering is off and the rows arrive already ranked.
	import PlusIcon from '@lucide/svelte/icons/plus';

	import { q } from '$lib/api/queries';
	import { searchQuery } from '$lib/search-query.svelte';
	import { matchHint, partitionHits } from './palette';
	import type { MapSearchHit } from '$lib/api/types/MapSearchHit';
	import { Badge } from '$lib/components/ui/badge';
	import * as Command from '$lib/components/ui/command';
	import SystemRow from '../pickers/SystemRow.svelte';
	import { SYSTEM_CELLS_5, SYSTEM_LIST_HINT, SYSTEM_ROW } from '../pickers/columns';
	import SystemMenu from '../system-menu/SystemMenu.svelte';
	import { NODE_W } from '$lib/map/geometry';
	import type { MapState } from '../../state/map-state.svelte';
	import Hint from '$lib/components/Hint.svelte';

	let { map, open = $bindable() }: { map: MapState; open: boolean } = $props();

	let query = $state('');
	// Undebounced, like the palette always was: every keystroke searches.
	const search = searchQuery({
		term: () => query,
		query: (settled) => q.searchMap(map.mapId, settled),
		enabled: () => open,
		minChars: 0,
		debounceMs: 0,
	});
	const results = $derived(search.results);

	const canWrite = $derived(map.canWrite);
	const partitioned = $derived(partitionHits(results));
	const threatGroups = $derived(partitioned.threatGroups);
	const onMap = $derived(partitioned.onMap);
	const offMap = $derived(partitioned.offMap);
	const intel = $derived(partitioned.intel);
	/** Opened from "connect to a new system": every pick becomes a connection as well. */
	const linking = $derived(map.linkFrom !== null);
	/**
	 * Opened from a ghost's "assign a system": every pick says what that hole leads to,
	 * on-map hits included: those merge the ghost into the placement already there.
	 */
	const assigning = $derived(map.assignGhostId !== null);

	// The list owns the tracks and rows are subgrids of them, so on-map and off-map rows line
	// up with each other. Tracks: four SystemRow cells, the hint/badge, Command's indicator.
	const HEADING = 'col-span-full px-2 pt-2 pb-1 text-xs font-medium text-muted-foreground';

	$effect(() => {
		if (open) {
			query = '';
		} else {
			// Closing without picking drops the pending placement, so the next Cmd+K is a
			// plain search again rather than quietly still linking.
			map.linkFrom = null;
			map.assignGhostId = null;
			map.searchAnchor = null;
		}
	});

	function activate(hit: MapSearchHit) {
		if (assigning) {
			assign(hit.system.id);
			return;
		}
		// Already on the map and a connection was asked for: join the two instead of panning.
		if (linking && hit.map_solar_system_id !== null) {
			connect(hit.map_solar_system_id);
			return;
		}
		map.activeId = hit.map_solar_system_id;
		open = false;
		// Pan the node into the middle, so a jump from the palette actually shows it.
		const system = map.systems.all.find((s) => s.id === hit.map_solar_system_id);
		if (!system) return;
		const r = map.camera.viewportRect();
		map.camera.pan = {
			x: r.width / 2 - (system.position_x + NODE_W / 2) * map.camera.zoom,
			y: r.height / 2 - (system.position_y + map.nodeH / 2) * map.camera.zoom,
		};
	}

	/** Join an already-placed system to whatever asked for the connection. */
	function connect(target: number) {
		const from = map.linkFrom;
		open = false;
		if (from === null || from === target) return;
		map.connections.add(from, target);
	}

	/** Say which system a ghost turned out to be. */
	function assign(solarSystemId: number) {
		const ghost = map.assignGhostId;
		map.assignGhostId = null;
		open = false;
		if (ghost === null) return;
		map.systems.assignGhost({ map_solar_system_id: ghost, solar_system_id: solarSystemId });
	}

	/** Jump to it if it is already placed, otherwise put it on the map. */
	function open_(hit: MapSearchHit) {
		if (assigning) {
			if (canWrite) assign(hit.system.id);
			return;
		}
		if (hit.map_solar_system_id !== null) activate(hit);
		else if (canWrite) add(hit);
	}

	function add(hit: MapSearchHit) {
		// In assign mode this row means "that is where the hole goes", not "place it too".
		if (assigning) {
			assign(hit.system.id);
			return;
		}
		open = false;
		// The anchor is where the canvas was right-clicked. Plain Cmd+K has none, so the system
		// lands in the middle of the view.
		map.systems.add(hit.system.id, { anchor: map.searchAnchor, connectFrom: map.linkFrom });
	}
</script>

<Command.Dialog
	bind:open
	shouldFilter={false}
	title={assigning ? 'Assign a system' : linking ? 'Connect to a system' : 'Search this map'}
	description={assigning
		? 'Pick the system this hole turned out to lead to.'
		: linking
			? 'Pick the system on the other side of the connection.'
			: 'Jump to a system on the map, or add one that is not on it yet.'}
>
	<Command.Input
		placeholder={assigning
			? 'This hole leads to…'
			: linking
				? 'Connect to…'
				: 'System, alias, occupier or notes…'}
		bind:value={query}
	/>
	<!-- Headings are plain cells, not Command.Group: a group wraps its items in an element of
	     its own, and a subgrid cannot cross that break in the ancestry. -->
	<Command.List class="p-1 {SYSTEM_LIST_HINT}" data-testid="palette-list">
		<Command.Empty class="col-span-full">
			{query.trim().length < 2 ? 'Type at least two characters to search.' : 'Nothing found.'}
		</Command.Empty>
		{#if onMap.length > 0}
			<div class={HEADING}>On this map</div>
			{#each onMap as hit (hit.map_solar_system_id)}
				<Command.Item
					value={`on-${hit.system.id}`}
					onSelect={() => activate(hit)}
					class={SYSTEM_ROW}
					data-testid="palette-hit"
				>
					<SystemMenu system={hit.system} class={SYSTEM_CELLS_5}>
						<SystemRow system={hit.system} />
						<Hint text={matchHint(hit) ?? undefined} focusable={false}>
							{#snippet child({ props })}
								<span class="truncate text-xs text-muted-foreground" {...props}>
									{matchHint(hit) ?? ''}
								</span>
							{/snippet}
						</Hint>
					</SystemMenu>
				</Command.Item>
			{/each}
		{/if}
		{#if canWrite && offMap.length > 0}
			<div class={HEADING}>
				{assigning ? 'Not on the map' : linking ? 'Add and connect' : 'Add to the map'}
			</div>
			{#each offMap as hit (hit.system.id)}
				<Command.Item
					value={`off-${hit.system.id}`}
					onSelect={() => add(hit)}
					class={SYSTEM_ROW}
					data-testid="palette-add"
				>
					<SystemMenu system={hit.system} class={SYSTEM_CELLS_5}>
						<SystemRow system={hit.system} />
						{#if assigning}
							<Badge variant="outline" class="justify-self-end">Assign</Badge>
						{:else}
							<Badge variant="outline" class="justify-self-end gap-1">
								<PlusIcon />
								Add
							</Badge>
						{/if}
					</SystemMenu>
				</Command.Item>
			{/each}
		{/if}
		{#if intel.length > 0}
			<div class={HEADING}>Mentioned in intel</div>
			{#each intel as hit (hit.map_solar_system_id)}
				<Command.Item
					value={`intel-${hit.system.id}`}
					onSelect={() => activate(hit)}
					class={SYSTEM_ROW}
					data-testid="palette-hit"
				>
					<SystemMenu system={hit.system} class={SYSTEM_CELLS_5}>
						<SystemRow system={hit.system} />
						<Hint text={matchHint(hit) ?? undefined} focusable={false}>
							{#snippet child({ props })}
								<span class="truncate text-xs text-muted-foreground" {...props}>
									{matchHint(hit) ?? ''}
								</span>
							{/snippet}
						</Hint>
					</SystemMenu>
				</Command.Item>
			{/each}
		{/if}
		{#each threatGroups as group (group.id)}
			<div class="{HEADING} flex items-center gap-2" data-testid="palette-threat-group">
				<Hint text={`${group.name} (${group.kind})`} clippedOnly focusable={false}>
					{#snippet child({ props })}
						<span class="min-w-0 truncate text-foreground" {...props}>
							{group.name}
						</span>
					{/snippet}
				</Hint>
				<span
					class="ml-auto shrink-0 font-mono whitespace-nowrap tabular-nums text-muted-foreground/60"
				>
					{group.hits.length} × {group.total.toLocaleString()} kills
				</span>
			</div>
			{#each group.hits as hit (hit.system.id)}
				<Command.Item
					value={`threat-${group.id}-${hit.system.id}`}
					onSelect={() => open_(hit)}
					class={SYSTEM_ROW}
					data-testid="palette-threat"
				>
					<SystemMenu system={hit.system} class={SYSTEM_CELLS_5}>
						<SystemRow system={hit.system} />
						<span
							class="text-right font-mono text-xs tabular-nums text-muted-foreground"
							data-testid="palette-threat-kills"
						>
							{hit.threat?.kills.toLocaleString()}
						</span>
					</SystemMenu>
				</Command.Item>
			{/each}
		{/each}
	</Command.List>
</Command.Dialog>

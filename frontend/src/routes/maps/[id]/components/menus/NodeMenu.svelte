<script lang="ts">
	// Right-clicking a system node: what only a node can do (pin, connect, status, home,
	// remove, and naming a ghost), around the same system items every other menu offers.
	import HomeIcon from '@lucide/svelte/icons/home';
	import MapIcon from '@lucide/svelte/icons/map';
	import PinIcon from '@lucide/svelte/icons/pin';
	import SearchIcon from '@lucide/svelte/icons/search';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';
	import WaypointsIcon from '@lucide/svelte/icons/waypoints';

	import type { MapSystemView } from '$lib/api/types/MapSystemView';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import { STATUS_ICONS, STATUS_OPTIONS, statusColor, statusLabel } from '$lib/map/status';
	import type { MapState } from '../../state/map-state.svelte';
	import SystemMenuItems from '../system-menu/SystemMenuItems.svelte';

	let { map, system: s }: { map: MapState; system: MapSystemView } = $props();

	const canWrite = $derived(map.canWrite);
	const info = $derived(s.kind === 'system' ? map.systemInfo(s.solar_system_id) : null);

	/** The palette doubles as the picker: on-map hits merge, off-map ones name the ghost. */
	function assignSystem() {
		map.assignGhostId = s.id;
		map.linkFrom = null;
		map.searchAnchor = null;
		map.paletteOpen = true;
	}

	function connectFrom() {
		map.linkFrom = s.id;
		// Anchor on the source node: the placement helper steps out from there and owns the
		// spacing, so every way of adding a system leaves the same gap.
		map.searchAnchor = { x: s.position_x, y: s.position_y };
		map.paletteOpen = true;
	}
</script>

{#if canWrite && s.kind === 'ghost'}
	<DropdownMenu.Item onSelect={assignSystem}>
		<SearchIcon class="size-4" />
		Assign a system
	</DropdownMenu.Item>
{/if}

{#if s.kind === 'system'}
	{#if canWrite}
		<DropdownMenu.Item onSelect={() => map.systems.setPinned(s.id, !s.is_pinned)}>
			<PinIcon class="size-4" />
			{s.is_pinned ? 'Unpin' : 'Pin'}
		</DropdownMenu.Item>
		<DropdownMenu.Item onSelect={connectFrom}>
			<WaypointsIcon class="size-4" />
			Add connection
		</DropdownMenu.Item>
		<DropdownMenu.Sub>
			<DropdownMenu.SubTrigger data-testid="status-subtrigger">
				<MapIcon class="size-4" />
				Status
			</DropdownMenu.SubTrigger>
			<DropdownMenu.SubContent class="w-40" data-testid="status-submenu">
				{#each STATUS_OPTIONS as st (st)}
					{@const Icon = STATUS_ICONS[st]}
					<DropdownMenu.Item onSelect={() => map.systems.setStatus(s.id, st)}>
						{statusLabel(st)}
						<Icon class="ml-auto size-3.5" style="color: {statusColor(st)}" />
					</DropdownMenu.Item>
				{/each}
			</DropdownMenu.SubContent>
		</DropdownMenu.Sub>
		<DropdownMenu.Separator />
	{/if}

	{#if info}
		<SystemMenuItems {map} system={info} />
	{/if}

	{#if canWrite}
		<DropdownMenu.Item onSelect={() => map.systems.setHome(s.id, !s.is_home)}>
			<HomeIcon class="size-4" />
			{s.is_home ? 'Unset Home System' : 'Set as Home System'}
		</DropdownMenu.Item>
	{/if}
{/if}

{#if canWrite && !s.is_pinned && !s.is_home}
	<DropdownMenu.Separator />
	<!-- Removes the whole marquee selection when one exists, else just this node. -->
	<DropdownMenu.Item variant="destructive" onSelect={() => map.removeSelection(s.id)}>
		<Trash2Icon class="size-4" />
		Remove
	</DropdownMenu.Item>
{/if}

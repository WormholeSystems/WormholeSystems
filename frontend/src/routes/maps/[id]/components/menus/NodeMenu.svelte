<script lang="ts">
	// Right-clicking a system node: pin, connect, status, external sites, waypoints, and the
	// home/rally flags.
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import CompassIcon from '@lucide/svelte/icons/compass';
	import FlagIcon from '@lucide/svelte/icons/flag';
	import HomeIcon from '@lucide/svelte/icons/home';
	import MapIcon from '@lucide/svelte/icons/map';
	import MapPinIcon from '@lucide/svelte/icons/map-pin';
	import NavigationIcon from '@lucide/svelte/icons/navigation';
	import BanIcon from '@lucide/svelte/icons/ban';
	import PinIcon from '@lucide/svelte/icons/pin';
	import RouteIcon from '@lucide/svelte/icons/route';
	import SearchIcon from '@lucide/svelte/icons/search';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';
	import UsersIcon from '@lucide/svelte/icons/users';
	import WaypointsIcon from '@lucide/svelte/icons/waypoints';

	import type { MapSystemView } from '$lib/api/types/MapSystemView';
	import type { SystemStatus } from '$lib/api/types/SystemStatus';
	import EveImage from '$lib/components/EveImage.svelte';
	import { isWormholeClass } from '$lib/map/classes';
	import { systemLinkGroups } from '$lib/map/external-links';
	import { STATUS_ICONS, STATUS_OPTIONS, statusColor, statusLabel } from '$lib/map/status';
	import type { MapState } from '../../state/map-state.svelte';
	import { item, panel, sub } from './chrome';
	import ExternalLinks from './ExternalLinks.svelte';

	let { map, system: s }: { map: MapState; system: MapSystemView } = $props();

	function close() {
		map.closeMenu();
	}

	/** The palette doubles as the picker: on-map hits merge, off-map ones name the ghost. */
	function assignSystem(id: number) {
		map.assignGhostId = id;
		map.linkFrom = null;
		map.searchAnchor = null;
		map.paletteOpen = true;
		close();
	}

	function connectFrom(id: number) {
		map.linkFrom = id;
		// Anchor on the source node: the placement helper steps out from there and owns the
		// spacing, so every way of adding a system leaves the same gap.
		const source = map.systems.all.find((sys) => sys.id === id);
		map.searchAnchor = source ? { x: source.position_x, y: source.position_y } : null;
		map.paletteOpen = true;
		close();
	}

	function setStatus(id: number, status: SystemStatus) {
		map.systems.setStatus(id, status);
		close();
	}

	/** Removes the whole marquee selection when one exists, else just this node. */
	function removeSystem(id: number) {
		const ids = map.selected.size > 0 ? [...map.selected] : [id];
		map.selected = new Set();
		map.systems.remove(ids);
		close();
	}

	const online = $derived(map.characters.online);

	function waypoint(characterId: number, destinationId: number, clearOthers: boolean) {
		map.waypoints.set(destinationId, characterId, clearOthers);
		close();
	}

	function waypointAll(destinationId: number, clearOthers: boolean) {
		map.waypoints.setAll(destinationId, clearOthers);
		close();
	}
</script>

{#snippet waypointSubmenu(label: string, destinationId: number, clearOthers: boolean)}
	{@const LabelIcon = clearOthers ? NavigationIcon : MapPinIcon}
	<div class={sub} data-testid="{clearOthers ? 'destination' : 'waypoint'}-subtrigger">
		<LabelIcon class="size-4" />
		{label}
		<ChevronRightIcon class="ml-auto size-3" />
		<div class={panel} data-testid="{clearOthers ? 'destination' : 'waypoint'}-submenu">
			{#if online.length === 0}
				<div class="px-3 py-1 text-xs text-muted-foreground">No characters online</div>
			{:else}
				{#each online as c (c.character_id)}
					<button class={item} onclick={() => waypoint(c.character_id, destinationId, clearOthers)}>
						<EveImage kind="character" id={c.character_id} class="size-5 rounded-lg" />
						{c.name}
					</button>
				{/each}
				{#if online.length > 1}
					<div class="my-0.5 border-t border-border"></div>
					<button class={item} onclick={() => waypointAll(destinationId, clearOthers)}>
						<UsersIcon class="size-4" />
						All Characters
					</button>
				{/if}
			{/if}
		</div>
	</div>
{/snippet}

{#if s.kind === 'ghost'}
	<button class={item} onclick={() => assignSystem(s.id)}>
		<SearchIcon class="size-4" />
		Assign a system
	</button>
	<div class="my-0.5 border-t border-border"></div>
{/if}
{#if s.kind === 'system'}
	<button
		class={item}
		onclick={() => {
			map.systems.setPinned(s.id, !s.is_pinned);
			close();
		}}
	>
		<PinIcon class="size-4" />
		{s.is_pinned ? 'Unpin' : 'Pin'}
	</button>
	<button class={item} onclick={() => connectFrom(s.id)}>
		<WaypointsIcon class="size-4" />
		Add connection
	</button>
{/if}

{#if s.kind === 'system'}
	<div class={sub} data-testid="status-subtrigger">
		<MapIcon class="size-4" />
		Status
		<ChevronRightIcon class="ml-auto size-3" />
		<div class={panel} data-testid="status-submenu">
			{#each STATUS_OPTIONS as st (st)}
				{@const Icon = STATUS_ICONS[st]}
				<button class={item} onclick={() => setStatus(s.id, st)}>
					{statusLabel(st)}
					<Icon class="ml-auto size-3.5" style="color: {statusColor(st)}" />
				</button>
			{/each}
		</div>
	</div>

	<div class="my-0.5 border-t border-border"></div>

	<ExternalLinks
		groups={systemLinkGroups({
			solarSystemId: s.solar_system_id,
			name: s.name,
			region: s.region,
			regionId: s.region_id,
			constellationId: s.constellation_id,
			isWormhole: isWormholeClass(s.wormhole_class_id),
		})}
	/>

	{#if !isWormholeClass(s.wormhole_class_id)}
		{@render waypointSubmenu('Set destination', s.solar_system_id, true)}
		{@render waypointSubmenu('Add waypoint', s.solar_system_id, false)}
	{/if}

	<div class={sub} data-testid="route-subtrigger">
		<RouteIcon class="size-4" />
		Route planner
		<ChevronRightIcon class="ml-auto size-3" />
		<div class={panel} data-testid="route-submenu">
			<button
				class={item}
				onclick={() => {
					map.route.fromId = s.solar_system_id;
					close();
				}}
			>
				<CompassIcon class="size-4" />
				Set as origin
			</button>
			<button
				class={item}
				onclick={() => {
					map.route.toId = s.solar_system_id;
					close();
				}}
			>
				<NavigationIcon class="size-4" />
				Set as destination
			</button>
			{#if map.canManage}
				{@const ignored = map.ignored.has(s.solar_system_id)}
				<div class="my-0.5 border-t border-border"></div>
				<button
					class={item}
					data-testid="ignore-for-everyone"
					onclick={() => {
						if (ignored) map.ignored.remove(s.solar_system_id);
						else map.ignored.add(s.solar_system_id);
						close();
					}}
				>
					<BanIcon class="size-4" />
					{ignored ? 'Stop ignoring for everyone' : 'Ignore for everyone'}
				</button>
			{/if}
		</div>
	</div>

	<div class="my-0.5 border-t border-border"></div>

	<button
		class={item}
		onclick={() => {
			map.systems.setHome(s.id, !s.is_home);
			close();
		}}
	>
		<HomeIcon class="size-4" />
		{s.is_home ? 'Unset Home System' : 'Set as Home System'}
	</button>
	<button
		class={item}
		onclick={() => {
			map.systems.setRally(s.id, !s.is_rally);
			close();
		}}
	>
		<FlagIcon class="size-4" />
		{s.is_rally ? 'Clear Rally Point' : 'Set as Rally Point'}
	</button>
{/if}

{#if !s.is_pinned && !s.is_home}
	<div class="my-0.5 border-t border-border"></div>
	<button class="{item} text-destructive hover:text-destructive" onclick={() => removeSystem(s.id)}>
		<Trash2Icon class="size-4" />
		Remove
	</button>
{/if}

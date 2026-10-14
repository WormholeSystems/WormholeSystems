<script lang="ts">
	// Systems are draggable DOM nodes on a panned/zoomed world, connections one SVG overlay.
	// Every mutation publishes a MapEvent server-side and the WS triggers a refetch; pan, zoom
	// and selection live outside the fetched data and nodes are keyed by id, so a refetch
	// keeps interaction state.
	import { solarSystemId } from '$lib/map/system';
	import { tickingMs } from '$lib/now.svelte';
	import { trackedPilots } from '$lib/map/tracked-pilots';
	import { deepLinkTarget, systemParamUrl } from '../state/deep-link';

	import { afterNavigate, replaceState } from '$app/navigation';
	import { page } from '$app/state';

	import { Button } from '$lib/components/ui/button';
	import type { MapSystemView } from '$lib/api/types/MapSystemView';
	import type { MapUserSettings } from '$lib/api/types/MapUserSettings';
	import type { MapView } from '$lib/api/types/MapView';
	import { NODE_W, railEndpoint, canvasBackground } from '$lib/map/helpers';
	import { connectionCountByPlacement, pilotsBySystem, sigCountsBySystem } from '$lib/map/grouping';
	import { curveBetween } from '$lib/map/edges';
	import CanvasControls from './canvas/CanvasControls.svelte';
	import ConnectionPopover from './connection/ConnectionPopover.svelte';
	import ContextMenu from './menus/ContextMenu.svelte';
	import LoadingCover from './canvas/LoadingCover.svelte';
	import MapEdge from './canvas/MapEdge.svelte';
	import { MapGestures } from '../state/map-gestures.svelte';
	import { setMapContext } from '$lib/components/system-menu/context';
	import { connectMapSession } from '../state/map-session.svelte';
	import { MapState } from '../state/map-state.svelte';
	import Scrollbars from './canvas/Scrollbars.svelte';
	import RallyBadge from './canvas/RallyBadge.svelte';
	import SystemNode from './canvas/SystemNode.svelte';
	import CommandPalette from './overlays/CommandPalette.svelte';
	import LayoutToolbar from './panels/LayoutToolbar.svelte';
	import PanelGrid from './panels/PanelGrid.svelte';
	import CleanMapDialog from './overlays/CleanMapDialog.svelte';
	import IntroductionDialog from './overlays/IntroductionDialog.svelte';
	import TourOverlay from './overlays/TourOverlay.svelte';
	import StatusBar from './canvas/StatusBar.svelte';
	import TrackingDialog from './tracking/TrackingDialog.svelte';
	import { JumpTracker } from '../state/tracking.svelte';

	// Mounted under `{#key mapId}`, so every map gets a fresh instance and construction
	// happens at component init, where queries and effects are legal.
	let {
		mapId,
		signedIn,
		seed,
	}: {
		mapId: number;
		signedIn: boolean;
		seed: { view: MapView | null; settings: MapUserSettings | null };
	} = $props();

	// svelte-ignore state_referenced_locally -- initial values are the point: the `{#key}`
	// above remounts this component whenever the map changes.
	const map = new MapState(mapId, signedIn, seed);
	const canWrite = $derived(map.canWrite);
	// Built with the map, so navigating between maps never carries a half-seen jump over.
	const tracker = new JumpTracker(map.trackerHost());
	// Every fresh reading is observed as it lands, whatever triggered the fetch.
	// Structural sharing means this fires only when the payload actually changed.
	$effect(() => {
		void map.characters.mine;
		tracker.observe();
	});
	// Where each followed pilot last took the selection, so clicking elsewhere holds until
	// one of them moves.
	const followed = new Map<number, number>();
	$effect(() => {
		if (!map.userSettings?.follow_character) return;
		const pilots = trackedPilots(map.characters.mine, map.userSettings.tracked_character_ids);
		for (const pilot of pilots) {
			const systemId = pilot.online ? (pilot.solar_system_id ?? null) : null;
			if (systemId === null || followed.get(pilot.character_id) === systemId) continue;
			// A jump into an unmapped hole lands here again once the tracker has drawn it.
			const placed = map.systems.all.find(
				(s) => s.kind === 'system' && s.solar_system_id === systemId,
			);
			if (!placed) continue;
			followed.set(pilot.character_id, systemId);
			map.activeId = placed.id;
		}
	});
	// The edge countdowns; one clock for every line rather than a timer per edge.
	const clock = tickingMs(30_000);
	const gestures = new MapGestures(map);
	// The app-wide system context menu reads the map through this getter.
	setMapContext(() => map);

	let viewportEl = $state<HTMLElement | null>(null);

	// The viewer's own picture behind the chain, if they put one there.
	const background = $derived(
		canvasBackground({
			gridLines: !map.layoutLocked,
			cell: map.grid.cell_size,
			image: map.userSettings?.background_image_url ?? null,
			mode: map.userSettings?.background_image_mode ?? 'grid',
		}),
	);

	// The walkthrough hands over to the spotlight tour the moment it closes.
	let tourOpen = $state(false);

	// The loading cover starts where the map's own chrome does, leaving the app's nav above
	// it. Measured rather than assumed, since the nav can wrap.
	let chromeEl = $state<HTMLElement | null>(null);
	let coverTop = $state(0);
	$effect(() => {
		const el = chromeEl;
		if (!el) return;
		const measure = () => (coverTop = el.getBoundingClientRect().top);
		measure();
		window.addEventListener('resize', measure);
		return () => window.removeEventListener('resize', measure);
	});

	$effect(() => {
		map.camera.viewportEl = viewportEl;
	});

	$effect(() => {
		map.camera.restoreZoom();
	});

	// A `?system=` deep link activates its system once the graph is in, exactly once:
	// clearing the param instead would race the load-time restore.
	let deepLinkApplied = false;
	$effect(() => {
		if (deepLinkApplied || !map.loaded) return;
		deepLinkApplied = true;
		if (map.activeId === null) {
			map.activeId = deepLinkTarget(map.systems.all, page.url.searchParams.get('system'));
		}
	});

	// `getBoundingClientRect` is not reactive, and the virtual scrollbars derive from this.
	$effect(() => {
		const el = viewportEl;
		if (!el) return;
		const observer = new ResizeObserver(([entry]) => {
			const box = entry.contentRect;
			map.camera.viewportSize = { width: box.width, height: box.height };
			map.camera.remeasure();
		});
		observer.observe(el);
		// The camera keeps the canvas's screen position between gestures; anything that can
		// shift it without resizing it has to say so. Captured, so a scrolling panel counts.
		const moved = () => map.camera.remeasure();
		window.addEventListener('scroll', moved, { capture: true, passive: true });
		window.addEventListener('resize', moved, { passive: true });
		return () => {
			observer.disconnect();
			window.removeEventListener('scroll', moved, { capture: true });
			window.removeEventListener('resize', moved);
		};
	});

	// Realtime: any frame on the map socket means "refetch".
	$effect(() => connectMapSession(map));

	// `replaceState` throws until the router has started, which is still mid-hydration on
	// first paint.
	let routerReady = $state(false);
	afterNavigate(() => (routerReady = true));

	// Only ever writes the param: clearing it would race the load-time restore, which reads
	// `?system=` before the map data has arrived and an active system exists.
	$effect(() => {
		const active = map.activeSystem;
		if (!routerReady || active?.kind !== 'system') return;
		const url = systemParamUrl(page.url, active.solar_system_id);
		if (url) replaceState(url, {});
	});

	// Wheel: plain scrolls the page, ctrl/meta is swallowed so pinch does not zoom the whole
	// app, shift pans the map. Needs a non-passive listener to be allowed to preventDefault.
	$effect(() => {
		const el = viewportEl;
		if (!el) return;
		const onWheel = (ev: WheelEvent) => {
			if (ev.ctrlKey || ev.metaKey) {
				ev.preventDefault();
				return;
			}
			if (!ev.shiftKey) return;
			ev.preventDefault();
			map.camera.panBy(-ev.deltaX, -ev.deltaY);
		};
		el.addEventListener('wheel', onWheel, { passive: false });
		return () => el.removeEventListener('wheel', onWheel);
	});

	function onKey(ev: KeyboardEvent) {
		if (ev.key === 'Delete' || ev.key === 'Backspace') {
			const ids = [...map.selected];
			if (ids.length > 0) {
				ev.preventDefault();
				map.selected = new Set();
				map.systems.remove(ids);
			}
		}
	}

	function handleNodeSelect(ev: PointerEvent, s: MapSystemView) {
		// The active system drives the side panels and the amber ring. The marquee selection
		// is untouched.
		if (ev.button !== 0) return;
		ev.stopPropagation();
		map.activeId = s.id;
	}

	const sigCounts = $derived(sigCountsBySystem(map.signatures.all));
	const pilots = $derived(pilotsBySystem(map.characters.all));
	const connCounts = $derived(connectionCountByPlacement(map.connections.all));

	function saveAlias(s: MapSystemView, alias: string | null, occupier: string | null) {
		map.systems.rename(s, alias, occupier);
	}
</script>

<svelte:window
	onkeydown={(ev) => {
		if (ev.key === 'Escape') {
			map.closeMenu();
			map.connectionPopover = null;
		}
		if (ev.key === 'k' && (ev.metaKey || ev.ctrlKey)) {
			ev.preventDefault();
			map.paletteOpen = !map.paletteOpen;
		}
	}}
/>

<div bind:this={chromeEl}>
	<StatusBar {map} onstarttour={() => (tourOpen = true)} />
	<TourOverlay bind:open={tourOpen} />
</div>

<CommandPalette {map} bind:open={map.paletteOpen} />
<CleanMapDialog {map} bind:open={map.cleanPrompt} />
<TrackingDialog {map} {tracker} />

{#if map.loadError}
	<!-- A withdrawn link and a map that was never shared look the same to a signed-out
	     visitor, so the answer is the same too. -->
	{#if !signedIn}
		<div class="flex flex-col items-center gap-4 p-12 text-center" data-testid="map-error">
			<p class="text-sm text-muted-foreground">
				This map is not open to watch. The link may have been withdrawn.
			</p>
			<Button href="/login" variant="outline">Sign in</Button>
		</div>
	{:else}
		<p class="p-12 text-center text-sm text-destructive" data-testid="map-error">
			{map.loadError}
		</p>
	{/if}
{:else}
	<!-- Mounted as soon as the data is in, behind the cover: the arrangement is painted
	     while nobody is looking at it, so nothing is ever seen moving into place. -->
	{#if map.ready}
		<PanelGrid {map} {canvas} />
		{#if map.panels.editing}
			<LayoutToolbar {map} />
		{/if}
	{/if}
	<LoadingCover {map} top={coverTop} />
{/if}

{#snippet canvas()}
	<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_static_element_interactions -->
	<div
		bind:this={viewportEl}
		data-testid="map-canvas"
		tabindex="0"
		class="group relative h-full w-full overflow-hidden bg-canvas ring-1 ring-border ring-offset-[-0.5px] outline-none select-none"
		onpointerdown={(ev) => gestures.onBackgroundDown(ev)}
		onpointerenter={() => map.camera.wakeScrollbars()}
		onpointermove={(ev) => gestures.onPointerMove(ev)}
		onpointerup={(ev) => gestures.onPointerUp(ev)}
		onkeydown={onKey}
		oncontextmenu={(ev) => {
			ev.preventDefault();
			map.openMenu(ev.clientX, ev.clientY, { kind: 'map' });
		}}
		style={background.viewport}
	>
		<IntroductionDialog {map} onfinished={() => (tourOpen = true)} />

		<div
			class="absolute top-0 left-0 origin-top-left"
			style={background.world}
			style:width="{map.grid.world_width}px"
			style:height="{map.grid.world_height}px"
			style:transform="translate({map.camera.pan.x}px, {map.camera.pan.y}px) scale({map.camera
				.zoom})"
		>
			<svg
				class="absolute top-0 left-0 overflow-visible"
				style:width="{map.grid.world_width}px"
				style:height="{map.grid.world_height}px"
			>
				{#each map.connections.all as c (c.id)}
					{@const geometry = map.edgeGeometry.get(c.id)}
					{#if geometry}
						<MapEdge {map} connection={c} {geometry} now={clock.current} />
					{/if}
				{/each}

				{#if map.linking}
					{@const from = map.positions.get(map.linking.from)}
					{#if from}
						{@const start = railEndpoint(
							from.x,
							from.x + NODE_W,
							from.y + map.nodeH / 2,
							map.linking.x,
						)}
						<path
							d={curveBetween(start, { x: map.linking.x, y: map.linking.y })}
							fill="none"
							stroke="var(--color-edge)"
							stroke-width="4"
							stroke-linecap="round"
							stroke-dasharray="2 6"
						/>
					{/if}
				{/if}

				{#if map.band}
					{@const b = map.band}
					<rect
						x={Math.min(b.x0, b.x1)}
						y={Math.min(b.y0, b.y1)}
						width={Math.abs(b.x1 - b.x0)}
						height={Math.abs(b.y1 - b.y0)}
						fill="rgba(99,102,241,0.12)"
						stroke="#6366f1"
						stroke-width="1"
					/>
				{/if}
			</svg>

			<!-- Keyed by id so a refetch diffs in place. -->
			{#each map.systems.all as s (s.id)}
				<SystemNode
					node={s}
					nodeH={map.nodeH}
					selected={map.selected.has(s.id)}
					highlighted={map.hoveredSystemId === s.id}
					pos={map.positions.get(s.id) ?? { x: 0, y: 0 }}
					sigCounts={sigCounts.get(solarSystemId(s) ?? -1) ?? {
						total: 0,
						uncategorized: 0,
						wormholes: 0,
					}}
					connectionCount={connCounts.get(s.id) ?? 0}
					pilots={pilots.get(solarSystemId(s) ?? -1) ?? []}
					showThreat={map.userSettings?.show_threat_level ?? true}
					draggable={!map.layoutLocked && canWrite}
					linkable={canWrite}
					editable={canWrite}
					signatureId={map.ghostSignatures.get(s.id) ?? null}
					onsavealias={(alias, occupier) => saveAlias(s, alias, occupier)}
					active={map.activeId === s.id}
					onselect={(ev) => handleNodeSelect(ev, s)}
					ondown={(ev) => gestures.onNodeDown(ev, s)}
					onlink={(ev) => gestures.onLinkDown(ev, s.id)}
					onmenu={(ev) => {
						ev.preventDefault();
						ev.stopPropagation();
						map.openMenu(ev.clientX, ev.clientY, { kind: 'node', system: s });
					}}
				/>
			{/each}
		</div>

		<Scrollbars {map} />

		<RallyBadge {map} />

		<CanvasControls {map} />

		{#if map.menu}
			<ContextMenu {map} menu={map.menu} />
		{/if}
		<ConnectionPopover {map} />
	</div>
{/snippet}

// The map page's shared state. Interaction state is owned here, never derived from the
// fetched data, so it survives refetches. The fetched data itself lives in the query
// cache ([`createMapQueries`]); the domain namespaces (`systems`, `connections`, ...)
// are views over it plus every verb that changes it, so components read `map.x.all`
// and call `map.x.verb()` without ever touching the api or the cache themselves.

import { untrack } from 'svelte';

import { errorMessage } from '$lib/api/client';
import { key, q } from '$lib/api/queries';
import type { GridConfig } from '$lib/api/types/GridConfig';
import type { MapSystemView } from '$lib/api/types/MapSystemView';
import type { MapUserSettings } from '$lib/api/types/MapUserSettings';
import type { UpdateMapUserSettings } from '$lib/api/types/UpdateMapUserSettings';
import type { SystemDetails } from '$lib/api/types/SystemDetails';
import type { SystemSearchResult } from '$lib/api/types/SystemSearchResult';
import type { SocketState } from '$lib/ws';
import type { PanelId } from './layout/registry';
import type { RoutingSettings } from '$lib/routing/algorithm';
import { NODE_W, clamp, type Vec2 } from '$lib/map/geometry';
import { freeEdgeRouter, treeEdges, type EdgeGeometry } from '$lib/map/edges';
import { draggedPositions, type Drag } from '$lib/map/gestures';
import { compareForTree, computeTreeLayout } from '$lib/map/tree';
import {
	edgeTopologyKey,
	nodePositions,
	reusePositions,
	settlePending,
	treeShapeKey,
} from '$lib/map/positions';
import { routeOrigin } from '$lib/routing/origin';
import { ghostSignatureIds } from '$lib/map/ghosts';
import { layoutMode } from '$lib/map/layout-mode';
import { orphanedSystems } from '$lib/map/orphans';
import type { MapAction, RunAction } from '$lib/map/actions';
import { MapCamera } from './map-camera.svelte';
import { HistoryStore } from './history.svelte';
import { createMapQueries, type MapQueries, type MapSeed } from './map-queries.svelte';
import { PanelLayoutStore } from './panel-layout.svelte';
import { RoutePlanner, type RouteHost } from './route-planner.svelte';
import type { TrackerHost } from './tracking.svelte';
import type { MapEvent } from '$lib/api/types/MapEvent';
import { loadCatalog } from '$lib/map/signatures';
import { solarSystemId } from '$lib/map/system';
import { atLeast } from '$lib/map/roles';
import { systemResolver } from '$lib/resolve-cache.svelte';
import { CharactersApi } from './characters';
import { ConnectionsApi } from './connections';
import { SignaturesApi } from './signatures';
import { SystemsApi } from './systems';
import { IgnoredApi } from './ignored';
import { WatchlistApi } from './watchlist';
import { WaypointsApi } from './waypoints';
import { PlacementApi } from './placement';
import { BackgroundApi } from './background';

export type { Drag };

/** A placed system in the shape the resolver serves, so both sides answer alike. */
function searchResultOf(placed: Extract<MapSystemView, { kind: 'system' }>): SystemSearchResult {
	return {
		id: placed.solar_system_id,
		name: placed.name,
		security: placed.security_status,
		region: placed.region,
		region_id: placed.region_id,
		constellation_id: placed.constellation_id,
		wormhole_class_id: placed.wormhole_class_id,
		effect_name: placed.effect_name,
		sovereignty: placed.sovereignty,
		statics: placed.statics,
	};
}

/** An in-progress connection drag: from this placement to the current cursor (world coords). */
export interface Linking {
	from: number;
	x: number;
	y: number;
}

export type MenuTarget =
	{ kind: 'map' } | { kind: 'node'; system: MapSystemView } | { kind: 'connection'; id: number };

/** An open right-click menu, positioned at screen `(x, y)`. */
export interface Menu {
	x: number;
	y: number;
	/**
	 * The same point on the map, taken when the menu opened: anything that later moves the
	 * page (a menu taking focus, a scroll) must not move where an added system lands.
	 */
	world: Vec2;
	target: MenuTarget;
}

const defaultGrid: GridConfig = {
	cell_size: 20,
	world_width: 4000,
	world_height: 2000,
	viewport_height: 1400,
};

export class MapState {
	mapId: number;
	camera: MapCamera;
	panels: PanelLayoutStore;
	route: RoutePlanner;
	queries: MapQueries;

	// The domain namespaces: data plus verbs, one per noun.
	systems: SystemsApi;
	connections: ConnectionsApi;
	signatures: SignaturesApi;
	watchlist: WatchlistApi;
	ignored: IgnoredApi;
	characters: CharactersApi;
	waypoints: WaypointsApi;
	placement: PlacementApi;
	background: BackgroundApi;

	get data() {
		return this.queries.graph.data ?? null;
	}
	get grid() {
		return this.queries.grid.data ?? defaultGrid;
	}
	get eveScout() {
		return this.queries.eveScout.data ?? [];
	}
	private get ignoredRows() {
		return this.queries.ignored.data ?? [];
	}
	get userSettings() {
		return this.queries.settings.data ?? null;
	}

	get naming() {
		return this.data?.map.naming ?? null;
	}

	selected = $state<Set<number>>(new Set());
	drag = $state<Drag | null>(null);
	// Optimistic positions held from drop until the server confirms them, so a moved node
	// doesn't flash back to its old spot during the refetch round-trip.
	pending = $state<Record<number, { x: number; y: number }>>({});
	linking = $state<Linking | null>(null);
	band = $state<{ x0: number; y0: number; x1: number; y1: number } | null>(null);
	menu = $state<Menu | null>(null);
	panDrag = $state<{ cx: number; cy: number; px: number; py: number } | null>(null);
	paletteOpen = $state(false);
	/** The "clean map" confirmation, opened from the status bar hint or the map menu. */
	cleanPrompt = $state(false);
	// The connection details popover: which edge, anchored at which screen point.
	connectionPopover = $state<{ id: number; x: number; y: number } | null>(null);
	// Where a search-added system should land (world coords, top-left). Set by the
	// context menu (right-click spot / next to the source node); null = viewport center.
	searchAnchor = $state<{ x: number; y: number } | null>(null);
	// The active system (legacy model): set by clicking a node body, drives the side
	// panels and the amber ring. Independent of the marquee selection.
	activeId = $state<number | null>(null);
	// When set, a search pick also connects the new placement to this node.
	linkFrom = $state<number | null>(null);
	// When set, a search pick says which system this ghost placement turned out to be,
	// instead of placing a new one.
	assignGhostId = $state<number | null>(null);
	// A row hovered in a side panel: the node it names lights up. Owned here so any card
	// can point at the map.
	hoveredSystemId = $state<number | null>(null);
	// The history tree plus where the map sits in it (`history.data`), and the cursor
	// controls over it.
	history: HistoryStore;
	socket = $state<SocketState>('connecting');
	// The page holds a loader until both the graph and the arrangement have arrived, so
	// tiles are never painted in the built-in positions and then moved.
	get loaded() {
		return this.queries.graph.data !== undefined;
	}
	// A disabled query stays pending forever, so the signed-out case answers first.
	get settingsLoaded() {
		return !this.signedIn || !this.queries.settings.isPending;
	}
	// Only a first load with nothing to show gates the page; a later failure leaves the
	// view briefly stale instead, because the cache keeps the last good data.
	get loadError() {
		return this.queries.graph.data === undefined && this.queries.graph.isError
			? errorMessage(this.queries.graph.error)
			: '';
	}
	get ready() {
		return this.loaded && this.settingsLoaded;
	}

	private systemsView = $derived(this.data?.systems ?? []);
	private connectionsView = $derived(this.data?.connections ?? []);
	private systemsById = $derived(new Map(this.systemsView.map((s) => [s.id, s])));
	private connectionsById = $derived(new Map(this.connectionsView.map((c) => [c.id, c])));
	activeSystem = $derived(this.systemsView.find((s) => s.id === this.activeId) ?? null);
	routeOrigin = $derived.by(() =>
		routeOrigin(
			this.route.fromId,
			this.activeSystem ? solarSystemId(this.activeSystem) : null,
			this.characters.mine,
		),
	);

	// One derived per field, so the object below is rebuilt only when a routing setting
	// itself changed. Built straight off the settings it would be a new object after every
	// save of anything, and every route on the page would be searched again.
	private routePreference = $derived(this.userSettings?.route_preference ?? 'shorter');
	private securityPenalty = $derived(this.userSettings?.security_penalty ?? 50);
	private allowTimeStatus = $derived(this.userSettings?.route_allow_time_status ?? 'critical');
	private allowMassStatus = $derived(this.userSettings?.route_allow_mass_status ?? 'reduced');
	routingSettings = $derived<RoutingSettings>({
		preference: this.routePreference,
		securityPenalty: this.securityPenalty,
		allowTimeStatus: this.allowTimeStatus,
		allowMassStatus: this.allowMassStatus,
	});
	useEveScout = $derived(this.userSettings?.route_use_evescout ?? false);
	useWormholes = $derived(this.userSettings?.route_use_wormholes ?? true);

	nodeH = $derived(2 * this.grid.cell_size);

	layout = $derived.by(() =>
		layoutMode(this.data?.map ?? null, this.userSettings?.layout_override),
	);

	/** Automatic placement owns the positions, so dragging one would fight the layout. */
	layoutLocked = $derived(this.layout === 'tree');

	/**
	 * What the tree layout reads off the chain, as a string. A string compares by value, so
	 * a refetch that changed only statuses or masses leaves it equal and the layout below
	 * does not rerun.
	 */
	private treeShape = $derived(
		this.layout === 'tree' ? treeShapeKey(this.systemsView, this.connectionsView) : null,
	);

	/** Derived from the shape of the chain, recomputed only when that shape changes. */
	treePositions = $derived.by(() => {
		if (this.treeShape === null) return null;
		const gridSize = this.grid.cell_size;
		// The shape key above already says when this has to rerun; reading the arrays
		// tracked would rerun it on every refetch.
		return untrack(() => {
			const systems = new Map(this.systemsView.map((s) => [s.id, s]));
			return computeTreeLayout(
				{
					nodeIds: this.systemsView.map((s) => s.id),
					edges: this.connectionsView.map((c) => ({ from: c.from_system, to: c.to_system })),
					rootIds: this.systemsView.filter((s) => s.is_pinned).map((s) => s.id),
					homeId: this.systemsView.find((s) => s.is_home)?.id ?? null,
					fallbackRootId: null,
					compareNodes: compareForTree(systems),
				},
				{ gridSize },
			);
		});
	});

	ghostSignatures = $derived.by(() =>
		ghostSignatureIds(this.systemsView, this.connectionsView, this.queries.signatures.data ?? []),
	);

	/** Which connections exist and what they join; a status change leaves it equal. */
	private edgeTopology = $derived(edgeTopologyKey(this.connectionsView));
	private routeFreeEdges = freeEdgeRouter();

	/**
	 * Every connection's line and badge anchor, routed the way this layout draws them. Reruns
	 * when a node moves or the topology changes, not when a refetch only changed statuses.
	 */
	edgeGeometry = $derived.by<Map<number, EdgeGeometry>>(() => {
		void this.edgeTopology;
		const positions = this.positions;
		const nodeH = this.nodeH;
		const tree = this.layout === 'tree';
		return untrack(() =>
			tree
				? treeEdges(this.connectionsView, positions, nodeH)
				: this.routeFreeEdges(this.connectionsView, positions, nodeH),
		);
	});

	// The last lookup handed out, so the next one can keep the entries that did not move.
	// Plain field on purpose: it is a memo, not something anything should react to.
	private lastPositions: Map<number, Vec2> | null = null;

	// Position lookup: the automatic layout when one is active; else live drag, then an
	// optimistic override, then the server position. Entries keep their identity while
	// their coordinates do, so only the nodes and edges that moved redraw.
	positions = $derived.by(() => {
		const dragged = this.drag ? draggedPositions(this.drag) : new Map<number, Vec2>();
		const out = nodePositions(this.systemsView, this.treePositions, dragged, this.pending);
		const next = reusePositions(out, this.lastPositions);
		this.lastPositions = next;
		return next;
	});

	/**
	 * False for somebody watching through a share link or a public map: everything hung off
	 * an account is not fetched rather than fetched and refused.
	 */
	signedIn = $state(true);

	/** Editing takes the member role; below it the map is read-only. */
	canWrite = $derived(atLeast(this.data?.role, 'member'));
	/** The map-wide settings, the ignore list among them, are a manager's to change. */
	canManage = $derived(atLeast(this.data?.role, 'manager'));
	/** What the whole map keeps off its routes, as the router and the tracker read it. */
	ignoredIds = $derived(new Set(this.ignoredRows.map((row) => row.solar_system_id)));

	constructor(mapId: number, signedIn = true, seed: MapSeed = { view: null, settings: null }) {
		this.mapId = mapId;
		this.camera = new MapCamera(mapId);
		this.signedIn = signedIn;
		this.queries = createMapQueries(mapId, signedIn, seed, () => this.socket === 'open');
		const run: RunAction = (action, exec, detail) => this.run(action, exec, detail);
		this.systems = new SystemsApi({
			mapId,
			camera: this.camera,
			all: () => this.systemsView,
			byId: () => this.systemsById,
			orphaned: () => this.orphaned,
			grid: () => this.grid,
			run,
			notesLocal: (id, notes) =>
				this.queries.client.setQueryData(
					q.systemDetails(mapId, id).queryKey,
					(d: SystemDetails | undefined) => d && { ...d, notes },
				),
		});
		this.connections = new ConnectionsApi({
			mapId,
			all: () => this.connectionsView,
			byId: () => this.connectionsById,
			stale: () => this.queries.stale.data ?? [],
			systems: () => this.systemsView,
			run,
			refreshJumps: (connectionId) =>
				void this.queries.client.invalidateQueries({
					queryKey: key.connectionJumps(mapId, connectionId),
				}),
		});
		this.signatures = new SignaturesApi({
			mapId,
			all: () => this.queries.signatures.data ?? [],
			run,
		});
		this.watchlist = new WatchlistApi({
			mapId,
			all: () => this.queries.watchlist.data ?? [],
			run,
		});
		this.ignored = new IgnoredApi({
			mapId,
			all: () => this.queries.ignored.data ?? [],
			run,
		});
		this.characters = new CharactersApi({
			all: () => this.queries.characters.data ?? [],
			mine: () => this.queries.myCharacters.data ?? [],
			refresh: () => {
				if (this.signedIn)
					void this.queries.client.invalidateQueries({ queryKey: key.mapCharacters(mapId) });
			},
			refreshMine: () => {
				if (this.signedIn)
					void this.queries.client.invalidateQueries({ queryKey: key.myCharacters });
			},
		});
		this.waypoints = new WaypointsApi({ run });
		this.placement = new PlacementApi({
			mapId,
			mapLayout: () => this.data?.map.layout ?? null,
			run,
			patchSettingsLocal: (update) => this.queries.patchSettingsLocal(update),
			patchUserSettings: (patch) => this.patchUserSettings(patch),
		});
		this.background = new BackgroundApi({ mapId, writer: this.queries.backgroundWriter });
		this.history = new HistoryStore({
			mapId,
			data: () => this.queries.history.data ?? null,
			run,
		});
		this.panels = new PanelLayoutStore({
			hiddenPanels: () => this.userSettings?.hidden_panels ?? null,
			setHiddenPanels: (panels) =>
				this.queries.patchSettingsLocal((s) => ({ ...s, hidden_panels: panels })),
			save: (layouts, hidden) => {
				return this.run('saveLayout', () =>
					this.saveUserSettings(
						{ layout_breakpoints: layouts, hidden_panels: hidden },
						{ quiet: true },
					).then((saved) => saved.layout_breakpoints ?? null),
				);
			},
		});
		this.route = new RoutePlanner(this.routeHost());
		// Seeded from the page's load, so the first frame already has the arrangement; a
		// map opened without a seed gets it once the settings query lands.
		let layoutsSeeded = seed.settings != null;
		if (seed.settings) this.panels.seed(seed.settings.layout_breakpoints ?? null);
		$effect(() => {
			const settings = this.queries.settings.data;
			if (!settings || layoutsSeeded) return;
			layoutsSeeded = true;
			this.panels.seed(settings.layout_breakpoints ?? null);
		});

		// Each graph payload's arrival: the resolver learns the placed systems, and a
		// pending move is dropped once the server position matches it (ours landed) or
		// the system is gone.
		$effect(() => {
			const data = this.queries.graph.data;
			if (!data) return;
			this.shareResolved();
			const left = settlePending(
				untrack(() => this.pending),
				data.systems,
			);
			if (left) this.pending = left;
		});
	}

	/** The signature type catalog, cached forever in the query client. */
	loadCatalog() {
		return loadCatalog(this.queries.client);
	}

	/** An optimistic local edit of the viewer's settings, without a round trip. */
	patchSettingsLocal(update: (s: MapUserSettings) => MapUserSettings) {
		this.queries.patchSettingsLocal(update);
	}

	/** The jump tracker's narrow view of this map, commands included. */
	trackerHost(): TrackerHost {
		return {
			myCharacters: () => this.characters.mine,
			systems: () => this.systemsView,
			connections: () => this.connectionsView,
			sigs: () => this.signatures.all,
			grid: () => this.grid,
			settings: () => this.userSettings,
			naming: () => this.naming,
			ignored: () => this.ignoredIds,
			stargates: () => this.route.stargates,
			whenRoutingLoaded: () => this.route.whenLoaded(),
			loadCatalog: () => loadCatalog(this.queries.client),
			resolveSystem: (id) => systemResolver.resolve(id),
			trackJump: (cmd) => this.connections.trackJump(cmd),
			resolveGhost: (cmd) => this.systems.assignGhost(cmd),
		};
	}

	/** The planner's narrow view of this map; tables come from the query cache. */
	private routeHost(): RouteHost {
		return {
			mapId: this.mapId,
			systems: () => this.systemsView,
			connections: () => this.connectionsView,
			sigs: () => this.signatures.all,
			eveScout: () => this.eveScout,
			useEveScout: () => this.useEveScout,
			useWormholes: () => this.useWormholes,
			mapIgnored: () => this.ignoredIds,
			settings: () => this.routingSettings,
			loadTables: () => this.queries.client.ensureQueryData(q.routingGraph()),
		};
	}

	/** Panels there is nobody to fill in: both of these are about the account, not the map. */
	unavailablePanels = $derived(new Set<PanelId>(this.signedIn ? [] : ['characters', 'skyhooks']));

	/**
	 * A system in the shape every picker and the context menu expect, whether or not it is
	 * on the map. Reading an off-map system schedules its (deduplicated, batched) fetch as
	 * a side effect, so callers just read again: the answer is null until the row lands,
	 * and the read is reactive, so it lands on its own.
	 */
	systemInfo(id: number): SystemSearchResult | null {
		const placed = this.placedInfo.get(id);
		if (placed) return placed;
		const known = systemResolver.get(id);
		if (known === undefined) systemResolver.ensure([id]);
		return known ?? null;
	}

	/**
	 * The placed systems by solar system id, already in the resolver's shape. Built once per
	 * graph, since `systemInfo` is called from sort comparators and once per row.
	 */
	private placedInfo = $derived.by(() => {
		const out = new Map<number, SystemSearchResult>();
		for (const s of this.systemsView) {
			if (s.kind === 'system' && !out.has(s.solar_system_id)) {
				out.set(s.solar_system_id, searchResultOf(s));
			}
		}
		return out;
	});

	/** Placed systems arrive with the graph; the resolver learns them so no panel refetches one. */
	private shareResolved() {
		systemResolver.seed(this.placedInfo.values());
	}

	/** Refetch what one socket frame invalidated; the table lives in [`keysFor`]. */
	applyEvent(event: MapEvent | null) {
		this.queries.applyEvent(event);
	}

	/** Refetch the whole map subtree; resolves once the active refetches settle. */
	refetch() {
		return this.queries.invalidateAll();
	}

	/**
	 * Run an action and say how it went. What it says lives in [`MAP_ACTIONS`] rather than
	 * at the call site; the refetch policy lives on the mutation, in [`createMapQueries`].
	 * Private on purpose: components go through the domain namespaces.
	 */
	private run<T>(action: MapAction, exec: () => Promise<T>, detail?: string): Promise<T> {
		// The shared mutation hands back whatever `exec` resolved to.
		const write = this.queries.write.mutateAsync({ action, exec, detail }) as Promise<T>;
		// Already toasted by the mutation; this only keeps a dropped promise from being
		// reported as unhandled. A caller that waits still sees the rejection.
		write.catch(() => {});
		return write;
	}

	/**
	 * Write one or more of the viewer's own settings and move on. Everything goes through
	 * the one writer ([`createUserSettingsWriter`]), which toasts a failure.
	 */
	patchUserSettings(patch: UpdateMapUserSettings): void {
		this.queries.settingsWriter.save(this.mapId, patch);
	}

	/** The same write, for a caller that waits on the stored settings. */
	saveUserSettings(
		patch: UpdateMapUserSettings,
		opts?: { quiet?: boolean },
	): Promise<MapUserSettings> {
		return this.queries.settingsWriter.saveAsync(this.mapId, patch, opts);
	}

	private orphaned = $derived(orphanedSystems(this.systemsView, this.connectionsView));

	snap(v: number): number {
		return Math.round(v / this.grid.cell_size) * this.grid.cell_size;
	}

	clampNodeX(x: number): number {
		return clamp(x, 0, this.grid.world_width - NODE_W);
	}

	clampNodeY(y: number): number {
		return clamp(y, 0, this.grid.world_height - this.nodeH);
	}

	/**
	 * Take the marquee selection off the map, or just `fallbackId` when nothing is selected:
	 * removing one node of a selection removes them all.
	 */
	removeSelection(fallbackId?: number) {
		const ids =
			this.selected.size > 0 ? [...this.selected] : fallbackId === undefined ? [] : [fallbackId];
		if (ids.length === 0) return;
		this.selected = new Set();
		this.systems.remove(ids);
	}

	openMenu(x: number, y: number, target: MenuTarget) {
		this.camera.remeasure();
		this.menu = { x, y, world: this.camera.toWorld(x, y), target };
	}

	closeMenu() {
		this.menu = null;
	}
}

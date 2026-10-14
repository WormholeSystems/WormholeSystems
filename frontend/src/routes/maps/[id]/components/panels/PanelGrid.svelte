<script lang="ts">
	// The map page as a free-form grid: the canvas is a tile like any other. Tiles are
	// absolutely positioned from their {x,y,w,h} against the breakpoint's column count;
	// placement itself lives in `$lib/layout/grid`, this only turns pointers into calls.
	import { untrack } from 'svelte';
	import { browser } from '$app/environment';

	import XIcon from '@lucide/svelte/icons/x';

	import { bottom, compact, tileBox, type GridItem } from '$lib/layout/grid';
	import MapPanel from '$lib/components/map-panel/MapPanel.svelte';
	import MapPanelContent from '$lib/components/map-panel/MapPanelContent.svelte';
	import MapPanelHeader from '$lib/components/map-panel/MapPanelHeader.svelte';
	import { cn } from '$lib/utils';
	import NavigationCard from './NavigationCard.svelte';
	import CharactersCard from './CharactersCard.svelte';
	import EveScoutCard from './EveScoutCard.svelte';
	import KillmailsCard from './KillmailsCard.svelte';
	import NotesCard from './NotesCard.svelte';
	import SkyhooksCard from './SkyhooksCard.svelte';
	import SystemInfoCard from './SystemInfoCard.svelte';
	import ThreatCard from './ThreatCard.svelte';
	import SignaturesPanel from '../signatures/SignaturesPanel.svelte';
	import {
		type BreakpointKey,
		type PanelId,
		breakpointFor,
		panelMeta,
		resolveLayouts,
	} from '../../state/layout/registry';
	import {
		beginGesture,
		floatingBox,
		keyboardLayout,
		moveGesture,
		type GridMetrics,
		type PanelGesture,
	} from '../../state/layout/panel-gestures';
	import type { MapState } from '../../state/map-state.svelte';
	import type { Snippet } from 'svelte';

	let { map, canvas }: { map: MapState; canvas: Snippet } = $props();

	let gridEl = $state<HTMLElement | null>(null);
	// Only the drag maths needs pixels; tiles are placed in percentages, so a stale width
	// here never moves anything on screen.
	let gridWidth = $state(1200);
	let windowWidth = $state(browser ? window.innerWidth : 1536);

	// Tiles animate as they are dragged around each other, but not into place on load: the
	// first paint is the arrangement, not something to glide towards.
	let settled = $state(false);
	$effect(() => {
		const frame = requestAnimationFrame(() => (settled = true));
		return () => cancelAnimationFrame(frame);
	});

	// While editing you pick a breakpoint; otherwise it follows the window.
	const activeKey = $derived<BreakpointKey>(
		map.panels.editing ? map.panels.breakpoint : breakpointFor(windowWidth),
	);
	const layouts = $derived(resolveLayouts(map.panels.draft));
	const layout = $derived(layouts[activeKey]);
	const hidden = $derived(new Set(map.userSettings?.hidden_panels ?? []));
	// Compacted after filtering, so hiding a panel closes the hole rather than leaving a gap.
	// The stored positions are untouched, so an unhidden panel returns where it was.
	const items = $derived(
		compact(
			layout.items.filter((i) => !hidden.has(i.i) && !map.unavailablePanels.has(i.i as PanelId)),
			layout.cols,
		),
	);

	const colWidth = $derived(gridWidth / layout.cols);
	const rows = $derived(bottom(items));

	/** Empty rows kept below the layout while arranging, so there is somewhere to drag a
	 *  tile *to* when it is already at the bottom. */
	const EDIT_SLACK_ROWS = 3;
	/** Rows the grid had when the gesture started. Holding that floor stops the page
	 *  shrinking under the pointer as tiles reflow, which would jump the scroll position. */
	let gestureFloor = $state(0);
	const gridRows = $derived(
		map.panels.editing ? Math.max(rows, gestureFloor) + EDIT_SLACK_ROWS : rows,
	);

	let gesture = $state<PanelGesture | null>(null);

	const metrics = $derived<GridMetrics>({
		cols: layout.cols,
		rowHeight: layout.row_height,
		colWidth,
		gridWidth,
	});

	// Rendered in a fixed order, never the layout's: tiles are positioned with left/top, and
	// reordering a keyed `each` detaches the focused tile, so a second arrow key goes nowhere.
	const shown = $derived([...(gesture?.live ?? items)].sort((a, b) => a.i.localeCompare(b.i)));

	const placeholder = $derived(
		gesture?.live ? (gesture.live.find((i) => i.i === gesture!.id) ?? null) : null,
	);

	const floating = $derived(gesture === null ? null : floatingBox(gesture, metrics));

	function commit(next: GridItem[]) {
		map.panels.setItems(activeKey, next);
	}

	function onPointerDown(
		ev: PointerEvent & { currentTarget: HTMLElement },
		id: PanelId,
		kind: 'move' | 'resize',
	) {
		if (!map.panels.editing) return;
		ev.preventDefault();
		ev.stopPropagation();
		const origin = items.find((i) => i.i === id);
		if (!origin) return;
		ev.currentTarget.setPointerCapture(ev.pointerId);
		gestureFloor = rows;
		gesture = beginGesture(id, kind, { x: ev.clientX, y: ev.clientY }, origin);
	}

	function onPointerMove(ev: PointerEvent) {
		if (!gesture) return;
		gesture = moveGesture(gesture, { x: ev.clientX, y: ev.clientY }, items, metrics);
	}

	function onPointerUp() {
		const g = gesture;
		gesture = null;
		gestureFloor = 0;
		if (g?.live) commit(g.live);
	}

	function onKeyDown(ev: KeyboardEvent, id: PanelId) {
		if (!map.panels.editing) return;
		const next = keyboardLayout(items, id, ev.key, ev.shiftKey, layout.cols);
		if (!next) return;
		ev.preventDefault();
		commit(next);
	}

	$effect(() => {
		const el = gridEl;
		if (!el) return;
		const observer = new ResizeObserver(([entry]) => (gridWidth = entry.contentRect.width));
		observer.observe(el);
		return () => observer.disconnect();
	});

	// Entering edit mode starts from whatever breakpoint the window is actually at, and
	// snapshots the hidden set so Discard can put it back.
	$effect(() => {
		if (map.panels.editing) {
			untrack(() => {
				map.panels.breakpoint = breakpointFor(windowWidth);
				map.panels.rememberHidden();
			});
		}
	});
</script>

<svelte:window bind:innerWidth={windowWidth} />

{#snippet tile(item: GridItem)}
	{@const meta = panelMeta(item.i as PanelId)}
	{@const held = gesture?.id === item.i ? floating : null}
	{@const box = tileBox(item, layout.cols, layout.row_height)}
	<div
		class={cn(
			'absolute',
			settled && 'transition-[left,top,width,height] duration-150',
			// The held tile follows the pointer directly, so it must not animate. Releasing
			// re-enables the transition and it glides into the placeholder's slot.
			held && 'z-30 shadow-2xl duration-0',
		)}
		style:width={held ? `${held.width}px` : box.width}
		style:height={held ? `${held.height}px` : box.height}
		style:left={held ? `${held.left}px` : box.left}
		style:top={held ? `${held.top}px` : box.top}
		data-testid="panel-tile"
		data-panel={item.i}
		data-x={item.x}
		data-y={item.y}
		data-w={item.w}
		data-h={item.h}
	>
		<div class="relative h-full">
			{#if item.i === 'map'}
				<div class="h-full">{@render canvas()}</div>
			{:else if item.i === 'navigation'}
				<NavigationCard {map} />
			{:else if item.i === 'system-info'}
				{#if map.activeSystem}
					<SystemInfoCard system={map.activeSystem} />
				{:else}
					{@render empty(item.i as PanelId, meta.label)}
				{/if}
			{:else if item.i === 'threat'}
				{#if map.activeSystem}
					<ThreatCard system={map.activeSystem} />
				{:else}
					{@render empty(item.i as PanelId, meta.label)}
				{/if}
			{:else if item.i === 'signatures'}
				{#if map.activeSystem}
					<SignaturesPanel {map} system={map.activeSystem} />
				{:else}
					{@render empty(item.i as PanelId, meta.label)}
				{/if}
			{:else if item.i === 'characters'}
				<CharactersCard {map} />
			{:else if item.i === 'skyhooks'}
				<SkyhooksCard {map} />
			{:else if item.i === 'killmails'}
				<KillmailsCard {map} />
			{:else if item.i === 'evescout'}
				<EveScoutCard {map} />
			{:else if item.i === 'notes'}
				{#if map.activeSystem}
					<NotesCard {map} system={map.activeSystem} />
				{:else}
					{@render empty(item.i as PanelId, meta.label)}
				{/if}
			{/if}

			{#if map.panels.editing}
				<!-- The shield makes a drag anywhere on the card move the tile instead of
				     reaching the content under it: without it, dragging across the canvas would
				     pan the map. The header strip is part of it, since a card's title bar is the
				     first place anyone grabs. Buttons rather than divs where it has to focus and
				     take the arrow keys. -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div
					class="absolute inset-x-0 top-0 z-20 flex h-9 cursor-move items-center justify-between bg-muted/90 pr-1 pl-3 backdrop-blur-sm"
					data-testid="tile-header"
					data-panel={item.i}
					onpointerdown={(ev) => onPointerDown(ev, item.i as PanelId, 'move')}
				>
					<span class="font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
						{meta.label}
					</span>
					{#if meta.removable}
						<button
							type="button"
							class="flex size-6 items-center justify-center rounded text-muted-foreground hover:bg-destructive hover:text-destructive-foreground"
							aria-label="Hide {meta.label}"
							data-testid="tile-hide"
							data-panel={item.i}
							onpointerdown={(ev) => ev.stopPropagation()}
							onclick={() => map.panels.hidePanel(item.i)}
						>
							<XIcon class="size-3.5" />
						</button>
					{/if}
				</div>
				<button
					type="button"
					class="absolute inset-x-0 top-9 bottom-0 z-20 cursor-move focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-ring"
					aria-label="Move {meta.label}. Arrow keys move, shift and arrows resize."
					data-testid="tile-shield"
					data-panel={item.i}
					onpointerdown={(ev) => onPointerDown(ev, item.i as PanelId, 'move')}
					onkeydown={(ev) => onKeyDown(ev, item.i as PanelId)}
				></button>
				<button
					type="button"
					class="group/resize absolute right-[3px] bottom-[3px] z-40 flex size-6 cursor-se-resize items-end justify-end p-1.5"
					aria-label="Resize {meta.label}"
					data-testid="tile-resize"
					data-panel={item.i}
					onpointerdown={(ev) => onPointerDown(ev, item.i as PanelId, 'resize')}
				>
					<span
						class="size-2 border-r-2 border-b-2 border-muted-foreground/70 group-hover/resize:border-foreground"
					></span>
				</button>
				<!-- A sibling drawn over the content, not an outline: as an outline it was painted
				     underneath the panel's own header background. -->
				<div
					class="pointer-events-none absolute inset-[3px] z-40 border-2 border-dashed border-muted-foreground/60"
					data-testid="tile-frame"
				></div>
			{/if}
		</div>
	</div>
{/snippet}

{#snippet empty(id: PanelId, label: string)}
	<MapPanel testid="{id}-empty">
		<MapPanelHeader>{label}</MapPanelHeader>
		<MapPanelContent>
			<div class="flex h-full flex-col items-center justify-center gap-2 p-4">
				<p class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">
					Select a system
				</p>
			</div>
		</MapPanelContent>
	</MapPanel>
{/snippet}

<!-- `overflow-x: clip` rather than `hidden`: it stops a shadow or a reflowing tile from
     flashing a horizontal scrollbar without making this a scroll container.
     `data-dragging` is on while a gesture previews, so tests wait on it clearing rather
     than on a timeout. -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	bind:this={gridEl}
	class="relative w-full overflow-x-clip"
	style:height="{gridRows * layout.row_height}px"
	data-testid="panel-grid"
	data-breakpoint={activeKey}
	data-dragging={gesture !== null}
	onpointermove={onPointerMove}
	onpointerup={onPointerUp}
	onpointercancel={onPointerUp}
>
	{#if placeholder}
		{@const ghost = tileBox(placeholder, layout.cols, layout.row_height)}
		<!-- Under the tiles so the held one stays readable. No transition: it shows which cell
		     the tile has picked, and easing would lag behind that. -->
		<div
			class="absolute z-0 border-2 border-dashed border-muted-foreground/60 bg-muted/50"
			data-testid="tile-placeholder"
			data-panel={placeholder.i}
			data-x={placeholder.x}
			data-y={placeholder.y}
			data-w={placeholder.w}
			data-h={placeholder.h}
			style:width={ghost.width}
			style:height={ghost.height}
			style:left={ghost.left}
			style:top={ghost.top}
		></div>
	{/if}
	{#each shown as item (item.i)}
		{@render tile(item)}
	{/each}
</div>

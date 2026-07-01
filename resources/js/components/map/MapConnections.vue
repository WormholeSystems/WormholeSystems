<script setup lang="ts">
import MapConnection from '@/components/map/MapConnection.vue';
import {
    beginMapDrag,
    endMapDrag,
    is_layout_locked,
    item_anchor_offset,
    useMapConnections,
    useMapMouse,
    useMapScale,
    useNewConnection,
    useNodeSizes,
    useSelection,
} from '@/composables/map';
import type { Coordinates, TProcessedConnection } from '@/composables/map/types';
import { TMapConnection } from '@/pages/maps';
import { useElementSize, useEventListener } from '@vueuse/core';
import { computed, ref, useTemplateRef } from 'vue';

const container = useTemplateRef('container');

// Drive the SVG viewBox from a reactive size: reading container.clientWidth/Height in the
// template doesn't re-evaluate on resize, so the drawing would otherwise keep a stale box
// when the window (and the canvas) resizes.
const { width: viewBoxWidth, height: viewBoxHeight } = useElementSize(container);

const connections = useMapConnections();

const { scale } = useMapScale();
const { nodeSizes } = useNodeSizes();

type NodeBox = { minX: number; maxX: number; minY: number; maxY: number; centerX: number; centerY: number };

/** The scaled bounding box of a node, derived from its measured base size. */
function nodeBox(anchor: Coordinates, id: number): NodeBox | null {
    const size = nodeSizes.get(id);
    if (!size) return null;
    const s = scale.value;
    const minX = anchor.x - item_anchor_offset.x * s;
    const minY = anchor.y - item_anchor_offset.y * s;
    const maxX = minX + size.width * s;
    const maxY = minY + size.height * s;
    return { minX, maxX, minY, maxY, centerX: (minX + maxX) / 2, centerY: (minY + maxY) / 2 };
}

/** The scaled centre of a node, falling back to the raw anchor until it has been measured. */
function nodeCenter(anchor: Coordinates | null, id: number): Coordinates | null {
    if (!anchor) return null;
    const box = nodeBox(anchor, id);
    return box ? { x: box.centerX, y: box.centerY } : anchor;
}

/**
 * The point on a node's edge facing `target`: where the straight line from the node's
 * centre to `target` crosses the node's bounding box. Keeps the whole connection visible
 * instead of running it under the node.
 */
function edgePointToward(box: NodeBox, target: Coordinates): Coordinates {
    const dx = target.x - box.centerX;
    const dy = target.y - box.centerY;
    const halfWidth = (box.maxX - box.minX) / 2;
    const halfHeight = (box.maxY - box.minY) / 2;
    const t = Math.min(dx !== 0 ? halfWidth / Math.abs(dx) : Infinity, dy !== 0 ? halfHeight / Math.abs(dy) : Infinity);
    return { x: box.centerX + dx * t, y: box.centerY + dy * t };
}

type Endpoints = { from: Coordinates; to: Coordinates; fromNormal: Coordinates; toNormal: Coordinates };

/**
 * Connects the centre of each box's facing edge, leaving perpendicular to it.
 *
 * Prefer the left/right edges whenever the boxes are separated horizontally — then the
 * smoothstep drops its long vertical run through the clear lane between the columns
 * (down, then right into the node) instead of routing down-over-down with the second
 * vertical run cutting straight through the column of stacked siblings. Top/bottom
 * edges are only used when the boxes share a column, so there is no horizontal lane.
 */
function edgeCenterConnection(source: NodeBox, target: NodeBox): Endpoints {
    const dx = target.centerX - source.centerX;
    const dy = target.centerY - source.centerY;

    const separatedX = target.minX > source.maxX || source.minX > target.maxX;
    const separatedY = target.minY > source.maxY || source.minY > target.maxY;
    const useHorizontal = separatedX || (!separatedY && Math.abs(dx) >= Math.abs(dy));

    if (useHorizontal) {
        const rightward = dx >= 0;
        return {
            from: { x: rightward ? source.maxX : source.minX, y: source.centerY },
            to: { x: rightward ? target.minX : target.maxX, y: target.centerY },
            fromNormal: { x: rightward ? 1 : -1, y: 0 },
            toNormal: { x: rightward ? -1 : 1, y: 0 },
        };
    }

    const downward = dy >= 0;
    return {
        from: { x: source.centerX, y: downward ? source.maxY : source.minY },
        to: { x: target.centerX, y: downward ? target.minY : target.maxY },
        fromNormal: { x: 0, y: downward ? 1 : -1 },
        toNormal: { x: 0, y: downward ? -1 : 1 },
    };
}

// Spacing between connections that leave the same node edge, in base units.
const PARALLEL_SPACING = 14;
// Spacing between the perpendicular runs of those connections' bends.
const BEND_SPACING = 16;

type Port = { endpoint: Coordinates; normal: Coordinates; box: NodeBox; sortKey: number };

/**
 * Fans out connections that share a node edge so parallel lines don't overlap:
 * each shared edge spreads its endpoints along itself, ordered by the other end's
 * position so the lines stay untangled.
 */
function spreadSharedEdges(ports: Port[]): void {
    if (ports.length < 2) return;
    const alongY = ports[0].normal.x !== 0;
    const box = ports[0].box;
    const extent = alongY ? box.maxY - box.minY : box.maxX - box.minX;
    const spacing = Math.min(PARALLEL_SPACING * scale.value, (extent * 0.7) / (ports.length - 1));
    ports.sort((a, b) => a.sortKey - b.sortKey);
    ports.forEach((port, i) => {
        const offset = (i - (ports.length - 1) / 2) * spacing;
        if (alongY) {
            port.endpoint.y += offset;
        } else {
            port.endpoint.x += offset;
        }
    });
}

type RenderedConnection = {
    connection: TProcessedConnection;
    from: Coordinates | null;
    to: Coordinates | null;
    fromNormal?: Coordinates;
    toNormal?: Coordinates;
    bend?: number;
    variant: 'default' | 'orthogonal';
};

// A connection whose endpoints both resolved to a measured node box, so it gets the
// edge-routed orthogonal treatment. The geometry fields are mutated in place by the
// fan-out / bend passes below.
type RoutedConnection = RenderedConnection & {
    from: Coordinates;
    to: Coordinates;
    fromNormal: Coordinates;
    toNormal: Coordinates;
    variant: 'orthogonal';
    sourceBox: NodeBox;
    targetBox: NodeBox;
    /** Perpendicular distance and signed offset of the far end, for fan ordering. */
    distance: number;
    signed: number;
};

const drawnConnections = computed<RenderedConnection[]>(() => {
    // The free layout keeps the original curve styling, but anchors each connection to the
    // node edge facing the other system rather than the stored top-left anchor.
    if (!is_layout_locked.value) {
        return connections.value.map((connection) => {
            const sourceBox = connection.source.position ? nodeBox(connection.source.position, connection.source.id) : null;
            const targetBox = connection.target.position ? nodeBox(connection.target.position, connection.target.id) : null;

            // Until both nodes are measured, fall back to node centres (or raw anchors).
            if (!sourceBox || !targetBox) {
                return {
                    connection,
                    from: nodeCenter(connection.source.position, connection.source.id),
                    to: nodeCenter(connection.target.position, connection.target.id),
                    variant: 'default',
                };
            }

            const sourceCenter = { x: sourceBox.centerX, y: sourceBox.centerY };
            const targetCenter = { x: targetBox.centerX, y: targetBox.centerY };
            return {
                connection,
                from: edgePointToward(sourceBox, targetCenter),
                to: edgePointToward(targetBox, sourceCenter),
                variant: 'default',
            };
        });
    }

    const routed: RoutedConnection[] = [];
    const items: RenderedConnection[] = connections.value.map((connection) => {
        const sourceBox = connection.source.position ? nodeBox(connection.source.position, connection.source.id) : null;
        const targetBox = connection.target.position ? nodeBox(connection.target.position, connection.target.id) : null;
        // Until both nodes have been measured, fall back to the anchor endpoints.
        if (!sourceBox || !targetBox) {
            return { connection, from: connection.source.position, to: connection.target.position, variant: 'default' };
        }
        const ends = edgeCenterConnection(sourceBox, targetBox);
        const item: RoutedConnection = {
            connection,
            from: { ...ends.from },
            to: { ...ends.to },
            fromNormal: ends.fromNormal,
            toNormal: ends.toNormal,
            variant: 'orthogonal',
            sourceBox,
            targetBox,
            distance: 0,
            signed: 0,
        };
        routed.push(item);
        return item;
    });

    // Fan out the endpoints that share a node edge so parallel lines don't overlap.
    const edges = new Map<string, Port[]>();
    const register = (endpoint: Coordinates, normal: Coordinates, box: NodeBox, other: NodeBox): void => {
        const port: Port = { endpoint, normal, box, sortKey: normal.x !== 0 ? other.centerY : other.centerX };
        const key = `${box.centerX},${box.centerY}|${normal.x},${normal.y}`;
        (edges.get(key) ?? edges.set(key, []).get(key)!).push(port);
    };
    for (const item of routed) {
        register(item.from, item.fromNormal, item.sourceBox, item.targetBox);
        register(item.to, item.toNormal, item.targetBox, item.sourceBox);
    }
    for (const ports of edges.values()) {
        spreadSharedEdges(ports);
    }

    // Stagger the perpendicular run of connections that fan out from the same node so
    // they nest instead of stacking. Group by the node the fan leaves on its primary
    // axis (the left node for horizontal links, the top node for vertical), regardless
    // of which end is the stored source, and order each group by how far its far end
    // sits so the runs don't cross.
    const fans = new Map<string, RoutedConnection[]>();
    for (const item of routed) {
        const horizontal = item.fromNormal.x !== 0;
        const sourceFirst = horizontal ? item.sourceBox.centerX <= item.targetBox.centerX : item.sourceBox.centerY <= item.targetBox.centerY;
        const primary = sourceFirst ? item.sourceBox : item.targetBox;
        const other = sourceFirst ? item.targetBox : item.sourceBox;
        const along = horizontal ? other.centerY - primary.centerY : other.centerX - primary.centerX;
        item.distance = Math.abs(along);
        item.signed = along;
        const key = `${horizontal ? 'h' : 'v'}|${primary.centerX},${primary.centerY}`;
        (fans.get(key) ?? fans.set(key, []).get(key)!).push(item);
    }
    for (const group of fans.values()) {
        if (group.length < 2) continue;
        const horizontal = group[0].fromNormal.x !== 0;
        // Keep the whole fan inside the lane between the two columns: when there are more
        // connections than BEND_SPACING would fit in the gap, shrink the spacing so the
        // outermost runs still clear the neighbouring nodes instead of cutting through them.
        const gap = Math.min(...group.map((item) => Math.abs(horizontal ? item.to.x - item.from.x : item.to.y - item.from.y)));
        const spacing = Math.min(BEND_SPACING * scale.value, (gap * 0.8) / (group.length - 1));
        // Farthest target bends closest to the node; ties (above vs below) split by side.
        group.sort((a, b) => b.distance - a.distance || a.signed - b.signed);
        group.forEach((item, i) => {
            const base = horizontal ? (item.from.x + item.to.x) / 2 : (item.from.y + item.to.y) / 2;
            item.bend = base + (i - (group.length - 1) / 2) * spacing;
        });
    }

    return items;
});

const { origin } = useNewConnection();

const mouse = useMapMouse();

const { selection, setSelectionStart, setSelectionEnd, clearSelection } = useSelection();

const dragging = ref(false);
const background_press = ref<{ x: number; y: number } | null>(null);

/** Shift works everywhere; Cmd covers macOS, where Ctrl+click is a right-click. */
function wantsSelection(event: PointerEvent): boolean {
    return event.shiftKey || event.ctrlKey || event.metaKey;
}

const emit = defineEmits<{
    (e: 'connectionContextMenu', event: MouseEvent, connection: TMapConnection): void;
    (e: 'connectionClick', event: MouseEvent, connection: TMapConnection): void;
}>();

function handleDragStart(event: PointerEvent) {
    if (event.button !== 0) return;
    // In the tree layout a plain left drag pans; hold Shift/Cmd to box-select instead.
    if (is_layout_locked.value && !wantsSelection(event)) {
        // Remember the press so a click without dragging (i.e. not a pan) clears the
        // current selection on release.
        background_press.value = { x: event.clientX, y: event.clientY };
        return;
    }
    dragging.value = true;
    beginMapDrag();
    setSelectionStart(event.offsetX, event.offsetY);
}

function handleDragMove() {
    if (!dragging.value) return;
    setSelectionEnd(mouse.value.x, mouse.value.y);
}

function handleDragEnd(event: PointerEvent) {
    if (background_press.value) {
        const press = background_press.value;
        background_press.value = null;
        // A background click that did not turn into a pan clears the selection.
        if (Math.abs(event.clientX - press.x) < 4 && Math.abs(event.clientY - press.y) < 4) {
            clearSelection();
        }
    }

    if (!dragging.value) return;
    dragging.value = false;
    endMapDrag();
    const bounds = container.value?.getBoundingClientRect();
    if (!bounds) return;

    const x = event.clientX - bounds.left;
    const y = event.clientY - bounds.top;

    setSelectionEnd(x, y);
}

useEventListener('pointerup', handleDragEnd);
</script>

<template>
    <div class="" ref="container" @pointerdown="handleDragStart" @pointermove="handleDragMove">
        <svg class="h-full w-full text-neutral-700" xmlns="http://www.w3.org/2000/svg" :viewBox="`0 0 ${viewBoxWidth} ${viewBoxHeight}`">
            <MapConnection
                v-for="{ connection, from, to, fromNormal, toNormal, bend, variant } in drawnConnections"
                :key="connection.id"
                :from="from!"
                :to="to!"
                :from_normal="fromNormal"
                :to_normal="toNormal"
                :bend="bend"
                :variant="variant"
                :type="connection.type"
                :preserve_mass="connection.preserve_mass"
                :ship_size="connection.ship_size"
                :mass_status="connection.mass_status"
                :lifetime="connection.lifetime_status"
                :is_highlighted="connection.is_on_route"
                :is_on_rally_route="connection.is_on_rally_route"
                :rally_route_reversed="connection.rally_route_reversed"
                @connection-context-menu="(event) => emit('connectionContextMenu', event, connection)"
                @connection-click="(event) => emit('connectionClick', event, connection)"
            />
            <MapConnection v-if="origin" :from="origin.position!" :to="mouse" />
            <rect
                v-if="dragging && selection?.start"
                :x="Math.min(selection.start.x, mouse.x)"
                :y="Math.min(selection.start.y, mouse.y)"
                :width="Math.abs(selection.start.x - mouse.x)"
                :height="Math.abs(selection.start.y - mouse.y)"
                class="fill-amber-500/10 stroke-amber-500 stroke-1"
                :rx="4"
                :ry="4"
                stroke-dasharray="2,2"
            />
        </svg>
    </div>
</template>

<style scoped></style>

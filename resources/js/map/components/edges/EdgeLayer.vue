<script setup lang="ts">
import ConnectedEdge from '@/map/components/edges/ConnectedEdge.vue';
import Edge from '@/map/components/edges/Edge.vue';
import { nodeRect } from '@/map/core/coords';
import { computeTreeEdgeGeometries } from '@/map/core/geometry/treeRouting';
import type { EdgeGeometry, EdgeInput, Rect, Vec2 } from '@/map/core/types';
import { useMapStore } from '@/map/store/mapStore';
import type { TMapConnection } from '@/pages/maps';
import { useElementSize } from '@vueuse/core';
import { computed, useTemplateRef } from 'vue';

/**
 * Pending "new connection" ghost edge: the gesture layer (a later phase) feeds the
 * drag-origin anchor and live pointer position in BASE units; while both are set
 * the layer draws the old neutral ghost curve (an Edge without a connection).
 */
type Props = {
    pendingFrom?: Vec2 | null;
    pendingTo?: Vec2 | null;
};

const { pendingFrom = null, pendingTo = null } = defineProps<Props>();

const emit = defineEmits<{
    (e: 'connectionContextMenu', event: MouseEvent, connection: TMapConnection): void;
    (e: 'connectionClick', event: MouseEvent, connection: TMapConnection): void;
}>();

const store = useMapStore();

const container = useTemplateRef('container');

// Drive the SVG viewBox from a reactive size: reading container.clientWidth/Height in the
// template doesn't re-evaluate on resize, so the drawing would otherwise keep a stale box
// when the window (and the canvas) resizes.
const { width: viewBoxWidth, height: viewBoxHeight } = useElementSize(container);

const connectionIds = computed(() => [...store.connections.keys()]);

// The tree layout routes all edges in one global pass (its fan-out spacing needs
// every edge that shares a node); each ConnectedEdge then picks its entry. In the
// free layout this stays null and never tracks the entity maps, so per-edge
// geometry computeds are the only readers of the drag hot path.
const treeGeometries = computed<Map<number, EdgeGeometry> | null>(() => {
    if (!store.isLayoutLocked.value) return null;

    const edges: EdgeInput[] = [];
    const nodeIds = new Set<number>();
    for (const connection of store.connections.values()) {
        edges.push({ id: connection.id, sourceId: connection.from_map_solarsystem_id, targetId: connection.to_map_solarsystem_id });
        nodeIds.add(connection.from_map_solarsystem_id);
        nodeIds.add(connection.to_map_solarsystem_id);
    }

    const rects = new Map<number, Rect>();
    const anchors = new Map<number, Vec2>();
    for (const nodeId of nodeIds) {
        const anchor = store.renderPosition(nodeId);
        if (!anchor) continue;
        anchors.set(nodeId, anchor);
        const size = store.nodeSizes.get(nodeId);
        if (size) {
            rects.set(nodeId, nodeRect(anchor, size));
        }
    }

    return computeTreeEdgeGeometries(edges, rects, anchors);
});

const pendingGeometry = computed<EdgeGeometry | null>(() => {
    if (!pendingFrom || !pendingTo) return null;
    return { id: -1, kind: 'curve', from: pendingFrom, to: pendingTo };
});
</script>

<template>
    <div ref="container">
        <svg class="h-full w-full text-neutral-700" xmlns="http://www.w3.org/2000/svg" :viewBox="`0 0 ${viewBoxWidth} ${viewBoxHeight}`">
            <ConnectedEdge
                v-for="id in connectionIds"
                :id="id"
                :key="id"
                :tree-geometries="treeGeometries"
                @connection-click="(event, connection) => emit('connectionClick', event, connection)"
                @connection-context-menu="(event, connection) => emit('connectionContextMenu', event, connection)"
            />
            <Edge v-if="pendingGeometry" :geometry="pendingGeometry" :scale="store.scale.value" />
        </svg>
    </div>
</template>

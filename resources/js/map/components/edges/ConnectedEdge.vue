<script setup lang="ts">
import Edge from '@/map/components/edges/Edge.vue';
import { nodeRect } from '@/map/core/coords';
import { freeEdgeGeometry } from '@/map/core/geometry/freeRouting';
import type { EdgeGeometry, Rect, Vec2 } from '@/map/core/types';
import { useMapStore } from '@/map/store/mapStore';
import type { TMapConnection } from '@/pages/maps';
import { computed } from 'vue';

type Props = {
    id: number;
    /**
     * Tree-layout geometry computed once for the whole layer (the fan-out passes
     * need every edge); null in the free layout, where each edge routes itself
     * from just its two endpoints so dragging a node only recomputes its edges.
     */
    treeGeometries?: ReadonlyMap<number, EdgeGeometry> | null;
};

const { id, treeGeometries = null } = defineProps<Props>();

const emit = defineEmits<{
    (e: 'connectionContextMenu', event: MouseEvent, connection: TMapConnection): void;
    (e: 'connectionClick', event: MouseEvent, connection: TMapConnection): void;
}>();

const store = useMapStore();

const connection = computed(() => store.connections.get(id) ?? null);

/** A node's endpoint for free routing: its rect once measured, else the raw anchor. */
function freeEndpoint(mapSolarsystemId: number): Rect | Vec2 | null {
    const anchor = store.renderPosition(mapSolarsystemId);
    if (!anchor) return null;
    const size = store.nodeSizes.get(mapSolarsystemId);
    return size ? nodeRect(anchor, size) : anchor;
}

const geometry = computed<EdgeGeometry | null>(() => {
    if (treeGeometries) {
        return treeGeometries.get(id) ?? null;
    }
    const resolved = connection.value;
    if (!resolved) return null;
    const source = freeEndpoint(resolved.from_map_solarsystem_id);
    const target = freeEndpoint(resolved.to_map_solarsystem_id);
    if (!source || !target) return null;
    return freeEdgeGeometry(id, source, target);
});

const isOnRoute = computed(() => store.routeConnectionIds.value.has(id));
const rallyDirection = computed<'forward' | 'reverse' | null>(() => store.rallyEdgeDirections.value.get(id) ?? null);

function handleConnectionClick(event: MouseEvent): void {
    if (connection.value) {
        emit('connectionClick', event, connection.value);
    }
}

function handleConnectionContextMenu(event: MouseEvent): void {
    if (connection.value) {
        emit('connectionContextMenu', event, connection.value);
    }
}
</script>

<template>
    <Edge
        v-if="geometry && connection"
        :geometry="geometry"
        :connection="connection"
        :is-on-route="isOnRoute"
        :rally-direction="rallyDirection"
        :scale="store.scale.value"
        @connection-click="handleConnectionClick"
        @connection-context-menu="handleConnectionContextMenu"
    />
</template>

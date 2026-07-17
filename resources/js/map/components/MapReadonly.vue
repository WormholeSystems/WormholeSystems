<script setup lang="ts">
import Edge from '@/map/components/edges/Edge.vue';
import NodeCard from '@/map/components/nodes/NodeCard.vue';
import { ANCHOR_OFFSET, nodeRect } from '@/map/core/coords';
import { freeEdgeGeometry } from '@/map/core/geometry/freeRouting';
import type { EdgeGeometry, Size } from '@/map/core/types';
import type { TMapConnection, TMapSolarsystem } from '@/pages/maps';
import { TCharacter } from '@/types/models';

/**
 * Store-free, read-only rendering of plain map data (the landing page's demo
 * map). Same contract as the old MapView: base-unit positions scaled by the
 * `scale` prop, non-interactive cards, and the same free-layout curved edges
 * the live map draws.
 */
type TReadonlyConnection = TMapConnection & {
    source: TMapSolarsystem;
    target: TMapSolarsystem;
    is_on_route?: boolean;
};

const {
    solarsystems,
    connections,
    scale = 1,
    grid_size = 20,
    home_solarsystem_id = null,
    rally_solarsystem_id = null,
    pilots = {},
} = defineProps<{
    solarsystems: TMapSolarsystem[];
    connections: TReadonlyConnection[];
    scale?: number;
    /** Grid cell size in base units; matches the live map default (20). */
    grid_size?: number;
    home_solarsystem_id?: number | null;
    rally_solarsystem_id?: number | null;
    pilots?: Record<number, TCharacter[]>;
}>();

/**
 * Fixed-width node cards keep the edge routing fully deterministic: the
 * geometry is correct on the very first paint (no post-mount measurement pass,
 * so no connection jump while the map fades in) and costs nothing to compute.
 * Nodes are assumed to span two grid fields high (the card's base height).
 */
const NODE_SIZE: Size = { width: 180, height: 40 };

/**
 * The same per-edge free-layout routing the live map uses when the layout is
 * unlocked: rounded curves between rail endpoints on each node.
 */
function connectionGeometry(connection: TReadonlyConnection): EdgeGeometry {
    const sourceAnchor = connection.source.position ?? { x: 0, y: 0 };
    const targetAnchor = connection.target.position ?? { x: 0, y: 0 };
    return freeEdgeGeometry(connection.id, nodeRect(sourceAnchor, NODE_SIZE), nodeRect(targetAnchor, NODE_SIZE));
}

/** The node's top-left in screen pixels: the scaled anchor minus the scaled anchor offset. */
function nodeTransform(system: TMapSolarsystem): string {
    const x = (system.position?.x ?? 0) * scale - ANCHOR_OFFSET.x * scale;
    const y = (system.position?.y ?? 0) * scale - ANCHOR_OFFSET.y * scale;
    return `translate(${x}px, ${y}px)`;
}
</script>

<template>
    <div class="bg-grid relative h-full w-full overflow-hidden" :style="{ backgroundSize: `${grid_size * scale}px ${grid_size * scale}px` }">
        <svg class="pointer-events-none absolute inset-0 h-full w-full text-neutral-700" xmlns="http://www.w3.org/2000/svg">
            <Edge
                v-for="connection in connections"
                :key="connection.id"
                :geometry="connectionGeometry(connection)"
                :connection="connection"
                :is-on-route="connection.is_on_route ?? false"
                :scale="scale"
            />
        </svg>

        <div
            v-for="solarsystem in solarsystems"
            :key="solarsystem.id"
            class="pointer-events-none absolute select-none"
            :style="{ transform: nodeTransform(solarsystem) }"
        >
            <div class="origin-top-left" :style="{ scale }">
                <NodeCard
                    :system="solarsystem"
                    :pilots="pilots[solarsystem.id] ?? []"
                    :is-selected="false"
                    :is-hovered="false"
                    :is-active="false"
                    :is-home="home_solarsystem_id === solarsystem.id"
                    :is-rally="rally_solarsystem_id === solarsystem.solarsystem_id"
                    :fixed-width="true"
                />
            </div>
        </div>
    </div>
</template>

<style scoped>
.bg-grid {
    background-image: linear-gradient(to right, var(--grid) 1px, transparent 1px), linear-gradient(to bottom, var(--grid) 1px, transparent 1px);
}
</style>

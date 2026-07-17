<script setup lang="ts">
import Edge from '@/map/components/edges/Edge.vue';
import NodeCard from '@/map/components/nodes/NodeCard.vue';
import { ANCHOR_OFFSET, nodeRect } from '@/map/core/coords';
import { computeTreeEdgeGeometries } from '@/map/core/geometry/treeRouting';
import type { EdgeGeometry, EdgeInput, Rect, Size, Vec2 } from '@/map/core/types';
import type { TMapConnection, TMapSolarsystem } from '@/pages/maps';
import { TCharacter } from '@/types/models';
import { computed, onMounted, ref, type ComponentPublicInstance } from 'vue';

/**
 * Store-free, read-only rendering of plain map data (the landing page's demo
 * map). Same contract as the old MapView: base-unit positions scaled by the
 * `scale` prop, non-interactive cards, and the same tree-routed elbow edges
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
 * Node sizes for edge routing: measured from the DOM after mount, with the
 * card's known base dimensions as the SSR/first-paint estimate.
 */
const ESTIMATED_NODE_WIDTH = 120;

const nodeEls = new Map<number, HTMLElement>();
const measuredSizes = ref<Map<number, Size>>(new Map());

function registerNode(id: number, el: Element | ComponentPublicInstance | null) {
    if (el instanceof HTMLElement) {
        nodeEls.set(id, el);
    } else {
        nodeEls.delete(id);
    }
}

onMounted(() => {
    const sizes = new Map<number, Size>();
    for (const [id, el] of nodeEls) {
        const card = el.firstElementChild as HTMLElement | null;
        if (!card) continue;
        sizes.set(id, { width: card.offsetWidth, height: card.offsetHeight });
    }
    measuredSizes.value = sizes;
});

function nodeSize(system: TMapSolarsystem): Size {
    return (
        measuredSizes.value.get(system.id) ?? {
            width: ESTIMATED_NODE_WIDTH,
            height: (pilots[system.id]?.length ?? 0) > 0 ? 60 : 40,
        }
    );
}

/**
 * The same global tree-routing pass the live map runs: all edges routed
 * together so fan-outs at shared nodes space themselves out.
 */
const treeGeometries = computed<Map<number, EdgeGeometry>>(() => {
    const edges: EdgeInput[] = connections.map((connection) => ({
        id: connection.id,
        sourceId: connection.source.id,
        targetId: connection.target.id,
    }));

    const rects = new Map<number, Rect>();
    const anchors = new Map<number, Vec2>();
    for (const system of solarsystems) {
        const anchor = system.position ?? { x: 0, y: 0 };
        anchors.set(system.id, anchor);
        rects.set(system.id, nodeRect(anchor, nodeSize(system)));
    }

    return computeTreeEdgeGeometries(edges, rects, anchors);
});

function connectionGeometry(connection: TReadonlyConnection): EdgeGeometry {
    return (
        treeGeometries.value.get(connection.id) ?? {
            id: connection.id,
            kind: 'curve',
            from: connection.source.position ?? { x: 0, y: 0 },
            to: connection.target.position ?? { x: 0, y: 0 },
        }
    );
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
            :ref="(el) => registerNode(solarsystem.id, el)"
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
                    :fixed-width="false"
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

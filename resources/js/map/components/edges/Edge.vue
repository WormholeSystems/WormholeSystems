<script setup lang="ts">
import EdgeBadges, { type EdgeIndicator } from '@/map/components/edges/EdgeBadges.vue';
import { scalePoint } from '@/map/core/coords';
import { edgePathAndCenter } from '@/map/core/geometry/paths';
import type { EdgeGeometry } from '@/map/core/types';
import type { TMapConnection } from '@/pages/maps';
import type { TShipSize } from '@/types/models';
import { computed } from 'vue';

type Props = {
    geometry: EdgeGeometry;
    /**
     * The connection being drawn, or null for the pending "new connection" ghost:
     * with no connection every status field reads undefined, which reproduces the
     * old bare `<MapConnection :from :to />` (neutral dashed curve, no badges).
     */
    connection?: TMapConnection | null;
    isOnRoute?: boolean;
    rallyDirection?: 'forward' | 'reverse' | null;
    scale: number;
};

const { geometry, connection = null, isOnRoute = false, rallyDirection = null, scale } = defineProps<Props>();

const emit = defineEmits<{
    (e: 'connectionContextMenu', event: MouseEvent): void;
    (e: 'connectionClick', event: MouseEvent): void;
}>();

const isStargate = computed(() => connection?.type === 'stargate');

/** 'elbow' is the tree-style rounded elbow + slim stroke; 'curve' the original free-layout style. */
const isOrthogonal = computed(() => geometry.kind === 'elbow');

const massStatus = computed(() => connection?.mass_status);
const lifetime = computed(() => connection?.lifetime_status);

/** The SVG path and badge-cluster centre in screen pixels; the one place scale is applied. */
const path = computed(() => edgePathAndCenter(geometry, scale));

const scaledFrom = computed(() => scalePoint(geometry.from, scale));
const scaledTo = computed(() => scalePoint(geometry.to, scale));

function getShipSizeLabel(size?: TShipSize | null): string | null {
    if (size === 'frigate') return 'S';
    if (size === 'medium') return 'M';
    if (size === 'xlarge') return 'XL';

    return null;
}

const indicators = computed<EdgeIndicator[]>(() => {
    const items: EdgeIndicator[] = [];

    if (isStargate.value) {
        items.push({
            type: 'gate',
            fill: 'var(--color-sky-500)',
            stroke: 'var(--color-sky-600)',
        });
    }

    if (connection?.preserve_mass) {
        items.push({
            type: 'preserve',
            fill: 'var(--color-emerald-500)',
            stroke: 'var(--color-emerald-600)',
        });
    }

    const shipSizeLabel = getShipSizeLabel(connection?.ship_size);
    if (shipSizeLabel) {
        items.push({
            type: 'text',
            label: shipSizeLabel,
            fill: 'var(--color-neutral-500)',
            stroke: 'var(--color-neutral-600)',
        });
    }

    if (massStatus.value && massStatus.value !== 'fresh') {
        items.push({
            type: 'weight',
            fill: massStatus.value === 'critical' ? 'var(--color-red-500)' : 'var(--color-amber-500)',
            stroke: massStatus.value === 'critical' ? 'var(--color-red-600)' : 'var(--color-amber-600)',
        });
    }

    if (lifetime.value && lifetime.value !== 'healthy') {
        items.push({
            type: 'clock',
            fill: lifetime.value === 'critical' ? 'var(--color-red-500)' : 'var(--color-purple-500)',
            stroke: lifetime.value === 'critical' ? 'var(--color-red-600)' : 'var(--color-purple-600)',
        });
    }

    return items;
});

function getDashArray(): string | undefined {
    if (!massStatus.value) return '0';
    if (lifetime.value === 'eol' || lifetime.value === 'critical') return '2,6';
    return undefined;
}
</script>

<template>
    <g pointer-events="visiblePainted" class="group text-neutral-300 dark:text-neutral-700">
        <!-- Stargates are permanent, so they draw a single solid line instead of the wormhole's mass/lifetime styling. -->
        <path
            v-if="isStargate"
            :d="path.d"
            stroke="currentColor"
            fill="none"
            :stroke-width="isOrthogonal ? 1.5 : 4"
            stroke-linejoin="round"
            stroke-linecap="round"
            :data-highlighted="isOnRoute"
            class="cursor-pointer text-sky-500 transition-colors duration-200 ease-in-out group-hover:text-sky-400"
        />
        <path
            v-if="!isStargate && (massStatus === 'fresh' || lifetime === 'eol' || lifetime === 'critical')"
            :d="path.d"
            stroke="currentColor"
            fill="none"
            :stroke-width="isOrthogonal ? 1.5 : 4"
            stroke-linejoin="round"
            stroke-linecap="round"
            :stroke-dasharray="getDashArray()"
            :data-lifetime="lifetime"
            :data-highlighted="isOnRoute"
            class="cursor-pointer text-neutral-300 transition-colors duration-200 ease-in-out group-hover:text-neutral-200 dark:text-neutral-700 dark:group-hover:text-neutral-600"
        />
        <path
            v-if="!isStargate && massStatus !== 'fresh'"
            :d="path.d"
            stroke="currentColor"
            fill="none"
            :stroke-width="isOrthogonal ? 1.5 : 4"
            stroke-linejoin="round"
            stroke-linecap="round"
            stroke-dasharray="2,6"
            stroke-dashoffset="4"
            :data-connection-status="massStatus"
            :data-highlighted="isOnRoute"
            class="cursor-pointer transition-colors duration-200 ease-in-out"
        />
        <!-- Rally route animated overlay -->
        <template v-if="rallyDirection">
            <path
                :d="path.d"
                stroke="var(--color-pink-400)"
                fill="none"
                :stroke-width="isOrthogonal ? 4 : 6"
                stroke-linejoin="round"
                stroke-opacity="0.3"
            />
            <path
                :d="path.d"
                stroke="var(--color-pink-400)"
                fill="none"
                :stroke-width="isOrthogonal ? 1.5 : 3"
                stroke-linejoin="round"
                stroke-linecap="round"
                stroke-dasharray="8,12"
                :class="rallyDirection === 'reverse' ? 'rally-route-animated-reverse' : 'rally-route-animated'"
            />
        </template>
        <!-- Connection status indicators -->
        <EdgeBadges :indicators="indicators" :center="path.center" />
        <path
            :d="path.d"
            stroke="transparent"
            fill="none"
            stroke-width="24"
            class="hover-path cursor-pointer transition-colors duration-200"
            @contextmenu="(event) => emit('connectionContextMenu', event)"
            @click="(event) => emit('connectionClick', event)"
            @pointerdown.stop
        />
        <!-- Original style draws solid endpoints; the orthogonal style meets the node edge instead. -->
        <template v-if="!isOrthogonal">
            <circle :cx="scaledFrom.x" :cy="scaledFrom.y" r="4" fill="currentColor" />
            <circle :cx="scaledTo.x" :cy="scaledTo.y" r="4" fill="currentColor" />
        </template>
    </g>
</template>

<style scoped>
[data-lifetime='critical'] {
    color: var(--color-red-500);
}

.group:hover [data-lifetime='critical'] {
    color: var(--color-red-400);
}

[data-connection-status='critical'] {
    color: var(--color-red-500);
}

.group:hover [data-connection-status='critical'] {
    color: var(--color-red-400);
}

[data-lifetime='eol'] {
    color: var(--color-purple-500);
}

.group:hover [data-lifetime='eol'] {
    color: var(--color-purple-400);
}

[data-connection-status='reduced'] {
    color: var(--color-orange-500);
}

.group:hover [data-connection-status='reduced'] {
    color: var(--color-orange-400);
}

[data-highlighted='true'] {
    color: var(--color-amber-500);
}

.group:hover [data-highlighted='true'] {
    color: var(--color-amber-400);
}

.rally-route-animated {
    animation: rally-march 0.8s linear infinite;
}

.rally-route-animated-reverse {
    animation: rally-march-reverse 0.8s linear infinite;
}

@keyframes rally-march {
    to {
        stroke-dashoffset: -20;
    }
}

@keyframes rally-march-reverse {
    to {
        stroke-dashoffset: 20;
    }
}
</style>

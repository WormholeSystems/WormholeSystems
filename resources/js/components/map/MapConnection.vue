<script setup lang="ts">
import { TConnectionType, TLifetimeStatus, TMassStatus, TShipSize } from '@/types/models';
import { Clock, Heart, Orbit, Weight } from 'lucide-vue-next';
import { computed } from 'vue';

type Position = {
    x: number;
    y: number;
};

type Indicator = {
    type: 'text' | 'clock' | 'weight' | 'gate' | 'preserve';
    label?: string;
    fill: string;
    stroke: string;
};

type Props = {
    from: Position;
    to: Position;
    /** Outward edge normals at from/to; when set the curve leaves each node perpendicular to its edge. */
    from_normal?: Position | null;
    to_normal?: Position | null;
    /** Position of the elbow's perpendicular run along the main axis; defaults to the midpoint. */
    bend?: number | null;
    /** 'orthogonal' draws the tree-style rounded elbow + slim stroke; 'default' the original curve. */
    variant?: 'default' | 'orthogonal';
    type?: TConnectionType;
    preserve_mass?: boolean;
    ship_size?: TShipSize | null;
    mass_status?: TMassStatus;
    lifetime?: TLifetimeStatus;
    is_highlighted?: boolean;
    is_on_rally_route?: boolean;
    rally_route_reversed?: boolean;
};

const {
    from,
    to,
    from_normal,
    to_normal,
    bend,
    variant = 'default',
    type = 'wormhole',
    preserve_mass = false,
    mass_status,
    lifetime,
    ship_size,
} = defineProps<Props>();

const isStargate = computed(() => type === 'stargate');

const emit = defineEmits<{
    (e: 'connectionContextMenu', event: MouseEvent): void;
    (e: 'connectionClick', event: MouseEvent): void;
}>();

const isOrthogonal = computed(() => variant === 'orthogonal');

const CORNER_RADIUS = 10;

// The two right-angle turn points of the orthogonal elbow (on the axis the
// connection exits), or null when the original curve should be drawn.
const elbow = computed<[Position, Position] | null>(() => {
    if (!isOrthogonal.value || !from_normal || !to_normal) return null;
    if (from_normal.x !== 0) {
        const midX = bend ?? (from.x + to.x) / 2;
        return [
            { x: midX, y: from.y },
            { x: midX, y: to.y },
        ];
    }
    const midY = bend ?? (from.y + to.y) / 2;
    return [
        { x: from.x, y: midY },
        { x: to.x, y: midY },
    ];
});

// Orthogonal "smoothstep" routing leaves each node perpendicular to its edge and
// turns through a rounded corner; otherwise fall back to the original simple curve.
const curve = computed(() => {
    const corners = elbow.value;
    if (!corners) {
        const cp1x = from.x + (to.x - from.x) / 1.5;
        const cp2x = to.x - (to.x - from.x) / 1.5;
        return `M ${from.x} ${from.y} C ${cp1x} ${from.y}, ${cp2x} ${to.y}, ${to.x} ${to.y}`;
    }
    return roundedPath([from, corners[0], corners[1], to], CORNER_RADIUS);
});

const center = computed(() => {
    const corners = elbow.value;
    if (!corners) return midPoint(from, to);
    return { x: (corners[0].x + corners[1].x) / 2, y: (corners[0].y + corners[1].y) / 2 };
});

/** Polyline through `points` with each interior corner rounded by up to `radius`. */
function roundedPath(points: Position[], radius: number): string {
    const pts = points.filter((point, i) => i === 0 || Math.hypot(point.x - points[i - 1].x, point.y - points[i - 1].y) > 0.01);
    if (pts.length < 2) return '';
    let d = `M ${pts[0].x} ${pts[0].y}`;
    for (let i = 1; i < pts.length - 1; i++) {
        const prev = pts[i - 1];
        const curr = pts[i];
        const next = pts[i + 1];
        const lenIn = Math.hypot(curr.x - prev.x, curr.y - prev.y);
        const lenOut = Math.hypot(next.x - curr.x, next.y - curr.y);
        const r = Math.min(radius, lenIn / 2, lenOut / 2);
        const start = { x: curr.x + ((prev.x - curr.x) / lenIn) * r, y: curr.y + ((prev.y - curr.y) / lenIn) * r };
        const end = { x: curr.x + ((next.x - curr.x) / lenOut) * r, y: curr.y + ((next.y - curr.y) / lenOut) * r };
        d += ` L ${start.x} ${start.y} Q ${curr.x} ${curr.y} ${end.x} ${end.y}`;
    }
    const last = pts[pts.length - 1];
    d += ` L ${last.x} ${last.y}`;
    return d;
}

function getShipSizeLabel(size?: TShipSize | null): string | null {
    if (size === 'frigate') return 'S';
    if (size === 'medium') return 'M';
    if (size === 'xlarge') return 'XL';

    return null;
}

const indicators = computed<Indicator[]>(() => {
    const items: Indicator[] = [];

    if (isStargate.value) {
        items.push({
            type: 'gate',
            fill: 'var(--color-sky-500)',
            stroke: 'var(--color-sky-600)',
        });
    }

    if (preserve_mass) {
        items.push({
            type: 'preserve',
            fill: 'var(--color-emerald-500)',
            stroke: 'var(--color-emerald-600)',
        });
    }

    const shipSizeLabel = getShipSizeLabel(ship_size);
    if (shipSizeLabel) {
        items.push({
            type: 'text',
            label: shipSizeLabel,
            fill: 'var(--color-neutral-500)',
            stroke: 'var(--color-neutral-600)',
        });
    }

    if (mass_status && mass_status !== 'fresh') {
        items.push({
            type: 'weight',
            fill: mass_status === 'critical' ? 'var(--color-red-500)' : 'var(--color-amber-500)',
            stroke: mass_status === 'critical' ? 'var(--color-red-600)' : 'var(--color-amber-600)',
        });
    }

    if (lifetime && lifetime !== 'healthy') {
        items.push({
            type: 'clock',
            fill: lifetime === 'critical' ? 'var(--color-red-500)' : 'var(--color-purple-500)',
            stroke: lifetime === 'critical' ? 'var(--color-red-600)' : 'var(--color-purple-600)',
        });
    }

    return items;
});

const indicatorsTotalWidth = computed(() => {
    const count = indicators.value.length;
    if (count === 0) return 0;
    return count * 18 + 8;
});

function midPoint(from: Position, to: Position): Position {
    return {
        x: (from.x + to.x) / 2,
        y: (from.y + to.y) / 2,
    };
}

function getDashArray() {
    if (!mass_status) return '0';
    if (lifetime === 'eol' || lifetime === 'critical') return '2,6';
}
</script>

<template>
    <g pointer-events="visiblePainted" class="group text-neutral-300 dark:text-neutral-700">
        <!-- Stargates are permanent, so they draw a single solid line instead of the wormhole's mass/lifetime styling. -->
        <path
            v-if="isStargate"
            :d="curve"
            stroke="currentColor"
            fill="none"
            :stroke-width="isOrthogonal ? 1.5 : 4"
            stroke-linejoin="round"
            stroke-linecap="round"
            :data-highlighted="is_highlighted"
            class="cursor-pointer text-sky-500 transition-colors duration-200 ease-in-out group-hover:text-sky-400"
        />
        <path
            v-if="!isStargate && (mass_status === 'fresh' || lifetime === 'eol' || lifetime === 'critical')"
            :d="curve"
            stroke="currentColor"
            fill="none"
            :stroke-width="isOrthogonal ? 1.5 : 4"
            stroke-linejoin="round"
            stroke-linecap="round"
            :stroke-dasharray="getDashArray()"
            :data-lifetime="lifetime"
            :data-highlighted="is_highlighted"
            class="cursor-pointer text-neutral-300 transition-colors duration-200 ease-in-out group-hover:text-neutral-200 dark:text-neutral-700 dark:group-hover:text-neutral-600"
        />
        <path
            v-if="!isStargate && mass_status !== 'fresh'"
            :d="curve"
            stroke="currentColor"
            fill="none"
            :stroke-width="isOrthogonal ? 1.5 : 4"
            stroke-linejoin="round"
            stroke-linecap="round"
            stroke-dasharray="2,6"
            stroke-dashoffset="4"
            :data-connection-status="mass_status"
            :data-highlighted="is_highlighted"
            class="cursor-pointer transition-colors duration-200 ease-in-out"
        />
        <!-- Rally route animated overlay -->
        <template v-if="is_on_rally_route">
            <path
                :d="curve"
                stroke="var(--color-pink-400)"
                fill="none"
                :stroke-width="isOrthogonal ? 4 : 6"
                stroke-linejoin="round"
                stroke-opacity="0.3"
            />
            <path
                :d="curve"
                stroke="var(--color-pink-400)"
                fill="none"
                :stroke-width="isOrthogonal ? 1.5 : 3"
                stroke-linejoin="round"
                stroke-linecap="round"
                stroke-dasharray="8,12"
                :class="rally_route_reversed ? 'rally-route-animated-reverse' : 'rally-route-animated'"
            />
        </template>
        <!-- Connection status indicators -->
        <foreignObject
            v-if="indicators.length"
            :x="center.x - indicatorsTotalWidth / 2"
            :y="center.y - 10"
            :width="indicatorsTotalWidth"
            height="20"
            class="pointer-events-none"
        >
            <div
                class="flex h-full items-center justify-center gap-0.5 rounded-full border border-neutral-300 bg-white px-1 dark:border-neutral-700 dark:bg-neutral-900"
            >
                <template v-for="(indicator, i) in indicators" :key="i">
                    <span v-if="indicator.type === 'text'" class="text-[13px] leading-none font-bold" :style="{ color: indicator.fill }">
                        {{ indicator.label }}
                    </span>
                    <Weight v-else-if="indicator.type === 'weight'" class="size-3.5" :style="{ color: indicator.fill }" />
                    <Clock v-else-if="indicator.type === 'clock'" class="size-3.5" :style="{ color: indicator.fill }" />
                    <Orbit v-else-if="indicator.type === 'gate'" class="size-3.5" :style="{ color: indicator.fill }" />
                    <Heart v-else-if="indicator.type === 'preserve'" class="size-3.5" :style="{ color: indicator.fill }" />
                </template>
            </div>
        </foreignObject>
        <path
            :d="curve"
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
            <circle :cx="from.x" :cy="from.y" r="4" fill="currentColor" />
            <circle :cx="to.x" :cy="to.y" r="4" fill="currentColor" />
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

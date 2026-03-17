<script setup lang="ts">
import { TLifetimeStatus, TMassStatus, TShipSize } from '@/types/models';
import { Clock, Weight } from 'lucide-vue-next';
import { computed } from 'vue';

type Position = {
    x: number;
    y: number;
};

type Indicator = {
    type: 'text' | 'clock' | 'weight';
    label?: string;
    fill: string;
    stroke: string;
};

type Props = {
    from: Position;
    to: Position;
    ship_size?: TShipSize | null;
    mass_status?: TMassStatus;
    lifetime?: TLifetimeStatus;
    is_highlighted?: boolean;
    is_on_rally_route?: boolean;
    rally_route_reversed?: boolean;
};

const { from, to, mass_status, lifetime, ship_size } = defineProps<Props>();

const emit = defineEmits<{
    (e: 'connectionContextMenu', event: MouseEvent): void;
    (e: 'connectionClick', event: MouseEvent): void;
}>();

const curve = computed(() => bezierCurve(from, to));
const center = computed(() => midPoint(from, to));

const indicators = computed<Indicator[]>(() => {
    const items: Indicator[] = [];

    if (ship_size && ship_size !== 'large') {
        items.push({
            type: 'text',
            label: ship_size === 'frigate' ? 'S' : 'M',
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

function bezierCurve(from: Position, to: Position): string {
    const cp1x = from.x + (to.x - from.x) / 1.5;
    const cp1y = from.y;
    const cp2x = to.x - (to.x - from.x) / 1.5;
    const cp2y = to.y;

    return `M ${from.x} ${from.y} C ${cp1x} ${cp1y}, ${cp2x} ${cp2y}, ${to.x} ${to.y}`;
}

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
        <path
            v-if="mass_status === 'fresh' || lifetime === 'eol' || lifetime === 'critical'"
            :d="curve"
            stroke="currentColor"
            fill="none"
            stroke-width="4"
            :stroke-dasharray="getDashArray()"
            :data-lifetime="lifetime"
            :data-highlighted="is_highlighted"
            class="cursor-pointer text-neutral-300 transition-colors duration-200 ease-in-out group-hover:text-neutral-200 dark:text-neutral-700 dark:group-hover:text-neutral-600"
        />
        <path
            v-if="mass_status !== 'fresh'"
            :d="curve"
            stroke="currentColor"
            fill="none"
            stroke-width="4"
            stroke-dasharray="2,6"
            stroke-dashoffset="4"
            :data-connection-status="mass_status"
            :data-highlighted="is_highlighted"
            class="cursor-pointer transition-colors duration-200 ease-in-out"
        />
        <!-- Rally route animated overlay -->
        <template v-if="is_on_rally_route">
            <path :d="curve" stroke="var(--color-pink-400)" fill="none" stroke-width="6" stroke-opacity="0.3" />
            <path
                :d="curve"
                stroke="var(--color-pink-400)"
                fill="none"
                stroke-width="3"
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
        <circle :cx="from.x" :cy="from.y" r="8" fill="currentColor" />
        <circle :cx="to.x" :cy="to.y" r="8" fill="currentColor" />
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

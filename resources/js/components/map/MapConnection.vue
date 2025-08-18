<script setup lang="ts">
import { TMassStatus } from '@/types/models';
import { computed } from 'vue';

type Position = {
    x: number;
    y: number;
};

type Props = {
    from: Position;
    to: Position;
    extra?: string;
    mass_status?: TMassStatus;
    is_eol?: boolean;
    is_highlighted?: boolean;
};

const { from, to, extra, mass_status, is_eol } = defineProps<Props>();

const emit = defineEmits<{
    (e: 'connectionContextMenu', event: MouseEvent): void;
    (e: 'connectionClick', event: MouseEvent): void;
}>();

const curve = computed(() => bezierCurve(from, to));
const center = computed(() => midPoint(from, to));

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
    if (is_eol) return '2,6';
}
</script>

<template>
    <g pointer-events="visiblePainted" class="group text-neutral-300 dark:text-neutral-700">
        <path
            v-if="mass_status === 'fresh' || is_eol"
            :d="curve"
            stroke="currentColor"
            fill="none"
            stroke-width="4"
            :stroke-dasharray="getDashArray()"
            :data-eol="is_eol"
            :data-highlighted="is_highlighted"
            class="cursor-pointer text-neutral-300 transition-colors duration-200 ease-in-out group-hover:text-neutral-200 data-[eol=true]:text-purple-500 data-[highlighted=true]:text-amber-500 dark:text-neutral-700 dark:group-hover:text-neutral-600 dark:data-[eol=true]:text-purple-500 dark:data-[highlighted=true]:text-amber-500"
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
            class="cursor-pointer transition-colors duration-200 ease-in-out data-[connection-status=critical]:text-red-500 data-[connection-status=reduced]:text-orange-500 data-[highlighted=true]:text-amber-500 dark:data-[connection-status=critical]:text-red-500 dark:data-[connection-status=reduced]:text-orange-500 dark:data-[highlighted=true]:text-amber-500"
        />
        <rect
            class="pointer-events-none fill-white stroke-neutral-300 dark:fill-neutral-900 dark:stroke-neutral-700"
            :x="center.x - 12"
            :y="center.y - 8"
            width="24"
            height="16"
            rx="2"
            ry="2"
            v-if="extra"
        />
        <text
            :x="center.x"
            :y="center.y + 4"
            text-anchor="middle"
            fill="currentColor"
            font-size="12"
            font-weight="bold"
            class="pointer-events-none text-neutral-700 select-none dark:text-neutral-300"
            v-if="extra"
        >
            {{ extra }}
        </text>
        <path
            :d="curve"
            stroke="transparent"
            fill="none"
            stroke-width="24"
            class="cursor-pointer transition-colors duration-200"
            @contextmenu="(event) => emit('connectionContextMenu', event)"
            @click="(event) => emit('connectionClick', event)"
            @pointerdown.stop
        />
        <circle :cx="from.x" :cy="from.y" r="8" fill="currentColor" />
        <circle :cx="to.x" :cy="to.y" r="8" fill="currentColor" />
    </g>
</template>

<style scoped></style>

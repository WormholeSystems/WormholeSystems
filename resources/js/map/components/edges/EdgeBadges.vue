<script lang="ts">
/** One badge in the cluster drawn at the connection's centre. */
export type EdgeIndicator = {
    type: 'text' | 'clock' | 'weight' | 'gate' | 'preserve';
    label?: string;
    fill: string;
    stroke: string;
};
</script>

<script setup lang="ts">
import type { Vec2 } from '@/map/core/types';
import { Clock, Heart, Orbit, Weight } from 'lucide-vue-next';
import { computed } from 'vue';

type Props = {
    indicators: EdgeIndicator[];
    /** Centre of the badge cluster in screen pixels (already scaled). */
    center: Vec2;
};

const { indicators, center } = defineProps<Props>();

const totalWidth = computed(() => {
    const count = indicators.length;
    if (count === 0) return 0;
    return count * 18 + 8;
});
</script>

<template>
    <foreignObject
        v-if="indicators.length"
        :x="center.x - totalWidth / 2"
        :y="center.y - 10"
        :width="totalWidth"
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
</template>

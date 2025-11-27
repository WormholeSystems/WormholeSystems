<script setup lang="ts">
import { TUniverseSolarsystem } from '@/types/universe-map';
import { computed } from 'vue';

const props = defineProps<{
    solarsystem: TUniverseSolarsystem;
    x: number;
    y: number;
}>();

// Security color
const securityColor = computed(() => {
    const sec = props.solarsystem.security;
    if (props.solarsystem.class) {
        const colors: Record<number, string> = {
            1: '#67e8f9', // cyan-300
            2: '#3b82f6', // blue-500
            3: '#d8b4fe', // purple-300
            4: '#8b5cf6', // violet-500
            5: '#fb923c', // orange-400
            6: '#ef4444', // red-500
        };
        return colors[props.solarsystem.class] ?? '#a3a3a3';
    }
    if (sec >= 0.5) return '#22c55e'; // green-500
    if (sec >= 0.1) return '#f97316'; // orange-500
    return '#ef4444'; // red-500
});

// Border color based on sovereignty or security
const borderColor = computed(() => {
    // Faction warfare systems
    if (props.solarsystem.sovereignty?.faction) {
        return '#fbbf24'; // amber
    }
    // Alliance sovereignty
    if (props.solarsystem.sovereignty?.alliance) {
        return '#60a5fa'; // blue-400
    }
    return securityColor.value;
});

// Format security status
const securityDisplay = computed(() => {
    if (props.solarsystem.class) {
        return `C${props.solarsystem.class}`;
    }
    return props.solarsystem.security.toFixed(1);
});

// Get sovereignty ticker
const sovereigntyTicker = computed(() => {
    if (props.solarsystem.sovereignty?.alliance) {
        return props.solarsystem.sovereignty.alliance.ticker;
    }
    if (props.solarsystem.sovereignty?.corporation) {
        return props.solarsystem.sovereignty.corporation.ticker;
    }
    if (props.solarsystem.sovereignty?.faction) {
        return props.solarsystem.sovereignty.faction.name.slice(0, 4).toUpperCase();
    }
    return null;
});

// Short system name for display
const shortName = computed(() => {
    const name = props.solarsystem.name;
    if (name.length <= 10) return name;
    return name;
});
</script>

<template>
    <div
        class="universe-system group absolute"
        :style="{
            left: x + 'px',
            top: y + 'px',
            transform: 'translate(-50%, -50%)',
        }"
    >
        <!-- Main System Box (Dotlan-inspired) -->
        <div
            class="relative flex min-w-[70px] flex-col overflow-hidden rounded border bg-neutral-900/90 text-[9px] leading-tight shadow-lg backdrop-blur-sm transition-all duration-150 hover:z-50 hover:scale-110 hover:shadow-xl"
            :style="{
                borderColor: borderColor,
                boxShadow: `0 0 8px ${borderColor}33`,
            }"
        >
            <!-- Header with system name -->
            <div
                class="flex items-center justify-between gap-1 px-1.5 py-0.5"
                :style="{
                    backgroundColor: borderColor + '20',
                }"
            >
                <span class="truncate font-semibold text-white" :title="solarsystem.name">
                    {{ shortName }}
                </span>
            </div>

            <!-- Content Row -->
            <div class="flex items-center justify-between gap-2 px-1.5 py-0.5">
                <!-- Security Status -->
                <span class="font-mono font-bold" :style="{ color: securityColor }">
                    {{ securityDisplay }}
                </span>

                <!-- Sovereignty Ticker -->
                <span v-if="sovereigntyTicker" class="truncate text-neutral-400" :title="sovereigntyTicker">
                    {{ sovereigntyTicker }}
                </span>
            </div>

            <!-- Region/Constellation (shown on hover) -->
            <div
                class="max-h-0 overflow-hidden px-1.5 text-[8px] text-neutral-500 transition-all duration-150 group-hover:max-h-12 group-hover:py-0.5"
            >
                <div class="truncate">{{ solarsystem.constellation.name }}</div>
                <div class="truncate text-neutral-600">{{ solarsystem.region.name }}</div>
            </div>
        </div>

        <!-- Connection dot (for future connections) -->
        <div
            class="absolute top-1/2 left-1/2 h-1 w-1 -translate-x-1/2 -translate-y-1/2 rounded-full opacity-0 transition-opacity group-hover:opacity-100"
            :style="{ backgroundColor: borderColor }"
        />
    </div>
</template>

<style scoped>
.universe-system {
    pointer-events: auto;
}

.universe-system:hover {
    z-index: 100;
}
</style>

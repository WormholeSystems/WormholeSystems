<script setup lang="ts">
import UniverseMapCanvas from '@/components/universe-map/UniverseMapCanvas.vue';
import UniverseMapControls from '@/components/universe-map/UniverseMapControls.vue';
import UniverseMapRegionSelector from '@/components/universe-map/UniverseMapRegionSelector.vue';
import AppLayout from '@/layouts/AppLayout.vue';
import SeoHead from '@/layouts/SeoHead.vue';
import { TUniverseMapProps } from '@/types/universe-map';
import { computed, provide, ref } from 'vue';

const { solarsystems, connections, regions, bounds } = defineProps<TUniverseMapProps>();

// Pan & zoom state
const scale = ref(0.5);
const panOffset = ref({ x: 0, y: 0 });

// Canvas ref for calling exposed methods
const canvasRef = ref<InstanceType<typeof UniverseMapCanvas> | null>(null);

// Currently focused region
const focusedRegionId = ref<number | null>(null);

// Provide scale and pan to child components
provide('universeMapScale', scale);
provide('universeMapPanOffset', panOffset);

const selectedRegionName = computed(() => {
    if (!focusedRegionId.value) return 'All Regions';
    const region = regions.find((r) => r.id === focusedRegionId.value);
    return region?.name ?? 'All Regions';
});

function handleRegionSelect(regionId: number | null) {
    focusedRegionId.value = regionId;
    canvasRef.value?.focusOnRegion(regionId);
}
</script>

<template>
    <AppLayout>
        <SeoHead
            :title="`Universe Map - ${selectedRegionName}`"
            description="Explore the EVE Online universe with our interactive 2D map. View solar systems, security status, and sovereignty information."
            keywords="EVE Online, universe map, solar systems, navigation, space exploration"
        />

        <div class="flex h-[calc(100vh-4rem)] flex-col gap-4 p-4">
            <!-- Header Controls -->
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-4">
                    <h1 class="font-display text-2xl font-bold tracking-tight">Universe Map</h1>
                    <UniverseMapRegionSelector :regions="regions" @select-region="handleRegionSelect" />
                </div>
                <UniverseMapControls v-model:scale="scale" />
            </div>

            <!-- Map Canvas -->
            <div class="relative flex-1 overflow-hidden rounded-xl border border-neutral-200 bg-neutral-950 dark:border-neutral-800">
                <UniverseMapCanvas
                    ref="canvasRef"
                    :solarsystems="solarsystems"
                    :connections="connections"
                    :bounds="bounds"
                    v-model:scale="scale"
                    v-model:pan-offset="panOffset"
                />
            </div>
        </div>
    </AppLayout>
</template>

<script setup lang="ts">
import { Kbd } from '@/components/ui/kbd';
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group';
import UniverseMapCanvas from '@/components/universe-map/UniverseMapCanvas.vue';
import UniverseMapControls from '@/components/universe-map/UniverseMapControls.vue';
import UniverseMapRegionSelector from '@/components/universe-map/UniverseMapRegionSelector.vue';
import UniverseMapSearch from '@/components/universe-map/UniverseMapSearch.vue';
import UniverseSystemContextMenu from '@/components/universe-map/UniverseSystemContextMenu.vue';
import UniverseSystemPanel from '@/components/universe-map/UniverseSystemPanel.vue';
import AppLayout from '@/layouts/AppLayout.vue';
import SeoHead from '@/layouts/SeoHead.vue';
import { TUniverseMapProps, TUniverseSolarsystem } from '@/types/universe-map';
import { router } from '@inertiajs/vue3';
import { Eraser, PenTool, Search, Square } from 'lucide-vue-next';
import { computed, provide, ref, watch } from 'vue';

// Context menu state
const contextMenuSystem = ref<TUniverseSolarsystem | null>(null);
const contextMenuPosition = ref({ x: 0, y: 0 });

const props = defineProps<TUniverseMapProps>();

// Search ref
const searchRef = ref<InstanceType<typeof UniverseMapSearch> | null>(null);

// Pan & zoom state
const scale = ref(0.5);
const panOffset = ref({ x: 0, y: 0 });

// Canvas ref for calling exposed methods
const canvasRef = ref<InstanceType<typeof UniverseMapCanvas> | null>(null);

// Currently focused region
const focusedRegionId = ref<number | null>(null);

// Intel tool state - empty string means no tool selected (not in intel mode)
const selectedTool = ref<string>('');

function handleSystemClick(system: TUniverseSolarsystem) {
    router.reload({
        data: { system: system.id },
        only: ['selectedSystemDetails'],
    });
}

function closeSystemPanel() {
    canvasRef.value?.clearSystemSelection();
    router.reload({
        data: { system: undefined },
        only: ['selectedSystemDetails'],
    });
}

// Provide scale and pan to child components
provide('universeMapScale', scale);
provide('universeMapPanOffset', panOffset);

const selectedRegionName = computed(() => {
    if (!focusedRegionId.value) return 'All Regions';
    const region = props.regions.find((r) => r.id === focusedRegionId.value);
    return region?.name ?? 'All Regions';
});

function handleRegionSelect(regionId: number | null) {
    focusedRegionId.value = regionId;
    canvasRef.value?.focusOnRegion(regionId);
}

function handleFocusRegion(regionId: number) {
    focusedRegionId.value = regionId;
    canvasRef.value?.focusOnRegion(regionId);
}

function handleFocusConstellation(constellationId: number) {
    canvasRef.value?.focusOnConstellation(constellationId);
}

function handleFocusSystem(systemId: number) {
    canvasRef.value?.focusOnSystem(systemId);
}

// Search handlers
function handleSearchSelectSystem(systemId: number) {
    canvasRef.value?.focusOnSystem(systemId);
    // Also load system details
    router.reload({
        data: { system: systemId },
        only: ['selectedSystemDetails'],
    });
}

function handleSearchSelectConstellation(constellationId: number) {
    canvasRef.value?.focusOnConstellation(constellationId);
}

function handleSearchSelectRegion(regionId: number) {
    focusedRegionId.value = regionId;
    canvasRef.value?.focusOnRegion(regionId);
}

function handleSelectAdjacentSystem(systemId: number) {
    canvasRef.value?.focusOnSystem(systemId);
    router.reload({
        data: { system: systemId },
        only: ['selectedSystemDetails'],
    });
}

function handleSystemContextMenu(system: TUniverseSolarsystem, position: { x: number; y: number }) {
    contextMenuSystem.value = system;
    contextMenuPosition.value = position;
}

function closeContextMenu() {
    contextMenuSystem.value = null;
}

// Sync tool selection with canvas drawing mode
watch(selectedTool, (tool) => {
    if (!canvasRef.value) return;

    if (tool) {
        canvasRef.value.isDrawingMode = true;
        canvasRef.value.drawMode = tool as 'rect' | 'polygon' | 'eraser';
    } else {
        canvasRef.value.isDrawingMode = false;
    }
});

// Sync canvas state back to tool selection (e.g., when Escape is pressed)
watch(
    () => canvasRef.value?.isDrawingMode,
    (isDrawing) => {
        if (!isDrawing) {
            selectedTool.value = '';
        }
    },
);
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
                    <UniverseMapRegionSelector :regions="props.regions" @select-region="handleRegionSelect" />

                    <!-- Search trigger -->
                    <button
                        @click="
                            ($event.currentTarget as HTMLElement)?.blur();
                            searchRef?.open();
                        "
                        class="flex h-9 w-64 items-center gap-2 rounded-md border border-input bg-transparent px-3 text-sm text-muted-foreground shadow-xs transition-[color,box-shadow] hover:border-ring hover:ring-[3px] hover:ring-ring/50 dark:bg-input/30"
                    >
                        <Search class="h-4 w-4" />
                        <span class="flex-1 text-left">Search...</span>
                        <Kbd>⌘K</Kbd>
                    </button>

                    <!-- Intel Controls -->
                    <div class="ml-4 flex items-center gap-2 border-l border-neutral-700 pl-4">
                        <ToggleGroup v-model="selectedTool" type="single" variant="outline" size="sm">
                            <ToggleGroupItem value="rect" title="Rectangle (click & drag)">
                                <Square class="h-4 w-4" />
                            </ToggleGroupItem>
                            <ToggleGroupItem value="polygon" title="Polygon (click points, double-click to finish)">
                                <PenTool class="h-4 w-4" />
                            </ToggleGroupItem>
                            <ToggleGroupItem
                                value="eraser"
                                title="Eraser (click to delete)"
                                class="data-[state=on]:bg-red-600 data-[state=on]:text-white"
                            >
                                <Eraser class="h-4 w-4" />
                            </ToggleGroupItem>
                        </ToggleGroup>

                        <!-- Color picker -->
                        <div v-if="selectedTool && selectedTool !== 'eraser'" class="ml-2 flex items-center gap-1">
                            <button
                                v-for="color in canvasRef?.annotationColors"
                                :key="color"
                                @click="canvasRef!.selectedColor = color"
                                class="h-5 w-5 rounded-full border-2 transition-transform hover:scale-110"
                                :class="canvasRef?.selectedColor === color ? 'scale-110 border-white' : 'border-transparent'"
                                :style="{ backgroundColor: color }"
                            />
                        </div>
                    </div>
                </div>
                <UniverseMapControls 
                    v-model:scale="scale" 
                    @zoom-in="canvasRef?.zoomIn()"
                    @zoom-out="canvasRef?.zoomOut()"
                    @reset="canvasRef?.centerMap()"
                />
            </div>

            <!-- Map Canvas -->
            <div class="relative flex-1 overflow-hidden rounded-xl border border-neutral-200 bg-neutral-950 dark:border-neutral-800">
                <UniverseMapCanvas
                    ref="canvasRef"
                    :solarsystems="props.solarsystems"
                    :connections="props.connections"
                    :bounds="props.bounds"
                    v-model:scale="scale"
                    v-model:pan-offset="panOffset"
                    @system-click="handleSystemClick"
                    @system-contextmenu="handleSystemContextMenu"
                />

                <!-- System Detail Panel -->
                <UniverseSystemPanel
                    :details="props.selectedSystemDetails ?? null"
                    @close="closeSystemPanel"
                    @focus-region="handleFocusRegion"
                    @focus-constellation="handleFocusConstellation"
                    @focus-system="handleFocusSystem"
                    @select-system="handleSelectAdjacentSystem"
                />
            </div>
        </div>

        <!-- Search Dialog -->
        <UniverseMapSearch
            ref="searchRef"
            :solarsystems="props.solarsystems"
            :regions="props.regions"
            @select-system="handleSearchSelectSystem"
            @select-constellation="handleSearchSelectConstellation"
            @select-region="handleSearchSelectRegion"
        />

        <!-- System Context Menu -->
        <UniverseSystemContextMenu :system="contextMenuSystem" :position="contextMenuPosition" @close="closeContextMenu" />
    </AppLayout>
</template>

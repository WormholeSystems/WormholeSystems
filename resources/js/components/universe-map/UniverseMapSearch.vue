<script setup lang="ts">
import { Dialog, DialogContent } from '@/components/ui/dialog';
import { Kbd } from '@/components/ui/kbd';
import { ScrollArea } from '@/components/ui/scroll-area';
import { TUniverseRegion, TUniverseSolarsystem } from '@/types/universe-map';
import { useMagicKeys, whenever } from '@vueuse/core';
import { Globe, MapPin, Navigation, Search } from 'lucide-vue-next';
import { computed, ref, watch } from 'vue';

const props = defineProps<{
    solarsystems: TUniverseSolarsystem[];
    regions: TUniverseRegion[];
}>();

const emit = defineEmits<{
    'select-system': [systemId: number];
    'select-constellation': [constellationId: number];
    'select-region': [regionId: number];
}>();

const isOpen = ref(false);
const searchQuery = ref('');
const selectedIndex = ref(0);

// Get unique constellations from systems
const constellations = computed(() => {
    const map = new Map<number, { id: number; name: string; regionName: string }>();
    for (const system of props.solarsystems) {
        if (!map.has(system.constellation_id)) {
            map.set(system.constellation_id, {
                id: system.constellation_id,
                name: system.constellation.name,
                regionName: system.region.name,
            });
        }
    }
    return Array.from(map.values()).sort((a, b) => a.name.localeCompare(b.name));
});

// Sort results: items starting with query first, then alphabetically
function sortByRelevance<T extends { name: string }>(items: T[], query: string): T[] {
    return items.sort((a, b) => {
        const aLower = a.name.toLowerCase();
        const bLower = b.name.toLowerCase();
        const aStartsWith = aLower.startsWith(query);
        const bStartsWith = bLower.startsWith(query);

        // Prioritize items that start with the query
        if (aStartsWith && !bStartsWith) return -1;
        if (!aStartsWith && bStartsWith) return 1;

        // Then sort alphabetically
        return aLower.localeCompare(bLower);
    });
}

// Search results
const results = computed(() => {
    const query = searchQuery.value.toLowerCase().trim();
    if (!query) return { systems: [], constellations: [], regions: [] };

    const maxResults = 10;

    const matchedSystems = props.solarsystems.filter((s) => s.name.toLowerCase().includes(query));
    const systems = sortByRelevance(matchedSystems, query)
        .slice(0, maxResults)
        .map((s) => ({
            id: s.id,
            name: s.name,
            subtitle: `${s.constellation.name} • ${s.region.name}`,
            security: s.security,
        }));

    const matchedConstellationsList = constellations.value.filter((c) => c.name.toLowerCase().includes(query));
    const matchedConstellations = sortByRelevance(matchedConstellationsList, query)
        .slice(0, maxResults)
        .map((c) => ({
            id: c.id,
            name: c.name,
            subtitle: c.regionName,
        }));

    const matchedRegionsList = props.regions.filter((r) => r.name.toLowerCase().includes(query));
    const matchedRegions = sortByRelevance(matchedRegionsList, query)
        .slice(0, maxResults)
        .map((r) => ({
            id: r.id,
            name: r.name,
            subtitle: 'Region',
        }));

    return {
        systems,
        constellations: matchedConstellations,
        regions: matchedRegions,
    };
});

// Flatten results for keyboard navigation
const flatResults = computed(() => {
    const flat: { type: 'system' | 'constellation' | 'region'; id: number; name: string; subtitle: string; security?: number }[] = [];

    for (const r of results.value.regions) {
        flat.push({ type: 'region', ...r });
    }
    for (const c of results.value.constellations) {
        flat.push({ type: 'constellation', ...c });
    }
    for (const s of results.value.systems) {
        flat.push({ type: 'system', ...s });
    }

    return flat;
});

// Reset state when dialog opens/closes
watch(isOpen, (open) => {
    if (open) {
        searchQuery.value = '';
        selectedIndex.value = 0;
    }
});

// Reset selected index when results change
watch(flatResults, () => {
    selectedIndex.value = 0;
});

// Keyboard shortcuts
const { ctrl_k, meta_k } = useMagicKeys();

whenever(
    () => ctrl_k.value || meta_k.value,
    () => {
        isOpen.value = true;
    },
);

function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'ArrowDown') {
        event.preventDefault();
        selectedIndex.value = Math.min(selectedIndex.value + 1, flatResults.value.length - 1);
    } else if (event.key === 'ArrowUp') {
        event.preventDefault();
        selectedIndex.value = Math.max(selectedIndex.value - 1, 0);
    } else if (event.key === 'Enter') {
        event.preventDefault();
        selectResult(flatResults.value[selectedIndex.value]);
    }
}

function selectResult(result: { type: 'system' | 'constellation' | 'region'; id: number } | undefined) {
    if (!result) return;

    isOpen.value = false;

    // Blur any focused element to prevent focus jumping to panel buttons
    if (document.activeElement instanceof HTMLElement) {
        document.activeElement.blur();
    }

    if (result.type === 'system') {
        emit('select-system', result.id);
    } else if (result.type === 'constellation') {
        emit('select-constellation', result.id);
    } else if (result.type === 'region') {
        emit('select-region', result.id);
    }
}

function getSecurityColor(security: number): string {
    if (security >= 0.5) return 'text-green-400';
    if (security > 0) return 'text-yellow-400';
    return 'text-red-400';
}

function open() {
    isOpen.value = true;
}

defineExpose({ open });
</script>

<template>
    <Dialog v-model:open="isOpen">
        <DialogContent class="fixed top-[20%] max-w-lg translate-y-0 gap-0 overflow-hidden p-0" @close-auto-focus.prevent>
            <!-- Search Input -->
            <div class="flex items-center gap-3 border-b border-neutral-800 px-4 py-3 pr-12">
                <Search class="h-5 w-5 text-neutral-500" />
                <input
                    v-model="searchQuery"
                    type="text"
                    placeholder="Search systems, constellations, regions..."
                    class="flex-1 bg-transparent text-sm outline-none placeholder:text-neutral-500"
                    autofocus
                    @keydown="handleKeyDown"
                />
            </div>

            <!-- Results -->
            <ScrollArea class="h-[350px]">
                <div class="p-2">
                    <!-- No query -->
                    <div v-if="!searchQuery.trim()" class="px-3 py-8 text-center text-sm text-neutral-500">Start typing to search...</div>

                    <!-- No results -->
                    <div v-else-if="flatResults.length === 0" class="px-3 py-8 text-center text-sm text-neutral-500">
                        No results found for "{{ searchQuery }}"
                    </div>

                    <!-- Results -->
                    <template v-else>
                        <!-- Regions -->
                        <div v-if="results.regions.length" class="mb-2">
                            <div class="px-2 py-1 text-xs font-medium tracking-wider text-neutral-500 uppercase">Regions</div>
                            <button
                                v-for="(region, i) in results.regions"
                                :key="`region-${region.id}`"
                                @click="selectResult({ type: 'region', id: region.id })"
                                class="flex w-full items-center gap-3 rounded-md px-2 py-2 text-left text-sm transition-colors"
                                :class="
                                    flatResults.findIndex((r) => r.type === 'region' && r.id === region.id) === selectedIndex
                                        ? 'bg-neutral-800 text-white'
                                        : 'text-neutral-300 hover:bg-neutral-800/50'
                                "
                            >
                                <Globe class="h-4 w-4 text-blue-400" />
                                <span class="flex-1">{{ region.name }}</span>
                            </button>
                        </div>

                        <!-- Constellations -->
                        <div v-if="results.constellations.length" class="mb-2">
                            <div class="px-2 py-1 text-xs font-medium tracking-wider text-neutral-500 uppercase">Constellations</div>
                            <button
                                v-for="constellation in results.constellations"
                                :key="`constellation-${constellation.id}`"
                                @click="selectResult({ type: 'constellation', id: constellation.id })"
                                class="flex w-full items-center gap-3 rounded-md px-2 py-2 text-left text-sm transition-colors"
                                :class="
                                    flatResults.findIndex((r) => r.type === 'constellation' && r.id === constellation.id) === selectedIndex
                                        ? 'bg-neutral-800 text-white'
                                        : 'text-neutral-300 hover:bg-neutral-800/50'
                                "
                            >
                                <Navigation class="h-4 w-4 text-cyan-400" />
                                <span class="flex-1">{{ constellation.name }}</span>
                                <span class="text-xs text-neutral-500">{{ constellation.subtitle }}</span>
                            </button>
                        </div>

                        <!-- Systems -->
                        <div v-if="results.systems.length">
                            <div class="px-2 py-1 text-xs font-medium tracking-wider text-neutral-500 uppercase">Systems</div>
                            <button
                                v-for="system in results.systems"
                                :key="`system-${system.id}`"
                                @click="selectResult({ type: 'system', id: system.id })"
                                class="flex w-full items-center gap-3 rounded-md px-2 py-2 text-left text-sm transition-colors"
                                :class="
                                    flatResults.findIndex((r) => r.type === 'system' && r.id === system.id) === selectedIndex
                                        ? 'bg-neutral-800 text-white'
                                        : 'text-neutral-300 hover:bg-neutral-800/50'
                                "
                            >
                                <MapPin class="h-4 w-4 text-amber-400" />
                                <span class="flex-1">{{ system.name }}</span>
                                <span v-if="system.security !== undefined" class="font-mono text-xs" :class="getSecurityColor(system.security)">
                                    {{ system.security.toFixed(1) }}
                                </span>
                                <span class="text-xs text-neutral-500">{{ system.subtitle }}</span>
                            </button>
                        </div>
                    </template>
                </div>
            </ScrollArea>

            <!-- Footer -->
            <div class="flex items-center justify-between border-t border-neutral-800 px-4 py-2 text-xs text-neutral-500">
                <div class="flex items-center gap-4">
                    <span class="flex items-center gap-1">
                        <Kbd>↑</Kbd>
                        <Kbd>↓</Kbd>
                        to navigate
                    </span>
                    <span class="flex items-center gap-1">
                        <Kbd>↵</Kbd>
                        to select
                    </span>
                </div>
            </div>
        </DialogContent>
    </Dialog>
</template>

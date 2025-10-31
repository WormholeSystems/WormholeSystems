<script setup lang="ts">
import EveScoutConnection from '@/components/eve-scout/EveScoutConnection.vue';
import EveScoutConnectionPlaceholder from '@/components/eve-scout/EveScoutConnectionPlaceholder.vue';
import { Button } from '@/components/ui/button';
import { CardAction, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useMap } from '@/composables/useMap';
import { useSelectedMapSolarsystem } from '@/composables/useSelectedMapSolarsystem';
import EveScoutConnections from '@/routes/eve-scout-connections';
import { TEveScoutConnection } from '@/types/eve-scout';
import { Deferred, router } from '@inertiajs/vue3';
import { useLocalStorage } from '@vueuse/core';
import { ArrowDown, ArrowUp, ExternalLink, Plus } from 'lucide-vue-next';
import { computed, ref } from 'vue';

const map = useMap();

const { eve_scout_connections } = defineProps<{
    eve_scout_connections?: TEveScoutConnection[];
}>();

const selectedMapSolarsystem = useSelectedMapSolarsystem();

const description = computed(() => {
    if (selectedMapSolarsystem.value?.solarsystem?.name) {
        return `Showing jump distances from ${selectedMapSolarsystem.value.solarsystem.name}`;
    }
    return 'Select a system on the map to see jump distances';
});

function addAllToMap(specialSystem: string) {
    router.post(
        EveScoutConnections.store().url,
        {
            map_id: map.value.id,
            solarsystem_name: specialSystem === 'Thera' ? 'Thera' : 'Turnur',
        },
        {
            preserveState: true,
            preserveScroll: true,
            only: ['map', 'selected_map_solarsystem', 'eve_scout_connections'],
        },
    );
}

type SortColumn = 'system' | 'region' | 'sovereignty' | 'jumps' | 'type' | 'sig_in' | 'sig_out' | 'time';
type SortDirection = 'asc' | 'desc';

const theraSortColumn = ref<SortColumn>('system');
const theraSortDirection = ref<SortDirection>('asc');
const turnurSortColumn = ref<SortColumn>('system');
const turnurSortDirection = ref<SortDirection>('asc');

function getOtherSystem(connection: TEveScoutConnection, specialSystem: string) {
    return connection.in_system.name === specialSystem ? connection.out_system : connection.in_system;
}

function sortConnections(connections: TEveScoutConnection[], specialSystem: string, sortColumn: SortColumn, sortDirection: SortDirection) {
    return [...connections].sort((a, b) => {
        const aSystem = getOtherSystem(a, specialSystem);
        const bSystem = getOtherSystem(b, specialSystem);

        let comparison = 0;

        switch (sortColumn) {
            case 'system':
                // First sort by security status/class
                const aIsWH = aSystem.class !== null;
                const bIsWH = bSystem.class !== null;

                if (aIsWH && bIsWH) {
                    // Both are wormholes, sort by class
                    comparison = (aSystem.class || 0) - (bSystem.class || 0);
                } else if (!aIsWH && !bIsWH) {
                    // Both are k-space, sort by security
                    comparison = (bSystem.security || 0) - (aSystem.security || 0);
                } else {
                    // One is WH, one is k-space. K-space first
                    comparison = aIsWH ? 1 : -1;
                }

                // If security/class is same, sort by name
                if (comparison === 0) {
                    comparison = aSystem.name.localeCompare(bSystem.name);
                }
                break;
            case 'region':
                comparison = (aSystem.region?.name || '').localeCompare(bSystem.region?.name || '');
                break;
            case 'sovereignty':
                const aSov =
                    aSystem.sovereignty?.alliance?.name || aSystem.sovereignty?.corporation?.name || aSystem.sovereignty?.faction?.name || '';
                const bSov =
                    bSystem.sovereignty?.alliance?.name || bSystem.sovereignty?.corporation?.name || bSystem.sovereignty?.faction?.name || '';
                comparison = aSov.localeCompare(bSov);
                break;
            case 'jumps':
                const aJumps = a.jumps_from_selected ?? 999;
                const bJumps = b.jumps_from_selected ?? 999;
                comparison = aJumps - bJumps;
                break;
            case 'type':
                comparison = a.wormhole_type.localeCompare(b.wormhole_type);
                break;
            case 'sig_in':
                const aInSig = a.in_system.name === specialSystem ? a.in_signature : a.out_signature;
                const bInSig = b.in_system.name === specialSystem ? b.in_signature : b.out_signature;
                comparison = aInSig.localeCompare(bInSig);
                break;
            case 'sig_out':
                const aOutSig = a.in_system.name === specialSystem ? a.out_signature : a.in_signature;
                const bOutSig = b.in_system.name === specialSystem ? b.out_signature : b.in_signature;
                comparison = aOutSig.localeCompare(bOutSig);
                break;
            case 'time':
                const aTime = a.remaining_hours || 999;
                const bTime = b.remaining_hours || 999;
                comparison = aTime - bTime;
                break;
        }

        return sortDirection === 'asc' ? comparison : -comparison;
    });
}

function handleTheraSort(column: SortColumn) {
    if (theraSortColumn.value === column) {
        theraSortDirection.value = theraSortDirection.value === 'asc' ? 'desc' : 'asc';
    } else {
        theraSortColumn.value = column;
        theraSortDirection.value = 'asc';
    }
}

function handleTurnurSort(column: SortColumn) {
    if (turnurSortColumn.value === column) {
        turnurSortDirection.value = turnurSortDirection.value === 'asc' ? 'desc' : 'asc';
    } else {
        turnurSortColumn.value = column;
        turnurSortDirection.value = 'asc';
    }
}

// Filter and sort connections
const theraConnections = computed(() => {
    const filtered = eve_scout_connections?.filter((conn) => conn.in_system.name === 'Thera' || conn.out_system.name === 'Thera') || [];
    return sortConnections(filtered, 'Thera', theraSortColumn.value, theraSortDirection.value);
});

const turnurConnections = computed(() => {
    const filtered = eve_scout_connections?.filter((conn) => conn.in_system.name === 'Turnur' || conn.out_system.name === 'Turnur') || [];
    return sortConnections(filtered, 'Turnur', turnurSortColumn.value, turnurSortDirection.value);
});

const activeTab = useLocalStorage<'thera' | 'turnur'>('eve-scout-active-tab', 'thera');
</script>

<template>
    <MapPanel>
        <CardHeader>
            <CardTitle>EVE Scout Connections</CardTitle>
            <CardDescription>{{ description }}</CardDescription>
            <CardAction class="flex gap-2">
                <Tooltip v-if="activeTab === 'thera' && theraConnections.length > 0">
                    <TooltipTrigger as-child>
                        <Button @click="addAllToMap('Thera')" size="icon" variant="secondary">
                            <Plus class="size-4" />
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent>Add all Thera connections to map</TooltipContent>
                </Tooltip>
                <Tooltip v-if="activeTab === 'turnur' && turnurConnections.length > 0">
                    <TooltipTrigger as-child>
                        <Button @click="addAllToMap('Turnur')" size="icon" variant="secondary">
                            <Plus class="size-4" />
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent>Add all Turnur connections to map</TooltipContent>
                </Tooltip>
                <Tooltip>
                    <TooltipTrigger as-child>
                        <Button variant="secondary" size="icon" as-child>
                            <a href="https://www.eve-scout.com/" target="_blank" rel="noopener noreferrer">
                                <ExternalLink class="size-4" />
                            </a>
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent>View on EVE Scout</TooltipContent>
                </Tooltip>
            </CardAction>
        </CardHeader>
        <MapPanelContent inner-class="border-0 bg-transparent">
            <div class="relative">
                <Deferred data="eve_scout_connections">
                    <Tabs v-model="activeTab" default-value="thera" class="w-full">
                        <TabsList class="grid w-full grid-cols-2">
                            <TabsTrigger value="thera">
                                Thera
                                <span v-if="theraConnections.length" class="ml-1.5 text-xs text-muted-foreground"
                                    >({{ theraConnections.length }})</span
                                >
                            </TabsTrigger>
                            <TabsTrigger value="turnur">
                                Turnur
                                <span v-if="turnurConnections.length" class="ml-1.5 text-xs text-muted-foreground"
                                    >({{ turnurConnections.length }})</span
                                >
                            </TabsTrigger>
                        </TabsList>
                        <TabsContent value="thera" class="mt-2">
                            <div v-if="theraConnections.length" class="space-y-2">
                                <div
                                    class="grid grid-cols-[auto_auto_auto_auto_auto_auto_auto_auto] gap-2 rounded-lg border bg-white dark:bg-neutral-900/40"
                                >
                                    <!-- Table Header -->
                                    <div
                                        class="col-span-full grid grid-cols-subgrid items-center border-b bg-muted/50 px-2 py-1.5 text-xs font-medium text-muted-foreground"
                                    >
                                        <button @click="handleTheraSort('system')" class="flex items-center gap-1 hover:text-foreground">
                                            <span>Destination</span>
                                            <ArrowUp v-if="theraSortColumn === 'system' && theraSortDirection === 'asc'" class="size-3" />
                                            <ArrowDown v-if="theraSortColumn === 'system' && theraSortDirection === 'desc'" class="size-3" />
                                        </button>
                                        <button @click="handleTheraSort('region')" class="flex items-center gap-1 hover:text-foreground">
                                            <span>Region</span>
                                            <ArrowUp v-if="theraSortColumn === 'region' && theraSortDirection === 'asc'" class="size-3" />
                                            <ArrowDown v-if="theraSortColumn === 'region' && theraSortDirection === 'desc'" class="size-3" />
                                        </button>
                                        <button @click="handleTheraSort('sovereignty')" class="flex items-center gap-1 hover:text-foreground">
                                            <span>Sovereignty</span>
                                            <ArrowUp v-if="theraSortColumn === 'sovereignty' && theraSortDirection === 'asc'" class="size-3" />
                                            <ArrowDown v-if="theraSortColumn === 'sovereignty' && theraSortDirection === 'desc'" class="size-3" />
                                        </button>
                                        <button
                                            @click="handleTheraSort('jumps')"
                                            class="flex items-center justify-center gap-1 hover:text-foreground"
                                        >
                                            <span>Jumps</span>
                                            <ArrowUp v-if="theraSortColumn === 'jumps' && theraSortDirection === 'asc'" class="size-3" />
                                            <ArrowDown v-if="theraSortColumn === 'jumps' && theraSortDirection === 'desc'" class="size-3" />
                                        </button>
                                        <button @click="handleTheraSort('type')" class="flex items-center justify-center gap-1 hover:text-foreground">
                                            <span>WH</span>
                                            <ArrowUp v-if="theraSortColumn === 'type' && theraSortDirection === 'asc'" class="size-3" />
                                            <ArrowDown v-if="theraSortColumn === 'type' && theraSortDirection === 'desc'" class="size-3" />
                                        </button>
                                        <button
                                            @click="handleTheraSort('sig_in')"
                                            class="flex items-center justify-center gap-1 hover:text-foreground"
                                        >
                                            <span>In</span>
                                            <ArrowUp v-if="theraSortColumn === 'sig_in' && theraSortDirection === 'asc'" class="size-3" />
                                            <ArrowDown v-if="theraSortColumn === 'sig_in' && theraSortDirection === 'desc'" class="size-3" />
                                        </button>
                                        <button
                                            @click="handleTheraSort('sig_out')"
                                            class="flex items-center justify-center gap-1 hover:text-foreground"
                                        >
                                            <span>Out</span>
                                            <ArrowUp v-if="theraSortColumn === 'sig_out' && theraSortDirection === 'asc'" class="size-3" />
                                            <ArrowDown v-if="theraSortColumn === 'sig_out' && theraSortDirection === 'desc'" class="size-3" />
                                        </button>
                                        <button @click="handleTheraSort('time')" class="flex items-center justify-center gap-1 hover:text-foreground">
                                            <span>Expires</span>
                                            <ArrowUp v-if="theraSortColumn === 'time' && theraSortDirection === 'asc'" class="size-3" />
                                            <ArrowDown v-if="theraSortColumn === 'time' && theraSortDirection === 'desc'" class="size-3" />
                                        </button>
                                    </div>
                                    <!-- Table Body -->
                                    <EveScoutConnection
                                        v-for="connection in theraConnections"
                                        :key="`${connection.in_signature}-${connection.out_signature}`"
                                        :connection="connection"
                                        special-system="Thera"
                                    />
                                </div>
                            </div>
                            <div v-else class="rounded-lg border bg-white p-4 text-center text-sm text-muted-foreground dark:bg-neutral-900/40">
                                No Thera connections
                            </div>
                        </TabsContent>
                        <TabsContent value="turnur" class="mt-2">
                            <div v-if="turnurConnections.length" class="space-y-2">
                                <div
                                    class="grid grid-cols-[auto_auto_auto_auto_auto_auto_auto_auto] gap-2 rounded-lg border bg-white dark:bg-neutral-900/40"
                                >
                                    <!-- Table Header -->
                                    <div
                                        class="col-span-full grid grid-cols-subgrid items-center border-b bg-muted/50 px-2 py-1.5 text-xs font-medium text-muted-foreground"
                                    >
                                        <button @click="handleTurnurSort('system')" class="flex items-center gap-1 hover:text-foreground">
                                            <span>Destination</span>
                                            <ArrowUp v-if="turnurSortColumn === 'system' && turnurSortDirection === 'asc'" class="size-3" />
                                            <ArrowDown v-if="turnurSortColumn === 'system' && turnurSortDirection === 'desc'" class="size-3" />
                                        </button>
                                        <button @click="handleTurnurSort('region')" class="flex items-center gap-1 hover:text-foreground">
                                            <span>Region</span>
                                            <ArrowUp v-if="turnurSortColumn === 'region' && turnurSortDirection === 'asc'" class="size-3" />
                                            <ArrowDown v-if="turnurSortColumn === 'region' && turnurSortDirection === 'desc'" class="size-3" />
                                        </button>
                                        <button @click="handleTurnurSort('sovereignty')" class="flex items-center gap-1 hover:text-foreground">
                                            <span>Sovereignty</span>
                                            <ArrowUp v-if="turnurSortColumn === 'sovereignty' && turnurSortDirection === 'asc'" class="size-3" />
                                            <ArrowDown v-if="turnurSortColumn === 'sovereignty' && turnurSortDirection === 'desc'" class="size-3" />
                                        </button>
                                        <button
                                            @click="handleTurnurSort('jumps')"
                                            class="flex items-center justify-center gap-1 hover:text-foreground"
                                        >
                                            <span>Jumps</span>
                                            <ArrowUp v-if="turnurSortColumn === 'jumps' && turnurSortDirection === 'asc'" class="size-3" />
                                            <ArrowDown v-if="turnurSortColumn === 'jumps' && turnurSortDirection === 'desc'" class="size-3" />
                                        </button>
                                        <button
                                            @click="handleTurnurSort('type')"
                                            class="flex items-center justify-center gap-1 hover:text-foreground"
                                        >
                                            <span>WH</span>
                                            <ArrowUp v-if="turnurSortColumn === 'type' && turnurSortDirection === 'asc'" class="size-3" />
                                            <ArrowDown v-if="turnurSortColumn === 'type' && turnurSortDirection === 'desc'" class="size-3" />
                                        </button>
                                        <button
                                            @click="handleTurnurSort('sig_in')"
                                            class="flex items-center justify-center gap-1 hover:text-foreground"
                                        >
                                            <span>In</span>
                                            <ArrowUp v-if="turnurSortColumn === 'sig_in' && turnurSortDirection === 'asc'" class="size-3" />
                                            <ArrowDown v-if="turnurSortColumn === 'sig_in' && turnurSortDirection === 'desc'" class="size-3" />
                                        </button>
                                        <button
                                            @click="handleTurnurSort('sig_out')"
                                            class="flex items-center justify-center gap-1 hover:text-foreground"
                                        >
                                            <span>Out</span>
                                            <ArrowUp v-if="turnurSortColumn === 'sig_out' && turnurSortDirection === 'asc'" class="size-3" />
                                            <ArrowDown v-if="turnurSortColumn === 'sig_out' && turnurSortDirection === 'desc'" class="size-3" />
                                        </button>
                                        <button
                                            @click="handleTurnurSort('time')"
                                            class="flex items-center justify-center gap-1 hover:text-foreground"
                                        >
                                            <span>Expires</span>
                                            <ArrowUp v-if="turnurSortColumn === 'time' && turnurSortDirection === 'asc'" class="size-3" />
                                            <ArrowDown v-if="turnurSortColumn === 'time' && turnurSortDirection === 'desc'" class="size-3" />
                                        </button>
                                    </div>
                                    <!-- Table Body -->
                                    <EveScoutConnection
                                        v-for="connection in turnurConnections"
                                        :key="`${connection.in_signature}-${connection.out_signature}`"
                                        :connection="connection"
                                        special-system="Turnur"
                                    />
                                </div>
                            </div>
                            <div v-else class="rounded-lg border bg-white p-4 text-center text-sm text-muted-foreground dark:bg-neutral-900/40">
                                No Turnur connections
                            </div>
                        </TabsContent>
                    </Tabs>
                    <template #fallback>
                        <div class="space-y-2 p-2">
                            <EveScoutConnectionPlaceholder v-for="i in 3" :key="i" />
                        </div>
                    </template>
                </Deferred>
            </div>
        </MapPanelContent>
    </MapPanel>
</template>

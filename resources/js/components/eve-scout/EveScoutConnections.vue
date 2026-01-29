<script setup lang="ts">
import EveScoutConnection from '@/components/eve-scout/EveScoutConnection.vue';
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
import MapPanelHeader from '@/components/ui/map-panel/MapPanelHeader.vue';
import MapPanelHeaderActionButton from '@/components/ui/map-panel/MapPanelHeaderActionButton.vue';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useJumpCounts } from '@/composables/useJumpCounts';
import { useMap } from '@/composables/useMap';
import { useSelectedMapSolarsystem } from '@/composables/useSelectedMapSolarsystem';
import { useShowMap } from '@/composables/useShowMap';
import { useStaticSolarsystems } from '@/composables/useStaticSolarsystems';
import type { TResolvedSolarsystem } from '@/pages/maps';
import EveScoutConnections from '@/routes/eve-scout-connections';
import type { TEveScoutConnection } from '@/types/eve-scout';
import type { TStaticSolarsystem } from '@/types/static-data';
import { router, usePoll } from '@inertiajs/vue3';
import { useLocalStorage, useNow } from '@vueuse/core';
import { ArrowDown, ArrowUp, ExternalLink, Plus } from 'lucide-vue-next';
import { computed, ref } from 'vue';

const map = useMap();
const page = useShowMap();

const { eve_scout_connections } = defineProps<{
    eve_scout_connections?: TEveScoutConnection[];
}>();

const five_minutes_in_ms = 5 * 60 * 1000;

// Refresh EVE Scout data every 5 minutes
usePoll(five_minutes_in_ms, { only: ['eve_scout_connections'] });

const now = useNow({ interval: 60_000 });

function formatTimestamp(connections: TEveScoutConnection[], currentTime: Date): string | null {
    if (!connections?.length) return null;

    const mostRecent = connections.reduce(
        (latest, conn) => {
            if (!conn.created_at) return latest;
            if (!latest) return conn.created_at;
            return conn.created_at > latest ? conn.created_at : latest;
        },
        null as string | null,
    );

    if (!mostRecent) return null;

    const date = new Date(mostRecent);
    const diffMs = currentTime.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMins / 60);

    if (diffMins < 1) return 'just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    return `${Math.floor(diffHours / 24)}d ago`;
}

const selectedMapSolarsystem = useSelectedMapSolarsystem();
const { resolveSolarsystem } = useStaticSolarsystems();
const ignoredSystems = computed(() => page.props.ignored_systems ?? []);

// Collect all unique target IDs and classify them
const allTargetIds = computed(() => {
    if (!eve_scout_connections?.length) {
        return [] as number[];
    }

    const ids = new Set<number>();
    eve_scout_connections.forEach((connection) => {
        ids.add(connection.in_system_id);
        ids.add(connection.out_system_id);
    });

    return Array.from(ids);
});

// WH targets: both Thera and Turnur allowed as midpoints (full EVE Scout routing)
const whTargetIds = computed(() => allTargetIds.value.filter((id) => resolveSolarsystem(id).class !== null));

// K-space targets per tab: only the OTHER special system is allowed as midpoint
// (Thera tab blocks Thera, Turnur tab blocks Turnur)
function getKspaceTargetsForTab(specialSystem: string) {
    return computed(() => {
        const ids = new Set<number>();
        for (const conn of eve_scout_connections ?? []) {
            const inSystem = resolveSolarsystem(conn.in_system_id);
            const outSystem = resolveSolarsystem(conn.out_system_id);
            const isSpecial = inSystem.name === specialSystem || outSystem.name === specialSystem;
            if (!isSpecial) continue;
            const other = inSystem.name === specialSystem ? outSystem : inSystem;
            if (other.class === null) ids.add(other.id);
        }
        return [...ids];
    });
}

function getSpecialSystemId(specialSystem: string) {
    return computed(() => {
        for (const conn of eve_scout_connections ?? []) {
            const inSystem = resolveSolarsystem(conn.in_system_id);
            const outSystem = resolveSolarsystem(conn.out_system_id);
            if (inSystem.name === specialSystem) return inSystem.id;
            if (outSystem.name === specialSystem) return outSystem.id;
        }
        return null;
    });
}

const theraKspaceTargetIds = getKspaceTargetsForTab('Thera');
const turnurKspaceTargetIds = getKspaceTargetsForTab('Turnur');
const theraSystemId = getSpecialSystemId('Thera');
const turnurSystemId = getSpecialSystemId('Turnur');

const sharedParams = {
    fromId: computed(() => selectedMapSolarsystem.value?.solarsystem_id ?? null),
    mapConnections: computed(() => map.value.map_connections ?? []),
    mapSolarsystems: computed(() => map.value.map_solarsystems ?? []),
};

// WH: full EVE Scout routing, no blocking
const { jumpsByTarget: whJumps, routesByTarget: whRoutes } = useJumpCounts({
    ...sharedParams,
    targets: whTargetIds,
    ignoredSystems,
    includeEveScout: true,
});

// Thera k-space: EVE Scout enabled, block Thera as midpoint (Turnur shortcuts OK)
const { jumpsByTarget: theraKspaceJumps, routesByTarget: theraKspaceRoutes } = useJumpCounts({
    ...sharedParams,
    targets: theraKspaceTargetIds,
    ignoredSystems: computed(() => {
        const base = page.props.ignored_systems ?? [];
        return theraSystemId.value ? [...base, theraSystemId.value] : [...base];
    }),
    includeEveScout: true,
});

// Turnur k-space: EVE Scout enabled, block Turnur as midpoint (Thera shortcuts OK)
const { jumpsByTarget: turnurKspaceJumps, routesByTarget: turnurKspaceRoutes } = useJumpCounts({
    ...sharedParams,
    targets: turnurKspaceTargetIds,
    ignoredSystems: computed(() => {
        const base = page.props.ignored_systems ?? [];
        return turnurSystemId.value ? [...base, turnurSystemId.value] : [...base];
    }),
    includeEveScout: true,
});

// Merge results per tab
const theraJumpsByTarget = computed(() => new Map([...theraKspaceJumps.value, ...whJumps.value]));
const theraRoutesByTarget = computed(() => new Map([...theraKspaceRoutes.value, ...whRoutes.value]));
const turnurJumpsByTarget = computed(() => new Map([...turnurKspaceJumps.value, ...whJumps.value]));
const turnurRoutesByTarget = computed(() => new Map([...turnurKspaceRoutes.value, ...whRoutes.value]));

const enrichedConnections = computed(() => {
    if (!eve_scout_connections) {
        return [] as TEnrichedEveScoutConnection[];
    }

    return eve_scout_connections.map((connection) => ({
        ...connection,
        in_system: resolveSolarsystem(connection.in_system_id),
        out_system: resolveSolarsystem(connection.out_system_id),
    }));
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

type TEnrichedEveScoutConnection = TEveScoutConnection & {
    in_system: TStaticSolarsystem;
    out_system: TStaticSolarsystem;
    route?: TResolvedSolarsystem[] | null;
};

const theraSortColumn = ref<SortColumn>('system');
const theraSortDirection = ref<SortDirection>('asc');
const turnurSortColumn = ref<SortColumn>('system');
const turnurSortDirection = ref<SortDirection>('asc');

function getOtherSystem(connection: TEnrichedEveScoutConnection, specialSystem: string) {
    return connection.in_system.name === specialSystem ? connection.out_system : connection.in_system;
}

function sortConnections(connections: TEnrichedEveScoutConnection[], specialSystem: string, sortColumn: SortColumn, sortDirection: SortDirection) {
    return [...connections].sort((a, b) => {
        const aSystem = getOtherSystem(a, specialSystem);
        const bSystem = getOtherSystem(b, specialSystem);

        let comparison = 0;

        switch (sortColumn) {
            case 'system': {
                const aIsWH = aSystem.class !== null;
                const bIsWH = bSystem.class !== null;

                if (aIsWH && bIsWH) {
                    comparison = (aSystem.class || 0) - (bSystem.class || 0);
                } else if (!aIsWH && !bIsWH) {
                    comparison = (bSystem.security || 0) - (aSystem.security || 0);
                } else {
                    comparison = aIsWH ? 1 : -1;
                }

                if (comparison === 0) {
                    comparison = aSystem.name.localeCompare(bSystem.name);
                }
                break;
            }
            case 'region':
                comparison = (aSystem.region?.name || '').localeCompare(bSystem.region?.name || '');
                break;
            case 'sovereignty': {
                const aSov =
                    aSystem.sovereignty?.alliance?.name || aSystem.sovereignty?.corporation?.name || aSystem.sovereignty?.faction?.name || '';
                const bSov =
                    bSystem.sovereignty?.alliance?.name || bSystem.sovereignty?.corporation?.name || bSystem.sovereignty?.faction?.name || '';
                comparison = aSov.localeCompare(bSov);
                break;
            }
            case 'jumps': {
                const aJumps = a.jumps_from_selected ?? 999;
                const bJumps = b.jumps_from_selected ?? 999;
                comparison = aJumps - bJumps;
                break;
            }
            case 'type':
                comparison = a.wormhole_type.localeCompare(b.wormhole_type);
                break;
            case 'sig_in': {
                const aInSig = a.in_system.name === specialSystem ? a.in_signature : a.out_signature;
                const bInSig = b.in_system.name === specialSystem ? b.in_signature : b.out_signature;
                comparison = aInSig.localeCompare(bInSig);
                break;
            }
            case 'sig_out': {
                const aOutSig = a.in_system.name === specialSystem ? a.out_signature : a.in_signature;
                const bOutSig = b.in_system.name === specialSystem ? b.out_signature : b.in_signature;
                comparison = aOutSig.localeCompare(bOutSig);
                break;
            }
            case 'time': {
                const aTime = a.remaining_hours || 999;
                const bTime = b.remaining_hours || 999;
                comparison = aTime - bTime;
                break;
            }
        }

        return sortDirection === 'asc' ? comparison : -comparison;
    });
}

function resolveJumpCount(connection: TEnrichedEveScoutConnection, specialSystem: string): number | null {
    if (!selectedMapSolarsystem.value) {
        return null;
    }

    const otherSystem = getOtherSystem(connection, specialSystem);
    const jumps = specialSystem === 'Thera' ? theraJumpsByTarget.value : turnurJumpsByTarget.value;
    return jumps.get(otherSystem.id) ?? null;
}

function resolveRoute(connection: TEnrichedEveScoutConnection, specialSystem: string): TResolvedSolarsystem[] | null {
    if (!selectedMapSolarsystem.value) {
        return null;
    }

    const otherSystem = getOtherSystem(connection, specialSystem);
    const routes = specialSystem === 'Thera' ? theraRoutesByTarget.value : turnurRoutesByTarget.value;
    const routeResult = routes.get(otherSystem.id);

    if (!routeResult) {
        return null;
    }

    return routeResult.route
        .map<TResolvedSolarsystem | null>((step, index) => {
            const solarsystem = resolveSolarsystem(step.id);
            if (!solarsystem) {
                return null;
            }

            return {
                ...solarsystem,
                connection_type: routeResult.route[index + 1]?.via ?? null,
            };
        })
        .filter((entry): entry is TResolvedSolarsystem => entry !== null);
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
    const filtered = enrichedConnections.value
        .filter((conn) => conn.in_system.name === 'Thera' || conn.out_system.name === 'Thera')
        .map((connection) => ({
            ...connection,
            jumps_from_selected: resolveJumpCount(connection, 'Thera'),
            route: resolveRoute(connection, 'Thera'),
        }));

    return sortConnections(filtered, 'Thera', theraSortColumn.value, theraSortDirection.value);
});

const turnurConnections = computed(() => {
    const filtered = enrichedConnections.value
        .filter((conn) => conn.in_system.name === 'Turnur' || conn.out_system.name === 'Turnur')
        .map((connection) => ({
            ...connection,
            jumps_from_selected: resolveJumpCount(connection, 'Turnur'),
            route: resolveRoute(connection, 'Turnur'),
        }));

    return sortConnections(filtered, 'Turnur', turnurSortColumn.value, turnurSortDirection.value);
});

const theraLastUpdated = computed(() => formatTimestamp(theraConnections.value, now.value));
const turnurLastUpdated = computed(() => formatTimestamp(turnurConnections.value, now.value));

const activeTab = useLocalStorage<'thera' | 'turnur'>('eve-scout-active-tab', 'thera');
</script>

<template>
    <MapPanel>
        <MapPanelHeader>
            Eve Scout
            <span v-if="activeTab === 'thera' && theraLastUpdated" class="ml-2 text-muted-foreground/60">{{ theraLastUpdated }}</span>
            <span v-if="activeTab === 'turnur' && turnurLastUpdated" class="ml-2 text-muted-foreground/60">{{ turnurLastUpdated }}</span>
            <template #actions>
                <Tooltip v-if="activeTab === 'thera' && theraConnections.length > 0">
                    <TooltipTrigger as-child>
                        <MapPanelHeaderActionButton @click="addAllToMap('Thera')" size="icon">
                            <Plus class="size-4" />
                        </MapPanelHeaderActionButton>
                    </TooltipTrigger>
                    <TooltipContent>Add all Thera connections to map</TooltipContent>
                </Tooltip>
                <Tooltip v-if="activeTab === 'turnur' && turnurConnections.length > 0">
                    <TooltipTrigger as-child>
                        <MapPanelHeaderActionButton @click="addAllToMap('Turnur')" size="icon">
                            <Plus class="size-4" />
                        </MapPanelHeaderActionButton>
                    </TooltipTrigger>
                    <TooltipContent>Add all Turnur connections to map</TooltipContent>
                </Tooltip>
                <Tooltip>
                    <TooltipTrigger as-child>
                        <MapPanelHeaderActionButton size="icon" as-child>
                            <a href="https://www.eve-scout.com/" target="_blank" rel="noopener noreferrer">
                                <ExternalLink class="size-4" />
                            </a>
                        </MapPanelHeaderActionButton>
                    </TooltipTrigger>
                    <TooltipContent>View on EVE Scout</TooltipContent>
                </Tooltip>
            </template>
        </MapPanelHeader>
        <MapPanelContent>
            <Tabs v-model="activeTab" default-value="thera" class="flex h-full flex-col">
                <TabsList class="grid h-8 w-full shrink-0 grid-cols-2 rounded-none border-b border-border/50 bg-muted/20 p-0">
                    <TabsTrigger
                        value="thera"
                        class="h-8 rounded-none border-r border-border/30 font-mono text-[10px] tracking-wider uppercase data-[state=active]:bg-muted/30 data-[state=active]:text-foreground data-[state=active]:shadow-none"
                    >
                        Thera
                        <span v-if="theraConnections.length" class="ml-1 text-amber-400">{{ theraConnections.length }}</span>
                    </TabsTrigger>
                    <TabsTrigger
                        value="turnur"
                        class="h-8 rounded-none font-mono text-[10px] tracking-wider uppercase data-[state=active]:bg-muted/30 data-[state=active]:text-foreground data-[state=active]:shadow-none"
                    >
                        Turnur
                        <span v-if="turnurConnections.length" class="ml-1 text-amber-400">{{ turnurConnections.length }}</span>
                    </TabsTrigger>
                </TabsList>

                <TabsContent value="thera" class="mt-0 flex-1 overflow-y-auto">
                    <div class="grid grid-cols-[1.5rem_auto_auto_1.25rem_2rem_auto_auto_auto_auto] gap-x-2">
                        <div
                            class="col-span-full grid grid-cols-subgrid border-b border-border/30 bg-muted/20 px-3 py-1.5 font-mono text-[10px] tracking-wider text-muted-foreground uppercase"
                        >
                            <span></span>
                            <button @click="handleTheraSort('system')" class="flex items-center gap-1 hover:text-foreground">
                                <span>System</span>
                                <ArrowUp v-if="theraSortColumn === 'system' && theraSortDirection === 'asc'" class="size-3" />
                                <ArrowDown v-if="theraSortColumn === 'system' && theraSortDirection === 'desc'" class="size-3" />
                            </button>
                            <button @click="handleTheraSort('region')" class="flex items-center gap-1 hover:text-foreground">
                                <span>Region</span>
                                <ArrowUp v-if="theraSortColumn === 'region' && theraSortDirection === 'asc'" class="size-3" />
                                <ArrowDown v-if="theraSortColumn === 'region' && theraSortDirection === 'desc'" class="size-3" />
                            </button>
                            <span></span>
                            <button @click="handleTheraSort('jumps')" class="flex items-center justify-end gap-1 hover:text-foreground">
                                <span>J</span>
                                <ArrowUp v-if="theraSortColumn === 'jumps' && theraSortDirection === 'asc'" class="size-3" />
                                <ArrowDown v-if="theraSortColumn === 'jumps' && theraSortDirection === 'desc'" class="size-3" />
                            </button>
                            <button @click="handleTheraSort('sig_in')" class="flex items-center gap-1 hover:text-foreground">
                                <span>In</span>
                                <ArrowUp v-if="theraSortColumn === 'sig_in' && theraSortDirection === 'asc'" class="size-3" />
                                <ArrowDown v-if="theraSortColumn === 'sig_in' && theraSortDirection === 'desc'" class="size-3" />
                            </button>
                            <button @click="handleTheraSort('sig_out')" class="flex items-center gap-1 hover:text-foreground">
                                <span>Out</span>
                                <ArrowUp v-if="theraSortColumn === 'sig_out' && theraSortDirection === 'asc'" class="size-3" />
                                <ArrowDown v-if="theraSortColumn === 'sig_out' && theraSortDirection === 'desc'" class="size-3" />
                            </button>
                            <span>WH</span>
                            <button @click="handleTheraSort('time')" class="flex items-center gap-1 hover:text-foreground">
                                <span>TTL</span>
                                <ArrowUp v-if="theraSortColumn === 'time' && theraSortDirection === 'asc'" class="size-3" />
                                <ArrowDown v-if="theraSortColumn === 'time' && theraSortDirection === 'desc'" class="size-3" />
                            </button>
                        </div>
                        <EveScoutConnection
                            v-for="connection in theraConnections"
                            :key="`${connection.in_signature}-${connection.out_signature}`"
                            :connection="connection"
                            special-system="Thera"
                        />
                        <div v-if="!theraConnections.length" class="col-span-full flex h-full flex-col items-center justify-center gap-2 p-4">
                            <p class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">No Thera connections</p>
                        </div>
                    </div>
                </TabsContent>

                <TabsContent value="turnur" class="mt-0 flex-1 overflow-y-auto">
                    <div class="grid grid-cols-[1.5rem_auto_auto_1.25rem_2rem_auto_auto_auto_auto] gap-x-2">
                        <div
                            class="col-span-full grid grid-cols-subgrid border-b border-border/30 bg-muted/20 px-3 py-1.5 font-mono text-[10px] tracking-wider text-muted-foreground uppercase"
                        >
                            <span></span>
                            <button @click="handleTurnurSort('system')" class="flex items-center gap-1 hover:text-foreground">
                                <span>System</span>
                                <ArrowUp v-if="turnurSortColumn === 'system' && turnurSortDirection === 'asc'" class="size-3" />
                                <ArrowDown v-if="turnurSortColumn === 'system' && turnurSortDirection === 'desc'" class="size-3" />
                            </button>
                            <button @click="handleTurnurSort('region')" class="flex items-center gap-1 hover:text-foreground">
                                <span>Region</span>
                                <ArrowUp v-if="turnurSortColumn === 'region' && turnurSortDirection === 'asc'" class="size-3" />
                                <ArrowDown v-if="turnurSortColumn === 'region' && turnurSortDirection === 'desc'" class="size-3" />
                            </button>
                            <span></span>
                            <button @click="handleTurnurSort('jumps')" class="flex items-center justify-end gap-1 hover:text-foreground">
                                <span>J</span>
                                <ArrowUp v-if="turnurSortColumn === 'jumps' && turnurSortDirection === 'asc'" class="size-3" />
                                <ArrowDown v-if="turnurSortColumn === 'jumps' && turnurSortDirection === 'desc'" class="size-3" />
                            </button>
                            <button @click="handleTurnurSort('sig_in')" class="flex items-center gap-1 hover:text-foreground">
                                <span>In</span>
                                <ArrowUp v-if="turnurSortColumn === 'sig_in' && turnurSortDirection === 'asc'" class="size-3" />
                                <ArrowDown v-if="turnurSortColumn === 'sig_in' && turnurSortDirection === 'desc'" class="size-3" />
                            </button>
                            <button @click="handleTurnurSort('sig_out')" class="flex items-center gap-1 hover:text-foreground">
                                <span>Out</span>
                                <ArrowUp v-if="turnurSortColumn === 'sig_out' && turnurSortDirection === 'asc'" class="size-3" />
                                <ArrowDown v-if="turnurSortColumn === 'sig_out' && turnurSortDirection === 'desc'" class="size-3" />
                            </button>
                            <span>WH</span>
                            <button @click="handleTurnurSort('time')" class="flex items-center gap-1 hover:text-foreground">
                                <span>TTL</span>
                                <ArrowUp v-if="turnurSortColumn === 'time' && turnurSortDirection === 'asc'" class="size-3" />
                                <ArrowDown v-if="turnurSortColumn === 'time' && turnurSortDirection === 'desc'" class="size-3" />
                            </button>
                        </div>
                        <EveScoutConnection
                            v-for="connection in turnurConnections"
                            :key="`${connection.in_signature}-${connection.out_signature}`"
                            :connection="connection"
                            special-system="Turnur"
                        />
                        <div v-if="!turnurConnections.length" class="col-span-full flex h-full flex-col items-center justify-center gap-2 p-4">
                            <p class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">No Turnur connections</p>
                        </div>
                    </div>
                </TabsContent>
            </Tabs>
        </MapPanelContent>
    </MapPanel>
</template>

<script setup lang="ts">
import Skyhook from '@/components/map-skyhooks/Skyhook.vue';
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
import MapPanelHeader from '@/components/ui/map-panel/MapPanelHeader.vue';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useJumpCounts } from '@/composables/useJumpCounts';
import { useMap } from '@/composables/useMap';
import { useSelectedMapSolarsystem } from '@/composables/useSelectedMapSolarsystem';
import { useShowMap } from '@/composables/useShowMap';
import { useStaticSolarsystems } from '@/composables/useStaticSolarsystems';
import type { TRaidableSkyhook, TResolvedSolarsystem } from '@/pages/maps';
import { Deferred, usePoll } from '@inertiajs/vue3';
import { useLocalStorage, useNow } from '@vueuse/core';
import { ArrowDown, ArrowUp } from 'lucide-vue-next';
import { computed } from 'vue';

const { map_skyhooks } = defineProps<{
    map_skyhooks?: TRaidableSkyhook[];
}>();

const five_minutes_in_ms = 5 * 60 * 1000;
usePoll(five_minutes_in_ms, { only: ['map_skyhooks'] });

const map = useMap();
const page = useShowMap();
const selectedMapSolarsystem = useSelectedMapSolarsystem();
const { resolveSolarsystem } = useStaticSolarsystems();

const fromId = computed(() => selectedMapSolarsystem.value?.solarsystem_id ?? null);
const targetIds = computed(() => {
    if (!map_skyhooks) return [] as number[];
    return [...new Set(map_skyhooks.map((s) => s.solarsystem_id))];
});

const { jumpsByTarget, routesByTarget } = useJumpCounts({
    fromId,
    targets: targetIds,
    mapConnections: computed(() => map.value.map_connections ?? []),
    mapSolarsystems: computed(() => map.value.map_solarsystems ?? []),
    ignoredSystems: computed(() => page.props.ignored_systems ?? []),
    includeEveScout: true,
});

function resolveRoute(solarsystemId: number): TResolvedSolarsystem[] | null {
    const routeResult = routesByTarget.value.get(solarsystemId);
    if (!routeResult) return null;
    return routeResult.route
        .map<TResolvedSolarsystem | null>((step, index) => {
            const solarsystem = resolveSolarsystem(step.id);
            if (!solarsystem) return null;
            return {
                ...solarsystem,
                connection_type: routeResult.route[index + 1]?.via ?? null,
            };
        })
        .filter((entry): entry is TResolvedSolarsystem => entry !== null);
}

type SkyhookSortColumn = 'jumps' | 'system' | 'region' | 'status';
type SkyhookSortDirection = 'asc' | 'desc';
type SkyhookTab = 'lava' | 'ice';
type SkyhookStatusFilter = 'upcoming' | 'active' | 'ending';

const storedTab = useLocalStorage<string>('skyhooks-active-tab', 'lava');
const activeTab = computed<SkyhookTab>({
    get: () => (storedTab.value === 'ice' ? 'ice' : 'lava'),
    set: (value) => {
        storedTab.value = value;
    },
});
const sortColumn = useLocalStorage<SkyhookSortColumn>('skyhooks-sort-column', 'jumps');
const sortDirection = useLocalStorage<SkyhookSortDirection>('skyhooks-sort-direction', 'asc');
const activeStatuses = useLocalStorage<SkyhookStatusFilter[]>('skyhooks-status-filters', ['upcoming', 'active', 'ending']);

const fifteen_minutes_in_ms = 15 * 60 * 1000;

const now = useNow({ interval: 30_000 });

type DecoratedSkyhook = TRaidableSkyhook & {
    jumps: number | null;
    route: TResolvedSolarsystem[] | null;
    status: 'raidable' | 'upcoming' | 'ended';
    statusTime: number;
};

const decorated = computed<DecoratedSkyhook[]>(() => {
    if (!map_skyhooks) return [];

    return map_skyhooks.map((skyhook) => {
        const jumps = fromId.value === skyhook.solarsystem_id ? 0 : (jumpsByTarget.value.get(skyhook.solarsystem_id) ?? null);
        const route = resolveRoute(skyhook.solarsystem_id);
        const start = new Date(skyhook.theft_vulnerability_start);
        const end = new Date(skyhook.theft_vulnerability_end);
        let status: 'raidable' | 'upcoming' | 'ended';
        let statusTime: number;
        if (now.value >= end) {
            status = 'ended';
            statusTime = now.value.getTime() - end.getTime();
        } else if (now.value >= start) {
            status = 'raidable';
            statusTime = end.getTime() - now.value.getTime();
        } else {
            status = 'upcoming';
            statusTime = start.getTime() - now.value.getTime();
        }
        return { ...skyhook, jumps, route, status, statusTime };
    });
});

const visibleSkyhooks = computed(() =>
    decorated.value.filter((s) => {
        if (s.status === 'ended') return false;
        if (s.status === 'upcoming') return activeStatuses.value.includes('upcoming');
        const ending = s.statusTime < fifteen_minutes_in_ms;
        return ending ? activeStatuses.value.includes('ending') : activeStatuses.value.includes('active');
    }),
);

const tabCounts = computed(() => ({
    lava: visibleSkyhooks.value.filter((s) => s.planet_type === 'lava').length,
    ice: visibleSkyhooks.value.filter((s) => s.planet_type === 'ice').length,
}));

const filteredByTab = computed(() => {
    if (activeTab.value === 'ice') return visibleSkyhooks.value.filter((s) => s.planet_type === 'ice');
    return visibleSkyhooks.value.filter((s) => s.planet_type === 'lava');
});

function compareJumps(a: DecoratedSkyhook, b: DecoratedSkyhook): number {
    if (a.jumps === null && b.jumps === null) return 0;
    if (a.jumps === null) return 1;
    if (b.jumps === null) return -1;
    return a.jumps - b.jumps;
}

function compareSystem(a: DecoratedSkyhook, b: DecoratedSkyhook): number {
    const aName = a.planet_name ?? String(a.planet_id);
    const bName = b.planet_name ?? String(b.planet_id);
    return aName.localeCompare(bName);
}

function compareRegion(a: DecoratedSkyhook, b: DecoratedSkyhook): number {
    const aRegion = resolveSolarsystem(a.solarsystem_id).region?.name ?? '';
    const bRegion = resolveSolarsystem(b.solarsystem_id).region?.name ?? '';
    return aRegion.localeCompare(bRegion);
}

function compareStatus(a: DecoratedSkyhook, b: DecoratedSkyhook): number {
    if (a.status !== b.status) return a.status === 'raidable' ? -1 : 1;
    return a.statusTime - b.statusTime;
}

const sortedSkyhooks = computed(() => {
    const list = [...filteredByTab.value];
    const direction = sortDirection.value === 'asc' ? 1 : -1;

    return list.sort((a, b) => {
        let cmp = 0;
        switch (sortColumn.value) {
            case 'jumps':
                cmp = compareJumps(a, b);
                break;
            case 'system':
                cmp = compareSystem(a, b);
                break;
            case 'region':
                cmp = compareRegion(a, b);
                break;
            case 'status':
                cmp = compareStatus(a, b);
                break;
        }
        if (cmp !== 0) return direction * cmp;
        return new Date(a.theft_vulnerability_start).getTime() - new Date(b.theft_vulnerability_start).getTime();
    });
});

function handleSort(column: SkyhookSortColumn) {
    if (sortColumn.value === column) {
        sortDirection.value = sortDirection.value === 'asc' ? 'desc' : 'asc';
    } else {
        sortColumn.value = column;
        sortDirection.value = 'asc';
    }
}
</script>

<template>
    <MapPanel>
        <MapPanelHeader card-id="skyhooks">
            Raidable Skyhooks
            <span v-if="visibleSkyhooks.length" class="ml-1 text-amber-400">{{ visibleSkyhooks.length }}</span>
            <template #actions>
                <ToggleGroup v-model="activeStatuses" type="multiple" size="sm" variant="outline">
                    <Tooltip>
                        <TooltipTrigger as-child>
                            <ToggleGroupItem value="upcoming" aria-label="Show upcoming">
                                <span class="size-2 rounded-full bg-amber-400" />
                            </ToggleGroupItem>
                        </TooltipTrigger>
                        <TooltipContent>Upcoming</TooltipContent>
                    </Tooltip>
                    <Tooltip>
                        <TooltipTrigger as-child>
                            <ToggleGroupItem value="active" aria-label="Show active">
                                <span class="size-2 rounded-full bg-emerald-400" />
                            </ToggleGroupItem>
                        </TooltipTrigger>
                        <TooltipContent>Currently raidable</TooltipContent>
                    </Tooltip>
                    <Tooltip>
                        <TooltipTrigger as-child>
                            <ToggleGroupItem value="ending" aria-label="Show about to end">
                                <span class="size-2 rounded-full bg-red-400" />
                            </ToggleGroupItem>
                        </TooltipTrigger>
                        <TooltipContent>Ending soon (&lt; 15m)</TooltipContent>
                    </Tooltip>
                </ToggleGroup>
            </template>
        </MapPanelHeader>
        <MapPanelContent>
            <Tabs v-model="activeTab" default-value="lava" class="flex h-full flex-col">
                <TabsList class="grid h-8 w-full shrink-0 grid-cols-2 rounded-none border-b border-border/50 bg-muted/20 p-0">
                    <TabsTrigger
                        value="lava"
                        class="h-8 rounded-none border-r border-border/30 font-mono text-[10px] tracking-wider uppercase data-[state=active]:bg-muted/30 data-[state=active]:text-foreground data-[state=active]:shadow-none"
                    >
                        Lava
                        <span v-if="tabCounts.lava" class="ml-1 text-amber-400">{{ tabCounts.lava }}</span>
                    </TabsTrigger>
                    <TabsTrigger
                        value="ice"
                        class="h-8 rounded-none font-mono text-[10px] tracking-wider uppercase data-[state=active]:bg-muted/30 data-[state=active]:text-foreground data-[state=active]:shadow-none"
                    >
                        Ice
                        <span v-if="tabCounts.ice" class="ml-1 text-amber-400">{{ tabCounts.ice }}</span>
                    </TabsTrigger>
                </TabsList>
                <div class="flex-1 overflow-y-auto">
                    <Deferred data="map_skyhooks">
                        <template v-if="sortedSkyhooks.length">
                            <div class="grid grid-cols-[1rem_auto_auto_1rem_2rem_auto] gap-x-2">
                                <div
                                    class="col-span-full grid grid-cols-subgrid border-b border-border/30 bg-muted/20 px-3 py-1.5 font-mono text-[10px] tracking-wider text-muted-foreground uppercase"
                                >
                                    <span></span>
                                    <button @click="handleSort('system')" class="flex items-center gap-1 hover:text-foreground">
                                        <span>System</span>
                                        <ArrowUp v-if="sortColumn === 'system' && sortDirection === 'asc'" class="size-3" />
                                        <ArrowDown v-if="sortColumn === 'system' && sortDirection === 'desc'" class="size-3" />
                                    </button>
                                    <button @click="handleSort('region')" class="flex items-center gap-1 hover:text-foreground">
                                        <span>Region</span>
                                        <ArrowUp v-if="sortColumn === 'region' && sortDirection === 'asc'" class="size-3" />
                                        <ArrowDown v-if="sortColumn === 'region' && sortDirection === 'desc'" class="size-3" />
                                    </button>
                                    <span></span>
                                    <button @click="handleSort('jumps')" class="flex items-center justify-end gap-1 hover:text-foreground">
                                        <span>J</span>
                                        <ArrowUp v-if="sortColumn === 'jumps' && sortDirection === 'asc'" class="size-3" />
                                        <ArrowDown v-if="sortColumn === 'jumps' && sortDirection === 'desc'" class="size-3" />
                                    </button>
                                    <button @click="handleSort('status')" class="flex items-center justify-end gap-1 hover:text-foreground">
                                        <span>Timer</span>
                                        <ArrowUp v-if="sortColumn === 'status' && sortDirection === 'asc'" class="size-3" />
                                        <ArrowDown v-if="sortColumn === 'status' && sortDirection === 'desc'" class="size-3" />
                                    </button>
                                </div>
                                <TransitionGroup name="list">
                                    <Skyhook
                                        v-for="skyhook in sortedSkyhooks"
                                        :key="skyhook.planet_id"
                                        :skyhook="skyhook"
                                        :jumps="skyhook.jumps"
                                        :route="skyhook.route"
                                    />
                                </TransitionGroup>
                            </div>
                        </template>
                        <div v-else class="flex h-full flex-col items-center justify-center gap-2 p-4">
                            <p class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">No raidable skyhooks</p>
                        </div>
                        <template #fallback>
                            <div class="flex h-full animate-pulse items-center justify-center gap-2 p-4">
                                <p class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">Loading skyhooks...</p>
                            </div>
                        </template>
                    </Deferred>
                </div>
            </Tabs>
        </MapPanelContent>
    </MapPanel>
</template>

<style scoped>
.list-move,
.list-enter-active,
.list-leave-active {
    transition: all 0.3s ease;
}

.list-enter-from,
.list-leave-to {
    opacity: 0;
    transform: translateX(20px);
}

.list-leave-active {
    position: absolute;
    opacity: 0;
    transition-duration: 0ms;
}
</style>

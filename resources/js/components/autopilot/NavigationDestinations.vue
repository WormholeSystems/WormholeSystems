<script setup lang="ts">
import MapRouteSolarsystem from '@/components/autopilot/MapRouteSolarsystem.vue';
import { useDestinationRoutes } from '@/composables/useDestinationRoutes';
import useHasWritePermission from '@/composables/useHasWritePermission';
import { useMap } from '@/composables/useMap';
import { useSelectedMapSolarsystem } from '@/composables/useSelectedMapSolarsystem';
import { useStaticSolarsystems } from '@/composables/useStaticSolarsystems';
import type { TResolvedMapRouteSolarsystem } from '@/pages/maps';
import type { WorkerRouteStep } from '@/routing/types';
import { useLocalStorage } from '@vueuse/core';
import { ArrowDown, ArrowUp } from 'lucide-vue-next';
import { computed } from 'vue';

const { destinations, ignored_systems } = defineProps<{
    destinations: TResolvedMapRouteSolarsystem[];
    ignored_systems: number[];
}>();

const map = useMap();
const selected = useSelectedMapSolarsystem();
const { resolveSolarsystem } = useStaticSolarsystems();

const { routesByDestination } = useDestinationRoutes({
    fromId: computed(() => selected.value?.solarsystem_id ?? null),
    destinations: computed(() => destinations),
    mapConnections: computed(() => map.value.map_connections ?? []),
    mapSolarsystems: computed(() => map.value.map_solarsystems ?? []),
    ignoredSystems: computed(() => ignored_systems),
});

const resolvedDestinations = computed(() =>
    destinations.map((destination) => {
        const routeResult = routesByDestination.value.get(destination.id);
        const route = routeResult?.route?.map((step: WorkerRouteStep) => resolveSolarsystem(step.id)) ?? [];

        return {
            ...destination,
            solarsystem: destination.solarsystem ?? resolveSolarsystem(destination.solarsystem_id),
            route,
        } satisfies TResolvedMapRouteSolarsystem;
    }),
);

type SortColumn = 'system' | 'jumps' | 'region';
type SortDirection = 'asc' | 'desc';

const sortColumn = useLocalStorage<SortColumn>('navigation-destinations-sort-column', 'system');
const sortDirection = useLocalStorage<SortDirection>('navigation-destinations-sort-direction', 'asc');

function handleSort(column: SortColumn) {
    if (sortColumn.value === column) {
        sortDirection.value = sortDirection.value === 'asc' ? 'desc' : 'asc';
    } else {
        sortColumn.value = column;
        sortDirection.value = 'asc';
    }
}

const sorted = computed(() => {
    return resolvedDestinations.value.toSorted((a, b) => {
        let comparison = 0;

        switch (sortColumn.value) {
            case 'system': {
                const aIsWH = a.solarsystem.class !== null;
                const bIsWH = b.solarsystem.class !== null;

                if (aIsWH && bIsWH) {
                    comparison = (a.solarsystem.class || 0) - (b.solarsystem.class || 0);
                } else if (!aIsWH && !bIsWH) {
                    comparison = (b.solarsystem.security || 0) - (a.solarsystem.security || 0);
                } else {
                    comparison = aIsWH ? 1 : -1;
                }

                if (comparison === 0) {
                    comparison = a.solarsystem.name.localeCompare(b.solarsystem.name);
                }
                break;
            }
            case 'jumps': {
                const aJumps = a.route ? a.route.length - 1 : 999;
                const bJumps = b.route ? b.route.length - 1 : 999;
                comparison = aJumps - bJumps;
                break;
            }
            case 'region':
                comparison = (a.solarsystem.region?.name || '').localeCompare(b.solarsystem.region?.name || '');
                break;
        }

        return sortDirection.value === 'asc' ? comparison : -comparison;
    });
});

const can_write = useHasWritePermission();
</script>

<template>
    <div
        :class="can_write ? 'grid-cols-[auto_1fr_auto_1fr_auto_auto]' : 'grid-cols-[auto_1fr_auto_1fr_auto]'"
        class="grid gap-x-4 overflow-hidden rounded border bg-white text-xs dark:bg-neutral-900/40"
    >
        <div class="col-span-full grid grid-cols-subgrid border-b bg-muted/50 px-2 py-1.5 text-xs font-medium text-muted-foreground">
            <div></div>
            <button @click="handleSort('system')" class="flex items-center gap-1 hover:text-foreground">
                <span>System</span>
                <ArrowUp v-if="sortColumn === 'system' && sortDirection === 'asc'" class="size-3" />
                <ArrowDown v-if="sortColumn === 'system' && sortDirection === 'desc'" class="size-3" />
            </button>
            <button @click="handleSort('jumps')" class="flex items-center justify-center gap-1 hover:text-foreground">
                <span>Jumps</span>
                <ArrowUp v-if="sortColumn === 'jumps' && sortDirection === 'asc'" class="size-3" />
                <ArrowDown v-if="sortColumn === 'jumps' && sortDirection === 'desc'" class="size-3" />
            </button>
            <button @click="handleSort('region')" class="flex items-center gap-1 hover:text-foreground">
                <span>Region</span>
                <ArrowUp v-if="sortColumn === 'region' && sortDirection === 'asc'" class="size-3" />
                <ArrowDown v-if="sortColumn === 'region' && sortDirection === 'desc'" class="size-3" />
            </button>
        </div>

        <MapRouteSolarsystem v-for="route in sorted" :key="route.solarsystem.id" :map_route="route" />

        <div v-if="!sorted?.length" class="col-span-full py-4 text-center text-muted-foreground">
            <div class="mb-1 text-sm">🎯</div>
            <div>No destinations</div>
        </div>
    </div>
</template>

<style scoped></style>

<script setup lang="ts">
import MapRouteSolarsystem from '@/components/autopilot/MapRouteSolarsystem.vue';
import useHasWritePermission from '@/composables/useHasWritePermission';
import { TMapRouteSolarsystem } from '@/types/models';
import { useLocalStorage } from '@vueuse/core';
import { ArrowDown, ArrowUp } from 'lucide-vue-next';
import { computed } from 'vue';

const { destinations } = defineProps<{
    destinations: TMapRouteSolarsystem[];
}>();

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
    return destinations.toSorted((a, b) => {
        let comparison = 0;

        switch (sortColumn.value) {
            case 'system':
                // First sort by security status/class
                const aIsWH = a.solarsystem.class !== null;
                const bIsWH = b.solarsystem.class !== null;

                if (aIsWH && bIsWH) {
                    // Both are wormholes, sort by class
                    comparison = (a.solarsystem.class || 0) - (b.solarsystem.class || 0);
                } else if (!aIsWH && !bIsWH) {
                    // Both are k-space, sort by security
                    comparison = (b.solarsystem.security || 0) - (a.solarsystem.security || 0);
                } else {
                    // One is WH, one is k-space. K-space first
                    comparison = aIsWH ? 1 : -1;
                }

                // If security/class is same, sort by name
                if (comparison === 0) {
                    comparison = a.solarsystem.name.localeCompare(b.solarsystem.name);
                }
                break;
            case 'jumps':
                const aJumps = a.route ? a.route.length - 1 : 999;
                const bJumps = b.route ? b.route.length - 1 : 999;
                comparison = aJumps - bJumps;
                break;
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

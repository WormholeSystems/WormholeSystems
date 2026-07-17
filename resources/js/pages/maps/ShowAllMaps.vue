<script setup lang="ts">
import Logo from '@/components/icons/Logo.vue';
import PlusIcon from '@/components/icons/PlusIcon.vue';
import CreateMapDialog from '@/components/map/CreateMapDialog.vue';
import MapCard from '@/components/map/MapCard.vue';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { useSearch } from '@/composables/useSearch';
import AppLayout from '@/layouts/AppLayout.vue';
import SeoHead from '@/layouts/SeoHead.vue';
import { TMapSummary } from '@/pages/maps/index';
import { Archive, ChevronDown, SearchIcon } from 'lucide-vue-next';
import { computed, ref } from 'vue';

const { maps } = defineProps<{
    maps: TMapSummary[];
}>();

const search = useSearch('search', ['maps']);
const showArchived = ref(false);

const activeMaps = computed(() => maps.filter((map) => !map.map_user_setting?.is_archived));
const archivedMaps = computed(() => maps.filter((map) => Boolean(map.map_user_setting?.is_archived)));

const totals = computed(() => ({
    maps: activeMaps.value.length,
    systems: activeMaps.value.reduce((sum, map) => sum + map.map_solarsystems_count, 0),
    connections: activeMaps.value.reduce((sum, map) => sum + map.map_connections_count, 0),
}));

const stats = computed(() => [
    { label: 'Maps', value: totals.value.maps },
    { label: 'Systems', value: totals.value.systems },
    { label: 'Connections', value: totals.value.connections },
]);
</script>

<template>
    <AppLayout>
        <SeoHead
            title="Maps"
            description="Manage and explore your wormhole mapping networks. Create, edit, and navigate through dangerous wormhole space with collaborative mapping tools."
            keywords="wormhole maps, eve online mapping, wormhole navigation, space exploration"
        />
        <div class="p-8">
            <!-- Header -->
            <div class="flex flex-wrap items-end justify-between gap-4">
                <div>
                    <h1 class="font-display text-3xl font-bold tracking-tight text-foreground">Maps</h1>
                    <p class="mt-2 text-muted-foreground">Manage and explore your wormhole mapping networks</p>
                </div>
                <CreateMapDialog>
                    <Button class="flex items-center gap-2">
                        <PlusIcon class="h-4 w-4" />
                        Create New Map
                    </Button>
                </CreateMapDialog>
            </div>

            <!-- Summary -->
            <div v-if="maps.length > 0" class="mt-8 grid grid-cols-3 divide-x divide-border/50 overflow-hidden rounded bg-card ring-1 ring-border">
                <div v-for="stat in stats" :key="stat.label" class="px-5 py-4">
                    <div class="font-display text-2xl font-bold tracking-tight tabular-nums">{{ stat.value.toLocaleString() }}</div>
                    <div class="mt-1 font-mono text-[10px] tracking-wider text-muted-foreground uppercase">{{ stat.label }}</div>
                </div>
            </div>

            <!-- Controls -->
            <div v-if="maps.length > 0" class="mt-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                <div class="relative max-w-md flex-1">
                    <SearchIcon class="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                    <Input v-model="search" placeholder="Search maps..." class="pl-9" />
                </div>
                <Button v-if="archivedMaps.length > 0" variant="outline" size="sm" class="gap-2" @click="showArchived = !showArchived">
                    <Archive class="h-4 w-4" />
                    {{ showArchived ? 'Hide archived' : `Show archived (${archivedMaps.length})` }}
                    <ChevronDown class="h-4 w-4 transition-transform" :class="{ 'rotate-180': showArchived }" />
                </Button>
            </div>

            <!-- Active maps -->
            <div v-if="activeMaps.length > 0" class="mt-6 grid grid-cols-[repeat(auto-fill,minmax(20rem,1fr))] gap-4">
                <MapCard v-for="map in activeMaps" :key="map.id" :map="map" />
            </div>

            <!-- All active maps archived -->
            <div v-else-if="maps.length > 0" class="mt-10 grid justify-items-center gap-3 text-center">
                <div class="grid size-16 place-items-center rounded-full bg-muted">
                    <Archive class="h-7 w-7 text-muted-foreground" />
                </div>
                <h3 class="text-lg font-semibold">No active maps</h3>
                <p class="max-w-md text-sm text-muted-foreground">
                    {{ search ? `No active maps match "${search}".` : 'Every map you can access is archived.' }}
                </p>
                <Button v-if="archivedMaps.length > 0 && !showArchived" variant="outline" size="sm" class="gap-2" @click="showArchived = true">
                    <Archive class="h-4 w-4" />
                    Show archived ({{ archivedMaps.length }})
                </Button>
            </div>

            <!-- Archived maps -->
            <div v-if="showArchived && archivedMaps.length > 0" class="mt-10">
                <div class="flex items-center gap-3">
                    <h2 class="font-mono text-[10px] tracking-wider text-muted-foreground uppercase">Archived · {{ archivedMaps.length }}</h2>
                    <span class="h-px flex-1 bg-border/50" />
                </div>
                <div class="mt-5 grid grid-cols-[repeat(auto-fill,minmax(20rem,1fr))] gap-4">
                    <MapCard v-for="map in archivedMaps" :key="map.id" :map="map" />
                </div>
            </div>

            <!-- Empty state (no maps at all) -->
            <div v-if="maps.length === 0" class="mt-16 grid justify-items-center gap-4">
                <div class="grid size-24 place-items-center rounded-full bg-neutral-200 p-4 dark:bg-neutral-800" v-if="search.length">
                    <SearchIcon class="h-8 w-8 text-muted-foreground" />
                </div>
                <Logo class="h-16 w-16 text-muted-foreground" v-else />
                <h3 class="text-xl font-semibold">
                    {{ search ? 'No maps found' : 'No maps yet' }}
                </h3>
                <p class="max-w-md text-center text-muted-foreground">
                    <template v-if="search">
                        No maps match your search for "{{ search }}". Try adjusting your search terms or create a new map.
                    </template>
                    <template v-else> Get started by creating your first wormhole map to begin tracking connections and systems. </template>
                </p>
                <div class="flex gap-2">
                    <Button v-if="search" variant="outline" @click="search = ''">Clear Search</Button>
                    <CreateMapDialog>
                        <Button class="flex items-center gap-2">
                            <PlusIcon class="h-4 w-4" />
                            {{ search ? 'Create New Map' : 'Create Your First Map' }}
                        </Button>
                    </CreateMapDialog>
                </div>
            </div>
        </div>
    </AppLayout>
</template>

<script setup lang="ts">
import MapConnections from '@/components/MapConnections.vue';
import MapSolarsystem from '@/components/MapSolarsystem.vue';
import { Combobox, ComboboxAnchor, ComboboxEmpty, ComboboxGroup, ComboboxInput, ComboboxItem, ComboboxList } from '@/components/ui/combobox';
import { useMap as useNewMap, useMapSolarsystems } from '@/composables/map';
import AppLayout from '@/layouts/AppLayout.vue';
import { TMap, TSolarsystem } from '@/types/models';
import { Head, Link, router } from '@inertiajs/vue3';
import { useDebounceFn, useMagicKeys, whenever } from '@vueuse/core';
import { computed, ref, useTemplateRef } from 'vue';

const { map, solarsystems, search } = defineProps<{
    map: TMap;
    search: string;
    solarsystems: TSolarsystem[];
}>();

const container = useTemplateRef('map-container');

useNewMap(
    () => map,
    () => container.value!,
);

const { map_solarsystems, map_solarsystems_selected } = useMapSolarsystems();

const { Delete } = useMagicKeys();

whenever(Delete, () => {
    if (!map_solarsystems_selected.value.length) return;

    router.delete(route('map-selection.destroy'), {
        data: {
            map_solarsystem_ids: map_solarsystems_selected.value.map((map_solarsystem) => map_solarsystem.id),
        },
    });
});

const debounced = useDebounceFn(handleSearch, 200, {
    maxWait() {
        return 100;
    },
});

const new_solarsystems = computed(() => {
    return solarsystems.filter((solarsystem) => {
        return !map.map_solarsystems.some((map_solarsystem) => map_solarsystem.solarsystem_id === solarsystem.id);
    });
});

const existing_solarsystems = computed(() => {
    return solarsystems.filter((solarsystem) => {
        return map.map_solarsystems.some((map_solarsystem) => map_solarsystem.solarsystem_id === solarsystem.id);
    });
});

function handleSearch() {
    router.reload({
        data: {
            search: search_input.value,
        },
    });
}

const search_input = ref(search);

function handleSolarsystemSelect(solarsystem: TSolarsystem) {
    router.post(route('map-solarsystems.store'), {
        map_id: map.id,
        solarsystem_id: solarsystem.id,
        position_x: 200,
        position_y: 200,
    });
}
</script>

<template>
    <AppLayout>
        <Head title="ShowMap" />

        <div class="p-8">
            <div class="relative">
                <div class="relative h-400 w-full overflow-scroll rounded-lg border bg-neutral-900/50">
                    <div class="relative grid h-400 w-1000" @dragover.prevent ref="map-container">
                        <MapConnections />
                        <MapSolarsystem v-for="solarsystem in map_solarsystems" :key="solarsystem.id" :map_solarsystem="solarsystem" />
                    </div>
                </div>
                <Combobox class="absolute top-4 left-1/2 -translate-x-1/2 rounded-lg border">
                    <ComboboxAnchor>
                        <ComboboxInput v-model:model-value="search_input" @update:modelValue="debounced" />
                    </ComboboxAnchor>
                    <ComboboxList>
                        <ComboboxEmpty> No results found</ComboboxEmpty>
                        <ComboboxGroup heading="Search Results" v-if="new_solarsystems.length > 0">
                            <ComboboxItem
                                v-for="solarsystem in new_solarsystems"
                                :key="solarsystem.id"
                                :value="solarsystem.name"
                                @select="() => handleSolarsystemSelect(solarsystem)"
                            />
                        </ComboboxGroup>
                        <ComboboxGroup heading="Already in Map" v-if="existing_solarsystems.length > 0">
                            <ComboboxItem
                                v-for="solarsystem in existing_solarsystems"
                                :key="solarsystem.id"
                                :value="solarsystem.name"
                                @select="() => handleSolarsystemSelect(solarsystem)"
                            />
                        </ComboboxGroup>
                    </ComboboxList>
                </Combobox>
            </div>
        </div>
        <ul>
            <li :key="solarsystem.id" v-for="solarsystem in map.map_solarsystems">
                <span>{{ solarsystem.name }}</span>
                <Link :href="route('map-solarsystems.destroy', solarsystem.id)" method="delete" as="button"> Delete </Link>
            </li>
        </ul>
        <ul>
            <li v-for="map_connection in map.map_connections" :key="map_connection.id">
                <span>{{ map_connection.from_map_solarsystem_id }} {{ '<-->' }} {{ map_connection.to_map_solarsystem_id }}</span>
            </li>
        </ul>
    </AppLayout>
</template>
<style>
::-webkit-scrollbar {
    width: 8px;
    height: 8px;
}

::-webkit-scrollbar-thumb {
    background-color: var(--color-neutral-700);
    border-radius: 4px;
}

::-webkit-scrollbar-track {
    background-color: transparent;
}

::-webkit-scrollbar-corner {
    background-color: transparent;
}

::-webkit-scrollbar-button {
    display: none;
}
</style>

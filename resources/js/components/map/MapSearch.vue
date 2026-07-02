<script setup lang="ts">
import SolarsystemSearchResult from '@/components/solarsystem/SolarsystemSearchResult.vue';
import { Combobox, ComboboxAnchor, ComboboxEmpty, ComboboxGroup, ComboboxInput, ComboboxItem, ComboboxList } from '@/components/ui/combobox';
import { createMapSolarsystem, useSolarsystemSearch } from '@/map/api';
import type { TMap } from '@/pages/maps';
import type { TStaticSolarsystem } from '@/types/static-data';
import { ref } from 'vue';

const { map } = defineProps<{
    map: TMap;
}>();

const search = ref('');
const { new_solarsystems, existing_solarsystems, getAlias } = useSolarsystemSearch(search, () => map.map_solarsystems);

function handleSolarsystemSelect(solarsystem: TStaticSolarsystem) {
    createMapSolarsystem(solarsystem.id);
}
</script>

<template>
    <Combobox class="w-full max-w-sm rounded border border-border/50 bg-background">
        <ComboboxAnchor>
            <ComboboxInput v-model="search" class="w-full max-w-none" />
        </ComboboxAnchor>
        <ComboboxList>
            <ComboboxEmpty> No results found</ComboboxEmpty>
            <ComboboxGroup
                heading="Search Results"
                v-if="new_solarsystems.length > 0"
                class="grid grid-cols-[auto_1fr_1fr_auto] items-center gap-x-2"
            >
                <ComboboxItem
                    v-for="solarsystem in new_solarsystems"
                    :key="solarsystem.id"
                    :value="solarsystem.name"
                    @select.prevent="() => handleSolarsystemSelect(solarsystem)"
                    class="col-span-full grid grid-cols-subgrid"
                >
                    <SolarsystemSearchResult :solarsystem="solarsystem" :alias="getAlias(solarsystem.id)" />
                </ComboboxItem>
            </ComboboxGroup>
            <ComboboxGroup
                heading="Already in Map"
                v-if="existing_solarsystems.length > 0"
                class="grid grid-cols-[auto_1fr_1fr_auto] items-center gap-x-2"
            >
                <ComboboxItem
                    v-for="solarsystem in existing_solarsystems"
                    :key="solarsystem.id"
                    :value="solarsystem.name"
                    disabled
                    class="col-span-full grid grid-cols-subgrid"
                >
                    <SolarsystemSearchResult :solarsystem="solarsystem" :alias="getAlias(solarsystem.id)" />
                </ComboboxItem>
            </ComboboxGroup>
        </ComboboxList>
    </Combobox>
</template>

<style scoped></style>

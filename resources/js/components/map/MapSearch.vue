<script setup lang="ts">
import SolarsystemEffect from '@/components/map/SolarsystemEffect.vue';
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import { Combobox, ComboboxAnchor, ComboboxEmpty, ComboboxGroup, ComboboxInput, ComboboxItem, ComboboxList } from '@/components/ui/combobox';
import { createMapSolarsystem } from '@/composables/map';
import { useSearch } from '@/composables/useSearch';
import { TMap, TSolarsystem } from '@/types/models';
import { computed } from 'vue';

const { map, solarsystems } = defineProps<{
    map: TMap;
    solarsystems: TSolarsystem[];
}>();

const search = useSearch('search', ['solarsystems']);

function handleSolarsystemSelect(solarsystem: TSolarsystem) {
    createMapSolarsystem(solarsystem.id);
}

const new_solarsystems = computed(() => {
    return solarsystems.filter((solarsystem) => {
        return !map.map_solarsystems?.some((map_solarsystem) => map_solarsystem.solarsystem_id === solarsystem.id);
    });
});

const existing_solarsystems = computed(() => {
    return solarsystems.filter((solarsystem) => {
        return map.map_solarsystems?.some((map_solarsystem) => map_solarsystem.solarsystem_id === solarsystem.id);
    });
});
</script>

<template>
    <Combobox class="absolute top-4 left-1/2 -translate-x-1/2 rounded-lg border bg-white dark:bg-neutral-900">
        <ComboboxAnchor>
            <ComboboxInput v-model="search" />
        </ComboboxAnchor>
        <ComboboxList>
            <ComboboxEmpty> No results found</ComboboxEmpty>
            <ComboboxGroup heading="Search Results" v-if="new_solarsystems.length > 0" class="grid grid-cols-[auto_1fr_auto]">
                <ComboboxItem
                    v-for="solarsystem in new_solarsystems"
                    :key="solarsystem.id"
                    :value="solarsystem.name"
                    @select.prevent="() => handleSolarsystemSelect(solarsystem)"
                    class="col-span-full grid grid-cols-subgrid"
                >
                    <div class="justify-self-center">
                        <SolarsystemClass :wormhole_class="solarsystem.class" :security="solarsystem.security" :name="solarsystem.name" />
                    </div>
                    <span class="whitespace-nowrap">{{ solarsystem.name }}</span>
                    <span class="truncate text-muted-foreground" v-if="!solarsystem.class">{{ solarsystem.region?.name }}</span>
                    <div class="justify-self-end" v-else-if="solarsystem.effect">
                        <SolarsystemEffect :effect="solarsystem.effect" />
                    </div>
                </ComboboxItem>
            </ComboboxGroup>
            <ComboboxGroup heading="Already in Map" v-if="existing_solarsystems.length > 0" class="grid grid-cols-[auto_1fr_auto]">
                <ComboboxItem
                    v-for="solarsystem in existing_solarsystems"
                    :key="solarsystem.id"
                    :value="solarsystem.name"
                    class="col-span-full grid grid-cols-subgrid"
                    disabled
                >
                    <div class="justify-self-center">
                        <SolarsystemClass :wormhole_class="solarsystem.class" :security="solarsystem.security" :name="solarsystem.name" />
                    </div>
                    <span class="whitespace-nowrap">{{ solarsystem.name }}</span>
                    <span class="truncate text-muted-foreground" v-if="!solarsystem.class">{{ solarsystem.region?.name }}</span>
                    <div class="justify-self-end" v-else-if="solarsystem.effect">
                        <SolarsystemEffect :effect="solarsystem.effect" />
                    </div>
                </ComboboxItem>
            </ComboboxGroup>
        </ComboboxList>
    </Combobox>
</template>

<style scoped></style>

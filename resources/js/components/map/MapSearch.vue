<script setup lang="ts">
import SolarsystemEffect from '@/components/map/SolarsystemEffect.vue';
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import { Combobox, ComboboxAnchor, ComboboxEmpty, ComboboxGroup, ComboboxInput, ComboboxItem, ComboboxList } from '@/components/ui/combobox';
import { TMap, TSolarsystem } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { useDebounceFn } from '@vueuse/core';
import { computed, ref } from 'vue';

const { map, search, solarsystems } = defineProps<{
    map: TMap;
    search: string;
    solarsystems: TSolarsystem[];
}>();

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
        position_x: 500,
        position_y: 200,
    });
}

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
</script>

<template>
    <Combobox class="absolute top-4 left-1/2 -translate-x-1/2 rounded-lg border bg-neutral-900">
        <ComboboxAnchor>
            <ComboboxInput v-model:model-value="search_input" @update:modelValue="debounced" />
        </ComboboxAnchor>
        <ComboboxList>
            <ComboboxEmpty> No results found</ComboboxEmpty>
            <ComboboxGroup heading="Search Results" v-if="new_solarsystems.length > 0" class="grid grid-cols-[auto_1fr_auto]">
                <ComboboxItem
                    v-for="solarsystem in new_solarsystems"
                    :key="solarsystem.id"
                    :value="solarsystem.name"
                    @select="() => handleSolarsystemSelect(solarsystem)"
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

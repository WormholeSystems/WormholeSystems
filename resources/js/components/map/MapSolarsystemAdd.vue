<script setup lang="ts">
import SolarsystemEffect from '@/components/map/SolarsystemEffect.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Combobox, ComboboxAnchor, ComboboxEmpty, ComboboxGroup, ComboboxInput, ComboboxItem, ComboboxList } from '@/components/ui/combobox';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { createMapSolarsystem } from '@/composables/map';
import { useShowMap } from '@/composables/useShowMap';
import { useStaticData } from '@/composables/useStaticData';
import { TStaticSolarsystem } from '@/types/static-data';
import { computed, ref } from 'vue';

const search = ref('');

const page = useShowMap();
const { staticData, loadStaticData } = useStaticData();

void loadStaticData();

const adding = ref(false);

const filteredSolarsystems = computed(() => {
    const query = search.value.trim().toLowerCase();
    if (!query) {
        return [] as TStaticSolarsystem[];
    }

    const solarsystems = staticData.value?.solarsystems ?? [];

    return solarsystems.filter((solarsystem) => solarsystem.name.toLowerCase().includes(query)).slice(0, 25);
});

const new_solarsystems = computed(() => {
    return filteredSolarsystems.value.filter(
        (solarsystem) => !page.props.map.map_solarsystems?.some((map_solarsystem) => map_solarsystem.solarsystem_id === solarsystem.id),
    );
});

const existing_solarsystems = computed(() => {
    return filteredSolarsystems.value.filter((solarsystem) =>
        page.props.map.map_solarsystems?.some((map_solarsystem) => map_solarsystem.solarsystem_id === solarsystem.id),
    );
});

function handleSolarsystemSelect(solarsystem: TStaticSolarsystem) {
    createMapSolarsystem(solarsystem.id);
}
</script>

<template>
    <Dialog v-model:open="adding">
        <slot />
        <DialogContent>
            <DialogHeader>
                <DialogTitle> Add Solarsystem</DialogTitle>
                <DialogDescription> Add a solarsystem to the map;</DialogDescription>
            </DialogHeader>
            <Combobox class="rounded-lg border bg-neutral-900">
                <ComboboxAnchor>
                    <ComboboxInput v-model="search" auto-focus />
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
        </DialogContent>
    </Dialog>
</template>

<style scoped></style>

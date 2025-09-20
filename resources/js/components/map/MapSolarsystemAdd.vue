<script setup lang="ts">
import SolarsystemEffect from '@/components/map/SolarsystemEffect.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Combobox, ComboboxAnchor, ComboboxEmpty, ComboboxGroup, ComboboxInput, ComboboxItem, ComboboxList } from '@/components/ui/combobox';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { createMapSolarsystem } from '@/composables/map';
import { useSearch } from '@/composables/useSearch';
import { TShowMapProps } from '@/pages/maps';
import { AppPageProps } from '@/types';
import { TSolarsystem } from '@/types/models';
import { usePage } from '@inertiajs/vue3';
import { computed, ref } from 'vue';

const search = useSearch('search', ['solarsystems']);

const page = usePage<AppPageProps<TShowMapProps>>();

const adding = ref(false);

const new_solarsystems = computed(() => {
    return page.props.solarsystems.filter(
        (solarsystem) => !page.props.map.map_solarsystems?.some((map_solarsystem) => map_solarsystem.solarsystem_id === solarsystem.id),
    );
});

const existing_solarsystems = computed(() => {
    return page.props.solarsystems.filter((solarsystem) =>
        page.props.map.map_solarsystems?.some((map_solarsystem) => map_solarsystem.solarsystem_id === solarsystem.id),
    );
});

function handleSolarsystemSelect(solarsystem: TSolarsystem) {
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

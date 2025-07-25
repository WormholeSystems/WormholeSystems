<script setup lang="ts">
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import SolarsystemEffect from '@/components/map/SolarsystemEffect.vue';
import { Button } from '@/components/ui/button';
import { Combobox, ComboboxAnchor, ComboboxEmpty, ComboboxGroup, ComboboxInput, ComboboxItem, ComboboxList } from '@/components/ui/combobox';
import { ContextMenuContent, ContextMenuItem, ContextMenuSeparator } from '@/components/ui/context-menu';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { useMapAction, useMapSolarsystems } from '@/composables/map';
import { useSearch } from '@/composables/useSearch';
import { TShowMapProps } from '@/pages/maps';
import { AppPageProps } from '@/types';
import { TSolarsystem } from '@/types/models';
import { usePage } from '@inertiajs/vue3';
import { computed, ref } from 'vue';

const { removeAllMapSolarsystems, organizeMapSolarsystems, removeSelectedMapSolarsystems } = useMapAction();

const { map_solarsystems_selected } = useMapSolarsystems();

const is_clearing_map = ref(false);

const { addMapSolarsystem } = useMapAction();

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
    addMapSolarsystem(solarsystem.id);
    adding.value = false;
}

function handleDelete() {
    is_clearing_map.value = true;
}

function handleConfirmDelete() {
    removeAllMapSolarsystems();
    is_clearing_map.value = false;
}

function handeCancelDelete() {
    is_clearing_map.value = false;
}
</script>

<template>
    <ContextMenuContent>
        <ContextMenuItem @select="adding = true"> Add Solarsystem</ContextMenuItem>
        <template v-if="map_solarsystems_selected.length">
            <ContextMenuSeparator />
            <ContextMenuItem @select="removeSelectedMapSolarsystems"> Delete selection</ContextMenuItem>
            <ContextMenuItem @select="organizeMapSolarsystems"> Organize selection</ContextMenuItem>
            <ContextMenuSeparator />
        </template>
        <ContextMenuItem @select="handleDelete"> Clear map</ContextMenuItem>
    </ContextMenuContent>
    <Dialog v-model:open="is_clearing_map">
        <DialogContent>
            <DialogHeader>
                <DialogTitle> Clear map</DialogTitle>
                <DialogDescription>
                    Are you sure you want to clear the map? This will remove all solarsystems that are not pinned.
                </DialogDescription>
            </DialogHeader>
            <DialogFooter>
                <Button variant="outline" @click="handeCancelDelete"> Cancel</Button>
                <Button variant="destructive" @click="handleConfirmDelete"> Clear map</Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>
    <Dialog v-model:open="adding">
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

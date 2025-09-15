<script setup lang="ts">
import SolarsystemEffect from '@/components/map/SolarsystemEffect.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import { Combobox, ComboboxAnchor, ComboboxEmpty, ComboboxGroup, ComboboxInput, ComboboxItem, ComboboxList } from '@/components/ui/combobox';
import {
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuSeparator,
    ContextMenuSub,
    ContextMenuSubContent,
    ContextMenuSubTrigger,
} from '@/components/ui/context-menu';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Label } from '@/components/ui/label';
import { NumberField, NumberFieldContent, NumberFieldDecrement, NumberFieldIncrement, NumberFieldInput } from '@/components/ui/number-field';
import {
    createMapSolarsystem,
    deleteAllMapSolarsystems,
    deleteSelectedMapSolarsystems,
    organizeMapSolarsystems,
    useMapSolarsystems,
} from '@/composables/map';
import { useSearch } from '@/composables/useSearch';
import { TShowMapProps } from '@/pages/maps';
import { AppPageProps } from '@/types';
import { TSolarsystem } from '@/types/models';
import { usePage } from '@inertiajs/vue3';
import { Position } from '@vueuse/core';
import { computed, ref } from 'vue';

const { position } = defineProps<{
    position: Position | null;
}>();

// deleteAllMapSolarsystems, organizeMapSolarsystems, deleteSelectedMapSolarsystems, createMapSolarsystem imported directly

const { map_solarsystems_selected } = useMapSolarsystems();

const is_clearing_map = ref(false);

const search = useSearch('search', ['solarsystems']);

const page = usePage<AppPageProps<TShowMapProps>>();

const adding = ref(false);

const spacing = ref(0);

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
    createMapSolarsystem(solarsystem.id, position);
    adding.value = false;
}

function handleDelete() {
    is_clearing_map.value = true;
}

function handleConfirmDelete() {
    deleteAllMapSolarsystems();
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
            <ContextMenuItem @select="deleteSelectedMapSolarsystems"> Delete selection</ContextMenuItem>
            <ContextMenuSub>
                <ContextMenuSubTrigger> Organize selection</ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <div class="grid gap-2 p-1">
                        <NumberField v-model:model-value="spacing" :min="0" :max="4">
                            <Label for="spacing">Spacing</Label>
                            <NumberFieldContent>
                                <NumberFieldDecrement />
                                <NumberFieldInput />
                                <NumberFieldIncrement />
                            </NumberFieldContent>
                        </NumberField>
                        <Button as-child class="w-full">
                            <ContextMenuItem @select="() => organizeMapSolarsystems(spacing)"> Organize </ContextMenuItem>
                        </Button>
                    </div>
                </ContextMenuSubContent>
            </ContextMenuSub>
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
                <Button variant="secondary" @click="handeCancelDelete"> Cancel</Button>
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

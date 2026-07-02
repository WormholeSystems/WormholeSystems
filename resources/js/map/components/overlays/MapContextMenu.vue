<script setup lang="ts">
import SolarsystemSearchResult from '@/components/solarsystem/SolarsystemSearchResult.vue';
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
import { createMapSolarsystem } from '@/map/actions/createMapSolarsystem';
import { deleteAllMapSolarsystems } from '@/map/actions/deleteAllMapSolarsystems';
import { deleteSelectedMapSolarsystems } from '@/map/actions/deleteSelectedMapSolarsystems';
import { organizeMapSolarsystems } from '@/map/actions/organizeMapSolarsystems';
import { getSelectedMapSolarsystems } from '@/map/actions/selectedMapSolarsystems';
import type { Vec2 } from '@/map/core/types';
import { useSolarsystemSearch } from '@/map/interactions/useSolarsystemSearch';
import { useMapStore } from '@/map/store/mapStore';
import { TStaticSolarsystem } from '@/types/static-data';
import { computed, ref } from 'vue';

/**
 * The map-background context menu. Unlike the old version this receives the
 * right-click position in BASE units (the new createMapSolarsystem action
 * persists base coordinates), and reads the selection from the store instead
 * of the old module-level state.
 */
const { position } = defineProps<{
    position: Vec2 | null;
}>();

const store = useMapStore();

const hasSelection = computed(() => getSelectedMapSolarsystems(store).length > 0);

const is_clearing_map = ref(false);

const search = ref('');

const mapSolarsystems = computed(() => [...store.systems.values()]);

const { new_solarsystems, existing_solarsystems, getAlias } = useSolarsystemSearch(search, () => mapSolarsystems.value);

const adding = ref(false);

const spacing = ref(0);

function handleSolarsystemSelect(solarsystem: TStaticSolarsystem) {
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
        <template v-if="hasSelection">
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
                            class="col-span-full grid grid-cols-subgrid"
                        >
                            <SolarsystemSearchResult :solarsystem="solarsystem" :alias="getAlias(solarsystem.id)" />
                        </ComboboxItem>
                    </ComboboxGroup>
                </ComboboxList>
            </Combobox>
        </DialogContent>
    </Dialog>
</template>

<style scoped></style>

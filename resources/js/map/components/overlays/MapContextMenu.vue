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
import { cleanMapSolarsystems } from '@/map/actions/cleanMapSolarsystems';
import { getClearableMapSolarsystems } from '@/map/actions/clearableMapSolarsystems';
import { createMapSolarsystem } from '@/map/actions/createMapSolarsystem';
import { deleteAllMapSolarsystems } from '@/map/actions/deleteAllMapSolarsystems';
import { deleteSelectedMapSolarsystems } from '@/map/actions/deleteSelectedMapSolarsystems';
import { organizeMapSolarsystems } from '@/map/actions/organizeMapSolarsystems';
import { getOrphanedMapSolarsystems } from '@/map/actions/orphanedMapSolarsystems';
import { getSelectedMapSolarsystems } from '@/map/actions/selectedMapSolarsystems';
import type { Vec2 } from '@/map/core/types';
import { useSolarsystemSearch } from '@/map/interactions/useSolarsystemSearch';
import { useMapStore } from '@/map/store/mapStore';
import { TStaticSolarsystem } from '@/types/static-data';
import { Eraser, LayoutGrid, Plus, Trash2 } from 'lucide-vue-next';
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

const clearable_count = computed(() => getClearableMapSolarsystems(store).length);
const orphaned_count = computed(() => getOrphanedMapSolarsystems(store).length);

const is_clearing_map = ref(false);
const is_cleaning_map = ref(false);

const search = ref('');

const mapSolarsystems = computed(() => [...store.systems.values()]);

const { new_solarsystems, existing_solarsystems, getAlias } = useSolarsystemSearch(search, () => mapSolarsystems.value);

const adding = ref(false);

const spacing = ref(0);

function handleSolarsystemSelect(solarsystem: TStaticSolarsystem) {
    createMapSolarsystem(solarsystem.id, position);
    adding.value = false;
}

function handleConfirmClear() {
    deleteAllMapSolarsystems();
    is_clearing_map.value = false;
}

function handleConfirmClean() {
    cleanMapSolarsystems();
    is_cleaning_map.value = false;
}
</script>

<template>
    <ContextMenuContent>
        <ContextMenuItem @select="adding = true">
            <Plus class="size-4" />
            Add Solarsystem
        </ContextMenuItem>
        <template v-if="hasSelection">
            <ContextMenuSeparator />
            <ContextMenuItem @select="deleteSelectedMapSolarsystems">
                <Trash2 class="size-4" />
                Delete selection
            </ContextMenuItem>
            <ContextMenuSub>
                <ContextMenuSubTrigger>
                    <LayoutGrid class="size-4" />
                    Organize selection
                </ContextMenuSubTrigger>
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
        </template>
        <ContextMenuSeparator />
        <ContextMenuItem @select="is_cleaning_map = true" :disabled="orphaned_count === 0">
            <Eraser class="size-4" />
            Clean map
        </ContextMenuItem>
        <ContextMenuItem @select="is_clearing_map = true" :disabled="clearable_count === 0" class="text-destructive focus:text-destructive">
            <Trash2 class="size-4" />
            Clear map
        </ContextMenuItem>
    </ContextMenuContent>
    <Dialog v-model:open="is_clearing_map">
        <DialogContent>
            <DialogHeader>
                <DialogTitle> Clear map</DialogTitle>
                <DialogDescription>
                    Are you sure you want to clear the map? This will remove {{ clearable_count }}
                    {{ clearable_count === 1 ? 'solarsystem' : 'solarsystems' }}. Only pinned systems and the home system are kept.
                </DialogDescription>
            </DialogHeader>
            <DialogFooter>
                <Button variant="secondary" @click="is_clearing_map = false"> Cancel</Button>
                <Button variant="destructive" @click="handleConfirmClear"> Clear map</Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>
    <Dialog v-model:open="is_cleaning_map">
        <DialogContent>
            <DialogHeader>
                <DialogTitle> Clean map</DialogTitle>
                <DialogDescription>
                    Are you sure you want to clean the map? This will remove {{ orphaned_count }}
                    {{ orphaned_count === 1 ? 'solarsystem' : 'solarsystems' }} not connected to a pinned or home system.
                </DialogDescription>
            </DialogHeader>
            <DialogFooter>
                <Button variant="secondary" @click="is_cleaning_map = false"> Cancel</Button>
                <Button variant="destructive" @click="handleConfirmClean"> Clean map</Button>
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

<script setup lang="ts">
import VirtualizedSolarsystemList from '@/components/solarsystem/VirtualizedSolarsystemList.vue';
import { Combobox, ComboboxAnchor, ComboboxInput } from '@/components/ui/combobox';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { type TComboboxSection } from '@/lib/comboboxSections';
import { createMapSolarsystem } from '@/map/actions/createMapSolarsystem';
import { useAddConnectionDialog } from '@/map/interactions/useAddConnectionDialog';
import { useSolarsystemSearch } from '@/map/interactions/useSolarsystemSearch';
import { useMapStore } from '@/map/store/mapStore';
import { TStaticSolarsystem } from '@/types/static-data';
import { computed, ref, watch } from 'vue';

const { origin, closeAddConnection } = useAddConnectionDialog();

const search = ref('');

const store = useMapStore();
const mapSolarsystems = computed(() => [...store.systems.values()]);
const { new_solarsystems, existing_solarsystems, getAlias } = useSolarsystemSearch(search, () => mapSolarsystems.value);

const isOpen = computed({
    get: () => origin.value !== null,
    set: (value: boolean) => {
        if (!value) {
            closeAddConnection();
        }
    },
});

// Start each open with a clean query.
watch(origin, (value) => {
    if (value) {
        search.value = '';
    }
});

const originName = computed(() => origin.value?.alias || origin.value?.solarsystem?.name || 'this system');

const search_sections = computed<TComboboxSection<TStaticSolarsystem>[]>(() => [
    { key: 'new', heading: 'New systems', items: new_solarsystems.value },
    { key: 'existing', heading: 'Already on the map', items: existing_solarsystems.value },
]);

// Works for both groups: a new system is created and linked, an existing one is just
// linked (the backend finds it in place without moving it and skips a duplicate link).
function handleSolarsystemSelect(solarsystem: TStaticSolarsystem) {
    if (!origin.value) {
        return;
    }

    createMapSolarsystem(solarsystem.id, null, origin.value.id);
    closeAddConnection();
}
</script>

<template>
    <Dialog v-model:open="isOpen">
        <DialogContent>
            <DialogHeader>
                <DialogTitle> Add connection</DialogTitle>
                <DialogDescription> Search for a system to connect to {{ originName }}.</DialogDescription>
            </DialogHeader>
            <Combobox class="rounded-lg border bg-neutral-900" :ignore-filter="true">
                <ComboboxAnchor>
                    <ComboboxInput v-model="search" auto-focus />
                </ComboboxAnchor>
                <VirtualizedSolarsystemList :sections="search_sections" :get-alias="getAlias" @select="handleSolarsystemSelect" />
            </Combobox>
        </DialogContent>
    </Dialog>
</template>

<style scoped></style>

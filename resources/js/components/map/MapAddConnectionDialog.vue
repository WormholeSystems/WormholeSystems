<script setup lang="ts">
import SolarsystemSearchResult from '@/components/solarsystem/SolarsystemSearchResult.vue';
import { Combobox, ComboboxAnchor, ComboboxEmpty, ComboboxGroup, ComboboxInput, ComboboxItem, ComboboxList } from '@/components/ui/combobox';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { createMapSolarsystem, useAddConnection, useSolarsystemSearch } from '@/composables/map';
import { useShowMap } from '@/composables/useShowMap';
import { TStaticSolarsystem } from '@/types/static-data';
import { computed, ref, watch } from 'vue';

const { origin, closeAddConnection } = useAddConnection();

const search = ref('');

const page = useShowMap();
const { new_solarsystems, existing_solarsystems, getAlias } = useSolarsystemSearch(search, () => page.props.map.map_solarsystems);

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
            <Combobox class="rounded-lg border bg-neutral-900">
                <ComboboxAnchor>
                    <ComboboxInput v-model="search" auto-focus />
                </ComboboxAnchor>
                <ComboboxList>
                    <ComboboxEmpty> No results found</ComboboxEmpty>
                    <ComboboxGroup
                        heading="New systems"
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
                        heading="Already on the map"
                        v-if="existing_solarsystems.length > 0"
                        class="grid grid-cols-[auto_1fr_1fr_auto] items-center gap-x-2"
                    >
                        <ComboboxItem
                            v-for="solarsystem in existing_solarsystems"
                            :key="solarsystem.id"
                            :value="solarsystem.name"
                            @select.prevent="() => handleSolarsystemSelect(solarsystem)"
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

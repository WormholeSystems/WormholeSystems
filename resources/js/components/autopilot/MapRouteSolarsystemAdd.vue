<script setup lang="ts">
import PlusIcon from '@/components/icons/PlusIcon.vue';
import VirtualizedSolarsystemList from '@/components/solarsystem/VirtualizedSolarsystemList.vue';
import { Combobox, ComboboxAnchor, ComboboxInput } from '@/components/ui/combobox';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Label } from '@/components/ui/label';
import MapPanelHeaderActionButton from '@/components/ui/map-panel/MapPanelHeaderActionButton.vue';
import { Switch } from '@/components/ui/switch';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useSolarsystemAliases } from '@/composables/useSolarsystemAliases';
import { useStaticData } from '@/composables/useStaticData';
import { type TComboboxSection } from '@/lib/comboboxSections';
import { MAX_SEARCH_RESULTS, takeRanked } from '@/lib/searchRank';
import type { TMap, TResolvedMapRouteSolarsystem } from '@/pages/maps';
import MapRouteSolarsystems from '@/routes/map-route-solarsystems';
import { TStaticSolarsystem } from '@/types/static-data';
import { router } from '@inertiajs/vue3';
import { computed, ref } from 'vue';

const { map, map_route_solarsystems, can_add_personal, can_add_shared } = defineProps<{
    map: TMap;
    map_route_solarsystems?: TResolvedMapRouteSolarsystem[];
    can_add_personal: boolean;
    can_add_shared: boolean;
}>();

const search = ref('');
const { staticData, loadStaticData } = useStaticData();
const { getAlias } = useSolarsystemAliases(() => map.map_solarsystems);

void loadStaticData();

const solarsystems = computed(() => staticData.value?.solarsystems ?? []);

const adding = ref(false);

const sharedSelected = ref(false);
const isShared = computed(() => can_add_shared && sharedSelected.value);

const filteredSolarsystems = computed(() => {
    const query = search.value.trim().toLowerCase();
    if (!query) {
        return [] as TStaticSolarsystem[];
    }

    return takeRanked(
        solarsystems.value,
        query,
        MAX_SEARCH_RESULTS,
        (solarsystem) => [solarsystem.name],
        (solarsystem) => solarsystem.name,
    );
});

const routeSolarsystemIds = computed(
    () => new Set((map_route_solarsystems ?? []).filter((route) => route.is_personal !== isShared.value).map((route) => route.solarsystem.id)),
);

const search_sections = computed<TComboboxSection<TStaticSolarsystem>[]>(() => {
    const new_solarsystems: TStaticSolarsystem[] = [];
    const existing_solarsystems: TStaticSolarsystem[] = [];

    for (const solarsystem of filteredSolarsystems.value) {
        (routeSolarsystemIds.value.has(solarsystem.id) ? existing_solarsystems : new_solarsystems).push(solarsystem);
    }

    return [
        { key: 'new', heading: 'Search Results', items: new_solarsystems },
        { key: 'existing', heading: 'Already in Watchlist', items: existing_solarsystems, selectable: false },
    ];
});

function handleSolarsystemSelect(solarsystem: TStaticSolarsystem) {
    router.post(
        MapRouteSolarsystems.store().url,
        {
            map_id: map.id,
            solarsystem_id: solarsystem.id,
            is_shared: isShared.value,
        },
        {
            preserveState: true,
            preserveScroll: true,
            onSuccess() {
                adding.value = false;
            },
        },
    );
}
</script>

<template>
    <Dialog v-model:open="adding">
        <Tooltip>
            <DialogTrigger as-child>
                <TooltipTrigger as-child>
                    <MapPanelHeaderActionButton size="icon">
                        <PlusIcon />
                    </MapPanelHeaderActionButton>
                </TooltipTrigger>
            </DialogTrigger>
            <TooltipContent> Add Solarsystem</TooltipContent>
        </Tooltip>
        <DialogContent>
            <DialogHeader>
                <DialogTitle> Add Solarsystem</DialogTitle>
                <DialogDescription v-if="isShared">
                    Add a solarsystem to the shared watchlist, visible to all map users. You can select a solarsystem from the map or enter its ID
                    manually.
                </DialogDescription>
                <DialogDescription v-else>
                    Add a solarsystem to your personal watchlist, visible only to you. You can select a solarsystem from the map or enter its ID
                    manually.
                </DialogDescription>
            </DialogHeader>
            <div v-if="can_add_shared && can_add_personal" class="flex items-center gap-2">
                <Switch id="map-route-solarsystem-shared" v-model="sharedSelected" />
                <Label for="map-route-solarsystem-shared">Shared with all map users</Label>
            </div>
            <Combobox class="rounded-lg border bg-neutral-900" :ignore-filter="true">
                <ComboboxAnchor>
                    <ComboboxInput v-model="search" auto-focus />
                </ComboboxAnchor>
                <VirtualizedSolarsystemList
                    class="w-115"
                    align="start"
                    :sections="search_sections"
                    :get-alias="getAlias"
                    @select="handleSolarsystemSelect"
                />
            </Combobox>
        </DialogContent>
    </Dialog>
</template>

<style scoped></style>

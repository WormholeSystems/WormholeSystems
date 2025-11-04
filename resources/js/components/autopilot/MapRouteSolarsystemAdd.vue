<script setup lang="ts">
import PlusIcon from '@/components/icons/PlusIcon.vue';
import SolarsystemEffect from '@/components/map/SolarsystemEffect.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import { Combobox, ComboboxAnchor, ComboboxEmpty, ComboboxGroup, ComboboxInput, ComboboxItem, ComboboxList } from '@/components/ui/combobox';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useSearch } from '@/composables/useSearch';
import { TMap, TSolarsystem } from '@/pages/maps';
import MapRouteSolarsystems from '@/routes/map-route-solarsystems';
import { router } from '@inertiajs/vue3';
import { computed, ref } from 'vue';

const { map, solarsystems, map_route_solarsystems } = defineProps<{
    map: TMap;
    solarsystems: TSolarsystem[];
    map_route_solarsystems?: { solarsystem: TSolarsystem }[];
}>();

const search = useSearch('search', ['solarsystems']);

const adding = ref(false);

const new_solarsystems = computed(() => {
    return solarsystems.filter((solarsystem) => !map_route_solarsystems?.some((route) => route.solarsystem.id === solarsystem.id));
});

const existing_solarsystems = computed(() => {
    return solarsystems.filter((solarsystem) => map_route_solarsystems?.some((route) => route.solarsystem.id === solarsystem.id));
});

function handleSolarsystemSelect(solarsystem: TSolarsystem) {
    router.post(
        MapRouteSolarsystems.store().url,
        {
            map_id: map.id,
            solarsystem_id: solarsystem.id,
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
                    <Button variant="secondary" size="icon">
                        <PlusIcon />
                    </Button>
                </TooltipTrigger>
            </DialogTrigger>
            <TooltipContent> Add Solarsystem</TooltipContent>
        </Tooltip>
        <DialogContent>
            <DialogHeader>
                <DialogTitle> Add Solarsystem</DialogTitle>
                <DialogDescription>
                    Add a solarsystem to the route list. You can select a solarsystem from the map or enter its ID manually.
                </DialogDescription>
            </DialogHeader>
            <Combobox class="rounded-lg border bg-neutral-900">
                <ComboboxAnchor>
                    <ComboboxInput v-model="search" auto-focus />
                </ComboboxAnchor>
                <ComboboxList class="w-115" align="start">
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
                    <ComboboxGroup heading="Already in Watchlist" v-if="existing_solarsystems.length > 0" class="grid grid-cols-[auto_1fr_auto]">
                        <ComboboxItem
                            v-for="solarsystem in existing_solarsystems"
                            :key="solarsystem.id"
                            :value="solarsystem.name"
                            class="col-span-full grid grid-cols-subgrid"
                            disabled
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

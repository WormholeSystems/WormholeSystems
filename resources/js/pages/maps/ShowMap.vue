<script setup lang="ts">
import MapCharacters from '@/components/characters/MapCharacters.vue';
import LockIcon from '@/components/icons/LockIcon.vue';
import QuestionIcon from '@/components/icons/QuestionIcon.vue';
import MapKillmails from '@/components/killmails/MapKillmails.vue';
import MapComponent from '@/components/map/MapComponent.vue';
import MapSearch from '@/components/map/MapSearch.vue';
import Tracker from '@/components/map/Tracker.vue';
import SelectedSolarsystem from '@/components/solarsystem/SelectedSolarsystem.vue';
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import AppLayout from '@/layouts/AppLayout.vue';
import { TMapConfig } from '@/types/map';
import { TCharacter, TKillmail, TMap, TMapSolarSystem, TSolarsystem } from '@/types/models';
import { Head, Link, router } from '@inertiajs/vue3';
import { echo } from '@laravel/echo-vue';

const { map, selected_map_solarsystem, map_killmails } = defineProps<{
    map: TMap;
    search: string;
    solarsystems: TSolarsystem[];
    config: TMapConfig;
    selected_map_solarsystem: TMapSolarSystem | null;
    map_killmails?: TKillmail[];
    map_characters: TCharacter[];
}>();

router.on('before', (event) => {
    event.detail.visit.headers['X-Socket-ID'] = echo().socketId() || '';
});
</script>

<template>
    <AppLayout>
        <Head :title="map.name" />

        <div class="space-y-4 p-4">
            <div class="">
                <div class="relative">
                    <MapComponent :map :config />
                    <MapSearch :map :search :solarsystems />
                    <div class="absolute top-4 right-4 flex gap-2">
                        <Tracker :map_characters v-if="map_characters" />
                        <Tooltip>
                            <TooltipTrigger>
                                <Button :variant="'outline'" as-child>
                                    <Link :href="route('map-access.show', map.slug)">
                                        <LockIcon />
                                    </Link>
                                </Button>
                            </TooltipTrigger>
                            <TooltipContent side="bottom">
                                <p class="text-sm">Map Access</p>
                                <p class="text-xs text-muted-foreground">Manage who can view and edit this map</p>
                            </TooltipContent>
                        </Tooltip>
                    </div>
                </div>
            </div>
            <div class="grid grid-cols-12 gap-4">
                <div class="col-span-6 2xl:col-span-5">
                    <SelectedSolarsystem v-if="selected_map_solarsystem" :map_solarsystem="selected_map_solarsystem" :map />
                    <div class="flex flex-col items-center justify-center gap-8 rounded-lg border border-dashed p-16 text-neutral-700" v-else>
                        <QuestionIcon class="text-4xl" />
                        <p class="text-center">Select a solarsystem to see more details</p>
                    </div>
                </div>
                <div class="col-span-6 2xl:col-span-3">
                    <MapCharacters :map_id="map.id" :map_characters />
                </div>
                <div class="col-span-6 2xl:col-span-4">
                    <MapKillmails :map_killmails="map_killmails" :map_id="map.id" />
                </div>
            </div>
        </div>
    </AppLayout>
</template>
<style>
::-webkit-scrollbar {
    width: 8px;
    height: 8px;
}

::-webkit-scrollbar-thumb {
    background-color: var(--color-neutral-700);
    border-radius: 4px;
}

::-webkit-scrollbar-track {
    background-color: transparent;
}

::-webkit-scrollbar-corner {
    background-color: transparent;
}

::-webkit-scrollbar-button {
    display: none;
}
</style>

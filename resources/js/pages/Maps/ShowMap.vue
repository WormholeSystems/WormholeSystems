<script setup lang="ts">
import MapCharacters from '@/components/characters/MapCharacters.vue';
import QuestionIcon from '@/components/icons/QuestionIcon.vue';
import MapKillmails from '@/components/killmails/MapKillmails.vue';
import MapComponent from '@/components/map/MapComponent.vue';
import MapSearch from '@/components/map/MapSearch.vue';
import Tracker from '@/components/map/Tracker.vue';
import SelectedSolarsystem from '@/components/solarsystem/SelectedSolarsystem.vue';
import AppLayout from '@/layouts/AppLayout.vue';
import { TMapConfig } from '@/types/map';
import { TCharacter, TKillmail, TMap, TMapSolarSystem, TSolarsystem } from '@/types/models';
import { Head, router } from '@inertiajs/vue3';
import { echo } from '@laravel/echo-vue';

const { map, selected_map_solarsystem, map_killmails } = defineProps<{
    map: TMap;
    search: string;
    solarsystems: TSolarsystem[];
    config: TMapConfig;
    selected_map_solarsystem: TMapSolarSystem | null;
    map_killmails?: TKillmail[];
    map_characters?: TCharacter[];
}>();

router.on('before', (event) => {
    event.detail.visit.headers['X-Socket-ID'] = echo().socketId() || '';
});
</script>

<template>
    <AppLayout>
        <Head title="ShowMap" />

        <div>
            <div class="relative">
                <MapComponent :map :config />
                <MapSearch :map :search :solarsystems />
                <Tracker :map_characters v-if="map_characters" />
            </div>
            <div class="grid grid-cols-3 gap-8 p-8">
                <SelectedSolarsystem v-if="selected_map_solarsystem" :map_solarsystem="selected_map_solarsystem" :map />
                <div class="flex flex-col items-center justify-center gap-8 rounded-lg border border-dashed text-neutral-700" v-else>
                    <QuestionIcon class="text-4xl" />
                    <p class="text-center">Select a solarsystem to see more details</p>
                </div>
                <MapCharacters :map_id="map.id" :map_characters />
                <MapKillmails :map_killmails="map_killmails" :map_id="map.id" />
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

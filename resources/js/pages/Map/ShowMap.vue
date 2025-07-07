<script setup lang="ts">
import MapKillmails from '@/components/killmails/MapKillmails.vue';
import MapComponent from '@/components/map/MapComponent.vue';
import MapSearch from '@/components/map/MapSearch.vue';
import SelectedSolarsystem from '@/components/solarsystem/SelectedSolarsystem.vue';
import AppLayout from '@/layouts/AppLayout.vue';
import { TMapConfig } from '@/types/map';
import { TKillmail, TMap, TMapSolarSystem, TSolarsystem } from '@/types/models';
import { Head, router } from '@inertiajs/vue3';
import { echo } from '@laravel/echo-vue';

const { map, selected_map_solarsystem, map_killmails } = defineProps<{
    map: TMap;
    search: string;
    solarsystems: TSolarsystem[];
    config: TMapConfig;
    selected_map_solarsystem: TMapSolarSystem | null;
    map_killmails?: TKillmail[];
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
            </div>
            <div class="grid grid-cols-3 gap-8 p-8">
                <SelectedSolarsystem v-if="selected_map_solarsystem" :map_solarsystem="selected_map_solarsystem" :map />
                <div class=""></div>

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

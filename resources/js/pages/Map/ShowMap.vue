<script setup lang="ts">
import MapComponentComponent from '@/components/map/MapComponent.vue';
import MapSearch from '@/components/map/MapSearch.vue';
import AppLayout from '@/layouts/AppLayout.vue';
import { TMap, TSolarsystem } from '@/types/models';
import { Head, Link } from '@inertiajs/vue3';

const { map } = defineProps<{
    map: TMap;
    search: string;
    solarsystems: TSolarsystem[];
}>();
</script>

<template>
    <AppLayout>
        <Head title="ShowMap" />

        <div class="p-8">
            <div class="relative">
                <MapComponentComponent :map="map" />
                <MapSearch :map :search :solarsystems />
            </div>
        </div>
        <ul>
            <li :key="solarsystem.id" v-for="solarsystem in map.map_solarsystems">
                <span>{{ solarsystem.name }}</span>
                <Link :href="route('map-solarsystems.destroy', solarsystem.id)" method="delete" as="button"> Delete </Link>
            </li>
        </ul>
        <ul>
            <li v-for="map_connection in map.map_connections" :key="map_connection.id">
                <span>{{ map_connection.from_map_solarsystem_id }} {{ '<-->' }} {{ map_connection.to_map_solarsystem_id }}</span>
            </li>
        </ul>
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

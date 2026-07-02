<script setup lang="ts">
import { TMapSolarsystem } from '@/pages/maps';

defineProps<{
    map_solarsystem: TMapSolarsystem;
    /** Truncate to a single line for the fixed-width tree layout. */
    truncate?: boolean;
}>();
</script>

<template>
    <span class="pointer-events-none col-start-2 row-start-1" :class="{ 'block min-w-0 truncate': truncate }">
        <span class="mr-1 inline-block" v-if="map_solarsystem.alias">{{ map_solarsystem.alias }}</span>
        <span :data-has-alias="map_solarsystem.alias !== null" class="data-[has-alias=true]:text-muted-foreground">{{
            map_solarsystem.solarsystem?.name
        }}</span>
        <span v-if="map_solarsystem.occupier_alias" class="text-muted-foreground"> ({{ map_solarsystem.occupier_alias }})</span>
    </span>
</template>

<style scoped></style>

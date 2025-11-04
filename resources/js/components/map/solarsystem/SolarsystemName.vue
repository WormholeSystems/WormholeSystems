<script setup lang="ts">
import { TMapSolarsystem } from '@/pages/maps';

defineProps<{
    map_solarsystem: TMapSolarsystem;
}>();
</script>

<template>
    <span class="pointer-events-none col-start-2 row-start-1">
        <span class="mr-1 inline-block" v-if="map_solarsystem.alias">{{ map_solarsystem.alias }}</span>
        <span :data-has-alias="map_solarsystem.alias !== null" class="data-[has-alias=true]:text-muted-foreground">{{
            map_solarsystem.solarsystem?.name
        }}</span>
        <span v-if="map_solarsystem.occupier_alias" class="text-muted-foreground"> ({{ map_solarsystem.occupier_alias }})</span>
    </span>
</template>

<style scoped></style>

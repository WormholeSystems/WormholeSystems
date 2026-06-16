<script setup lang="ts">
import MapConnection from '@/components/map/MapConnection.vue';
import MapSolarsystemButton from '@/components/map/MapSolarsystemButton.vue';
import type { TDataMapSolarSystem, TProcessedConnection } from '@/composables/map';
import { TCharacter } from '@/types/models';

const {
    solarsystems,
    connections,
    scale = 1,
    grid_size = 20,
    home_solarsystem_id = null,
    rally_solarsystem_id = null,
    pilots = {},
} = defineProps<{
    solarsystems: TDataMapSolarSystem[];
    connections: TProcessedConnection[];
    scale?: number;
    /** Grid cell size in base units; matches the live map default (20). */
    grid_size?: number;
    home_solarsystem_id?: number | null;
    rally_solarsystem_id?: number | null;
    pilots?: Record<number, TCharacter[]>;
}>();

function scaled(position: { x: number; y: number } | null | undefined): { x: number; y: number } {
    return { x: (position?.x ?? 0) * scale, y: (position?.y ?? 0) * scale };
}
</script>

<template>
    <div class="bg-grid relative h-full w-full overflow-hidden" :style="{ backgroundSize: `${grid_size * scale}px ${grid_size * scale}px` }">
        <svg class="pointer-events-none absolute inset-0 h-full w-full text-neutral-700" xmlns="http://www.w3.org/2000/svg">
            <MapConnection
                v-for="connection in connections"
                :key="connection.id"
                :from="scaled(connection.source.position)"
                :to="scaled(connection.target.position)"
                :ship_size="connection.ship_size"
                :mass_status="connection.mass_status"
                :lifetime="connection.lifetime_status"
                :is_highlighted="connection.is_on_route"
            />
        </svg>

        <div
            v-for="solarsystem in solarsystems"
            :key="solarsystem.id"
            class="absolute"
            :style="{ transform: `translate(${scaled(solarsystem.position).x}px, ${scaled(solarsystem.position).y}px)` }"
        >
            <div :style="{ transform: `translate(${-40 * scale}px, ${-20 * scale}px)`, transformOrigin: 'top left' }">
                <div :style="{ scale, transformOrigin: 'top left' }">
                    <MapSolarsystemButton
                        :map_solarsystem="solarsystem"
                        :pilots="pilots[solarsystem.id] ?? []"
                        :is_active="false"
                        :is_home="home_solarsystem_id === solarsystem.id"
                        :is_rally="rally_solarsystem_id === solarsystem.solarsystem_id"
                    />
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.bg-grid {
    background-image: linear-gradient(to right, var(--grid) 1px, transparent 1px), linear-gradient(to bottom, var(--grid) 1px, transparent 1px);
}
</style>

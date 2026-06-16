<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import KillmailView from '@/components/map-killmails/KillmailView.vue';
import { useMapSolarsystems } from '@/composables/map';
import { useStaticSolarsystems } from '@/composables/useStaticSolarsystems';
import { TKillmail } from '@/types/models';
import { computed } from 'vue';

const { killmail } = defineProps<{
    killmail: TKillmail;
}>();

const { map_solarsystems, setHoveredMapSolarsystem } = useMapSolarsystems();
const { resolveSolarsystem } = useStaticSolarsystems();

const map_solarsystem = computed(() => {
    return map_solarsystems.value.find((solarsystem) => solarsystem.solarsystem_id === killmail.solarsystem_id);
});

const staticSolarsystem = computed(() => resolveSolarsystem(killmail.solarsystem_id));

function onHover(hovered: boolean) {
    if (map_solarsystem.value) {
        setHoveredMapSolarsystem(map_solarsystem.value.id, hovered);
    }
}
</script>

<template>
    <div class="col-span-full grid grid-cols-subgrid">
        <DestinationContextMenu :solarsystem_id="staticSolarsystem.id">
            <KillmailView :killmail="killmail" :solarsystem="staticSolarsystem" :alias="map_solarsystem?.alias ?? null" @hover="onHover" />
        </DestinationContextMenu>
    </div>
</template>

<style scoped></style>

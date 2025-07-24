<script setup lang="ts">
import TypeImage from '@/components/images/TypeImage.vue';
import Affiliation from '@/components/killmails/Affiliation.vue';
import AttackerImage from '@/components/killmails/AttackerImage.vue';
import VictimImage from '@/components/killmails/VictimImage.vue';
import { TableCell, TableRow } from '@/components/ui/table';
import { useMapSolarsystems } from '@/composables/map';
import { TKillmail } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import { vElementHover } from '@vueuse/components';
import { useNow } from '@vueuse/core';
import { formatDistanceStrict } from 'date-fns';
import { computed } from 'vue';

const { killmail } = defineProps<{
    killmail: TKillmail;
}>();

const { map_solarsystems, setHoveredMapSolarsystem } = useMapSolarsystems();

function onHover(hovered: boolean) {
    setHoveredMapSolarsystem(map_solarsystem.value!.id, hovered);
}

const map_solarsystem = computed(() => {
    return map_solarsystems.value.find((solarsystem) => solarsystem.solarsystem_id === killmail.solarsystem.id);
});

const final_blow = computed(() => {
    return killmail.data.attackers.find((attacker) => attacker.final_blow)!;
});

const now = useNow();

const time_ago = computed(() => {
    return formatDistanceStrict(new UTCDate(killmail.time), now.value, {
        addSuffix: true,
    });
});
</script>

<template>
    <TableRow v-element-hover="onHover">
        <TableCell>
            <div class="flex gap-x-2">
                <a :href="`https://zkillboard.com/kill/${killmail.id}/`" target="_blank" rel="noopener noreferrer">
                    <TypeImage class="size-8 rounded-lg" :type_id="killmail.ship_type.id" :type_name="killmail.ship_type.name" />
                </a>
                <a :href="`https://zkillboard.com/character/${killmail.data.victim.character_id}/`" target="_blank" rel="noopener noreferrer">
                    <VictimImage class="size-8 overflow-hidden rounded-lg" :victim="killmail.data.victim" />
                </a>
                <Affiliation class="size-8 overflow-hidden rounded-lg" alt="Victim group" :affiliation="killmail.data.victim" />
            </div>
        </TableCell>
        <TableCell>
            <div class="flex gap-x-2">
                <TypeImage class="size-8 rounded-lg" :type_id="final_blow.ship_type_id" type_name="Attacker Ship" variant="icon" />
                <a :href="`https://zkillboard.com/character/${final_blow.character_id}/`" target="_blank" rel="noopener noreferrer">
                    <AttackerImage class="size-8 overflow-hidden rounded-lg" :attacker="final_blow" />
                </a>
                <Affiliation class="size-8 overflow-hidden rounded-lg" alt="Attacker group" :affiliation="final_blow" />
            </div>
        </TableCell>
        <TableCell>
            <div class="">
                <span v-if="map_solarsystem?.alias" class="flex gap-1"
                    >{{ map_solarsystem?.alias }}<span class="text-muted-foreground">{{ killmail.solarsystem.name }}</span></span
                >
                <span v-else> {{ killmail.solarsystem.name }}</span>
                <p class="text-muted-foreground">
                    {{ time_ago }}
                </p>
            </div>
        </TableCell>
    </TableRow>
</template>

<style scoped></style>

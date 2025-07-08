<script setup lang="ts">
import TypeImage from '@/components/images/TypeImage.vue';
import Affiliation from '@/components/killmails/Affiliation.vue';
import AttackerImage from '@/components/killmails/AttackerImage.vue';
import VictimImage from '@/components/killmails/VictimImage.vue';
import { useMapSolarsystems } from '@/composables/map';
import { TKillmail } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import { vElementHover } from '@vueuse/components';
import { useNow } from '@vueuse/core';
import { format, formatDistanceStrict } from 'date-fns';
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

const time = computed(() => {
    return format(new UTCDate(killmail.time), 'MMM do, HH:mm');
});

const time_ago = computed(() => {
    return formatDistanceStrict(new UTCDate(killmail.time), now.value, {
        addSuffix: true,
    });
});
</script>

<template>
    <div
        class="col-span-full grid h-12 grid-cols-subgrid items-center gap-2 border-b border-neutral-700 py-2 text-xs last:border-b-0"
        v-element-hover="onHover"
    >
        <div class="grid grid-cols-[auto_auto_auto] gap-0.5 gap-x-2">
            <a :href="`https://zkillboard.com/kill/${killmail.id}/`" target="_blank" rel="noopener noreferrer">
                <TypeImage class="size-8 rounded-lg" :type_id="killmail.ship_type.id" :type_name="killmail.ship_type.name" />
            </a>
            <a :href="`https://zkillboard.com/character/${killmail.data.victim.character_id}/`" target="_blank" rel="noopener noreferrer">
                <VictimImage class="size-8" :victim="killmail.data.victim" />
            </a>
            <Affiliation class="size-8" alt="Victim group" :affiliation="killmail.data.victim" />
        </div>
        <div class="">
            <span>{{ map_solarsystem?.alias }} {{ killmail.solarsystem.name }}</span>
            <p class="text-muted-foreground">
                <span>{{ killmail.solarsystem.region?.name }}</span>
            </p>
        </div>
        <div>
            <span>{{ time }}</span>
            <p class="text-muted-foreground">
                <span>{{ time_ago }}</span>
            </p>
        </div>
        <div class="grid grid-cols-[auto_auto_auto_auto] gap-0.5 gap-x-2">
            <TypeImage class="size-8 rounded-lg" :type_id="final_blow.ship_type_id" type_name="Attacker Ship" variant="icon" />
            <a :href="`https://zkillboard.com/character/${final_blow.character_id}/`" target="_blank" rel="noopener noreferrer">
                <AttackerImage class="size-8" :attacker="final_blow" />
            </a>
            <Affiliation class="size-8" alt="Attacker group" :affiliation="final_blow" />
        </div>
        <div class="grid text-right text-muted-foreground">
            <span>{{ killmail.data.attackers.length }} attackers</span>
            <span
                >{{
                    killmail.zkb.totalValue.toLocaleString('en-US', {
                        notation: 'compact',
                    })
                }}
                ISK</span
            >
        </div>
    </div>
</template>

<style scoped></style>

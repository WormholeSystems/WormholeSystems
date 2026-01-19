<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import QuestionIcon from '@/components/icons/QuestionIcon.vue';
import Affiliation from '@/components/map-killmails/Affiliation.vue';
import AttackerImage from '@/components/map-killmails/AttackerImage.vue';
import VictimImage from '@/components/map-killmails/VictimImage.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { useMapSolarsystems } from '@/composables/map';
import { useStaticSolarsystem, useStaticSolarsystems } from '@/composables/useStaticSolarsystems';
import { formatISK } from '@/lib/utils';
import { TKillmail } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import { vElementHover } from '@vueuse/components';
import { useNow } from '@vueuse/core';
import { differenceInDays, differenceInHours, differenceInMinutes } from 'date-fns';
import { computed } from 'vue';
import TypeImage from '../images/TypeImage.vue';

const { killmail } = defineProps<{
    killmail: TKillmail;
}>();

const { map_solarsystems, setHoveredMapSolarsystem } = useMapSolarsystems();
const { resolveSolarsystem } = useStaticSolarsystems();
const staticSolarsystemRef = useStaticSolarsystem(() => killmail.solarsystem_id);

function onHover(hovered: boolean) {
    if (map_solarsystem.value) {
        setHoveredMapSolarsystem(map_solarsystem.value.id, hovered);
    }
}

const map_solarsystem = computed(() => {
    return map_solarsystems.value.find((solarsystem) => solarsystem.solarsystem_id === killmail.solarsystem_id);
});

const staticSolarsystem = computed(() => staticSolarsystemRef.value ?? resolveSolarsystem(killmail.solarsystem_id));

const final_blow = computed(() => {
    return killmail.data.attackers.find((attacker) => attacker.final_blow)!;
});

const now = useNow();

const time_ago = computed(() => {
    const days_ago = differenceInDays(now.value, new UTCDate(killmail.time));
    if (days_ago > 1) {
        return `${days_ago}d ago`;
    }
    const hours_ago = differenceInHours(now.value, new UTCDate(killmail.time));
    if (hours_ago >= 1) {
        return `${hours_ago}h ago`;
    }
    const minutes_ago = differenceInMinutes(now.value, new UTCDate(killmail.time));
    if (minutes_ago >= 1) {
        return `${minutes_ago}m ago`;
    }
    return 'just now';
});

const total_worth = computed(() => {
    if (!killmail.zkb?.totalValue) return 'N/A';
    return formatISK(killmail.zkb.totalValue);
});
</script>

<template>
    <div class="col-span-full grid grid-cols-subgrid text-sm *:*:p-2" v-element-hover="onHover">
        <DestinationContextMenu :solarsystem_id="staticSolarsystem.id">
            <div class="contents">
                <div class="flex gap-x-2">
                    <a :href="`https://zkillboard.com/kill/${killmail.id}/`" target="_blank" rel="noopener noreferrer" v-if="killmail.ship_type">
                        <TypeImage class="size-6 rounded-lg" :type_id="killmail.ship_type.id" :type_name="killmail.ship_type.name" />
                    </a>
                    <QuestionIcon v-else class="size-6 rounded-lg text-muted-foreground" />
                    <a :href="`https://zkillboard.com/character/${killmail.data.victim.character_id}/`" target="_blank" rel="noopener noreferrer">
                        <VictimImage class="size-6 overflow-hidden rounded-lg" :victim="killmail.data.victim" />
                    </a>
                    <Affiliation class="size-6 overflow-hidden rounded-lg" alt="Victim group" :affiliation="killmail.data.victim" />
                </div>
                <div class="flex gap-x-2">
                    <a :href="`https://zkillboard.com/character/${final_blow.character_id}/`" target="_blank" rel="noopener noreferrer">
                        <AttackerImage class="size-6 overflow-hidden rounded-lg" :attacker="final_blow" />
                    </a>
                    <Affiliation class="size-6 overflow-hidden rounded-lg" alt="Attacker group" :affiliation="final_blow" />
                </div>
                <div class="grid grid-cols-[1.5rem_1fr] items-center gap-1">
                    <div class="flex size-6 items-center justify-center">
                        <SolarsystemClass :wormhole_class="staticSolarsystem.class" :security="staticSolarsystem.security" />
                    </div>
                    <div v-if="map_solarsystem?.alias" class="truncate font-medium hover:text-accent-foreground">
                        {{ map_solarsystem.alias }} <span class="text-muted-foreground">{{ staticSolarsystem.name }}</span>
                    </div>
                    <div v-else class="font-medium hover:text-accent-foreground">
                        {{ staticSolarsystem.name }}
                    </div>
                </div>
                <div class="hidden @lg:block">
                    <SolarsystemSovereignty
                        :sovereignty="staticSolarsystem.sovereignty"
                        :solarsystem-id="staticSolarsystem.id"
                        class="size-6 flex-shrink-0"
                    />
                </div>
                <span class="truncate text-right text-muted-foreground">
                    {{ time_ago }}
                </span>

                <span class="hidden text-right font-mono whitespace-nowrap text-muted-foreground @lg:block">
                    {{ total_worth }}
                </span>
            </div>
        </DestinationContextMenu>
    </div>
</template>

<style scoped></style>

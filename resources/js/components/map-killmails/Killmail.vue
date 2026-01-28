<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import QuestionIcon from '@/components/icons/QuestionIcon.vue';
import VictimImage from '@/components/map-killmails/VictimImage.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { useMapSolarsystems } from '@/composables/map';
import { useStaticSolarsystems } from '@/composables/useStaticSolarsystems';
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

function onHover(hovered: boolean) {
    if (map_solarsystem.value) {
        setHoveredMapSolarsystem(map_solarsystem.value.id, hovered);
    }
}

const map_solarsystem = computed(() => {
    return map_solarsystems.value.find((solarsystem) => solarsystem.solarsystem_id === killmail.solarsystem_id);
});

const staticSolarsystem = computed(() => resolveSolarsystem(killmail.solarsystem_id));

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
    <DestinationContextMenu :solarsystem_id="staticSolarsystem.id">
        <div class="flex items-center gap-2 border-b border-border/30 px-3 py-1.5 hover:bg-muted/30" v-element-hover="onHover">
            <a :href="`https://zkillboard.com/kill/${killmail.id}/`" target="_blank" rel="noopener noreferrer" v-if="killmail.ship_type">
                <TypeImage class="size-5 rounded" :type_id="killmail.ship_type.id" :type_name="killmail.ship_type.name" />
            </a>
            <QuestionIcon v-else class="size-5 rounded text-muted-foreground" />

            <a :href="`https://zkillboard.com/character/${killmail.data.victim.character_id}/`" target="_blank" rel="noopener noreferrer">
                <VictimImage class="size-5 overflow-hidden rounded" :victim="killmail.data.victim" />
            </a>

            <div class="flex min-w-0 flex-1 items-center gap-1.5">
                <SolarsystemClass :wormhole_class="staticSolarsystem.class" :security="staticSolarsystem.security" class="shrink-0" />
                <span v-if="map_solarsystem?.alias" class="truncate text-xs">
                    <span class="font-mono font-medium">{{ map_solarsystem.alias }}</span>
                </span>
                <span v-else class="truncate font-mono text-xs">{{ staticSolarsystem.name }}</span>
            </div>

            <span class="font-mono text-[10px] text-muted-foreground">{{ time_ago }}</span>

            <span class="hidden font-mono text-[10px] text-muted-foreground @lg:block">{{ total_worth }}</span>
        </div>
    </DestinationContextMenu>
</template>

<style scoped></style>

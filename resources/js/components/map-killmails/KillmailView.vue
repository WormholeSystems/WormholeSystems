<script setup lang="ts">
import QuestionIcon from '@/components/icons/QuestionIcon.vue';
import { AllianceLogo, CharacterImage, CorporationLogo, TypeImage } from '@/components/images';
import VictimImage from '@/components/map-killmails/VictimImage.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { formatISK } from '@/lib/utils';
import type { TResolvedSolarsystem } from '@/pages/maps';
import { TKillmail } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import { vElementHover } from '@vueuse/components';
import { useNow } from '@vueuse/core';
import { differenceInDays, differenceInHours, differenceInMinutes } from 'date-fns';
import { computed } from 'vue';

const {
    killmail,
    solarsystem,
    alias = null,
} = defineProps<{
    killmail: TKillmail;
    solarsystem: TResolvedSolarsystem;
    alias?: string | null;
}>();

const emit = defineEmits<{
    hover: [hovered: boolean];
}>();

function onHover(hovered: boolean): void {
    emit('hover', hovered);
}

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

const attackerCount = computed(() => killmail.data.attackers.length);

const attackerCountClass = computed(() => {
    if (killmail.zkb?.awox) return 'text-purple-400';
    if (killmail.zkb?.npc) return 'text-red-400';
    if (killmail.zkb?.solo) return 'text-amber-400';
    return 'text-muted-foreground';
});

const victimTicker = computed(() => {
    return killmail.victim_alliance?.ticker ?? killmail.victim_corporation?.ticker ?? null;
});

const finalBlow = computed(() => killmail.data.attackers.find((a) => a.final_blow) ?? null);
</script>

<template>
    <div class="col-span-full grid grid-cols-subgrid items-center border-b border-border/30 px-3 py-1.5 hover:bg-muted/30" v-element-hover="onHover">
        <!-- Victim info -->
        <a
            v-if="killmail.ship_type"
            class="block size-5"
            :href="`https://zkillboard.com/kill/${killmail.id}/`"
            target="_blank"
            rel="noopener noreferrer"
        >
            <TypeImage class="size-5 rounded" :type_id="killmail.ship_type.id" :type_name="killmail.ship_type.name" />
        </a>
        <QuestionIcon v-else class="size-5 rounded text-muted-foreground" />

        <a
            class="block size-5"
            :href="`https://zkillboard.com/character/${killmail.data.victim.character_id}/`"
            target="_blank"
            rel="noopener noreferrer"
        >
            <VictimImage class="size-5 overflow-hidden rounded" :victim="killmail.data.victim" />
        </a>

        <AllianceLogo
            v-if="killmail.victim_alliance"
            class="size-5 overflow-hidden rounded"
            :alliance_id="killmail.victim_alliance.id"
            :alliance_name="killmail.victim_alliance.name"
        />
        <CorporationLogo
            v-else-if="killmail.victim_corporation"
            class="size-5 overflow-hidden rounded"
            :corporation_id="killmail.victim_corporation.id"
            :corporation_name="killmail.victim_corporation.name"
        />
        <span v-else />

        <span v-if="victimTicker" class="truncate font-mono text-[10px] text-muted-foreground">[{{ victimTicker }}]</span>
        <span v-else />

        <!-- Location -->
        <SolarsystemClass :solarsystem_class="solarsystem.class" class="justify-self-center" />

        <span v-if="alias" class="truncate font-mono text-xs font-medium">{{ alias }}</span>
        <span v-else class="truncate font-mono text-xs">{{ solarsystem.name }}</span>

        <span class="truncate text-[10px] text-muted-foreground">{{ solarsystem.region?.name }}</span>

        <!-- Attacker info -->
        <a
            v-if="finalBlow?.ship_type_id"
            class="block size-5"
            :href="`https://zkillboard.com/kill/${killmail.id}/`"
            target="_blank"
            rel="noopener noreferrer"
        >
            <TypeImage class="size-5 rounded" :type_id="finalBlow.ship_type_id" type_name="Final Blow Ship" />
        </a>
        <span v-else />

        <a
            v-if="finalBlow?.character_id"
            class="block size-5"
            :href="`https://zkillboard.com/character/${finalBlow.character_id}/`"
            target="_blank"
            rel="noopener noreferrer"
        >
            <CharacterImage class="size-5 overflow-hidden rounded" :character_id="finalBlow.character_id" character_name="Final Blow" />
        </a>
        <CorporationLogo
            v-else-if="finalBlow?.corporation_id"
            class="size-5 overflow-hidden rounded"
            :corporation_id="finalBlow.corporation_id"
            corporation_name="Final Blow"
        />
        <span v-else />

        <span class="text-right font-mono text-[10px] font-medium" :class="attackerCountClass">{{ attackerCount }}</span>

        <!-- Meta -->
        <span class="text-right font-mono text-[10px] text-muted-foreground">{{ total_worth }}</span>

        <span class="font-mono text-[10px] text-muted-foreground">{{ time_ago }}</span>
    </div>
</template>

<style scoped></style>

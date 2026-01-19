<script setup lang="ts">
import AllianceLogo from '@/components/images/AllianceLogo.vue';
import CorporationLogo from '@/components/images/CorporationLogo.vue';
import FactionLogo from '@/components/images/FactionLogo.vue';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useSovereignty } from '@/composables/useSovereigntyData';
import { cn } from '@/lib/utils';
import { TSovereignty } from '@/types/models';
import { computed } from 'vue';

const props = defineProps<{
    sovereignty?: TSovereignty | null;
    solarsystemId?: number;
    class?: string;
}>();

const sovereignty = useSovereignty(() => props.solarsystemId ?? null);

const resolvedSovereignty = computed(() => {
    if (props.sovereignty) {
        return props.sovereignty;
    }

    if (!props.solarsystemId) {
        return null;
    }

    return sovereignty.value ?? null;
});

const hasSovereignty = computed(() => {
    return Boolean(resolvedSovereignty.value?.alliance || resolvedSovereignty.value?.corporation || resolvedSovereignty.value?.faction);
});

const sizeClass = computed(() => cn('size-4', props.class));
</script>

<template>
    <Tooltip :delay-duration="500" v-if="hasSovereignty">
        <TooltipTrigger as-child>
            <div :class="sizeClass">
                <AllianceLogo
                    v-if="resolvedSovereignty?.alliance"
                    :alliance_id="resolvedSovereignty.alliance.id"
                    :alliance_name="resolvedSovereignty.alliance.name"
                />
                <CorporationLogo
                    v-else-if="resolvedSovereignty?.corporation"
                    :corporation_id="resolvedSovereignty.corporation.id"
                    :corporation_name="resolvedSovereignty.corporation.name"
                />
                <FactionLogo
                    v-else-if="resolvedSovereignty?.faction"
                    :faction_id="resolvedSovereignty.faction.id"
                    :faction_name="resolvedSovereignty.faction.name"
                />
            </div>
        </TooltipTrigger>
        <TooltipContent class="text-sm">
            <div v-if="resolvedSovereignty?.alliance" class="flex items-center gap-2">
                <AllianceLogo :alliance_id="resolvedSovereignty.alliance.id" :alliance_name="resolvedSovereignty.alliance.name" class="size-6" />
                <span class="text-sm">{{ resolvedSovereignty.alliance.name }}</span>
                <span class="text-xs text-muted-foreground" v-if="resolvedSovereignty.alliance.ticker"
                    >({{ resolvedSovereignty.alliance.ticker }})</span
                >
            </div>
            <div v-else-if="resolvedSovereignty?.corporation" class="flex items-center gap-2">
                <CorporationLogo
                    :corporation_id="resolvedSovereignty.corporation.id"
                    :corporation_name="resolvedSovereignty.corporation.name"
                    class="size-6"
                />
                <span class="text-sm">{{ resolvedSovereignty.corporation.name }}</span>
                <span class="text-xs text-muted-foreground" v-if="resolvedSovereignty.corporation.ticker"
                    >({{ resolvedSovereignty.corporation.ticker }})</span
                >
            </div>
            <div v-else-if="resolvedSovereignty?.faction" class="flex items-center gap-2">
                <FactionLogo :faction_id="resolvedSovereignty.faction.id" :faction_name="resolvedSovereignty.faction.name" class="size-6" />
                <span class="text-sm">{{ resolvedSovereignty.faction.name }}</span>
            </div>
        </TooltipContent>
    </Tooltip>
    <template v-else>
        <slot name="fallback">
            <div :class="sizeClass"></div>
        </slot>
    </template>
</template>

<style scoped></style>

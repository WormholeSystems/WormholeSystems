<script setup lang="ts">
import AllianceLogo from '@/components/images/AllianceLogo.vue';
import CorporationLogo from '@/components/images/CorporationLogo.vue';
import FactionLogo from '@/components/images/FactionLogo.vue';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { cn } from '@/lib/utils';
import { TSovereignty } from '@/types/models';

const { sovereignty, ...props } = defineProps<{
    sovereignty: TSovereignty;
    class?: string;
}>();
</script>

<template>
    <Tooltip :delay-duration="500">
        <TooltipTrigger as-child>
            <div :class="cn('size-4', props.class)">
                <AllianceLogo v-if="sovereignty.alliance" :alliance_id="sovereignty.alliance.id" :alliance_name="sovereignty.alliance.name" />
                <CorporationLogo
                    v-else-if="sovereignty.corporation"
                    :corporation_id="sovereignty.corporation.id"
                    :corporation_name="sovereignty.corporation.name"
                />
                <FactionLogo v-else-if="sovereignty.faction" :faction_id="sovereignty.faction.id" :faction_name="sovereignty.faction.name" />
            </div>
        </TooltipTrigger>
        <TooltipContent class="text-sm">
            <div v-if="sovereignty.alliance" class="flex items-center gap-2">
                <AllianceLogo :alliance_id="sovereignty.alliance.id" :alliance_name="sovereignty.alliance.name" class="size-6" />
                <span class="text-sm">{{ sovereignty.alliance.name }}</span>
                <span class="text-xs text-muted-foreground" v-if="sovereignty.alliance.ticker">({{ sovereignty.alliance.ticker }})</span>
            </div>
            <div v-else-if="sovereignty.corporation" class="flex items-center gap-2">
                <CorporationLogo :corporation_id="sovereignty.corporation.id" :corporation_name="sovereignty.corporation.name" class="size-6" />
                <span class="text-sm">{{ sovereignty.corporation.name }}</span>
                <span class="text-xs text-muted-foreground" v-if="sovereignty.corporation.ticker">({{ sovereignty.corporation.ticker }})</span>
            </div>
            <div v-else-if="sovereignty.faction" class="flex items-center gap-2">
                <FactionLogo :faction_id="sovereignty.faction.id" :faction_name="sovereignty.faction.name" class="size-6" />
                <span class="text-sm">{{ sovereignty.faction.name }}</span>
            </div>
        </TooltipContent>
    </Tooltip>
</template>

<style scoped></style>

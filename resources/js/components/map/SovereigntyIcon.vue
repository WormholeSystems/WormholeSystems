<script setup lang="ts">
import AllianceLogo from '@/components/images/AllianceLogo.vue';
import CorporationLogo from '@/components/images/CorporationLogo.vue';
import FactionLogo from '@/components/images/FactionLogo.vue';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { TSovereignty } from '@/types/models';

const { sovereignty } = defineProps<{
    sovereignty: TSovereignty;
}>();
</script>

<template>
    <Popover>
        <PopoverTrigger class="pointer-events-auto cursor-pointer">
            <AllianceLogo
                v-if="sovereignty.alliance"
                :alliance_id="sovereignty.alliance.id"
                :alliance_name="sovereignty.alliance.name"
                class="size-4"
            />
            <CorporationLogo
                v-else-if="sovereignty.corporation"
                :corporation_id="sovereignty.corporation.id"
                :corporation_name="sovereignty.corporation.name"
                class="size-4"
            />
            <FactionLogo
                v-else-if="sovereignty.faction"
                :faction_id="sovereignty.faction.id"
                :faction_name="sovereignty.faction.name"
                class="size-4"
            />
        </PopoverTrigger>
        <PopoverContent side="bottom" class="text-sm">
            <div v-if="sovereignty.alliance" class="flex items-center gap-2">
                <AllianceLogo :alliance_id="sovereignty.alliance.id" :alliance_name="sovereignty.alliance.name" class="size-6" />
                <span class="text-sm">{{ sovereignty.alliance.name }}</span>
                <span class="text-xs text-muted-foreground">({{ sovereignty.alliance.ticker }})</span>
            </div>
            <div v-else-if="sovereignty.corporation" class="flex items-center gap-2">
                <CorporationLogo :corporation_id="sovereignty.corporation.id" :corporation_name="sovereignty.corporation.name" class="size-6" />
                <span class="text-sm">{{ sovereignty.corporation.name }}</span>
                <span class="text-xs text-muted-foreground">({{ sovereignty.corporation.ticker }})</span>
            </div>
            <div v-else-if="sovereignty.faction" class="flex items-center gap-2">
                <FactionLogo :faction_id="sovereignty.faction.id" :faction_name="sovereignty.faction.name" class="size-6" />
                <span class="text-sm">{{ sovereignty.faction.name }}</span>
            </div>
        </PopoverContent>
    </Popover>
</template>

<style scoped></style>

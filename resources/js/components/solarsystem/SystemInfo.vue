<script setup lang="ts">
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import StaticDetails from '@/components/solarsystem/StaticDetails.vue';
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
import MapPanelHeader from '@/components/ui/map-panel/MapPanelHeader.vue';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { useSovereignty } from '@/composables/useSovereigntyData';
import { isWormholeClass } from '@/const/solarsystemClasses';
import type { TResolvedSelectedMapSolarsystem } from '@/pages/maps';
import { computed } from 'vue';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TResolvedSelectedMapSolarsystem;
}>();

const system = computed(() => map_solarsystem.solarsystem);

// Load sovereignty data for k-space systems
const sovereignty = useSovereignty(() => system.value?.id);

// Check if sovereignty has actual data
const hasSovereignty = computed(() => {
    return sovereignty.value?.alliance || sovereignty.value?.corporation || sovereignty.value?.faction;
});

// Effect color
const effectColor = computed(() => {
    const effect = system.value?.effect?.name?.toLowerCase() ?? '';
    if (effect.includes('pulsar')) return 'text-blue-400';
    if (effect.includes('magnetar')) return 'text-pink-400';
    if (effect.includes('wolf')) return 'text-amber-600';
    if (effect.includes('black')) return 'text-neutral-400';
    if (effect.includes('cataclysmic')) return 'text-yellow-400';
    if (effect.includes('red giant')) return 'text-red-400';
    return 'text-muted-foreground';
});

// Dotlan link
const dotlanLink = computed(() => {
    const region = system.value?.region?.name?.replace(/ /g, '_');
    const name = system.value?.name?.replace(/ /g, '_');
    if (isWormholeClass(system.value?.class)) {
        return `https://evemaps.dotlan.net/system/${name}`;
    }
    return `https://evemaps.dotlan.net/map/${region}/${name}`;
});
</script>

<template>
    <MapPanel>
        <MapPanelHeader> System </MapPanelHeader>
        <MapPanelContent>
            <!-- Hero: System Name -->
            <div class="border-b border-border/50 px-3 py-3">
                <div class="flex items-center gap-2">
                    <SolarsystemClass :solarsystem_class="system?.class" />
                    <span class="truncate text-sm font-medium">
                        {{ map_solarsystem.alias || system?.name }}
                        <span v-if="map_solarsystem.alias" class="text-muted-foreground">({{ system?.name }})</span>
                    </span>
                    <span v-if="system?.effect?.name" class="shrink-0 text-[10px]" :class="effectColor">
                        {{ system.effect.name }}
                    </span>
                    <span v-if="system?.is_shattered" class="shrink-0 text-[10px] text-amber-500">Shattered</span>
                </div>
                <div v-if="map_solarsystem.occupier_alias" class="mt-1 text-[11px] text-muted-foreground">
                    Occupied by <span class="font-medium text-foreground">{{ map_solarsystem.occupier_alias }}</span>
                </div>
                <div class="mt-1 flex items-center gap-1 text-[11px] text-muted-foreground">
                    <span>{{ system?.region?.name }}</span>
                    <span v-if="system?.constellation?.name">· {{ system.constellation.name }}</span>
                    <span class="text-border">·</span>
                    <a
                        :href="`https://zkillboard.com/system/${system?.id}/`"
                        target="_blank"
                        rel="noopener"
                        class="transition-colors hover:text-foreground"
                        >zKill</a
                    >
                    <a :href="dotlanLink" target="_blank" rel="noopener" class="transition-colors hover:text-foreground">Dotlan</a>
                    <a
                        v-if="isWormholeClass(system?.class)"
                        :href="`https://anoik.is/systems/${system?.name}`"
                        target="_blank"
                        rel="noopener"
                        class="transition-colors hover:text-foreground"
                        >Anoik</a
                    >
                </div>
            </div>

            <!-- Statics (wormhole only) -->
            <div v-if="system?.statics?.length" class="border-b border-border/50 px-3 py-2">
                <div class="flex items-center gap-2">
                    <span class="text-[10px] tracking-wider text-muted-foreground uppercase">Statics</span>
                    <div class="flex gap-1.5">
                        <Popover v-for="(wh, idx) in system.statics" :key="idx">
                            <PopoverTrigger as-child>
                                <button
                                    :data-leads-to="wh.leads_to"
                                    class="font-mono text-xs font-medium transition-opacity hover:opacity-70 data-[leads-to=c1]:text-c1 data-[leads-to=c2]:text-c2 data-[leads-to=c3]:text-c3 data-[leads-to=c4]:text-c4 data-[leads-to=c5]:text-c5 data-[leads-to=c6]:text-c6 data-[leads-to=hs]:text-hs data-[leads-to=ls]:text-ls data-[leads-to=ns]:text-ns"
                                >
                                    {{ wh.name }}
                                    <span class="uppercase opacity-60">{{ wh.leads_to.replace('s', '') }}</span>
                                </button>
                            </PopoverTrigger>
                            <PopoverContent class="w-48 p-0" align="start">
                                <StaticDetails :wormhole="wh" />
                            </PopoverContent>
                        </Popover>
                    </div>
                </div>
            </div>

            <!-- Effect Details (wormhole only) -->
            <div v-if="system?.effect?.effects?.length" class="border-b border-border/50 px-3 py-2">
                <div class="flex flex-col gap-1">
                    <span class="text-[10px] tracking-wider text-muted-foreground uppercase">Effect</span>
                    <div class="grid grid-cols-2 gap-x-4 gap-y-0.5">
                        <div v-for="effect in system.effect.effects" :key="effect.id" class="flex items-center justify-between text-[11px]">
                            <span class="truncate text-muted-foreground">{{ effect.name }}</span>
                            <span :class="effect.type === 'Buff' ? 'text-green-400' : 'text-red-400'">
                                {{ effect.strength }}
                            </span>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Sovereignty (k-space only) -->
            <div v-if="hasSovereignty" class="px-3 py-2">
                <div class="flex flex-col gap-1.5">
                    <span class="text-[10px] tracking-wider text-muted-foreground uppercase">Sovereignty</span>
                    <!-- Alliance -->
                    <div v-if="sovereignty?.alliance" class="flex items-center gap-2">
                        <img
                            :src="`https://images.evetech.net/alliances/${sovereignty.alliance.id}/logo?size=32`"
                            :alt="sovereignty.alliance.name"
                            class="size-5 rounded"
                        />
                        <span class="text-xs">
                            <span class="font-medium">[{{ sovereignty.alliance.ticker }}]</span>
                            {{ sovereignty.alliance.name }}
                        </span>
                    </div>
                    <!-- Corporation (only if no alliance) -->
                    <div v-else-if="sovereignty?.corporation" class="flex items-center gap-2">
                        <img
                            :src="`https://images.evetech.net/corporations/${sovereignty.corporation.id}/logo?size=32`"
                            :alt="sovereignty.corporation.name"
                            class="size-5 rounded"
                        />
                        <span class="text-xs">
                            <span class="font-medium">[{{ sovereignty.corporation.ticker }}]</span>
                            {{ sovereignty.corporation.name }}
                        </span>
                    </div>
                    <!-- Faction -->
                    <div v-else-if="sovereignty?.faction" class="flex items-center gap-2">
                        <img
                            :src="`https://images.evetech.net/corporations/${sovereignty.faction.id}/logo?size=32`"
                            :alt="sovereignty.faction.name"
                            class="size-5 rounded"
                        />
                        <span class="text-xs">{{ sovereignty.faction.name }}</span>
                    </div>
                </div>
            </div>
        </MapPanelContent>
    </MapPanel>
</template>

<script setup lang="ts">
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
import MapPanelHeader from '@/components/ui/map-panel/MapPanelHeader.vue';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { useSovereignty } from '@/composables/useSovereigntyData';
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
    if (system.value?.class) {
        return `https://evemaps.dotlan.net/system/${name}`;
    }
    return `https://evemaps.dotlan.net/map/${region}/${name}`;
});

// Static leads-to colors
function getLeadsToColor(leadsTo: string): string {
    if (leadsTo.startsWith('C')) {
        const num = parseInt(leadsTo.slice(1));
        if (num === 1) return 'text-cyan-400';
        if (num === 2) return 'text-blue-400';
        if (num === 3) return 'text-purple-400';
        if (num === 4) return 'text-violet-400';
        if (num === 5) return 'text-orange-400';
        if (num === 6) return 'text-red-400';
    }
    if (leadsTo === 'Highsec' || leadsTo === 'H') return 'text-green-400';
    if (leadsTo === 'Lowsec' || leadsTo === 'L') return 'text-amber-400';
    if (leadsTo === 'Nullsec' || leadsTo === 'N') return 'text-red-400';
    return 'text-muted-foreground';
}

function formatLeadsTo(leadsTo: string): string {
    if (leadsTo === 'Highsec') return 'HS';
    if (leadsTo === 'Lowsec') return 'LS';
    if (leadsTo === 'Nullsec') return 'NS';
    return leadsTo;
}

function formatMass(mass: number | null): string {
    if (!mass) return '—';
    if (mass >= 1_000_000_000) return `${(mass / 1_000_000_000).toFixed(1)}B kg`;
    if (mass >= 1_000_000) return `${(mass / 1_000_000).toFixed(0)}M kg`;
    return `${mass.toLocaleString()} kg`;
}

function formatLifetime(hours: number | null): string {
    if (!hours) return '—';
    return `${hours}h`;
}
</script>

<template>
    <MapPanel>
        <MapPanelHeader> System </MapPanelHeader>
        <MapPanelContent>
            <!-- Hero: System Name -->
            <div class="border-b border-border/50 px-3 py-3">
                <div class="flex items-center gap-2">
                    <SolarsystemClass :wormhole_class="system?.class" :security="system?.security" />
                    <span class="truncate text-sm font-medium">
                        {{ map_solarsystem.alias || system?.name }}
                        <span v-if="map_solarsystem.alias" class="text-muted-foreground">({{ system?.name }})</span>
                    </span>
                    <span v-if="system?.effect?.name" class="shrink-0 text-[10px]" :class="effectColor">
                        {{ system.effect.name }}
                    </span>
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
                        v-if="system?.class"
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
                                    class="font-mono text-xs font-medium transition-opacity hover:opacity-70"
                                    :class="getLeadsToColor(wh.leads_to)"
                                >
                                    {{ wh.name }}
                                    <span class="opacity-60">{{ formatLeadsTo(wh.leads_to) }}</span>
                                </button>
                            </PopoverTrigger>
                            <PopoverContent class="w-44 p-0" align="start">
                                <div class="border-b border-border/50 px-3 py-2">
                                    <div class="font-mono text-xs font-medium" :class="getLeadsToColor(wh.leads_to)">
                                        {{ wh.name }} → {{ wh.leads_to }}
                                    </div>
                                </div>
                                <div class="space-y-1 px-3 py-2 text-[11px]">
                                    <div class="flex justify-between">
                                        <span class="text-muted-foreground">Mass</span>
                                        <span>{{ formatMass(wh.total_mass) }}</span>
                                    </div>
                                    <div class="flex justify-between">
                                        <span class="text-muted-foreground">Jump</span>
                                        <span>{{ formatMass(wh.maximum_jump_mass) }}</span>
                                    </div>
                                    <div class="flex justify-between">
                                        <span class="text-muted-foreground">Lifetime</span>
                                        <span>{{ formatLifetime(wh.maximum_lifetime) }}</span>
                                    </div>
                                </div>
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

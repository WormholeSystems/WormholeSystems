<script setup lang="ts">
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { ScrollArea } from '@/components/ui/scroll-area';
import { TUniverseSystemDetails } from '@/types/universe-map';
import { formatDistanceToNow } from 'date-fns';
import { Building2, ChevronDown, Circle, Gem, Globe, MapPin, Moon, Navigation, Orbit, Shield, Skull, Users, X } from 'lucide-vue-next';
import { computed } from 'vue';

// Planet type colors based on EVE planet classification
function getPlanetStyle(type: string | null): { color: string; bg: string } {
    if (!type) return { color: 'text-neutral-400', bg: 'bg-neutral-400' };

    const t = type.toLowerCase();
    if (t.includes('temperate')) return { color: 'text-green-400', bg: 'bg-green-400' };
    if (t.includes('oceanic')) return { color: 'text-blue-400', bg: 'bg-blue-400' };
    if (t.includes('ice')) return { color: 'text-sky-300', bg: 'bg-sky-300' };
    if (t.includes('gas')) return { color: 'text-neutral-400', bg: 'bg-neutral-400' };
    if (t.includes('lava')) return { color: 'text-red-500', bg: 'bg-red-500' };
    if (t.includes('barren')) return { color: 'text-orange-400', bg: 'bg-orange-400' };
    if (t.includes('storm')) return { color: 'text-yellow-400', bg: 'bg-yellow-400' };
    if (t.includes('plasma')) return { color: 'text-purple-400', bg: 'bg-purple-400' };
    if (t.includes('shattered')) return { color: 'text-gray-500', bg: 'bg-gray-500' };
    return { color: 'text-neutral-400', bg: 'bg-neutral-400' };
}

const props = defineProps<{
    details: TUniverseSystemDetails | null;
}>();

const emit = defineEmits<{
    close: [];
    'focus-region': [regionId: number];
    'focus-constellation': [constellationId: number];
    'focus-system': [systemId: number];
    'select-system': [systemId: number];
}>();

const securityColor = computed(() => {
    if (!props.details) return 'text-neutral-400';
    const sec = props.details.security;
    if (sec >= 0.5) return 'text-green-400';
    if (sec > 0) return 'text-yellow-400';
    return 'text-red-400';
});

const securityLabel = computed(() => {
    if (!props.details) return '';
    const sec = props.details.security;
    if (sec >= 0.5) return 'High Sec';
    if (sec > 0) return 'Low Sec';
    return 'Null Sec';
});

const sovHolder = computed(() => {
    if (!props.details?.sovereignty) return null;
    const sov = props.details.sovereignty;
    if (sov.alliance) return { type: 'Alliance', ...sov.alliance };
    if (sov.corporation) return { type: 'Corporation', ...sov.corporation };
    if (sov.faction) return { type: 'Faction', name: sov.faction.name, id: sov.faction.id };
    return null;
});

const sovLogoUrl = computed(() => {
    if (!sovHolder.value) return null;
    if (sovHolder.value.type === 'Alliance') {
        return `https://images.evetech.net/alliances/${sovHolder.value.id}/logo?size=64`;
    }
    if (sovHolder.value.type === 'Corporation' || sovHolder.value.type === 'Faction') {
        return `https://images.evetech.net/corporations/${sovHolder.value.id}/logo?size=64`;
    }
    return null;
});

function formatIsk(value: number): string {
    if (value >= 1_000_000_000) return `${(value / 1_000_000_000).toFixed(1)}B`;
    if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
    if (value >= 1_000) return `${(value / 1_000).toFixed(1)}K`;
    return value.toString();
}
</script>

<template>
    <Transition
        enter-active-class="transition-transform duration-300 ease-out"
        enter-from-class="translate-x-full"
        enter-to-class="translate-x-0"
        leave-active-class="transition-transform duration-200 ease-in"
        leave-from-class="translate-x-0"
        leave-to-class="translate-x-full"
    >
        <div
            v-if="details"
            class="absolute top-0 right-0 bottom-0 z-20 flex w-[420px] flex-col border-l border-neutral-800 bg-neutral-950/95 backdrop-blur-sm"
        >
            <!-- Header -->
            <div class="flex items-start justify-between border-b border-neutral-800 p-4">
                <div>
                    <div class="flex items-center gap-2">
                        <button
                            @click="emit('focus-system', details.id)"
                            class="text-xl font-bold transition-colors hover:text-yellow-400 hover:underline"
                        >
                            {{ details.name }}
                        </button>
                        <span :class="securityColor" class="font-mono text-sm">
                            {{ details.security.toFixed(1) }}
                        </span>
                    </div>
                    <div class="mt-1 flex items-center gap-2 text-sm text-neutral-400">
                        <MapPin class="h-3.5 w-3.5" />
                        <button
                            @click="emit('focus-constellation', details.constellation.id)"
                            class="transition-colors hover:text-white hover:underline"
                        >
                            {{ details.constellation.name }}
                        </button>
                        <span>•</span>
                        <button @click="emit('focus-region', details.region.id)" class="transition-colors hover:text-white hover:underline">
                            {{ details.region.name }}
                        </button>
                    </div>
                </div>
                <button @click="emit('close')" class="rounded p-1 text-neutral-400 transition-colors hover:bg-neutral-800 hover:text-white">
                    <X class="h-5 w-5" />
                </button>
            </div>

            <!-- Content -->
            <ScrollArea class="flex-1">
                <div class="space-y-4 p-4">
                    <!-- Security Status -->
                    <div class="flex items-center gap-4 rounded-lg bg-neutral-900/50 p-3">
                        <Shield class="h-5 w-5" :class="securityColor" />
                        <div>
                            <div class="text-sm font-medium" :class="securityColor">{{ securityLabel }}</div>
                            <div class="text-xs text-neutral-500">Security Status: {{ details.security.toFixed(2) }}</div>
                        </div>
                    </div>

                    <!-- Sovereignty -->
                    <div v-if="sovHolder" class="flex items-center gap-4 rounded-lg bg-neutral-900/50 p-3">
                        <img v-if="sovLogoUrl" :src="sovLogoUrl" :alt="sovHolder.name" class="h-10 w-10 rounded" />
                        <Users v-else class="h-5 w-5 text-blue-400" />
                        <div>
                            <div class="text-sm font-medium text-blue-400">{{ sovHolder.name }}</div>
                            <div class="text-xs text-neutral-500">{{ sovHolder.type }} Sovereignty</div>
                        </div>
                    </div>

                    <!-- Wormhole Class -->
                    <div v-if="details.wormhole_class" class="rounded-lg bg-purple-900/20 p-3">
                        <div class="text-sm font-medium text-purple-400">Wormhole Class {{ details.wormhole_class }}</div>
                        <div class="text-xs text-neutral-500">J-Space System</div>
                    </div>

                    <!-- Adjacent Systems -->
                    <Collapsible v-if="details.adjacent_systems.length" default-open class="rounded-lg bg-neutral-900/50">
                        <CollapsibleTrigger class="flex w-full items-center justify-between rounded-lg p-3 transition-colors hover:bg-neutral-800/50">
                            <div class="flex items-center gap-2">
                                <Navigation class="h-4 w-4 text-cyan-400" />
                                <span class="text-sm font-medium text-neutral-300">Adjacent Systems ({{ details.adjacent_systems.length }})</span>
                            </div>
                            <ChevronDown class="h-4 w-4 text-neutral-500 transition-transform [[data-state=open]>&]:rotate-180" />
                        </CollapsibleTrigger>
                        <CollapsibleContent class="px-3 pb-3">
                            <div class="space-y-1 border-t border-neutral-800 pt-1">
                                <button
                                    v-for="system in details.adjacent_systems"
                                    :key="system.id"
                                    @click="emit('select-system', system.id)"
                                    class="-mx-1 flex w-full items-center gap-2 rounded px-1 py-1.5 text-xs transition-colors hover:bg-neutral-800/50"
                                >
                                    <span
                                        class="h-1.5 w-1.5 shrink-0 rounded-full"
                                        :class="system.security >= 0.5 ? 'bg-green-400' : system.security > 0 ? 'bg-yellow-400' : 'bg-red-400'"
                                    />
                                    <span class="flex-1 text-left text-neutral-300">{{ system.name }}</span>
                                    <span v-if="system.region_id !== details.region.id" class="w-24 shrink-0 truncate text-right text-neutral-500">
                                        {{ system.region_name }}
                                    </span>
                                    <span
                                        class="w-8 shrink-0 text-right font-mono"
                                        :class="system.security >= 0.5 ? 'text-green-400' : system.security > 0 ? 'text-yellow-400' : 'text-red-400'"
                                    >
                                        {{ system.security.toFixed(1) }}
                                    </span>
                                </button>
                            </div>
                        </CollapsibleContent>
                    </Collapsible>

                    <!-- Planets -->
                    <Collapsible v-if="details.planets.length" class="rounded-lg bg-neutral-900/50">
                        <CollapsibleTrigger class="flex w-full items-center justify-between rounded-lg p-3 transition-colors hover:bg-neutral-800/50">
                            <div class="flex items-center gap-2">
                                <Orbit class="h-4 w-4 text-amber-400" />
                                <span class="text-sm font-medium text-neutral-300">Planets ({{ details.planets.length }})</span>
                                <span v-if="details.moons_count" class="text-xs text-neutral-500">• {{ details.moons_count }} moons</span>
                            </div>
                            <ChevronDown class="h-4 w-4 text-neutral-500 transition-transform [[data-state=open]>&]:rotate-180" />
                        </CollapsibleTrigger>
                        <CollapsibleContent class="px-3 pb-3">
                            <div class="space-y-1 border-t border-neutral-800 pt-1">
                                <Collapsible v-for="planet in details.planets" :key="planet.id" class="group">
                                    <CollapsibleTrigger
                                        class="-mx-1 flex w-full items-center gap-2 rounded px-1 py-1.5 text-xs hover:bg-neutral-800/30"
                                        :class="planet.moons.length ? 'cursor-pointer' : 'cursor-default'"
                                    >
                                        <Circle class="h-1.5 w-1.5 fill-current" :class="getPlanetStyle(planet.type).color" />
                                        <span class="flex-1 text-left text-neutral-300">{{ planet.name }}</span>
                                        <span v-if="planet.type" class="text-neutral-500">{{ planet.type }}</span>
                                        <template v-if="planet.moons.length">
                                            <span class="text-[10px] text-neutral-600">{{ planet.moons.length }}</span>
                                            <Moon class="h-3 w-3 text-neutral-600" />
                                            <ChevronDown class="h-3 w-3 text-neutral-600 transition-transform [[data-state=open]>&]:rotate-180" />
                                        </template>
                                    </CollapsibleTrigger>
                                    <CollapsibleContent v-if="planet.moons.length" class="ml-4 border-l border-neutral-800 pl-2">
                                        <div
                                            v-for="moon in planet.moons"
                                            :key="moon.id"
                                            class="flex items-center gap-2 py-0.5 text-[11px] text-neutral-500"
                                        >
                                            <Moon class="h-2.5 w-2.5" />
                                            <span>{{ moon.name }}</span>
                                        </div>
                                    </CollapsibleContent>
                                </Collapsible>
                            </div>
                        </CollapsibleContent>
                    </Collapsible>

                    <!-- Stations -->
                    <Collapsible v-if="details.stations.length" class="rounded-lg bg-neutral-900/50">
                        <CollapsibleTrigger class="flex w-full items-center justify-between rounded-lg p-3 transition-colors hover:bg-neutral-800/50">
                            <div class="flex items-center gap-2">
                                <Building2 class="h-4 w-4 text-blue-400" />
                                <span class="text-sm font-medium text-neutral-300">Stations ({{ details.stations.length }})</span>
                            </div>
                            <ChevronDown class="h-4 w-4 text-neutral-500 transition-transform [[data-state=open]>&]:rotate-180" />
                        </CollapsibleTrigger>
                        <CollapsibleContent class="px-3 pb-3">
                            <div class="space-y-1 border-t border-neutral-800 pt-1">
                                <div v-for="station in details.stations" :key="station.id" class="truncate py-1 text-xs text-neutral-400">
                                    {{ station.name }}
                                </div>
                            </div>
                        </CollapsibleContent>
                    </Collapsible>

                    <!-- Asteroid Belts -->
                    <Collapsible v-if="details.belts.length" class="rounded-lg bg-neutral-900/50">
                        <CollapsibleTrigger class="flex w-full items-center justify-between rounded-lg p-3 transition-colors hover:bg-neutral-800/50">
                            <div class="flex items-center gap-2">
                                <Gem class="h-4 w-4 text-amber-400" />
                                <span class="text-sm font-medium text-neutral-300">Asteroid Belts ({{ details.belts.length }})</span>
                            </div>
                            <ChevronDown class="h-4 w-4 text-neutral-500 transition-transform [[data-state=open]>&]:rotate-180" />
                        </CollapsibleTrigger>
                        <CollapsibleContent class="px-3 pb-3">
                            <div class="grid grid-cols-2 gap-1 border-t border-neutral-800 pt-1">
                                <div v-for="belt in details.belts" :key="belt.id" class="truncate py-0.5 text-xs text-neutral-400">
                                    {{ belt.name.replace(details.name + ' ', '') }}
                                </div>
                            </div>
                        </CollapsibleContent>
                    </Collapsible>

                    <!-- Stargates -->
                    <Collapsible v-if="details.stargates.length" class="rounded-lg bg-neutral-900/50">
                        <CollapsibleTrigger class="flex w-full items-center justify-between rounded-lg p-3 transition-colors hover:bg-neutral-800/50">
                            <div class="flex items-center gap-2">
                                <Navigation class="h-4 w-4 text-cyan-400" />
                                <span class="text-sm font-medium text-neutral-300">Stargates ({{ details.stargates.length }})</span>
                            </div>
                            <ChevronDown class="h-4 w-4 text-neutral-500 transition-transform [[data-state=open]>&]:rotate-180" />
                        </CollapsibleTrigger>
                        <CollapsibleContent class="px-3 pb-3">
                            <div class="space-y-1 border-t border-neutral-800 pt-1">
                                <div v-for="gate in details.stargates" :key="gate.id" class="py-1 text-xs text-neutral-400">
                                    {{ gate.name.replace('Stargate (', '').replace(')', '') }}
                                </div>
                            </div>
                        </CollapsibleContent>
                    </Collapsible>

                    <!-- Recent Killmails -->
                    <Collapsible v-if="details.killmails.length" class="rounded-lg bg-neutral-900/50">
                        <CollapsibleTrigger class="flex w-full items-center justify-between rounded-lg p-3 transition-colors hover:bg-neutral-800/50">
                            <div class="flex items-center gap-2">
                                <Skull class="h-4 w-4 text-red-400" />
                                <span class="text-sm font-medium text-neutral-300">Recent Kills ({{ details.killmails.length }})</span>
                            </div>
                            <ChevronDown class="h-4 w-4 text-neutral-500 transition-transform [[data-state=open]>&]:rotate-180" />
                        </CollapsibleTrigger>
                        <CollapsibleContent class="px-3 pb-3">
                            <div class="space-y-2 border-t border-neutral-800 pt-1">
                                <a
                                    v-for="km in details.killmails"
                                    :key="km.id"
                                    :href="`https://zkillboard.com/kill/${km.id}/`"
                                    target="_blank"
                                    class="flex items-center gap-2 rounded p-1 text-xs transition-colors hover:bg-neutral-800"
                                >
                                    <img
                                        v-if="km.ship_type_id"
                                        :src="`https://images.evetech.net/types/${km.ship_type_id}/icon?size=32`"
                                        class="h-6 w-6 rounded"
                                    />
                                    <div class="flex-1 truncate">
                                        <div class="text-neutral-300">{{ km.ship_type || 'Unknown Ship' }}</div>
                                        <div class="text-neutral-500">
                                            {{ formatDistanceToNow(new Date(km.time), { addSuffix: true }) }}
                                        </div>
                                    </div>
                                    <div class="text-red-400">{{ formatIsk(km.zkb.totalValue) }}</div>
                                </a>
                            </div>
                        </CollapsibleContent>
                    </Collapsible>

                    <!-- No Killmails -->
                    <div v-else class="rounded-lg bg-neutral-900/50 p-3">
                        <div class="flex items-center gap-2 text-sm text-neutral-500">
                            <Skull class="h-4 w-4" />
                            No recent kills
                        </div>
                    </div>

                    <!-- Location Details -->
                    <div class="rounded-lg bg-neutral-900/50 p-3">
                        <div class="mb-2 flex items-center gap-2">
                            <Globe class="h-4 w-4 text-neutral-400" />
                            <span class="text-sm font-medium text-neutral-300">Location</span>
                        </div>
                        <div class="grid grid-cols-2 gap-2 text-xs">
                            <div>
                                <span class="text-neutral-500">Region:</span>
                                <button
                                    @click="emit('focus-region', details.region.id)"
                                    class="ml-1 text-neutral-300 transition-colors hover:text-white hover:underline"
                                >
                                    {{ details.region.name }}
                                </button>
                            </div>
                            <div>
                                <span class="text-neutral-500">Constellation:</span>
                                <button
                                    @click="emit('focus-constellation', details.constellation.id)"
                                    class="ml-1 text-neutral-300 transition-colors hover:text-white hover:underline"
                                >
                                    {{ details.constellation.name }}
                                </button>
                            </div>
                            <div>
                                <span class="text-neutral-500">System ID:</span>
                                <span class="ml-1 font-mono text-neutral-300">{{ details.id }}</span>
                            </div>
                            <div>
                                <span class="text-neutral-500">Type:</span>
                                <span class="ml-1 text-neutral-300">{{ details.type }}</span>
                            </div>
                        </div>
                    </div>
                </div>
            </ScrollArea>
        </div>
    </Transition>
</template>

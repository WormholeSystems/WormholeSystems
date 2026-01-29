<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import QuickSelectButtons from '@/components/autopilot/QuickSelectButtons.vue';
import SystemComboboxList from '@/components/autopilot/SystemComboboxList.vue';
import ExtraWormholeIcon from '@/components/icons/ExtraWormholeIcon.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import SolarsystemEffect from '@/components/solarsystem/SolarsystemEffect.vue';
import { Button } from '@/components/ui/button';
import { Combobox, ComboboxAnchor, ComboboxInput } from '@/components/ui/combobox';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useIgnoreList } from '@/composables/useIgnoreList';
import { usePath } from '@/composables/usePath';
import { useRouteCalculator } from '@/composables/useRouteCalculator';
import { useStaticSolarsystem, useStaticSolarsystems } from '@/composables/useStaticSolarsystems';
import type { TMap, TResolvedMapRouteSolarsystem, TResolvedSelectedMapSolarsystem, TResolvedSolarsystem } from '@/pages/maps';
import type { ConnectionType } from '@/routing/types';
import type { TCharacter, TCharacterStatus } from '@/types/models';
import type { TStaticSolarsystem } from '@/types/static-data';
import { vElementHover } from '@vueuse/components';
import { X } from 'lucide-vue-next';
import { computed, ref, watch } from 'vue';

const { map, solarsystems, selected_map_solarsystem, ignored_systems, active_character, character_status, destinations } = defineProps<{
    map: TMap;
    solarsystems: TStaticSolarsystem[];
    selected_map_solarsystem?: TResolvedSelectedMapSolarsystem | null;
    ignored_systems: number[];
    active_character?: TCharacter | null;
    character_status?: TCharacterStatus | null;
    destinations: TResolvedMapRouteSolarsystem[];
}>();

const mapConnections = computed(() => map.map_connections ?? []);
const mapSolarsystems = computed(() => map.map_solarsystems ?? []);
const { ignoreSolarsystem, clearIgnoreList } = useIgnoreList();
const { setPath } = usePath();

const search = ref('');
const fromSystem = ref<TResolvedSolarsystem | null>(null);
const toSystem = ref<TResolvedSolarsystem | null>(null);

const { route: routeSteps, jumps: routeJumps } = useRouteCalculator({
    fromId: computed(() => fromSystem.value?.id ?? null),
    toId: computed(() => toSystem.value?.id ?? null),
    ignoredSystems: computed(() => ignored_systems),
    mapConnections,
    mapSolarsystems,
});

const { resolveSolarsystem } = useStaticSolarsystems();

type RouteEntry = {
    solarsystem: TStaticSolarsystem;
    connection_type: ConnectionType | null;
};

const enrichedRoute = computed<RouteEntry[]>(() =>
    routeSteps.value
        .map((step, index) => {
            const solarsystem = resolveSolarsystem(step.id);
            if (!solarsystem) {
                return null;
            }

            return {
                solarsystem,
                connection_type: routeSteps.value[index + 1]?.via ?? null,
            } as RouteEntry;
        })
        .filter((entry): entry is RouteEntry => Boolean(entry)),
);

const routeSolarsystems = computed(() => enrichedRoute.value.map((entry) => entry.solarsystem));
const hasRoute = computed(() => enrichedRoute.value.length > 0);
const activeCharacterSystem = useStaticSolarsystem(() => (active_character ? (character_status?.solarsystem_id ?? null) : null));

const filteredSolarsystems = computed(() => {
    const query = search.value.trim().toLowerCase();
    if (!query) {
        return [] as TStaticSolarsystem[];
    }

    return solarsystems.filter((solarsystem) => solarsystem.name.toLowerCase().includes(query)).slice(0, 25);
});

watch(
    () => routeSolarsystems.value,
    (value) => {
        if (value.length > 0) {
            setPath(value);
            return;
        }

        setPath(null);
    },
    { immediate: true, deep: true },
);

function handleFromSystemSelect(system: TResolvedSolarsystem) {
    fromSystem.value = system;
    search.value = '';
}

function handleToSystemSelect(system: TResolvedSolarsystem) {
    toSystem.value = system;
    search.value = '';
}

function onRouteHover(hovered: boolean) {
    setPath(hovered ? routeSolarsystems.value : null);
}

function handleIgnoreSolarsystem(solarsystem_id: number) {
    ignoreSolarsystem(solarsystem_id, {
        only: ['map_navigation', 'ignored_systems'],
    });
}

function handleClearIgnoreList() {
    clearIgnoreList({
        only: ['map_navigation', 'ignored_systems'],
    });
}

function clearFrom() {
    fromSystem.value = null;
}

function clearTo() {
    toSystem.value = null;
}
</script>

<template>
    <div class="bg-card p-6 pt-2">
        <div class="mb-1.5">
            <span class="text-xs font-medium text-muted-foreground">From</span>
        </div>
        <Combobox>
            <ComboboxAnchor class="w-full">
                <template v-if="fromSystem">
                    <div class="flex items-center-safe gap-2 rounded-lg border p-2">
                        <SolarsystemClass :wormhole_class="fromSystem.class" :security="fromSystem.security" class="shrink-0" />
                        <div class="min-w-0 flex-1 text-xs">
                            <span class="font-medium">{{ fromSystem.name }}</span>
                            <span class="text-muted-foreground"> · {{ fromSystem.region?.name }}</span>
                        </div>
                        <SolarsystemSovereignty :sovereignty="fromSystem.sovereignty" :solarsystem-id="fromSystem.id" class="shrink-0">
                            <template #fallback>
                                <SolarsystemEffect v-if="fromSystem.effect" :effect="fromSystem.effect.name" class="shrink-0" />
                                <span v-else />
                            </template>
                        </SolarsystemSovereignty>
                        <Button variant="ghost" size="icon" @click="clearFrom" class="h-6 w-6 shrink-0">
                            <X class="size-3" />
                        </Button>
                    </div>
                </template>
                <template v-else>
                    <ComboboxInput v-model="search" placeholder="Search system..." class="rounded-lg border" />
                </template>
            </ComboboxAnchor>
            <SystemComboboxList :solarsystems="filteredSolarsystems" @select="handleFromSystemSelect" />
        </Combobox>
        <QuickSelectButtons
            v-if="!fromSystem"
            :selected_map_solarsystem="selected_map_solarsystem"
            :active_character_system="activeCharacterSystem"
            :destinations="destinations"
            @select-system="handleFromSystemSelect"
        />

        <!-- To System -->
        <div class="mt-4 mb-1.5">
            <span class="text-xs font-medium text-muted-foreground">To</span>
        </div>
        <Combobox>
            <ComboboxAnchor class="w-full">
                <template v-if="toSystem">
                    <div class="flex items-center-safe gap-2 rounded-lg border p-2">
                        <SolarsystemClass :wormhole_class="toSystem.class" :security="toSystem.security" class="shrink-0" />
                        <div class="min-w-0 flex-1 text-xs">
                            <span class="font-medium">{{ toSystem.name }}</span>
                            <span class="text-muted-foreground"> · {{ toSystem.region?.name }}</span>
                        </div>
                        <SolarsystemSovereignty :sovereignty="toSystem.sovereignty" :solarsystem-id="toSystem.id" class="shrink-0">
                            <template #fallback>
                                <SolarsystemEffect v-if="toSystem.effect" :effect="toSystem.effect.name" class="shrink-0" />
                                <span v-else />
                            </template>
                        </SolarsystemSovereignty>
                        <Button variant="ghost" size="icon" @click="clearTo" class="h-6 w-6 shrink-0">
                            <X class="size-3" />
                        </Button>
                    </div>
                </template>
                <template v-else>
                    <ComboboxInput v-model="search" placeholder="Search system..." class="rounded-lg border" />
                </template>
            </ComboboxAnchor>
            <SystemComboboxList :solarsystems="filteredSolarsystems" @select="handleToSystemSelect" />
        </Combobox>
        <QuickSelectButtons
            v-if="!toSystem"
            :selected_map_solarsystem="selected_map_solarsystem"
            :active_character_system="activeCharacterSystem"
            :destinations="destinations"
            @select-system="handleToSystemSelect"
        />
    </div>

    <!-- Route Controls -->
    <div v-if="fromSystem && toSystem" class="flex items-center justify-between p-2">
        <span v-if="hasRoute" class="text-sm text-muted-foreground">{{ routeJumps }} jumps</span>
        <Button v-if="ignored_systems.length > 0" variant="secondary" size="sm" @click="handleClearIgnoreList">
            Clear Ignored ({{ ignored_systems.length }})
        </Button>
    </div>

    <!-- Route Results -->
    <div v-if="hasRoute" v-element-hover="onRouteHover" class="grid grid-cols-[auto_auto_1fr_auto_auto_auto] rounded-lg border bg-card text-sm">
        <DestinationContextMenu v-for="(entry, index) in enrichedRoute" :key="entry.solarsystem.id" :solarsystem_id="entry.solarsystem.id">
            <div
                class="col-span-full grid grid-cols-subgrid items-center gap-2 border-b px-2.5 py-1.5 transition-colors last:border-b-0 hover:bg-accent/50"
            >
                <div class="flex items-center justify-center text-xs font-medium text-muted-foreground">
                    {{ index + 1 }}
                </div>
                <div class="justify-self-center">
                    <SolarsystemClass :wormhole_class="entry.solarsystem.class" :security="entry.solarsystem.security" />
                </div>
                <div class="min-w-0 text-xs">
                    <span class="font-medium">{{ entry.solarsystem.name }}</span>
                    <span class="text-muted-foreground"> · {{ entry.solarsystem.region?.name }}</span>
                </div>
                <div class="flex items-center justify-center">
                    <SolarsystemSovereignty :sovereignty="entry.solarsystem.sovereignty" :solarsystem-id="entry.solarsystem.id">
                        <template #fallback>
                            <SolarsystemEffect v-if="entry.solarsystem.effect" :effect="entry.solarsystem.effect.name" />
                        </template>
                    </SolarsystemSovereignty>
                </div>
                <div class="flex items-center justify-center">
                    <Tooltip
                        v-if="
                            index < enrichedRoute.length - 1 &&
                            (enrichedRoute[index]?.connection_type === 'wormhole' || enrichedRoute[index]?.connection_type === 'evescout')
                        "
                    >
                        <TooltipTrigger as-child>
                            <ExtraWormholeIcon
                                class="size-3.5"
                                :class="enrichedRoute[index]?.connection_type === 'evescout' ? 'text-blue-400' : 'text-amber-500'"
                            />
                        </TooltipTrigger>
                        <TooltipContent>
                            {{
                                enrichedRoute[index]?.connection_type === 'evescout'
                                    ? `EVE Scout to ${enrichedRoute[index + 1]?.solarsystem.name}`
                                    : `Take wormhole to ${enrichedRoute[index + 1]?.solarsystem.name}`
                            }}
                        </TooltipContent>
                    </Tooltip>
                </div>

                <div class="flex items-center justify-center">
                    <Tooltip v-if="index !== 0 && index !== enrichedRoute.length - 1">
                        <TooltipTrigger as-child>
                            <Button variant="ghost" size="icon" class="h-6 w-6" @click="handleIgnoreSolarsystem(entry.solarsystem.id)">
                                <X class="h-3 w-3" />
                            </Button>
                        </TooltipTrigger>
                        <TooltipContent>Ignore this system</TooltipContent>
                    </Tooltip>
                </div>
            </div>
        </DestinationContextMenu>
    </div>
    <div v-else-if="fromSystem && toSystem" class="flex items-center justify-center py-6 text-sm text-muted-foreground">
        No route found between the selected systems.
    </div>
    <div v-else class="flex flex-col items-center justify-center gap-2 py-6 text-center text-sm text-muted-foreground">
        <p>Select both a starting system and destination to calculate the shortest path.</p>
        <p v-if="selected_map_solarsystem || activeCharacterSystem" class="text-xs">Use the quick select buttons above to get started.</p>
    </div>
</template>

<style scoped></style>

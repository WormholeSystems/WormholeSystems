<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import SystemComboboxList from '@/components/autopilot/SystemComboboxList.vue';
import ExtraWormholeIcon from '@/components/icons/ExtraWormholeIcon.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import SolarsystemEffect from '@/components/solarsystem/SolarsystemEffect.vue';
import { Combobox, ComboboxAnchor } from '@/components/ui/combobox';
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
import { ArrowUpDown, MapPin, Navigation, Search, X } from 'lucide-vue-next';
import { ComboboxInput as RekaComboboxInput } from 'reka-ui';
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

const pinnedDestinations = computed(() => destinations.filter((dest) => dest.is_pinned).slice(0, 3));

const hasQuickPicks = computed(() => selected_map_solarsystem?.solarsystem || activeCharacterSystem.value || pinnedDestinations.value.length > 0);

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

function swapSystems() {
    const temp = fromSystem.value;
    fromSystem.value = toSystem.value;
    toSystem.value = temp;
}

function clearFrom() {
    fromSystem.value = null;
}

function clearTo() {
    toSystem.value = null;
}
</script>

<template>
    <!-- From system bar -->
    <div class="border-b border-border/30 px-3 py-2">
        <Combobox>
            <ComboboxAnchor class="w-full">
                <template v-if="fromSystem">
                    <div class="flex h-8 items-center gap-1.5 rounded-md bg-muted/30 px-2">
                        <span class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">From</span>
                        <SolarsystemClass :wormhole_class="fromSystem.class" :security="fromSystem.security" class="shrink-0 text-xs" />
                        <span class="min-w-0 flex-1 truncate text-sm">
                            {{ fromSystem.name }}
                            <span class="text-xs text-muted-foreground">· {{ fromSystem.region?.name }}</span>
                        </span>
                        <SolarsystemSovereignty :sovereignty="fromSystem.sovereignty" :solarsystem-id="fromSystem.id" class="shrink-0">
                            <template #fallback>
                                <SolarsystemEffect v-if="fromSystem.effect" :effect="fromSystem.effect.name" class="shrink-0" />
                                <span v-else />
                            </template>
                        </SolarsystemSovereignty>
                        <button class="shrink-0 text-muted-foreground/40 hover:text-foreground" @click="clearFrom">
                            <X class="size-3" />
                        </button>
                    </div>
                </template>
                <template v-else>
                    <div class="flex items-center gap-2 rounded-md bg-muted/30 px-2">
                        <Search class="size-3.5 shrink-0 text-muted-foreground/40" />
                        <RekaComboboxInput
                            v-model="search"
                            placeholder="From system..."
                            class="h-8 min-w-0 flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground/40"
                        />
                    </div>
                </template>
            </ComboboxAnchor>
            <SystemComboboxList :solarsystems="filteredSolarsystems" @select="handleFromSystemSelect" />
        </Combobox>
        <div v-if="hasQuickPicks" class="mt-1.5 flex flex-wrap gap-1.5">
            <button
                v-if="selected_map_solarsystem?.solarsystem"
                class="inline-flex items-center gap-1.5 rounded-md border border-border/40 bg-muted/30 px-2 py-1 text-xs transition-colors hover:bg-muted/60"
                @click="handleFromSystemSelect(selected_map_solarsystem.solarsystem)"
            >
                <SolarsystemClass
                    :wormhole_class="selected_map_solarsystem.solarsystem.class"
                    :security="selected_map_solarsystem.solarsystem.security"
                    class="shrink-0 text-[10px]"
                />
                <span>{{ selected_map_solarsystem.solarsystem.name }}</span>
                <MapPin class="size-3 shrink-0 text-muted-foreground" />
            </button>
            <button
                v-if="activeCharacterSystem"
                class="inline-flex items-center gap-1.5 rounded-md border border-border/40 bg-muted/30 px-2 py-1 text-xs transition-colors hover:bg-muted/60"
                @click="handleFromSystemSelect(activeCharacterSystem)"
            >
                <SolarsystemClass
                    :wormhole_class="activeCharacterSystem.class"
                    :security="activeCharacterSystem.security"
                    class="shrink-0 text-[10px]"
                />
                <span>{{ activeCharacterSystem.name }}</span>
                <Navigation class="size-3 shrink-0 text-muted-foreground" />
            </button>
            <button
                v-for="dest in pinnedDestinations"
                :key="dest.id"
                class="inline-flex items-center gap-1.5 rounded-md border border-border/40 bg-muted/30 px-2 py-1 text-xs transition-colors hover:bg-muted/60"
                @click="handleFromSystemSelect(dest.solarsystem)"
            >
                <SolarsystemClass :wormhole_class="dest.solarsystem.class" :security="dest.solarsystem.security" class="shrink-0 text-[10px]" />
                <span>{{ dest.solarsystem.name }}</span>
            </button>
        </div>
    </div>

    <!-- Swap button -->
    <div class="flex justify-center border-b border-border/30 py-0.5">
        <button class="inline-flex items-center gap-1 text-muted-foreground/40 transition-colors hover:text-foreground" @click="swapSystems">
            <ArrowUpDown class="size-3" />
            <span class="font-mono text-[10px] tracking-wider uppercase">Swap</span>
        </button>
    </div>

    <!-- To system bar -->
    <div class="border-b border-border/30 px-3 py-2">
        <Combobox>
            <ComboboxAnchor class="w-full">
                <template v-if="toSystem">
                    <div class="flex h-8 items-center gap-1.5 rounded-md bg-muted/30 px-2">
                        <span class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">To</span>
                        <SolarsystemClass :wormhole_class="toSystem.class" :security="toSystem.security" class="shrink-0 text-xs" />
                        <span class="min-w-0 flex-1 truncate text-sm">
                            {{ toSystem.name }}
                            <span class="text-xs text-muted-foreground">· {{ toSystem.region?.name }}</span>
                        </span>
                        <SolarsystemSovereignty :sovereignty="toSystem.sovereignty" :solarsystem-id="toSystem.id" class="shrink-0">
                            <template #fallback>
                                <SolarsystemEffect v-if="toSystem.effect" :effect="toSystem.effect.name" class="shrink-0" />
                                <span v-else />
                            </template>
                        </SolarsystemSovereignty>
                        <button class="shrink-0 text-muted-foreground/40 hover:text-foreground" @click="clearTo">
                            <X class="size-3" />
                        </button>
                    </div>
                </template>
                <template v-else>
                    <div class="flex items-center gap-2 rounded-md bg-muted/30 px-2">
                        <Search class="size-3.5 shrink-0 text-muted-foreground/40" />
                        <RekaComboboxInput
                            v-model="search"
                            placeholder="To system..."
                            class="h-8 min-w-0 flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground/40"
                        />
                    </div>
                </template>
            </ComboboxAnchor>
            <SystemComboboxList :solarsystems="filteredSolarsystems" @select="handleToSystemSelect" />
        </Combobox>
        <div v-if="hasQuickPicks" class="mt-1.5 flex flex-wrap gap-1.5">
            <button
                v-if="selected_map_solarsystem?.solarsystem"
                class="inline-flex items-center gap-1.5 rounded-md border border-border/40 bg-muted/30 px-2 py-1 text-xs transition-colors hover:bg-muted/60"
                @click="handleToSystemSelect(selected_map_solarsystem.solarsystem)"
            >
                <SolarsystemClass
                    :wormhole_class="selected_map_solarsystem.solarsystem.class"
                    :security="selected_map_solarsystem.solarsystem.security"
                    class="shrink-0 text-[10px]"
                />
                <span>{{ selected_map_solarsystem.solarsystem.name }}</span>
                <MapPin class="size-3 shrink-0 text-muted-foreground" />
            </button>
            <button
                v-if="activeCharacterSystem"
                class="inline-flex items-center gap-1.5 rounded-md border border-border/40 bg-muted/30 px-2 py-1 text-xs transition-colors hover:bg-muted/60"
                @click="handleToSystemSelect(activeCharacterSystem)"
            >
                <SolarsystemClass
                    :wormhole_class="activeCharacterSystem.class"
                    :security="activeCharacterSystem.security"
                    class="shrink-0 text-[10px]"
                />
                <span>{{ activeCharacterSystem.name }}</span>
                <Navigation class="size-3 shrink-0 text-muted-foreground" />
            </button>
            <button
                v-for="dest in pinnedDestinations"
                :key="dest.id"
                class="inline-flex items-center gap-1.5 rounded-md border border-border/40 bg-muted/30 px-2 py-1 text-xs transition-colors hover:bg-muted/60"
                @click="handleToSystemSelect(dest.solarsystem)"
            >
                <SolarsystemClass :wormhole_class="dest.solarsystem.class" :security="dest.solarsystem.security" class="shrink-0 text-[10px]" />
                <span>{{ dest.solarsystem.name }}</span>
            </button>
        </div>
    </div>

    <!-- Route info bar -->
    <div v-if="fromSystem && toSystem" class="flex items-center justify-between border-b border-border/30 px-3 py-1.5">
        <span v-if="hasRoute" class="font-mono text-[10px] tracking-wider text-muted-foreground uppercase">{{ routeJumps }} jumps</span>
        <button
            v-if="ignored_systems.length > 0"
            class="font-mono text-[10px] text-muted-foreground hover:text-foreground"
            @click="handleClearIgnoreList"
        >
            Clear {{ ignored_systems.length }} ignored
        </button>
    </div>

    <!-- Route results -->
    <div v-if="hasRoute" v-element-hover="onRouteHover" class="grid grid-cols-[1.5rem_1.5rem_auto_1.25rem_1rem_1.25rem] gap-x-2">
        <DestinationContextMenu v-for="(entry, index) in enrichedRoute" :key="entry.solarsystem.id" :solarsystem_id="entry.solarsystem.id">
            <div class="col-span-full grid grid-cols-subgrid items-center border-b border-border/30 px-3 py-1 hover:bg-muted/30">
                <span class="text-center font-mono text-[10px] text-muted-foreground/60">{{ index + 1 }}</span>

                <SolarsystemClass :wormhole_class="entry.solarsystem.class" :security="entry.solarsystem.security" class="justify-self-center" />

                <span class="min-w-0 truncate text-xs">
                    {{ entry.solarsystem.name }}
                    <span class="text-[10px] text-muted-foreground">· {{ entry.solarsystem.region?.name }}</span>
                </span>

                <SolarsystemSovereignty
                    :sovereignty="entry.solarsystem.sovereignty"
                    :solarsystem-id="entry.solarsystem.id"
                    class="size-4 justify-self-center"
                >
                    <template #fallback>
                        <SolarsystemEffect v-if="entry.solarsystem.effect" :effect="entry.solarsystem.effect.name" />
                    </template>
                </SolarsystemSovereignty>

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
                    <button
                        v-if="index !== 0 && index !== enrichedRoute.length - 1"
                        class="text-muted-foreground/40 hover:text-destructive"
                        @click.stop="handleIgnoreSolarsystem(entry.solarsystem.id)"
                    >
                        <X class="size-3" />
                    </button>
                </div>
            </div>
        </DestinationContextMenu>
    </div>

    <!-- Empty states -->
    <div v-else-if="fromSystem && toSystem" class="flex h-full flex-col items-center justify-center p-4">
        <p class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">No route found</p>
    </div>
    <div v-else class="flex h-full flex-col items-center justify-center p-4">
        <p class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">Select origin and destination</p>
    </div>
</template>

<style scoped></style>

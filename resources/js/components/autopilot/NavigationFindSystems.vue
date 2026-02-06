<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import RoutePopover from '@/components/autopilot/RoutePopover.vue';
import SystemComboboxList from '@/components/autopilot/SystemComboboxList.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import SolarsystemEffect from '@/components/solarsystem/SolarsystemEffect.vue';
import { Combobox, ComboboxAnchor } from '@/components/ui/combobox';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { useClosestSystemsCalculator } from '@/composables/useClosestSystemsCalculator';
import { useStaticData } from '@/composables/useStaticData';
import { useStaticSolarsystem, useStaticSolarsystems } from '@/composables/useStaticSolarsystems';
import type { TMap, TResolvedMapRouteSolarsystem, TResolvedSelectedMapSolarsystem, TResolvedSolarsystem } from '@/pages/maps';
import type { TCharacter, TCharacterStatus } from '@/types/models';
import type { TStaticSolarsystem } from '@/types/static-data';
import { ArrowDown, ArrowUp, MapPin, Navigation, Search, X } from 'lucide-vue-next';
import { ComboboxInput as RekaComboboxInput } from 'reka-ui';
import { computed, ref } from 'vue';

const { map, solarsystems, selected_map_solarsystem, active_character, character_status, destinations, ignored_systems } = defineProps<{
    map: TMap;
    solarsystems: TStaticSolarsystem[];
    selected_map_solarsystem?: TResolvedSelectedMapSolarsystem | null;
    active_character?: TCharacter | null;
    character_status?: TCharacterStatus | null;
    destinations: TResolvedMapRouteSolarsystem[];
    ignored_systems: number[];
}>();

const mapConnections = computed(() => map.map_connections ?? []);
const mapSolarsystems = computed(() => map.map_solarsystems ?? []);

const { staticData, loadStaticData } = useStaticData();
void loadStaticData();

const search = ref('');
const fromSystem = ref<TResolvedSolarsystem | null>(null);
const condition = ref<string>('observatories');
const limit = ref(15);
const sortBy = ref<'name' | 'security' | 'region' | 'jumps'>('jumps');
const sortDirection = ref<'asc' | 'desc'>('asc');

const { results } = useClosestSystemsCalculator({
    fromId: computed(() => fromSystem.value?.id ?? null),
    condition,
    limit,
    ignoredSystems: computed(() => ignored_systems),
    mapConnections,
    mapSolarsystems,
});

const { resolveSolarsystem } = useStaticSolarsystems();

const resolvedResults = computed(() =>
    results.value.map((result) => ({
        ...result,
        solarsystem: resolveSolarsystem(result.solarsystem_id),
        resolvedRoute: result.route
            .map<TResolvedSolarsystem | null>((step, index) => {
                const solarsystem = resolveSolarsystem(step.id);
                if (!solarsystem) {
                    return null;
                }

                return {
                    ...solarsystem,
                    connection_type: result.route[index + 1]?.via ?? null,
                };
            })
            .filter((entry): entry is TResolvedSolarsystem => entry !== null),
    })),
);

const sortedResults = computed(() => {
    return [...resolvedResults.value].sort((a, b) => {
        let comparison = 0;

        switch (sortBy.value) {
            case 'name':
                comparison = a.solarsystem.name.localeCompare(b.solarsystem.name);
                break;
            case 'security':
                comparison = (a.solarsystem.security ?? 0) - (b.solarsystem.security ?? 0);
                break;
            case 'region':
                comparison = (a.solarsystem.region?.name ?? '').localeCompare(b.solarsystem.region?.name ?? '');
                break;
            case 'jumps':
                comparison = a.jumps - b.jumps;
                break;
        }

        return sortDirection.value === 'asc' ? comparison : -comparison;
    });
});

const baseConditionOptions = [
    { value: 'observatories', label: 'Jove Observatories', group: 'features' },
    { value: 'npc_stations', label: 'NPC Stations', group: 'features' },
    { value: 'highsec', label: 'High Security', group: 'security' },
    { value: 'lowsec', label: 'Low Security', group: 'security' },
    { value: 'nullsec', label: 'Null Security', group: 'security' },
];

const essentialServiceIds = new Set([5, 10, 13, 14, 15, 27]);

const serviceConditionOptions = computed(() => {
    const services = staticData.value?.services ?? [];
    return services
        .filter((service) => essentialServiceIds.has(service.id))
        .map((service) => ({
            value: `service_${service.id}`,
            label: service.name,
            group: 'services',
        }));
});

const conditionOptions = computed(() => [...baseConditionOptions, ...serviceConditionOptions.value]);

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

function handleSystemSelect(system: TResolvedSolarsystem) {
    fromSystem.value = system;
    search.value = '';
}

function handleLimitChange(value: number) {
    limit.value = value;
}

function handleSort(column: typeof sortBy.value) {
    if (sortBy.value === column) {
        sortDirection.value = sortDirection.value === 'asc' ? 'desc' : 'asc';
        return;
    }

    sortBy.value = column;
    sortDirection.value = 'asc';
}

function clearSystem() {
    fromSystem.value = null;
}
</script>

<template>
    <!-- System picker -->
    <div class="border-b border-border/30 px-3 py-2">
        <Combobox>
            <ComboboxAnchor class="w-full">
                <template v-if="fromSystem">
                    <div class="flex h-8 items-center gap-1.5 rounded-md bg-muted/30 px-2">
                        <SolarsystemClass :wormhole_class="fromSystem.class" :security="fromSystem.security" class="shrink-0 text-xs" />
                        <span class="min-w-0 flex-1 truncate text-sm">
                            {{ fromSystem.name }}
                            <span class="text-[10px] text-muted-foreground">· {{ fromSystem.region?.name }}</span>
                        </span>
                        <button class="shrink-0 text-muted-foreground/40 hover:text-foreground" @click="clearSystem">
                            <X class="size-3" />
                        </button>
                    </div>
                </template>
                <template v-else>
                    <div class="flex items-center gap-2 rounded-md bg-muted/30 px-2">
                        <Search class="size-3.5 shrink-0 text-muted-foreground/40" />
                        <RekaComboboxInput
                            v-model="search"
                            placeholder="Search system..."
                            class="h-8 min-w-0 flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground/40"
                        />
                    </div>
                </template>
            </ComboboxAnchor>
            <SystemComboboxList :solarsystems="filteredSolarsystems" @select="handleSystemSelect" />
        </Combobox>
        <div v-if="hasQuickPicks" class="mt-1.5 flex flex-wrap gap-1.5">
            <button
                v-if="selected_map_solarsystem?.solarsystem"
                class="inline-flex items-center gap-1.5 rounded-md border border-border/40 bg-muted/30 px-2 py-1 text-xs transition-colors hover:bg-muted/60"
                @click="handleSystemSelect(selected_map_solarsystem.solarsystem)"
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
                @click="handleSystemSelect(activeCharacterSystem)"
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
                @click="handleSystemSelect(dest.solarsystem)"
            >
                <SolarsystemClass :wormhole_class="dest.solarsystem.class" :security="dest.solarsystem.security" class="shrink-0 text-[10px]" />
                <span>{{ dest.solarsystem.name }}</span>
            </button>
        </div>
    </div>

    <!-- Condition & limit bar -->
    <div v-if="fromSystem" class="flex items-center gap-2 border-b border-border/30 px-3 py-2">
        <Select v-model="condition" class="min-w-0 flex-1">
            <SelectTrigger class="h-7 w-full border-0 bg-muted/30 px-2 text-xs shadow-none">
                <SelectValue placeholder="Condition..." />
            </SelectTrigger>
            <SelectContent>
                <div class="px-2 py-1 text-[10px] font-medium text-muted-foreground">Features</div>
                <SelectItem v-for="option in conditionOptions.filter((o) => o.group === 'features')" :key="option.value" :value="option.value">
                    {{ option.label }}
                </SelectItem>
                <div class="px-2 py-1 text-[10px] font-medium text-muted-foreground">Security</div>
                <SelectItem v-for="option in conditionOptions.filter((o) => o.group === 'security')" :key="option.value" :value="option.value">
                    {{ option.label }}
                </SelectItem>
                <template v-if="serviceConditionOptions.length">
                    <div class="px-2 py-1 text-[10px] font-medium text-muted-foreground">Station Services</div>
                    <SelectItem v-for="option in conditionOptions.filter((o) => o.group === 'services')" :key="option.value" :value="option.value">
                        {{ option.label }}
                    </SelectItem>
                </template>
            </SelectContent>
        </Select>
        <Select :model-value="limit.toString()" @update:model-value="(value) => handleLimitChange(parseInt(value as string))">
            <SelectTrigger class="h-7 w-14 shrink-0 border-0 bg-muted/30 px-2 text-xs shadow-none">
                <SelectValue />
            </SelectTrigger>
            <SelectContent>
                <SelectItem value="5">5</SelectItem>
                <SelectItem value="10">10</SelectItem>
                <SelectItem value="15">15</SelectItem>
                <SelectItem value="25">25</SelectItem>
                <SelectItem value="50">50</SelectItem>
            </SelectContent>
        </Select>
    </div>

    <!-- Results table -->
    <div v-if="sortedResults.length" class="grid grid-cols-[1.5rem_auto_auto_1.25rem_2rem] gap-x-2">
        <div
            class="col-span-full grid grid-cols-subgrid border-b border-border/30 bg-muted/20 px-3 py-1.5 font-mono text-[10px] tracking-wider text-muted-foreground uppercase"
        >
            <span></span>
            <button @click="handleSort('name')" class="flex items-center gap-1 hover:text-foreground">
                <span>System</span>
                <ArrowUp v-if="sortBy === 'name' && sortDirection === 'asc'" class="size-3" />
                <ArrowDown v-if="sortBy === 'name' && sortDirection === 'desc'" class="size-3" />
            </button>
            <button @click="handleSort('region')" class="flex items-center gap-1 hover:text-foreground">
                <span>Region</span>
                <ArrowUp v-if="sortBy === 'region' && sortDirection === 'asc'" class="size-3" />
                <ArrowDown v-if="sortBy === 'region' && sortDirection === 'desc'" class="size-3" />
            </button>
            <span></span>
            <button @click="handleSort('jumps')" class="flex items-center justify-end gap-1 hover:text-foreground">
                <span>J</span>
                <ArrowUp v-if="sortBy === 'jumps' && sortDirection === 'asc'" class="size-3" />
                <ArrowDown v-if="sortBy === 'jumps' && sortDirection === 'desc'" class="size-3" />
            </button>
        </div>

        <DestinationContextMenu v-for="result in sortedResults" :key="result.solarsystem.id" :solarsystem_id="result.solarsystem.id">
            <div class="col-span-full grid grid-cols-subgrid items-center border-b border-border/30 px-3 py-1.5 hover:bg-muted/30">
                <SolarsystemClass :wormhole_class="result.solarsystem.class" :security="result.solarsystem.security" class="justify-self-center" />

                <span class="truncate text-xs">{{ result.solarsystem.name }}</span>

                <span class="truncate text-[10px] text-muted-foreground">{{ result.solarsystem.region?.name ?? '' }}</span>

                <SolarsystemSovereignty
                    :sovereignty="result.solarsystem.sovereignty"
                    :solarsystem-id="result.solarsystem.id"
                    class="size-4 justify-self-center"
                >
                    <template #fallback>
                        <SolarsystemEffect v-if="result.solarsystem.effect" :effect="result.solarsystem.effect.name" />
                    </template>
                </SolarsystemSovereignty>

                <RoutePopover :route="result.resolvedRoute">
                    <span
                        v-if="result.jumps !== 2147483647"
                        class="cursor-pointer font-mono text-xs font-medium"
                        :class="{
                            'text-green-400': result.jumps < 8,
                            'text-amber-400': result.jumps >= 8 && result.jumps < 15,
                            'text-red-400': result.jumps >= 15,
                        }"
                    >
                        {{ result.jumps }}j
                    </span>
                    <span v-else class="font-mono text-[10px] text-muted-foreground/60">--</span>
                </RoutePopover>
            </div>
        </DestinationContextMenu>
    </div>

    <!-- Empty state -->
    <div v-else class="flex h-full flex-col items-center justify-center p-4">
        <p class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">
            {{ fromSystem ? 'No systems found' : 'Select a starting system' }}
        </p>
    </div>
</template>

<style scoped></style>

<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import QuickSelectButtons from '@/components/autopilot/QuickSelectButtons.vue';
import SystemComboboxList from '@/components/autopilot/SystemComboboxList.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import SolarsystemEffect from '@/components/solarsystem/SolarsystemEffect.vue';
import { Button } from '@/components/ui/button';
import { Combobox, ComboboxAnchor, ComboboxInput } from '@/components/ui/combobox';
import { Label } from '@/components/ui/label';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { useClosestSystemsCalculator } from '@/composables/useClosestSystemsCalculator';
import { useStaticSolarsystem, useStaticSolarsystems } from '@/composables/useStaticSolarsystems';
import type { TMap, TResolvedMapRouteSolarsystem, TResolvedSelectedMapSolarsystem, TResolvedSolarsystem } from '@/pages/maps';
import type { TCharacter, TCharacterStatus } from '@/types/models';
import type { TStaticSolarsystem } from '@/types/static-data';
import { ArrowDown, ArrowUp, X } from 'lucide-vue-next';
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

const search = ref('');
const fromSystem = ref<TResolvedSolarsystem | null>(null);
const condition = ref<'observatories' | 'npc_stations' | 'highsec' | 'lowsec' | 'nullsec'>('observatories');
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

const conditionOptions = [
    { value: 'observatories', label: 'Jove Observatories' },
    { value: 'npc_stations', label: 'NPC Stations' },
    { value: 'highsec', label: 'High Security' },
    { value: 'lowsec', label: 'Low Security' },
    { value: 'nullsec', label: 'Null Security' },
];

const activeCharacterSystem = useStaticSolarsystem(() => (active_character ? (character_status?.solarsystem_id ?? null) : null));

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
    <div class="bg-card p-6 pt-2">
        <div class="mb-1.5">
            <span class="text-xs font-medium text-muted-foreground">From</span>
        </div>
        <Combobox>
            <ComboboxAnchor class="w-full">
                <template v-if="fromSystem">
                    <div class="flex items-center gap-2 rounded-lg border p-2">
                        <SolarsystemClass :wormhole_class="fromSystem.class" :security="fromSystem.security" class="shrink-0" />
                        <div class="min-w-0 flex-1 text-xs">
                            <span class="font-medium">{{ fromSystem.name }}</span>
                            <span class="text-muted-foreground"> · {{ fromSystem.region?.name }}</span>
                        </div>
                        <Button variant="ghost" size="icon" @click="clearSystem" class="h-6 w-6 shrink-0">
                            <X class="size-3" />
                        </Button>
                    </div>
                </template>
                <template v-else>
                    <ComboboxInput v-model="search" placeholder="Search system..." class="rounded-lg border" />
                </template>
            </ComboboxAnchor>
            <SystemComboboxList :solarsystems="filteredSolarsystems" @select="handleSystemSelect" />
        </Combobox>
        <QuickSelectButtons
            v-if="!fromSystem"
            :selected_map_solarsystem="selected_map_solarsystem"
            :active_character_system="activeCharacterSystem"
            :destinations="destinations"
            @select-system="handleSystemSelect"
        />

        <div v-if="fromSystem" class="mt-4 space-y-1.5">
            <div class="text-xs font-medium text-muted-foreground">Target Conditions</div>
            <RadioGroup v-model="condition" class="flex flex-wrap gap-2">
                <Label
                    v-for="option in conditionOptions"
                    :key="option.value"
                    class="flex cursor-pointer items-center gap-2 rounded-lg border px-2.5 py-1.5 transition-colors hover:bg-muted/50"
                    :class="condition === option.value ? 'bg-primary/5' : ''"
                >
                    <RadioGroupItem :value="option.value" :id="option.value" class="focus-visible:ring-0" />
                    <span class="text-sm font-medium">{{ option.label }}</span>
                </Label>
            </RadioGroup>
        </div>

        <div v-if="fromSystem" class="flex items-center justify-between p-2">
            <div class="flex items-center gap-2">
                <span class="text-xs text-muted-foreground">Limit:</span>
                <Select :model-value="limit.toString()" @update:model-value="(value) => handleLimitChange(parseInt(value as string))">
                    <SelectTrigger class="h-8 w-20 text-xs">
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
            <span v-if="resolvedResults.length" class="text-sm text-muted-foreground">{{ resolvedResults.length }} systems</span>
        </div>

        <div v-if="resolvedResults.length" class="grid grid-cols-[auto_1fr_1fr_auto_auto] rounded-lg border bg-card text-sm">
            <div class="col-span-full grid grid-cols-subgrid border-b bg-muted/50 px-2 py-1.5 text-xs font-medium text-muted-foreground">
                <div></div>
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
                <div></div>
                <button @click="handleSort('jumps')" class="flex items-center justify-center gap-1 hover:text-foreground">
                    <span>Jumps</span>
                    <ArrowUp v-if="sortBy === 'jumps' && sortDirection === 'asc'" class="size-3" />
                    <ArrowDown v-if="sortBy === 'jumps' && sortDirection === 'desc'" class="size-3" />
                </button>
            </div>
            <DestinationContextMenu v-for="result in sortedResults" :key="result.solarsystem.id" :solarsystem_id="result.solarsystem.id">
                <div class="col-span-full grid grid-cols-subgrid items-center gap-2 border-b px-2.5 py-1.5 hover:bg-accent/50">
                    <div class="justify-self-center">
                        <SolarsystemClass :wormhole_class="result.solarsystem.class" :security="result.solarsystem.security" />
                    </div>
                    <div class="min-w-0 text-xs font-medium">{{ result.solarsystem.name }}</div>
                    <div class="min-w-0 text-xs text-muted-foreground">{{ result.solarsystem.region?.name ?? 'Unknown' }}</div>
                    <div class="flex items-center justify-center">
                        <SolarsystemSovereignty :sovereignty="result.solarsystem.sovereignty" :solarsystem-id="result.solarsystem.id">
                            <template #fallback>
                                <SolarsystemEffect v-if="result.solarsystem.effect" :effect="result.solarsystem.effect.name" />
                            </template>
                        </SolarsystemSovereignty>
                    </div>
                    <div class="flex items-center justify-center font-mono text-xs text-muted-foreground tabular-nums">
                        {{ result.jumps === 2147483647 ? '∞' : result.jumps }}
                    </div>
                </div>
            </DestinationContextMenu>
        </div>
        <div v-else class="flex flex-col items-center justify-center gap-2 py-6 text-center text-sm text-muted-foreground">
            <p v-if="!fromSystem">Select a starting system to find nearby matches</p>
            <p v-else>No systems found matching your criteria</p>
            <p v-if="!fromSystem && (selected_map_solarsystem || activeCharacterSystem)" class="text-xs">
                Use the quick select buttons above to get started.
            </p>
        </div>
    </div>
</template>

<style scoped></style>

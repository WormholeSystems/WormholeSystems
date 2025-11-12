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
import { useSearch } from '@/composables/useSearch';
import { TClosestSystem, TClosestSystems, TMap, TSelectedMapSolarsystem, TSolarsystem } from '@/pages/maps';
import { TCharacter, TCharacterStatus, TMapRouteSolarsystem } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { X } from 'lucide-vue-next';
import { computed, ref } from 'vue';

const { solarsystems, closest_systems, selected_map_solarsystem, active_character, character_status, destinations } = defineProps<{
    map: TMap;
    solarsystems: TSolarsystem[];
    selected_map_solarsystem?: TSelectedMapSolarsystem | null;
    closest_systems: TClosestSystems;
    active_character?: TCharacter | null;
    character_status?: TCharacterStatus | null;
    destinations: TMapRouteSolarsystem[];
}>();

const search = useSearch('search', ['solarsystems']);

const currentFromSystem = computed(() => closest_systems.from_system || null);
const currentCondition = computed(() => closest_systems.condition || 'observatories');
const currentLimit = computed(() => closest_systems.limit || 15);

// Sorting state
const sortBy = ref<'name' | 'security' | 'region' | 'jumps'>('jumps');
const sortDirection = ref<'asc' | 'desc'>('asc');

const conditionOptions = [
    { value: 'observatories', label: 'Jove Observatories', description: 'Systems with Jove observatories' },
    { value: 'npc_stations', label: 'NPC Stations', description: 'Systems with NPC stations' },
    { value: 'highsec', label: 'High Security', description: 'Security >= 0.5' },
    { value: 'lowsec', label: 'Low Security', description: 'Security 0.1-0.4' },
    { value: 'nullsec', label: 'Null Security', description: 'Security <= 0.0' },
];

const activeCharacterSystem = computed(() => {
    if (!active_character || !character_status?.solarsystem) return null;
    return character_status.solarsystem;
});

const results = computed((): TClosestSystem[] => {
    const rawResults = closest_systems.results || [];

    // Sort based on selected field and direction
    return [...rawResults].sort((a, b) => {
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

function reloadWithParams(params: Record<string, any>) {
    router.reload({
        data: params,
        only: ['map_navigation', 'map_user_settings'],
    });
}

function handleSystemSelect(system: TSolarsystem) {
    reloadWithParams({
        from_system: system.name,
        condition: currentCondition.value,
        limit: currentLimit.value,
    });
}

function clearSystem() {
    reloadWithParams({
        from_system: '',
    });
}

function handleConditionChange(condition: string) {
    if (currentFromSystem.value) {
        reloadWithParams({
            from_system: currentFromSystem.value.name,
            condition,
            limit: currentLimit.value,
        });
    }
}

function handleLimitChange(limit: number) {
    if (currentFromSystem.value) {
        reloadWithParams({
            from_system: currentFromSystem.value.name,
            condition: currentCondition.value,
            limit,
        });
    }
}
</script>

<template>
    <div class="bg-card p-6 pt-2">
        <!-- Starting System Selection -->
        <div class="mb-1.5">
            <span class="text-xs font-medium text-muted-foreground">From</span>
        </div>
        <Combobox>
            <ComboboxAnchor class="w-full">
                <template v-if="currentFromSystem">
                    <div class="flex items-center gap-2 rounded-lg border p-2">
                        <SolarsystemClass :wormhole_class="currentFromSystem.class" :security="currentFromSystem.security" class="shrink-0" />
                        <div class="min-w-0 flex-1 text-xs">
                            <span class="font-medium">{{ currentFromSystem.name }}</span>
                            <span class="text-muted-foreground"> · {{ currentFromSystem.region?.name }}</span>
                        </div>
                        <Button variant="ghost" size="icon" @click="clearSystem" class="h-6 w-6 shrink-0">
                            <X class="size-3" />
                        </Button>
                    </div>
                </template>
                <template v-else>
                    <ComboboxInput v-model="search" placeholder="Search system..." class="rounded-lg border" auto-focus />
                </template>
            </ComboboxAnchor>
            <SystemComboboxList :solarsystems="solarsystems" @select="handleSystemSelect" />
        </Combobox>
        <QuickSelectButtons
            v-if="!currentFromSystem"
            :selected_map_solarsystem="selected_map_solarsystem"
            :active_character_system="activeCharacterSystem"
            :destinations="destinations"
            @select-system="handleSystemSelect"
        />

        <!-- Condition Selection -->
        <div v-if="currentFromSystem" class="mt-4 space-y-1.5">
            <div class="text-xs font-medium text-muted-foreground">Find Systems Matching</div>
            <RadioGroup
                :model-value="currentCondition"
                @update:model-value="(value) => handleConditionChange(value as string)"
                class="flex flex-wrap gap-2"
            >
                <Label
                    :for="option.value"
                    v-for="option in conditionOptions"
                    :key="option.value"
                    class="flex cursor-pointer items-center space-x-2 rounded-lg border px-2.5 py-1.5 whitespace-nowrap transition-colors focus-within:ring-0 focus-within:outline-none hover:bg-muted/50"
                    :class="currentCondition === option.value ? 'bg-primary/5' : ''"
                >
                    <RadioGroupItem :value="option.value" :id="option.value" class="focus-visible:ring-0 focus-visible:ring-offset-0" />
                    <div class="text-sm font-medium">{{ option.label }}</div>
                </Label>
            </RadioGroup>
        </div>
    </div>

    <!-- Results Controls -->
    <div v-if="currentFromSystem" class="flex items-center justify-between p-2">
        <div class="flex items-center gap-2">
            <span class="text-xs text-muted-foreground">Limit:</span>
            <Select :model-value="currentLimit.toString()" @update:model-value="(value) => handleLimitChange(parseInt(value as string))">
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
        <span v-if="results.length" class="text-sm text-muted-foreground">{{ results.length }} systems</span>
    </div>

    <!-- Results -->
    <div v-if="results.length > 0" class="grid grid-cols-[auto_1fr_1fr_auto_auto] rounded-lg border bg-card text-sm">
        <DestinationContextMenu v-for="result in results" :key="result.solarsystem.id" :solarsystem_id="result.solarsystem.id">
            <div
                class="col-span-full grid grid-cols-subgrid items-center gap-2 border-b px-2.5 py-1.5 transition-colors last:border-b-0 hover:bg-accent/50"
            >
                <div class="justify-self-center">
                    <SolarsystemClass :wormhole_class="result.solarsystem.class" :security="result.solarsystem.security" />
                </div>
                <div class="min-w-0 text-xs font-medium">
                    {{ result.solarsystem.name }}
                </div>
                <div class="min-w-0 text-xs text-muted-foreground">
                    {{ result.solarsystem.region?.name ?? 'Unknown' }}
                </div>
                <div class="flex items-center justify-center">
                    <SolarsystemSovereignty v-if="result.solarsystem.sovereignty" :sovereignty="result.solarsystem.sovereignty" />
                    <SolarsystemEffect v-else-if="result.solarsystem.effect" :effect="result.solarsystem.effect.name" />
                </div>
                <div class="flex items-center justify-center font-mono text-xs text-muted-foreground tabular-nums">
                    {{ result.jumps === 2147483647 ? '∞' : result.jumps }}
                </div>
            </div>
        </DestinationContextMenu>
    </div>
    <div v-else class="flex flex-col items-center justify-center gap-2 py-6 text-center text-sm text-muted-foreground">
        <p v-if="!currentFromSystem">Select a starting system to find closest systems matching specific conditions</p>
        <p v-else>No systems found matching your criteria</p>
        <p v-if="!currentFromSystem && (selected_map_solarsystem || activeCharacterSystem)" class="text-xs">
            Use the quick select buttons above to get started.
        </p>
    </div>
</template>

<style scoped></style>

<script setup lang="ts">
import AutopilotSettings from '@/components/autopilot/AutopilotSettings.vue';
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import ChevronDownIcon from '@/components/icons/ChevronDownIcon.vue';
import ChevronUpIcon from '@/components/icons/ChevronUpIcon.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import SolarsystemEffect from '@/components/solarsystem/SolarsystemEffect.vue';
import { Button } from '@/components/ui/button';
import { Combobox, ComboboxAnchor, ComboboxEmpty, ComboboxInput, ComboboxItem, ComboboxList } from '@/components/ui/combobox';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Label } from '@/components/ui/label';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { useSearch } from '@/composables/useSearch';
import { TClosestSystem, TClosestSystems } from '@/pages/maps';
import { TMap, TMapSolarSystem, TSolarsystem } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { computed, ref } from 'vue';

const { solarsystems, closest_systems } = defineProps<{
    map: TMap;
    solarsystems: TSolarsystem[];
    selected_map_solarsystem?: TMapSolarSystem | null;
    closest_systems?: TClosestSystems | null;
}>();

const isOpen = ref(false);

const search = useSearch('search', ['solarsystems']);

const currentFromSystem = computed(() => closest_systems?.from_system || null);
const currentCondition = computed(() => closest_systems?.condition || 'observatories');
const currentLimit = computed(() => closest_systems?.limit || 15);

// Sorting state
const sortBy = ref<'name' | 'security' | 'region' | 'jumps'>('jumps');
const sortDirection = ref<'asc' | 'desc'>('asc');

const conditionOptions = [
    { value: 'observatories', label: 'Jove Observatories', description: 'Systems with Jove observatories' },
    { value: 'highsec', label: 'High Security', description: 'Security >= 0.5' },
    { value: 'lowsec', label: 'Low Security', description: 'Security 0.1-0.4' },
    { value: 'nullsec', label: 'Null Security', description: 'Security <= 0.0' },
];

function handleSort(column: 'name' | 'security' | 'region' | 'jumps') {
    if (sortBy.value === column) {
        sortDirection.value = sortDirection.value === 'asc' ? 'desc' : 'asc';
    } else {
        sortBy.value = column;
        sortDirection.value = 'asc';
    }
}

function isSortedBy(column: 'name' | 'security' | 'region' | 'jumps') {
    return sortBy.value === column;
}

const results = computed((): TClosestSystem[] => {
    const rawResults = closest_systems?.results || [];

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
        only: ['closest_systems', 'map_user_settings'],
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
    <Dialog v-model:open="isOpen">
        <DialogTrigger as-child>
            <slot />
        </DialogTrigger>
        <DialogContent class="flex h-[85vh] w-full max-w-7xl flex-col gap-0 p-0 focus:outline-0">
            <DialogHeader class="border-b px-6 py-4">
                <DialogTitle>Find Closest Systems</DialogTitle>
                <DialogDescription>Search for systems matching specific conditions</DialogDescription>
            </DialogHeader>

            <!-- Filters Bar -->
            <div class="space-y-4 border-b px-6 py-4">
                <!-- From System Selection -->
                <div class="space-y-2">
                    <div v-if="currentFromSystem" class="flex items-center justify-between rounded-lg border bg-muted/50 p-3">
                        <div class="flex min-w-0 items-center gap-3">
                            <SolarsystemClass :wormhole_class="currentFromSystem.class" :security="currentFromSystem.security" />
                            <div class="min-w-0">
                                <div class="truncate font-medium">{{ currentFromSystem.name }}</div>
                                <div class="truncate text-xs text-muted-foreground">{{ currentFromSystem.region?.name }}</div>
                            </div>
                        </div>
                        <Button variant="ghost" size="sm" @click="clearSystem" class="ml-2 shrink-0">Clear</Button>
                    </div>
                    <div v-else class="space-y-2">
                        <Combobox>
                            <ComboboxAnchor class="w-full rounded-lg border">
                                <ComboboxInput v-model="search" placeholder="Search starting system..." class="w-full border-0" auto-focus />
                            </ComboboxAnchor>
                            <ComboboxList class="max-h-80 w-96" align="start">
                                <ComboboxEmpty>No systems found.</ComboboxEmpty>
                                <ComboboxItem
                                    v-for="system in solarsystems"
                                    :key="system.id"
                                    :value="system.name"
                                    @select.prevent="handleSystemSelect(system)"
                                    class="grid grid-cols-[auto_1fr] items-center gap-2 py-2"
                                >
                                    <SolarsystemClass :wormhole_class="system.class" :security="system.security" />
                                    <div class="min-w-0">
                                        <div class="truncate font-medium">{{ system.name }}</div>
                                        <div class="truncate text-xs text-muted-foreground">{{ system.region?.name }}</div>
                                    </div>
                                </ComboboxItem>
                            </ComboboxList>
                        </Combobox>
                        <div
                            v-if="selected_map_solarsystem?.solarsystem"
                            class="flex items-center justify-between rounded-lg border border-dashed bg-muted/30 p-3"
                        >
                            <div class="flex min-w-0 items-center gap-3">
                                <SolarsystemClass
                                    :wormhole_class="selected_map_solarsystem.solarsystem.class"
                                    :security="selected_map_solarsystem.solarsystem.security"
                                />
                                <div class="min-w-0">
                                    <div class="truncate text-sm font-medium">{{ selected_map_solarsystem.solarsystem.name }}</div>
                                    <div class="truncate text-xs text-muted-foreground">Active System</div>
                                </div>
                            </div>
                            <Button variant="outline" size="sm" @click="handleSystemSelect(selected_map_solarsystem.solarsystem)" class="shrink-0">
                                Use This System
                            </Button>
                        </div>
                    </div>
                </div>

                <!-- Condition Selection -->
                <div v-if="currentFromSystem">
                    <RadioGroup
                        :model-value="currentCondition"
                        @update:model-value="(value) => handleConditionChange(value as string)"
                        class="flex flex-wrap gap-2"
                    >
                        <Label
                            :for="option.value"
                            v-for="option in conditionOptions"
                            :key="option.value"
                            class="flex cursor-pointer items-center space-x-2 rounded-lg border px-3 py-2 whitespace-nowrap transition-colors hover:bg-muted/50"
                        >
                            <RadioGroupItem :value="option.value" :id="option.value" />
                            <div class="text-sm font-medium">{{ option.label }}</div>
                        </Label>
                    </RadioGroup>
                </div>
            </div>

            <!-- Results Controls -->
            <div v-if="currentFromSystem" class="flex items-center justify-between border-b px-6 py-3">
                <div class="flex items-center gap-2">
                    <span class="text-sm text-muted-foreground">Results Limit:</span>
                    <Select :model-value="currentLimit.toString()" @update:model-value="(value) => handleLimitChange(parseInt(value as string))">
                        <SelectTrigger class="w-24">
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

                <!-- Autopilot Settings -->
                <AutopilotSettings />
            </div>

            <!-- Results -->
            <div class="flex flex-1 flex-col overflow-hidden">
                <div v-if="results.length > 0" class="flex flex-1 flex-col overflow-hidden">
                    <div class="flex items-center justify-between px-6 py-3">
                        <h4 class="font-medium">Results</h4>
                        <span class="text-sm text-muted-foreground">{{ results.length }} systems found</span>
                    </div>
                    <div class="flex-1 overflow-auto px-6 pb-6">
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead>
                                        <button @click="handleSort('name')" class="flex w-full items-center gap-1 hover:text-foreground">
                                            System
                                            <ChevronUpIcon v-if="isSortedBy('name') && sortDirection === 'asc'" class="size-3" />
                                            <ChevronDownIcon v-if="isSortedBy('name') && sortDirection === 'desc'" class="size-3" />
                                        </button>
                                    </TableHead>
                                    <TableHead class="w-24">
                                        <button
                                            @click="handleSort('security')"
                                            class="flex w-full items-center justify-center gap-1 hover:text-foreground"
                                        >
                                            Sec
                                            <ChevronUpIcon v-if="isSortedBy('security') && sortDirection === 'asc'" class="size-3" />
                                            <ChevronDownIcon v-if="isSortedBy('security') && sortDirection === 'desc'" class="size-3" />
                                        </button>
                                    </TableHead>
                                    <TableHead>
                                        <button @click="handleSort('region')" class="flex w-full items-center gap-1 hover:text-foreground">
                                            Region
                                            <ChevronUpIcon v-if="isSortedBy('region') && sortDirection === 'asc'" class="size-3" />
                                            <ChevronDownIcon v-if="isSortedBy('region') && sortDirection === 'desc'" class="size-3" />
                                        </button>
                                    </TableHead>
                                    <TableHead class="w-20 text-center">
                                        <button
                                            @click="handleSort('jumps')"
                                            class="flex w-full items-center justify-center gap-1 hover:text-foreground"
                                        >
                                            Jumps
                                            <ChevronUpIcon v-if="isSortedBy('jumps') && sortDirection === 'asc'" class="size-3" />
                                            <ChevronDownIcon v-if="isSortedBy('jumps') && sortDirection === 'desc'" class="size-3" />
                                        </button>
                                    </TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                <DestinationContextMenu
                                    v-for="result in results"
                                    :key="result.solarsystem.id"
                                    :solarsystem_id="result.solarsystem.id"
                                >
                                    <TableRow>
                                        <TableCell>
                                            <div class="flex items-center gap-2">
                                                <span class="font-medium">{{ result.solarsystem.name }}</span>
                                            </div>
                                        </TableCell>
                                        <TableCell class="text-center">
                                            <SolarsystemClass :wormhole_class="result.solarsystem.class" :security="result.solarsystem.security" />
                                        </TableCell>
                                        <TableCell>
                                            <div class="flex items-center gap-2">
                                                <span class="text-sm">{{ result.solarsystem.region?.name ?? 'Unknown' }}</span>
                                                <SolarsystemSovereignty
                                                    v-if="result.solarsystem.sovereignty"
                                                    :sovereignty="result.solarsystem.sovereignty"
                                                    class="ml-auto"
                                                />
                                                <SolarsystemEffect
                                                    v-else-if="result.solarsystem.effect"
                                                    :effect="result.solarsystem.effect"
                                                    class="ml-auto"
                                                />
                                            </div>
                                        </TableCell>
                                        <TableCell class="text-center font-mono text-sm">
                                            {{ result.jumps === 2147483647 ? '∞' : result.jumps }}
                                        </TableCell>
                                    </TableRow>
                                </DestinationContextMenu>
                            </TableBody>
                        </Table>
                    </div>
                </div>
                <div v-else class="flex flex-1 items-center justify-center">
                    <div class="text-center text-muted-foreground">
                        <p v-if="!currentFromSystem" class="text-sm">Select a starting system to begin</p>
                        <p v-else class="text-sm">No systems found matching your criteria</p>
                    </div>
                </div>
            </div>
        </DialogContent>
    </Dialog>
</template>

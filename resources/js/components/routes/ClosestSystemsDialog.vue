<script setup lang="ts">
import DestinationContextMenu from '@/components/DestinationContextMenu.vue';
import SearchIcon from '@/components/icons/SearchIcon.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import SolarsystemEffect from '@/components/SolarsystemEffect.vue';
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

const conditionOptions = [
    { value: 'observatories', label: 'Jove Observatories', description: 'Systems with Jove observatories' },
    { value: 'highsec', label: 'High Security', description: 'Security >= 0.5' },
    { value: 'lowsec', label: 'Low Security', description: 'Security 0.1-0.4' },
    { value: 'nullsec', label: 'Null Security', description: 'Security <= 0.0' },
];

const results = computed((): TClosestSystem[] => {
    return closest_systems?.results || [];
});

function reloadWithParams(params: Record<string, any>) {
    router.reload({
        data: params,
        only: ['closest_systems'],
    });
}

function handleSystemSelect(system: TSolarsystem) {
    reloadWithParams({
        from_system: system.name,
        condition: currentCondition.value,
        limit: currentLimit.value,
    });
}

function handleConditionChange(newCondition: string) {
    if (currentFromSystem.value) {
        reloadWithParams({
            from_system: currentFromSystem.value.name,
            condition: newCondition,
            limit: currentLimit.value,
        });
    }
}

function handleLimitChange(newLimit: number) {
    if (currentFromSystem.value) {
        reloadWithParams({
            from_system: currentFromSystem.value.name,
            condition: currentCondition.value,
            limit: newLimit,
        });
    }
}

function clearSystem() {
    reloadWithParams({
        from_system: '',
        condition: 'observatories',
        limit: 15,
    });
}
</script>

<template>
    <Dialog v-model:open="isOpen">
        <DialogTrigger as-child>
            <slot>
                <Button variant="secondary" size="icon">
                    <SearchIcon />
                </Button>
            </slot>
        </DialogTrigger>
        <DialogContent class="w-full flex-col gap-4 py-4 focus:outline-0 md:max-w-[800px]">
            <DialogHeader>
                <DialogTitle>Find Closest Systems</DialogTitle>
                <DialogDescription> Find the closest systems matching specific conditions from any location </DialogDescription>
            </DialogHeader>

            <div class="grid gap-4">
                <Label>From System</Label>
                <div v-if="currentFromSystem" class="flex items-center justify-between rounded-lg border p-3">
                    <div class="flex items-center gap-2">
                        <SolarsystemClass :wormhole_class="currentFromSystem.class" :security="currentFromSystem.security" />
                        <div>
                            <div class="font-medium">{{ currentFromSystem.name }}</div>
                            <div class="text-sm text-muted-foreground">{{ currentFromSystem.region?.name }}</div>
                        </div>
                    </div>
                    <Button variant="ghost" size="sm" @click="clearSystem">Clear</Button>
                </div>
                <Combobox v-else>
                    <ComboboxAnchor class="w-full rounded-lg border">
                        <ComboboxInput v-model="search" placeholder="Search for a solar system..." class="w-full border-0" auto-focus />
                    </ComboboxAnchor>
                    <ComboboxList class="max-h-80 w-80" align="start">
                        <ComboboxEmpty>No systems found.</ComboboxEmpty>
                        <ComboboxItem
                            v-for="system in solarsystems"
                            :key="system.id"
                            :value="system.name"
                            @select.prevent="handleSystemSelect(system)"
                            class="grid grid-cols-[auto_1fr_auto_auto] items-center gap-2 py-2"
                        >
                            <SolarsystemClass :wormhole_class="system.class" :security="system.security" />
                            <div class="min-w-0">
                                <div class="truncate font-medium">{{ system.name }}</div>
                            </div>
                            <div class="flex justify-center">
                                <SolarsystemSovereignty v-if="system.sovereignty" :sovereignty="system.sovereignty" />
                                <SolarsystemEffect v-else-if="system.effect" :effect="system.effect" />
                            </div>
                            <div class="min-w-0 truncate text-sm text-muted-foreground">
                                {{ system.region?.name }}
                            </div>
                        </ComboboxItem>
                    </ComboboxList>
                </Combobox>
            </div>
            <!-- Search Options -->
            <div class="space-y-6" :class="{ 'pointer-events-none opacity-50': !currentFromSystem }">
                <!-- Condition Selection -->
                <div class="space-y-3">
                    <Label class="text-base font-medium">Search Condition</Label>
                    <RadioGroup
                        :model-value="currentCondition"
                        @update:model-value="(value) => handleConditionChange(value as string)"
                        class="grid grid-cols-2 gap-4"
                        :disabled="!currentFromSystem"
                    >
                        <Label
                            :for="option.value"
                            v-for="option in conditionOptions"
                            :key="option.value"
                            class="flex items-start space-x-3 rounded-lg border p-4 transition-colors hover:bg-muted/50"
                        >
                            <RadioGroupItem :value="option.value" :id="option.value" class="mt-0.5" :disabled="!currentFromSystem" />
                            <div class="grid gap-1 leading-none">
                                {{ option.label }}
                                <p class="text-xs text-muted-foreground">{{ option.description }}</p>
                            </div>
                        </Label>
                    </RadioGroup>
                </div>

                <!-- Limit Selection -->
                <div class="flex items-center justify-between">
                    <Label class="text-base font-medium">Result Limit</Label>
                    <Select
                        :model-value="currentLimit.toString()"
                        @update:model-value="(value) => handleLimitChange(parseInt(value as string))"
                        :disabled="!currentFromSystem"
                    >
                        <SelectTrigger class="w-40">
                            <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="5">5 systems</SelectItem>
                            <SelectItem value="10">10 systems</SelectItem>
                            <SelectItem value="15">15 systems</SelectItem>
                            <SelectItem value="25">25 systems</SelectItem>
                            <SelectItem value="50">50 systems</SelectItem>
                        </SelectContent>
                    </Select>
                </div>
            </div>
            <!-- Results Table -->
            <div v-if="results.length > 0" class="flex-1 overflow-hidden">
                <div class="mb-2 flex items-center justify-between">
                    <h4 class="font-medium">Results</h4>
                    <span class="text-sm text-muted-foreground">{{ results.length }} systems found</span>
                </div>
                <div class="max-h-80 overflow-auto rounded-lg border">
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead>System</TableHead>
                                <TableHead>Security</TableHead>
                                <TableHead>Region</TableHead>
                                <TableHead></TableHead>
                                <TableHead>Jumps</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            <DestinationContextMenu v-for="result in results" :key="result.solarsystem.id" :solarsystem_id="result.solarsystem.id">
                                <TableRow>
                                    <TableCell>
                                        <div class="flex items-center gap-2">
                                            <span class="font-medium">{{ result.solarsystem.name }}</span>
                                        </div>
                                    </TableCell>
                                    <TableCell class="text-center font-mono text-sm">
                                        <SolarsystemClass :wormhole_class="result.solarsystem.class" :security="result.solarsystem.security" />
                                    </TableCell>
                                    <TableCell class="text-sm">
                                        {{ result.solarsystem.region?.name ?? 'Unknown' }}
                                    </TableCell>
                                    <TableCell>
                                        <div class="flex justify-center">
                                            <SolarsystemSovereignty
                                                v-if="result.solarsystem.sovereignty"
                                                :sovereignty="result.solarsystem.sovereignty"
                                            />
                                            <SolarsystemEffect v-else-if="result.solarsystem.effect" :effect="result.solarsystem.effect" />
                                        </div>
                                    </TableCell>
                                    <TableCell class="text-center font-mono">
                                        {{ result.distance === 2147483647 ? '∞' : result.distance }}
                                    </TableCell>
                                </TableRow>
                            </DestinationContextMenu>
                        </TableBody>
                    </Table>
                </div>
            </div>
            <div v-else class="text-center text-sm text-muted-foreground">No results to display.</div>
        </DialogContent>
    </Dialog>
</template>

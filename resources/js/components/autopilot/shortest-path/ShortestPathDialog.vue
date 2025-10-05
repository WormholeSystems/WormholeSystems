<script setup lang="ts">
import AutopilotSettings from '@/components/autopilot/AutopilotSettings.vue';
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import SelectedSolarsystem from '@/components/autopilot/shortest-path/SelectedSolarsystem.vue';
import TimesIcon from '@/components/icons/TimesIcon.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import SolarsystemEffect from '@/components/solarsystem/SolarsystemEffect.vue';
import { Button } from '@/components/ui/button';
import { Combobox, ComboboxAnchor, ComboboxEmpty, ComboboxInput, ComboboxItem, ComboboxList } from '@/components/ui/combobox';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useIgnoreList } from '@/composables/useIgnoreList';
import { usePath } from '@/composables/usePath';
import { useSearch } from '@/composables/useSearch';
import { TShortestPathDialogProps } from '@/pages/maps';
import { TMapSolarSystem, TSolarsystem } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { vElementHover } from '@vueuse/components';
import { computed, ref, watch } from 'vue';

const props = defineProps<
    TShortestPathDialogProps & {
        selected_map_solarsystem?: TMapSolarSystem | null;
    }
>();

const open = ref(false);
const fromSystem = ref<TSolarsystem | null>(null);
const toSystem = ref<TSolarsystem | null>(null);
const route = ref<TSolarsystem[]>([]);
const selectionMode = ref<'from' | 'to'>('from');

const { setPath } = usePath();
const { ignoreSolarsystem, clearIgnoreList } = useIgnoreList();

const search = useSearch('search', ['solarsystems']);

const canCalculate = computed(() => {
    return fromSystem.value && toSystem.value && fromSystem.value.id !== toSystem.value.id;
});

const jumps = computed(() => {
    return route.value.length > 0 ? route.value.length - 1 : 0;
});

function onRouteHover(hovered: boolean) {
    if (hovered && route.value.length > 0) {
        setPath(route.value);
    } else {
        setPath(null);
    }
}

function handleFromSystemSelect(system: TSolarsystem) {
    fromSystem.value = system;
    search.value = '';
    route.value = [];
    selectionMode.value = 'to';

    if (toSystem.value && system.id !== toSystem.value.id) {
        calculateShortestPath();
    }
}

function handleToSystemSelect(system: TSolarsystem) {
    toSystem.value = system;
    search.value = ''; // Clear search after selection
    route.value = [];

    // Auto-calculate if both systems are now selected
    if (fromSystem.value && system.id !== fromSystem.value.id) {
        calculateShortestPath();
    }
}

function handleSystemSelect(system: TSolarsystem) {
    if (selectionMode.value === 'from') {
        handleFromSystemSelect(system);
    } else {
        handleToSystemSelect(system);
    }
}

function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'ArrowLeft') {
        event.preventDefault();
        selectionMode.value = 'from';
    } else if (event.key === 'ArrowRight') {
        event.preventDefault();
        selectionMode.value = 'to';
    }
}

function calculateShortestPath() {
    if (!canCalculate.value) return;

    router.reload({
        data: {
            from_solarsystem_id: fromSystem.value!.id,
            to_solarsystem_id: toSystem.value!.id,
        },
        only: ['shortest_path', 'map_user_settings'],
    });
}

function handleIgnoreSolarsystem(solarsystem_id: number) {
    ignoreSolarsystem(solarsystem_id, {
        only: ['shortest_path', 'ignored_systems'],
        onSuccess() {
            if (route.value.length > 0) {
                setPath(route.value);
            }
        },
    });
}

function handleClearIgnoreList() {
    clearIgnoreList({
        only: ['shortest_path', 'ignored_systems'],
        onSuccess() {
            if (route.value.length > 0) {
                setPath(route.value);
            }
        },
    });
}

watch(
    () => props.shortest_path,
    (newShortestPath) => {
        if (newShortestPath) {
            route.value = newShortestPath.route;
            fromSystem.value = newShortestPath.from_solarsystem;
            toSystem.value = newShortestPath.to_solarsystem;
        }
    },
    { immediate: true },
);
</script>

<template>
    <Dialog v-model:open="open">
        <DialogTrigger as-child>
            <slot />
        </DialogTrigger>
        <DialogContent class="flex h-[85vh] w-full max-w-7xl flex-col gap-0 p-0 focus:outline-0">
            <DialogHeader class="border-b px-6 py-4">
                <DialogTitle>Find Shortest Path</DialogTitle>
                <DialogDescription>Calculate the shortest route between two solar systems</DialogDescription>
            </DialogHeader>

            <!-- System Selection -->
            <div class="space-y-4 border-b px-6 py-4">
                <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                    <SelectedSolarsystem :solarsystem="fromSystem">
                        <template #label>From</template>
                    </SelectedSolarsystem>
                    <SelectedSolarsystem :solarsystem="toSystem">
                        <template #label>To</template>
                    </SelectedSolarsystem>
                </div>

                <div class="space-y-2">
                    <Combobox>
                        <ComboboxAnchor class="w-full rounded-lg border">
                            <ComboboxInput
                                v-model="search"
                                placeholder="Search for a solar system..."
                                id="system-search"
                                @keydown="handleKeydown"
                                class="w-full border-0"
                                auto-focus
                            />
                        </ComboboxAnchor>
                        <ComboboxList class="w-[600px]">
                            <div class="grid grid-cols-[auto_1fr_auto_auto_7rem] p-2">
                                <ComboboxEmpty class="col-span-full">No systems found.</ComboboxEmpty>
                                <ComboboxItem
                                    v-for="system in solarsystems"
                                    :key="system.id"
                                    :value="system.name"
                                    @select="handleSystemSelect(system)"
                                    class="group col-span-full grid h-10 grid-cols-subgrid items-center gap-2 py-2"
                                >
                                    <div class="flex justify-center">
                                        <SolarsystemClass :wormhole_class="system.class" :security="system.security" />
                                    </div>
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
                                    <div class="hidden gap-1 justify-self-end group-data-highlighted:flex">
                                        <Button
                                            size="xs"
                                            :variant="selectionMode === 'from' ? 'default' : 'outline'"
                                            @click="selectionMode = 'from'"
                                            :disabled="fromSystem?.id === system.id"
                                        >
                                            From
                                        </Button>
                                        <Button
                                            size="xs"
                                            :variant="selectionMode === 'to' ? 'default' : 'outline'"
                                            @click="selectionMode = 'to'"
                                            :disabled="toSystem?.id === system.id"
                                        >
                                            To
                                        </Button>
                                    </div>
                                </ComboboxItem>
                            </div>
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
                        <div class="flex shrink-0 gap-2">
                            <Button variant="outline" size="sm" @click="handleFromSystemSelect(selected_map_solarsystem.solarsystem)">
                                Set as From
                            </Button>
                            <Button variant="outline" size="sm" @click="handleToSystemSelect(selected_map_solarsystem.solarsystem)">
                                Set as To
                            </Button>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Route Controls -->
            <div v-if="fromSystem && toSystem" class="flex items-center justify-between border-b px-6 py-3">
                <div class="flex items-center gap-2">
                    <Button v-if="ignored_systems.length > 0" variant="secondary" size="sm" @click="handleClearIgnoreList">
                        Clear Ignored ({{ ignored_systems.length }})
                    </Button>
                    <span v-if="route.length" class="text-sm text-muted-foreground">{{ jumps }} jumps</span>
                </div>
                <AutopilotSettings />
            </div>

            <!-- Route Results -->
            <div class="flex flex-1 flex-col overflow-hidden">
                <div v-if="route.length > 0" class="flex flex-1 flex-col overflow-hidden" v-element-hover="onRouteHover">
                    <div class="flex-1 overflow-y-auto">
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead class="w-16"></TableHead>
                                    <TableHead>System</TableHead>
                                    <TableHead>Region</TableHead>
                                    <TableHead class="w-24"></TableHead>
                                    <TableHead class="w-16"></TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                <DestinationContextMenu v-for="(system, index) in route" :key="system.id" :solarsystem_id="system.id">
                                    <TableRow>
                                        <TableCell>
                                            <div class="flex justify-center">
                                                <SolarsystemClass :wormhole_class="system.class" :security="system.security" />
                                            </div>
                                        </TableCell>
                                        <TableCell>
                                            <span class="font-medium">{{ system.name }}</span>
                                        </TableCell>
                                        <TableCell class="text-muted-foreground">
                                            {{ system.region?.name }}
                                        </TableCell>
                                        <TableCell>
                                            <div class="flex justify-center">
                                                <SolarsystemSovereignty v-if="system.sovereignty" :sovereignty="system.sovereignty" />
                                                <SolarsystemEffect v-else-if="system.effect" :effect="system.effect" />
                                            </div>
                                        </TableCell>
                                        <TableCell>
                                            <div class="flex justify-center">
                                                <Tooltip v-if="route && index !== 0 && index !== route.length - 1">
                                                    <TooltipTrigger as-child>
                                                        <Button
                                                            variant="ghost"
                                                            size="icon"
                                                            class="h-8 w-8"
                                                            @click="handleIgnoreSolarsystem(system.id)"
                                                        >
                                                            <TimesIcon class="h-4 w-4" />
                                                        </Button>
                                                    </TooltipTrigger>
                                                    <TooltipContent>Ignore this system</TooltipContent>
                                                </Tooltip>
                                            </div>
                                        </TableCell>
                                    </TableRow>
                                </DestinationContextMenu>
                            </TableBody>
                        </Table>
                    </div>
                </div>
                <div v-else-if="fromSystem && toSystem" class="flex flex-1 items-center justify-center text-sm text-muted-foreground">
                    No route found between the selected systems.
                </div>
                <div v-else class="flex flex-1 items-center justify-center text-sm text-muted-foreground">
                    Select both systems to calculate the shortest path.
                </div>
            </div>
        </DialogContent>
    </Dialog>
</template>

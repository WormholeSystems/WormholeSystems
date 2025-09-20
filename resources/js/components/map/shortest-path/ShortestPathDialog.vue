<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import TimesIcon from '@/components/icons/TimesIcon.vue';
import SelectedSolarsystem from '@/components/map/shortest-path/SelectedSolarsystem.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import SolarsystemEffect from '@/components/solarsystem/SolarsystemEffect.vue';
import { Button } from '@/components/ui/button';
import { Combobox, ComboboxAnchor, ComboboxEmpty, ComboboxInput, ComboboxItem, ComboboxList } from '@/components/ui/combobox';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useIgnoreList } from '@/composables/useIgnoreList';
import { usePath } from '@/composables/usePath';
import { useSearch } from '@/composables/useSearch';
import { TShortestPathDialogProps } from '@/pages/maps';
import { TSolarsystem } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { vElementHover } from '@vueuse/components';
import { computed, ref, watch } from 'vue';

const props = defineProps<TShortestPathDialogProps>();

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
        only: ['shortest_path'],
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
        <DialogContent class="flex min-h-[600px] max-w-2xl flex-col gap-4 py-4">
            <DialogHeader>
                <DialogTitle>Find Shortest Path</DialogTitle>
                <DialogDescription> Calculate the shortest route between two solar systems</DialogDescription>
            </DialogHeader>
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                <SelectedSolarsystem :solarsystem="fromSystem">
                    <template #label>From</template>
                </SelectedSolarsystem>
                <SelectedSolarsystem :solarsystem="toSystem">
                    <template #label>To</template>
                </SelectedSolarsystem>
            </div>

            <div class="grid gap-2">
                <Combobox class="rounded-lg border">
                    <ComboboxAnchor class="w-full">
                        <ComboboxInput
                            v-model="search"
                            placeholder="Search for a solar system..."
                            id="system-search"
                            @keydown="handleKeydown"
                            class="w-full"
                            auto-focus
                        />
                    </ComboboxAnchor>
                    <ComboboxList class="w-100">
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
            </div>

            <Button v-if="ignored_systems.length > 0" variant="secondary" @click="handleClearIgnoreList" class="w-full">
                Clear Ignored ({{ ignored_systems.length }})
            </Button>

            <div class="mt-4 h-full grow" v-element-hover="onRouteHover" v-if="route.length">
                <div class="mb-2 flex items-center justify-between">
                    <span class="text-sm font-medium">Route</span>
                    <span class="text-sm text-muted-foreground" v-if="route.length">{{ jumps }} jumps</span>
                </div>
                <div class="max-h-64 overflow-y-auto rounded border bg-muted/50">
                    <Table>
                        <TableHeader>
                            <TableRow class="text-xs">
                                <TableHead class="h-auto w-8 p-1"></TableHead>
                                <TableHead class="h-auto p-1">System</TableHead>
                                <TableHead class="h-auto p-1">Region</TableHead>
                                <TableHead class="h-auto w-8 p-1"></TableHead>
                                <TableHead class="h-auto w-8 p-1"></TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            <DestinationContextMenu v-for="(system, index) in route" :key="system.id" :solarsystem_id="system.id">
                                <TableRow class="text-xs">
                                    <TableCell class="h-auto p-1">
                                        <div class="flex justify-center">
                                            <SolarsystemClass :wormhole_class="system.class" :security="system.security" />
                                        </div>
                                    </TableCell>
                                    <TableCell class="h-auto p-1">
                                        <span class="font-medium">{{ system.name }}</span>
                                    </TableCell>
                                    <TableCell class="h-auto p-1 text-muted-foreground">
                                        {{ system.region?.name }}
                                    </TableCell>
                                    <TableCell class="h-auto p-1">
                                        <div class="flex justify-center">
                                            <SolarsystemSovereignty v-if="system.sovereignty" :sovereignty="system.sovereignty" />
                                            <SolarsystemEffect v-else-if="system.effect" :effect="system.effect" />
                                        </div>
                                    </TableCell>
                                    <TableCell class="h-auto p-1">
                                        <template v-if="route && index !== 0 && index !== route.length - 1">
                                            <Tooltip>
                                                <TooltipTrigger as-child>
                                                    <Button
                                                        variant="secondary"
                                                        size="icon"
                                                        class="h-6 w-6"
                                                        @click="handleIgnoreSolarsystem(system.id)"
                                                    >
                                                        <TimesIcon class="h-2.5 w-2.5" />
                                                    </Button>
                                                </TooltipTrigger>
                                                <TooltipContent>Ignore this system</TooltipContent>
                                            </Tooltip>
                                        </template>
                                    </TableCell>
                                </TableRow>
                            </DestinationContextMenu>
                        </TableBody>
                    </Table>
                </div>
            </div>
            <div
                v-else
                class="flex h-full grow items-center justify-center rounded-lg border border-dashed bg-muted/50 text-sm text-muted-foreground"
            >
                No route found. Select systems to calculate the shortest path.
            </div>

            <DialogFooter>
                <Button variant="outline" @click="open = false"> Close</Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>
</template>

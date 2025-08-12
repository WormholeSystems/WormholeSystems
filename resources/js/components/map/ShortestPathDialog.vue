<script setup lang="ts">
import DestinationContextMenu from '@/components/DestinationContextMenu.vue';
import TimesIcon from '@/components/icons/TimesIcon.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import SolarsystemEffect from '@/components/SolarsystemEffect.vue';
import { Button } from '@/components/ui/button';
import { Combobox, ComboboxAnchor, ComboboxEmpty, ComboboxInput, ComboboxItem, ComboboxList } from '@/components/ui/combobox';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Label } from '@/components/ui/label';
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
const loading = ref(false);
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
    search.value = ''; // Clear search after selection
    route.value = [];
    selectionMode.value = 'to'; // Switch to 'to' mode after selecting from
}

function handleToSystemSelect(system: TSolarsystem) {
    toSystem.value = system;
    search.value = ''; // Clear search after selection
    route.value = [];
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
        <DialogContent class="max-w-2xl">
            <DialogHeader>
                <DialogTitle>Find Shortest Path</DialogTitle>
                <DialogDescription> Calculate the shortest route between two solar systems</DialogDescription>
            </DialogHeader>

            <div class="grid gap-4 py-4">
                <!-- Selected Systems Display -->
                <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                    <div class="rounded-lg border p-3">
                        <Label class="text-xs text-muted-foreground">From</Label>
                        <div v-if="fromSystem" class="grid grid-cols-[1fr_auto]">
                            <div class="font-medium">{{ fromSystem.name }}</div>
                            <div class="">
                                <SolarsystemClass :wormhole_class="fromSystem.class" :security="fromSystem.security" />
                            </div>
                            <div class="text-sm text-muted-foreground">{{ fromSystem.region?.name }}</div>
                            <div class="">
                                <SolarsystemSovereignty v-if="fromSystem.sovereignty" :sovereignty="fromSystem.sovereignty" />
                            </div>
                        </div>
                        <div v-else class="mt-1 text-sm text-muted-foreground">No system selected</div>
                    </div>

                    <div class="rounded-lg border p-3">
                        <Label class="text-xs text-muted-foreground">To</Label>
                        <div v-if="toSystem" class="grid grid-cols-[1fr_auto]">
                            <div class="font-medium">{{ toSystem.name }}</div>
                            <div class="">
                                <SolarsystemClass :wormhole_class="toSystem.class" :security="toSystem.security" />
                            </div>
                            <div class="text-sm text-muted-foreground">{{ toSystem.region?.name }}</div>
                            <div class="">
                                <SolarsystemSovereignty v-if="toSystem.sovereignty" :sovereignty="toSystem.sovereignty" />
                            </div>
                        </div>
                        <div v-else class="mt-1 text-sm text-muted-foreground">No system selected</div>
                    </div>
                </div>

                <!-- System Search -->
                <div class="grid gap-2">
                    <Label for="system-search">Search Systems</Label>
                    <Combobox class="rounded-lg border">
                        <ComboboxAnchor class="w-full">
                            <ComboboxInput
                                v-model="search"
                                :placeholder="`Type to search for ${selectionMode} system...`"
                                id="system-search"
                                @keydown="handleKeydown"
                                class="w-full"
                            />
                        </ComboboxAnchor>
                        <ComboboxList class="w-100">
                            <ComboboxEmpty>No systems found.</ComboboxEmpty>
                            <ComboboxItem
                                v-for="system in solarsystems"
                                :key="system.id"
                                :value="system.name"
                                @select="handleSystemSelect(system)"
                                class="flex items-center justify-between"
                            >
                                <div class="flex flex-col items-start">
                                    <span class="font-medium">{{ system.name }}</span>
                                    <span class="text-sm text-muted-foreground">
                                        {{ system.region?.name }}
                                    </span>
                                </div>
                                <div class="flex gap-1">
                                    <Button
                                        size="sm"
                                        :variant="selectionMode === 'from' ? 'default' : 'outline'"
                                        @click="selectionMode = 'from'"
                                        :disabled="fromSystem?.id === system.id"
                                    >
                                        From
                                    </Button>
                                    <Button
                                        size="sm"
                                        :variant="selectionMode === 'to' ? 'default' : 'outline'"
                                        @click="selectionMode = 'to'"
                                        :disabled="toSystem?.id === system.id"
                                    >
                                        To
                                    </Button>
                                </div>
                            </ComboboxItem>
                        </ComboboxList>
                    </Combobox>
                </div>

                <div class="flex gap-2">
                    <Button @click="calculateShortestPath" :disabled="!canCalculate || loading" class="flex-1">
                        <span v-if="loading">Calculating...</span>
                        <span v-else>Calculate Route</span>
                    </Button>
                    <Button v-if="ignored_systems.length > 0" variant="secondary" @click="handleClearIgnoreList" class="shrink-0">
                        Clear Ignored ({{ ignored_systems.length }})
                    </Button>
                </div>

                <div v-if="route.length > 0" class="mt-4" v-element-hover="onRouteHover">
                    <div class="mb-2 flex items-center justify-between">
                        <span class="text-sm font-medium">Route Found</span>
                        <span class="text-sm text-muted-foreground">{{ jumps }} jumps</span>
                    </div>
                    <div class="max-h-64 overflow-y-auto rounded border bg-muted/20">
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
            </div>

            <DialogFooter>
                <Button variant="outline" @click="open = false"> Close</Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>
</template>

<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import QuickSelectButtons from '@/components/autopilot/QuickSelectButtons.vue';
import SystemComboboxList from '@/components/autopilot/SystemComboboxList.vue';
import TimesIcon from '@/components/icons/TimesIcon.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import SolarsystemEffect from '@/components/solarsystem/SolarsystemEffect.vue';
import { Button } from '@/components/ui/button';
import { Combobox, ComboboxAnchor, ComboboxInput } from '@/components/ui/combobox';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useIgnoreList } from '@/composables/useIgnoreList';
import { usePath } from '@/composables/usePath';
import { useSearch } from '@/composables/useSearch';
import { TMap, TSelectedMapSolarsystem, TShortestPath, TSolarsystem } from '@/pages/maps';
import { TCharacter, TCharacterStatus, TMapRouteSolarsystem } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { vElementHover } from '@vueuse/components';
import { X } from 'lucide-vue-next';
import { computed, ref, watch } from 'vue';

const { solarsystems, shortest_path, selected_map_solarsystem, ignored_systems, active_character, character_status, destinations } = defineProps<{
    map: TMap;
    solarsystems: TSolarsystem[];
    selected_map_solarsystem?: TSelectedMapSolarsystem | null;
    shortest_path: TShortestPath | null;
    ignored_systems: number[];
    active_character?: TCharacter | null;
    character_status?: TCharacterStatus | null;
    destinations: TMapRouteSolarsystem[];
}>();

const fromSystem = ref<TSolarsystem | null>(null);
const toSystem = ref<TSolarsystem | null>(null);
const route = ref<TSolarsystem[]>([]);

const { setPath } = usePath();
const { ignoreSolarsystem, clearIgnoreList } = useIgnoreList();

const search = useSearch('search', ['solarsystems']);

const canCalculate = computed(() => {
    return fromSystem.value && toSystem.value && fromSystem.value.id !== toSystem.value.id;
});

const jumps = computed(() => {
    return route.value.length > 0 ? route.value.length - 1 : 0;
});

const activeCharacterSystem = computed(() => {
    if (!active_character || !character_status?.solarsystem) return null;
    return character_status.solarsystem;
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

    if (toSystem.value && system.id !== toSystem.value.id) {
        calculateShortestPath();
    }
}

function handleToSystemSelect(system: TSolarsystem) {
    toSystem.value = system;
    search.value = '';
    route.value = [];

    if (fromSystem.value && system.id !== fromSystem.value.id) {
        calculateShortestPath();
    }
}

function calculateShortestPath() {
    if (!canCalculate.value) return;

    router.reload({
        data: {
            from_solarsystem_id: fromSystem.value!.id,
            to_solarsystem_id: toSystem.value!.id,
        },
        only: ['map_navigation', 'map_user_settings'],
    });
}

function handleIgnoreSolarsystem(solarsystem_id: number) {
    ignoreSolarsystem(solarsystem_id, {
        only: ['map_navigation', 'ignored_systems'],
        onSuccess() {
            if (route.value.length > 0) {
                setPath(route.value);
            }
        },
    });
}

function handleClearIgnoreList() {
    clearIgnoreList({
        only: ['map_navigation', 'ignored_systems'],
        onSuccess() {
            if (route.value.length > 0) {
                setPath(route.value);
            }
        },
    });
}

function clearFrom() {
    fromSystem.value = null;
    route.value = [];
    router.reload({
        data: {
            from_solarsystem_id: null,
            to_solarsystem_id: toSystem.value?.id || null,
        },
        only: ['map_navigation'],
    });
}

function clearTo() {
    toSystem.value = null;
    route.value = [];
    router.reload({
        data: {
            from_solarsystem_id: fromSystem.value?.id || null,
            to_solarsystem_id: null,
        },
        only: ['map_navigation'],
    });
}

watch(
    () => shortest_path,
    (newShortestPath) => {
        if (newShortestPath) {
            route.value = newShortestPath.route;
            fromSystem.value = newShortestPath.from_solarsystem;
            toSystem.value = newShortestPath.to_solarsystem;
        } else {
            route.value = [];
        }
    },
    { immediate: true },
);
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
                        <SolarsystemSovereignty v-if="fromSystem.sovereignty" :sovereignty="fromSystem.sovereignty" class="shrink-0" />
                        <SolarsystemEffect v-else-if="fromSystem.effect" :effect="fromSystem.effect.name" class="shrink-0" />
                        <span v-else />
                        <Button variant="ghost" size="icon" @click="clearFrom" class="h-6 w-6 shrink-0">
                            <X class="size-3" />
                        </Button>
                    </div>
                </template>
                <template v-else>
                    <ComboboxInput v-model="search" placeholder="Search system..." class="rounded-lg border" />
                </template>
            </ComboboxAnchor>
            <SystemComboboxList :solarsystems="solarsystems" @select="handleFromSystemSelect" />
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
                        <SolarsystemSovereignty v-if="toSystem.sovereignty" :sovereignty="toSystem.sovereignty" class="shrink-0" />
                        <SolarsystemEffect v-else-if="toSystem.effect" :effect="toSystem.effect.name" class="shrink-0" />
                        <span v-else />
                        <Button variant="ghost" size="icon" @click="clearTo" class="h-6 w-6 shrink-0">
                            <X class="size-3" />
                        </Button>
                    </div>
                </template>
                <template v-else>
                    <ComboboxInput v-model="search" placeholder="Search system..." class="rounded-lg border" />
                </template>
            </ComboboxAnchor>
            <SystemComboboxList :solarsystems="solarsystems" @select="handleToSystemSelect" />
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
        <span v-if="route.length" class="text-sm text-muted-foreground">{{ jumps }} jumps</span>
        <Button v-if="ignored_systems.length > 0" variant="secondary" size="sm" @click="handleClearIgnoreList">
            Clear Ignored ({{ ignored_systems.length }})
        </Button>
    </div>

    <!-- Route Results -->
    <div v-if="route.length > 0" v-element-hover="onRouteHover" class="grid grid-cols-[auto_auto_1fr_auto_auto] rounded-lg border bg-card text-sm">
        <DestinationContextMenu v-for="(system, index) in route" :key="system.id" :solarsystem_id="system.id">
            <div
                class="col-span-full grid grid-cols-subgrid items-center gap-2 border-b px-2.5 py-1.5 transition-colors last:border-b-0 hover:bg-accent/50"
            >
                <div class="flex items-center justify-center text-xs font-medium text-muted-foreground">
                    {{ index + 1 }}
                </div>
                <div class="justify-self-center">
                    <SolarsystemClass :wormhole_class="system.class" :security="system.security" />
                </div>
                <div class="min-w-0 text-xs">
                    <span class="font-medium">{{ system.name }}</span>
                    <span class="text-muted-foreground"> · {{ system.region?.name }}</span>
                </div>
                <div class="flex items-center justify-center">
                    <SolarsystemSovereignty v-if="system.sovereignty" :sovereignty="system.sovereignty" />
                    <SolarsystemEffect v-else-if="system.effect" :effect="system.effect.name" />
                </div>
                <div class="flex items-center justify-center">
                    <Tooltip v-if="route && index !== 0 && index !== route.length - 1">
                        <TooltipTrigger as-child>
                            <Button variant="ghost" size="icon" class="h-6 w-6" @click="handleIgnoreSolarsystem(system.id)">
                                <TimesIcon class="h-3 w-3" />
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

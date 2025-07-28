<script setup lang="ts">
import PinIcon from '@/components/icons/PinIcon.vue';
import SettingsIcon from '@/components/icons/SettingsIcon.vue';
import Spinner from '@/components/icons/Spinner.vue';
import TrashIcon from '@/components/icons/TrashIcon.vue';
import { CharacterImage } from '@/components/images';
import TypeImage from '@/components/images/TypeImage.vue';
import SovereigntyIcon from '@/components/map/SovereigntyIcon.vue';
import MapRouteSolarsystemAdd from '@/components/routes/MapRouteSolarsystemAdd.vue';
import RoutePopover from '@/components/routes/RoutePopover.vue';
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useHasWritePermission } from '@/composables/useHasPermission';
import { usePath } from '@/composables/usePath';
import useUser from '@/composables/useUser';
import { TCharacter, TMap, TMapRouteSolarsystem, TMapSolarSystem, TSolarsystem } from '@/types/models';
import { Deferred, router } from '@inertiajs/vue3';
import { vElementHover } from '@vueuse/components';
import { computed } from 'vue';

const { map_route_solarsystems, map, solarsystems, allow_eol, allow_crit, allow_eve_scout, map_characters } = defineProps<{
    map: TMap;
    solarsystems: TSolarsystem[];
    map_route_solarsystems?: TMapRouteSolarsystem[];
    selected_map_solarsystem?: TMapSolarSystem;
    allow_eol: boolean;
    allow_crit: boolean;
    allow_eve_scout: boolean;
    map_characters?: TCharacter[];
}>();

const user = useUser();
const activeCharacter = computed(() => {
    if (!map_characters || !user.value?.active_character) return null;
    return map_characters.find((character) => character.id === user.value.active_character.id);
});
const characterStatus = computed(() => activeCharacter.value?.status);

const sorted = computed(() => {
    return map_route_solarsystems?.toSorted((a, b) => {
        if (a.is_pinned && !b.is_pinned) return -1;
        if (!a.is_pinned && b.is_pinned) return 1;
        return a.solarsystem.name.localeCompare(b.solarsystem.name);
    });
});

const can_write = useHasWritePermission();

const { setPath } = usePath();

function handleToggleEol(value: boolean | 'indeterminate') {
    router.put(
        route('watchlist.update'),
        {
            allow_eol: value === true,
        },
        {
            preserveScroll: true,
            only: ['map_route_solarsystems', 'allow_eol', 'allow_crit', 'allow_eve_scout'],
            preserveState: true,
        },
    );
}

function handleToggleCrit(value: boolean | 'indeterminate') {
    router.put(
        route('watchlist.update'),
        {
            allow_crit: value === true,
        },
        {
            preserveScroll: true,
            only: ['map_route_solarsystems', 'allow_eol', 'allow_crit', 'allow_eve_scout'],
            preserveState: true,
        },
    );
}

function handleToggleEveScout(value: boolean | 'indeterminate') {
    router.put(
        route('watchlist.update'),
        {
            allow_eve_scout: value === true,
        },
        {
            preserveScroll: true,
            only: ['map_route_solarsystems', 'allow_eol', 'allow_crit', 'allow_eve_scout'],
            preserveState: true,
        },
    );
}

function togglePinned(routeItem: TMapRouteSolarsystem) {
    router.put(
        route('map-route-solarsystems.update', routeItem.id),
        {
            is_pinned: !routeItem.is_pinned,
        },
        {
            preserveScroll: true,
            preserveState: true,
            only: ['map_route_solarsystems'],
        },
    );
}

function removeRoute(routeItem: TMapRouteSolarsystem) {
    router.delete(route('map-route-solarsystems.destroy', routeItem.id), {
        preserveScroll: true,
        preserveState: true,
        only: ['map_route_solarsystems'],
    });
}

function handleHover(hovered: boolean, route: TSolarsystem[] | null) {
    if (hovered) {
        setPath(route);
    } else {
        setPath(null);
    }
}
</script>

<template>
    <Card class="bg-neutral-50 pb-0 dark:bg-transparent">
        <CardHeader>
            <CardTitle class="text-base">Autopilot</CardTitle>
            <CardDescription> Manage your autopilot routes and settings.</CardDescription>
            <CardAction class="flex gap-2">
                <Popover>
                    <PopoverTrigger>
                        <Button variant="outline" size="icon">
                            <SettingsIcon />
                        </Button>
                    </PopoverTrigger>
                    <PopoverContent class="w-64 p-3">
                        <div class="space-y-3">
                            <div class="text-sm font-medium">Route Settings</div>
                            <div class="space-y-2">
                                <div class="flex items-center gap-2">
                                    <Checkbox :model-value="allow_eol" @update:model-value="handleToggleEol" id="eol-checkbox" />
                                    <label for="eol-checkbox" class="cursor-pointer text-xs font-medium"> Allow EOL connections </label>
                                </div>

                                <div class="flex items-center gap-2">
                                    <Checkbox :model-value="allow_crit" @update:model-value="handleToggleCrit" id="crit-checkbox" />
                                    <label for="crit-checkbox" class="cursor-pointer text-xs font-medium"> Allow critical mass </label>
                                </div>

                                <div class="flex items-center gap-2">
                                    <Checkbox :model-value="allow_eve_scout" @update:model-value="handleToggleEveScout" id="thera-checkbox" />
                                    <label for="thera-checkbox" class="cursor-pointer text-xs font-medium"> Use Thera/Turnur </label>
                                </div>
                            </div>
                        </div>
                    </PopoverContent>
                </Popover>
                <MapRouteSolarsystemAdd :map :solarsystems :map_route_solarsystems v-if="can_write" />
            </CardAction>
        </CardHeader>

        <CardContent class="p-2 pt-0">
            <!-- Pilot Status -->
            <div
                class="mb-2 flex items-center gap-2 rounded border bg-white p-2 dark:bg-neutral-900/40"
                v-element-hover="(e) => handleHover(e, activeCharacter?.route ?? null)"
                v-if="activeCharacter"
            >
                <CharacterImage
                    v-if="activeCharacter"
                    :character_id="activeCharacter.id"
                    :character_name="activeCharacter.name"
                    class="size-8 rounded-lg"
                />
                <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                        <span class="truncate text-sm font-medium">{{ activeCharacter?.name || 'Unknown' }}</span>
                    </div>
                    <div class="flex items-center gap-1 text-xs text-muted-foreground">
                        <TypeImage
                            v-if="characterStatus?.ship_type"
                            :type_id="characterStatus.ship_type.id"
                            :type_name="characterStatus.ship_type.name"
                            class="size-3 rounded"
                        />
                        <span class="truncate">{{ characterStatus?.ship_name || characterStatus?.ship_type?.name || 'Unknown Ship' }}</span>
                        <span v-if="characterStatus?.solarsystem" class="text-muted-foreground">• {{ characterStatus.solarsystem.name }}</span>
                    </div>
                </div>
                <div v-if="activeCharacter?.route" class="flex-shrink-0">
                    <RoutePopover :route="activeCharacter.route">
                        <Button variant="outline" size="sm" class="font-mono text-xs">
                            {{ activeCharacter.route.length > 0 ? activeCharacter.route.length - 1 : '0' }}
                        </Button>
                    </RoutePopover>
                </div>
            </div>

            <!-- Destinations Grid -->
            <Deferred data="map_route_solarsystems">
                <template #fallback>
                    <div class="flex items-center justify-center gap-2 py-3 text-xs text-muted-foreground">
                        <Spinner class="size-3 animate-spin" />
                        <span>Loading routes...</span>
                    </div>
                </template>

                <div
                    :class="can_write ? 'grid-cols-[auto_1fr_auto_1fr_auto_auto]' : 'grid-cols-[auto_1fr_auto_1fr_auto]'"
                    class="grid gap-x-4 rounded border bg-white text-xs dark:bg-neutral-900/40"
                >
                    <!-- Grid Header -->
                    <div
                        class="col-span-full grid grid-cols-subgrid border-b bg-neutral-50 px-3 py-1 text-[10px] font-medium text-muted-foreground dark:bg-neutral-800/50"
                    >
                        <div></div>
                        <div>System</div>
                        <div class="text-center">Jumps</div>
                        <div>Region</div>
                        <div class="text-center">Sov</div>
                        <div class="text-center" v-if="can_write">Actions</div>
                    </div>

                    <!-- Grid Rows -->
                    <div
                        v-for="route in sorted"
                        :key="route.solarsystem.id"
                        class="group col-span-full grid grid-cols-subgrid items-center border-b px-3 py-1 last:border-b-0 hover:bg-neutral-50 dark:hover:bg-neutral-800/30"
                        v-element-hover="(e) => handleHover(e, route.route)"
                    >
                        <!-- Class Column -->
                        <div class="flex justify-center">
                            <SolarsystemClass :wormhole_class="route.solarsystem.class" :security="route.solarsystem.security" />
                        </div>

                        <!-- Name Column -->
                        <div class="min-w-0 truncate font-medium">
                            {{ route.solarsystem.name }}
                        </div>

                        <!-- Jumps Column -->
                        <div class="flex justify-center">
                            <RoutePopover :route="route.route">
                                <Button variant="outline" size="sm" class="h-5 w-8 px-0 font-mono text-[10px] font-medium">
                                    <span v-if="route.route && route.route.length > 0">
                                        {{ route.route.length - 1 }}
                                    </span>
                                    <span v-else>∞</span>
                                </Button>
                            </RoutePopover>
                        </div>

                        <!-- Region Column -->
                        <div class="min-w-0 truncate text-[10px] text-muted-foreground">
                            {{ route.solarsystem.region?.name || '' }}
                        </div>

                        <!-- Sovereignty Column -->
                        <div class="flex justify-center">
                            <SovereigntyIcon v-if="route.solarsystem.sovereignty" :sovereignty="route.solarsystem.sovereignty" />
                            <div v-else class="size-4"></div>
                        </div>

                        <!-- Actions Column -->
                        <div v-if="can_write" class="flex justify-center">
                            <Tooltip>
                                <TooltipTrigger>
                                    <Button
                                        variant="ghost"
                                        size="sm"
                                        @click="togglePinned(route)"
                                        :class="route.is_pinned ? 'text-yellow-600' : 'text-muted-foreground'"
                                        class="h-5 w-5 p-0"
                                    >
                                        <PinIcon class="size-3" />
                                    </Button>
                                </TooltipTrigger>
                                <TooltipContent>{{ route.is_pinned ? 'Unpin' : 'Pin' }}</TooltipContent>
                            </Tooltip>

                            <Tooltip>
                                <TooltipTrigger>
                                    <Button
                                        variant="ghost"
                                        size="sm"
                                        @click="removeRoute(route)"
                                        class="h-5 w-5 p-0 text-muted-foreground hover:text-red-600"
                                    >
                                        <TrashIcon class="size-3" />
                                    </Button>
                                </TooltipTrigger>
                                <TooltipContent>Remove</TooltipContent>
                            </Tooltip>
                        </div>
                    </div>

                    <!-- Empty State -->
                    <div v-if="!sorted?.length" class="col-span-full py-4 text-center text-xs text-muted-foreground">
                        <div class="mb-1 text-sm">🎯</div>
                        <div>No destinations</div>
                    </div>
                </div>
            </Deferred>
        </CardContent>
    </Card>
</template>

<style scoped></style>

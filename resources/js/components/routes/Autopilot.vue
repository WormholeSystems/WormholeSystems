<script setup lang="ts">
import SettingsIcon from '@/components/icons/SettingsIcon.vue';
import Spinner from '@/components/icons/Spinner.vue';
import { CharacterImage } from '@/components/images';
import TypeImage from '@/components/images/TypeImage.vue';
import MapRouteSolarsystem from '@/components/routes/MapRouteSolarsystem.vue';
import MapRouteSolarsystemAdd from '@/components/routes/MapRouteSolarsystemAdd.vue';
import RoutePopover from '@/components/routes/RoutePopover.vue';
import { Button } from '@/components/ui/button';
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { useHasWritePermission } from '@/composables/useHasPermission';
import { usePath } from '@/composables/usePath';
import useUser from '@/composables/useUser';
import Watchlist from '@/routes/watchlist';
import { TCharacter, TMap, TMapRouteSolarsystem, TMapSolarSystem, TMassStatus, TSolarsystem } from '@/types/models';
import { Deferred, router } from '@inertiajs/vue3';
import { vElementHover } from '@vueuse/components';
import { computed } from 'vue';

const { map_route_solarsystems, map, solarsystems, allow_eol, allow_mass, allow_eve_scout, map_characters } = defineProps<{
    map: TMap;
    solarsystems: TSolarsystem[];
    map_route_solarsystems?: TMapRouteSolarsystem[];
    selected_map_solarsystem?: TMapSolarSystem;
    allow_eol: boolean;
    allow_mass: TMassStatus;
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
        Watchlist.update().url,
        {
            allow_eol: value === true,
        },
        {
            preserveScroll: true,
            only: ['map_route_solarsystems', 'allow_eol', 'allow_mass', 'allow_eve_scout'],
            preserveState: true,
        },
    );
}

function handleToggleMass(value: string) {
    router.put(
        Watchlist.update().url,
        {
            allow_mass: value,
        },
        {
            preserveScroll: true,
            only: ['map_route_solarsystems', 'allow_eol', 'allow_mass', 'allow_eve_scout'],
            preserveState: true,
        },
    );
}

function handleToggleEveScout(value: boolean | 'indeterminate') {
    router.put(
        Watchlist.update().url,
        {
            allow_eve_scout: value === true,
        },
        {
            preserveScroll: true,
            only: ['map_route_solarsystems', 'allow_eol', 'allow_mass', 'allow_eve_scout'],
            preserveState: true,
        },
    );
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
                        <Button variant="secondary" size="icon">
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
                                    <Checkbox :model-value="allow_eve_scout" @update:model-value="handleToggleEveScout" id="thera-checkbox" />
                                    <label for="thera-checkbox" class="cursor-pointer text-xs font-medium"> Use Thera/Turnur </label>
                                </div>
                                <RadioGroup :model-value="allow_mass" @update:model-value="handleToggleMass">
                                    <div class="flex items-center gap-2">
                                        <RadioGroupItem value="critical" id="crit" />
                                        <Label class="cursor-pointer text-xs font-medium" for="crit"> Allow critical</Label>
                                    </div>
                                    <div class="flex items-center gap-2">
                                        <RadioGroupItem value="reduced" id="reduced" />
                                        <Label class="cursor-pointer text-xs font-medium" for="reduced">Allow reduced</Label>
                                    </div>
                                    <div class="flex items-center gap-2">
                                        <RadioGroupItem value="fresh" id="fresh" />
                                        <Label class="cursor-pointer text-xs font-medium" for="fresh">Only fresh</Label>
                                    </div>
                                </RadioGroup>
                            </div>
                        </div>
                    </PopoverContent>
                </Popover>
                <MapRouteSolarsystemAdd :map :solarsystems :map_route_solarsystems v-if="can_write" />
            </CardAction>
        </CardHeader>

        <CardContent class="p-1 pt-0">
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
                        <Button variant="secondary" size="sm" class="font-mono text-xs">
                            {{ activeCharacter.route.length > 0 ? activeCharacter.route.length - 1 : '0' }}
                        </Button>
                    </RoutePopover>
                </div>
            </div>
            <Deferred data="map_route_solarsystems">
                <template #fallback>
                    <div class="flex items-center justify-center gap-2 py-3 text-xs text-muted-foreground">
                        <Spinner class="size-3 animate-spin" />
                        <span>Loading routes...</span>
                    </div>
                </template>

                <div
                    :class="can_write ? 'grid-cols-[auto_1fr_auto_1fr_auto_auto]' : 'grid-cols-[auto_1fr_auto_1fr_auto]'"
                    class="grid gap-x-4 rounded border bg-white dark:bg-neutral-900/40"
                >
                    <div
                        class="col-span-full grid grid-cols-subgrid border-b bg-neutral-50 px-3 py-1 font-medium text-muted-foreground dark:bg-neutral-800/50"
                    >
                        <div></div>
                        <div>System</div>
                        <div class="text-center">Jumps</div>
                        <div>Region</div>
                    </div>

                    <MapRouteSolarsystem v-for="route in sorted" :key="route.solarsystem.id" :map_route="route" />

                    <div v-if="!sorted?.length" class="col-span-full py-4 text-center text-muted-foreground">
                        <div class="mb-1 text-sm">🎯</div>
                        <div>No destinations</div>
                    </div>
                </div>
            </Deferred>
        </CardContent>
    </Card>
</template>

<style scoped></style>

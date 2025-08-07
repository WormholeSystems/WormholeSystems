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
import { Label } from '@/components/ui/label';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { useMapSolarsystems } from '@/composables/map';
import { useHasWritePermission } from '@/composables/useHasPermission';
import { usePath } from '@/composables/usePath';
import useUser from '@/composables/useUser';
import MapUserSettings from '@/routes/map-user-settings';
import { TCharacter, TMap, TMapRouteSolarsystem, TMapSolarSystem, TMapUserSetting, TMassStatus, TSolarsystem } from '@/types/models';
import { Deferred, router } from '@inertiajs/vue3';
import { vElementHover } from '@vueuse/components';
import { computed } from 'vue';

const { map_route_solarsystems, map, solarsystems, map_characters, map_user_settings, map_solarsystem } = defineProps<{
    map: TMap;
    solarsystems: TSolarsystem[];
    map_route_solarsystems?: TMapRouteSolarsystem[];
    selected_map_solarsystem?: TMapSolarSystem;
    map_user_settings: TMapUserSetting;
    map_characters?: TCharacter[];
    map_solarsystem: TMapSolarSystem | null;
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

const { setHoveredMapSolarsystem } = useMapSolarsystems();

const { setPath } = usePath();

function updateMapUserSettings(settings: Partial<TMapUserSetting>) {
    router.put(MapUserSettings.update(map_user_settings.id).url, settings, {
        preserveScroll: true,
        only: ['map_route_solarsystems', 'map_user_settings'],
        preserveState: true,
    });
}

function handleToggleEol(value: boolean | 'indeterminate') {
    updateMapUserSettings({
        route_allow_eol: value === true,
    });
}

function handleToggleMass(value: string) {
    updateMapUserSettings({
        route_allow_mass_status: value as TMassStatus,
    });
}

function handleToggleEveScout(value: boolean | 'indeterminate') {
    updateMapUserSettings({
        route_use_evescout: value === true,
    });
}

function handleHover(hovered: boolean, route: TSolarsystem[] | null) {
    if (hovered) {
        setPath(route);
    } else {
        setPath(null);
    }
}

function handleSolarsystemHover(hovered: boolean) {
    setHoveredMapSolarsystem(map_solarsystem?.id ?? 0, hovered);
}
</script>

<template>
    <Card class="bg-neutral-50 pb-0 dark:bg-transparent">
        <CardHeader>
            <CardTitle class="text-base">Autopilot</CardTitle>
            <CardDescription>
                See how far you have to travel from
                <b v-element-hover="handleSolarsystemHover">
                    <span v-if="map_solarsystem?.alias">
                        <span class="text-primary">{{ map_solarsystem.alias }}</span> {{ map_solarsystem.name }}
                    </span>
                    <span v-else class="text-primary">{{ map_solarsystem?.name }}</span>
                </b>
            </CardDescription>
            <CardAction class="flex gap-2">
                <Popover>
                    <PopoverTrigger>
                        <Button variant="secondary" size="icon">
                            <SettingsIcon />
                        </Button>
                    </PopoverTrigger>
                    <PopoverContent class="w-64 p-3">
                        <div class="grid auto-cols-[auto_1fr_auto] gap-x-1 gap-y-1">
                            <h4 class="col-span-3 text-xs text-muted-foreground">Wormholes</h4>
                            <div class="col-span-3 grid grid-cols-subgrid">
                                <Checkbox :model-value="map_user_settings.route_allow_eol" @update:model-value="handleToggleEol" id="eol-checkbox" />
                                <label for="eol-checkbox" class="cursor-pointer text-xs font-medium"> Allow EOL </label>
                                <span class="text-xs text-muted-foreground">&lt; 4 hours</span>
                            </div>

                            <RadioGroup
                                :model-value="map_user_settings.route_allow_mass_status"
                                @update:model-value="handleToggleMass"
                                class="col-span-3 grid grid-cols-subgrid gap-1"
                            >
                                <RadioGroupItem value="critical" id="mass-critical" />
                                <Label for="mass-critical" class="cursor-pointer text-xs font-medium">Critical Mass</Label>
                                <span class="text-xs text-muted-foreground">&lt; 10%</span>
                                <RadioGroupItem value="reduced" id="mass-reduced" />
                                <Label for="mass-reduced" class="cursor-pointer text-xs font-medium">Reduced Mass</Label>
                                <span class="text-xs text-muted-foreground">&lt; 50%</span>
                                <RadioGroupItem value="fresh" id="mass-fresh" />
                                <Label for="mass-fresh" class="cursor-pointer text-xs font-medium">High Mass</Label>
                                <span class="text-xs text-muted-foreground">&gt; 50%</span>
                            </RadioGroup>
                            <h4 class="col-span-3 mt-2 text-xs text-muted-foreground">Information Sources</h4>
                            <div class="col-span-3 grid grid-cols-subgrid">
                                <Checkbox
                                    :model-value="map_user_settings.route_use_evescout"
                                    @update:model-value="handleToggleEveScout"
                                    id="evescout-checkbox"
                                />
                                <label for="evescout-checkbox" class="cursor-pointer text-xs font-medium"> Use EVE Scout </label>
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
                    <div class="col-span-full grid grid-cols-subgrid border-b px-3 py-2 text-sm font-medium text-muted-foreground">
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

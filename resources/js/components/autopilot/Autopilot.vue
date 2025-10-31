<script setup lang="ts">
import ActiveCharacterLocation from '@/components/autopilot/ActiveCharacterLocation.vue';
import AutopilotSettings from '@/components/autopilot/AutopilotSettings.vue';
import ClosestSystemsDialog from '@/components/autopilot/ClosestSystemsDialog.vue';
import MapRouteSolarsystemAdd from '@/components/autopilot/MapRouteSolarsystemAdd.vue';
import MapRouteSolarsystems from '@/components/autopilot/MapRouteSolarsystems.vue';
import ShortestPathDialog from '@/components/autopilot/shortest-path/ShortestPathDialog.vue';
import RouteIcon from '@/components/icons/RouteIcon.vue';
import SearchIcon from '@/components/icons/SearchIcon.vue';
import { Button } from '@/components/ui/button';
import { CardAction, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useMapSolarsystems } from '@/composables/map';
import useHasWritePermission from '@/composables/useHasWritePermission';
import useUser from '@/composables/useUser';
import { TClosestSystems, TShortestPath } from '@/pages/maps';
import { TCharacter, TMap, TMapRouteSolarsystem, TMapSolarSystem, TSolarsystem } from '@/types/models';
import { vElementHover } from '@vueuse/components';
import { computed } from 'vue';

const { map_route_solarsystems, map, solarsystems, map_characters, selected_map_solarsystem, shortest_path, ignored_systems, closest_systems } =
    defineProps<{
        map: TMap;
        solarsystems: TSolarsystem[];
        map_route_solarsystems?: TMapRouteSolarsystem[];
        selected_map_solarsystem?: TMapSolarSystem | null;
        map_characters: TCharacter[] | null;
        shortest_path?: TShortestPath | null;
        ignored_systems: number[];
        closest_systems?: TClosestSystems | null;
    }>();

const user = useUser();
const activeCharacter = computed(() => {
    if (!map_characters || !user.value?.active_character) return null;
    return map_characters.find((character) => character.id === user.value.active_character.id);
});
const characterStatus = computed(() => activeCharacter.value?.status);

const can_write = useHasWritePermission();

const { setHoveredMapSolarsystem } = useMapSolarsystems();

function handleSolarsystemHover(hovered: boolean) {
    setHoveredMapSolarsystem(selected_map_solarsystem?.id ?? 0, hovered);
}
</script>

<template>
    <MapPanel>
        <CardHeader>
            <CardTitle class="text-base">Autopilot</CardTitle>
            <CardDescription>
                <template v-if="selected_map_solarsystem">
                    See how far you have to travel from
                    <b v-element-hover="handleSolarsystemHover">
                        <span v-if="selected_map_solarsystem.alias">
                            <span class="text-primary">{{ selected_map_solarsystem.alias }}</span> {{ selected_map_solarsystem.name }}
                        </span>
                        <span v-else class="text-primary">{{ selected_map_solarsystem.name }}</span>
                    </b>
                </template>
                <template v-else> Select a solarsystem to see routes, or use the shortest path finder</template>
            </CardDescription>
            <CardAction class="flex gap-2">
                <Tooltip>
                    <ShortestPathDialog
                        :map="map"
                        :solarsystems="solarsystems"
                        :shortest_path="shortest_path"
                        :ignored_systems="ignored_systems"
                        :selected_map_solarsystem="selected_map_solarsystem"
                    >
                        <TooltipTrigger as-child>
                            <Button variant="secondary" size="icon">
                                <SearchIcon />
                            </Button>
                        </TooltipTrigger>
                    </ShortestPathDialog>

                    <TooltipContent>
                        <p>Find shortest path between systems</p>
                    </TooltipContent>
                </Tooltip>

                <Tooltip>
                    <ClosestSystemsDialog
                        :map="map"
                        :solarsystems="solarsystems"
                        :selected_map_solarsystem="selected_map_solarsystem"
                        :closest_systems="closest_systems"
                    >
                        <TooltipTrigger as-child>
                            <Button variant="secondary" size="icon">
                                <RouteIcon />
                            </Button>
                        </TooltipTrigger>
                    </ClosestSystemsDialog>

                    <TooltipContent>
                        <p>Find closest systems by condition</p>
                    </TooltipContent>
                </Tooltip>
                <AutopilotSettings />
                <MapRouteSolarsystemAdd :map :solarsystems :map_route_solarsystems v-if="can_write" />
            </CardAction>
        </CardHeader>

        <MapPanelContent inner-class="border-0 bg-transparent">
            <template v-if="selected_map_solarsystem">
                <ActiveCharacterLocation
                    :active-character="activeCharacter"
                    :character-status="characterStatus"
                    v-if="activeCharacter && characterStatus"
                />
                <MapRouteSolarsystems v-if="map_route_solarsystems" :map_route_solarsystems="map_route_solarsystems" />
            </template>
            <template v-else>
                <div class="flex flex-col items-center justify-center gap-4 py-8 text-center text-muted-foreground">
                    <RouteIcon class="text-2xl" />
                    <div>
                        <p class="mb-1 font-medium">No system selected</p>
                        <p class="text-xs">Select a system on the map to see routes, or use the shortest path finder above</p>
                    </div>
                </div>
            </template>
        </MapPanelContent>
    </MapPanel>
</template>

<style scoped></style>

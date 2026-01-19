<script setup lang="ts">
import ActiveCharacterLocation from '@/components/autopilot/ActiveCharacterLocation.vue';
import AutopilotSettings from '@/components/autopilot/AutopilotSettings.vue';
import MapRouteSolarsystemAdd from '@/components/autopilot/MapRouteSolarsystemAdd.vue';
import NavigationDestinations from '@/components/autopilot/NavigationDestinations.vue';
import NavigationFindSystems from '@/components/autopilot/NavigationFindSystems.vue';
import NavigationRoute from '@/components/autopilot/NavigationRoute.vue';
import MapIcon from '@/components/icons/MapIcon.vue';
import NetworkIcon from '@/components/icons/NetworkIcon.vue';
import RouteIcon from '@/components/icons/RouteIcon.vue';
import { CardAction, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { useMapSolarsystems } from '@/composables/map';
import useHasWritePermission from '@/composables/useHasWritePermission';
import { useStaticData } from '@/composables/useStaticData';
import { useStaticSolarsystems } from '@/composables/useStaticSolarsystems';
import useUser from '@/composables/useUser';
import type { TMap, TResolvedMapNavigation, TResolvedSelectedMapSolarsystem } from '@/pages/maps';
import { TCharacter } from '@/types/models';
import { vElementHover } from '@vueuse/components';
import { computed, ref } from 'vue';

const { map_navigation, map, map_characters, selected_map_solarsystem, ignored_systems } = defineProps<{
    map: TMap;
    map_navigation: TResolvedMapNavigation;
    selected_map_solarsystem?: TResolvedSelectedMapSolarsystem | null;
    map_characters: TCharacter[] | null;
    ignored_systems: number[];
}>();

const user = useUser();
const activeCharacter = computed(() => {
    if (!map_characters || !user.value?.active_character) return null;
    return map_characters.find((character) => character.id === user.value.active_character.id);
});
const characterStatus = computed(() => activeCharacter.value?.status);

const can_write = useHasWritePermission();

const { setHoveredMapSolarsystem } = useMapSolarsystems();
const { staticData, loadStaticData } = useStaticData();
const { resolveSolarsystem } = useStaticSolarsystems();

void loadStaticData();

const solarsystems = computed(() => staticData.value?.solarsystems ?? []);
const selectedSolarsystem = computed(() => {
    if (!selected_map_solarsystem) {
        return null;
    }

    return resolveSolarsystem(selected_map_solarsystem.solarsystem_id);
});

function handleSolarsystemHover(hovered: boolean) {
    setHoveredMapSolarsystem(selected_map_solarsystem?.id ?? 0, hovered);
}

const activeTab = ref('destinations');
</script>

<template>
    <MapPanel>
        <CardHeader>
            <CardTitle class="text-base">Navigation</CardTitle>
            <CardDescription>
                <template v-if="selected_map_solarsystem">
                    Navigate from
                    <b v-element-hover="handleSolarsystemHover">
                        <span v-if="selected_map_solarsystem.alias">
                            <span class="text-primary">{{ selected_map_solarsystem.alias }}</span> {{ selectedSolarsystem?.name ?? 'Unknown' }}
                        </span>
                        <span v-else class="text-primary">{{ selectedSolarsystem?.name ?? 'Unknown' }}</span>
                    </b>
                </template>
                <template v-else> Select a solarsystem to see routes, or use the tools below</template>
            </CardDescription>
            <CardAction class="flex gap-2">
                <AutopilotSettings />
                <MapRouteSolarsystemAdd :map :map_route_solarsystems="map_navigation.destinations" v-if="can_write" />
            </CardAction>
        </CardHeader>

        <MapPanelContent inner-class="border-0 bg-transparent">
            <Tabs v-model="activeTab" default-value="destinations" class="w-full">
                <TabsList class="mb-4 grid w-full grid-cols-3">
                    <TabsTrigger value="destinations"> <RouteIcon class="mr-1.5 inline size-3.5" />Watchlist </TabsTrigger>
                    <TabsTrigger value="route"> <NetworkIcon class="mr-1.5 inline size-3.5" />Route </TabsTrigger>
                    <TabsTrigger value="find-systems"> <MapIcon class="mr-1.5 inline size-3.5" />Find Closest </TabsTrigger>
                </TabsList>

                <TabsContent value="destinations">
                    <template v-if="selected_map_solarsystem">
                        <ActiveCharacterLocation
                            :active-character="activeCharacter"
                            :character-status="characterStatus"
                            v-if="activeCharacter && characterStatus"
                        />
                        <NavigationDestinations :destinations="map_navigation.destinations" :ignored_systems="ignored_systems" />
                    </template>
                    <template v-else>
                        <div class="flex flex-col items-center justify-center gap-4 py-8 text-center text-muted-foreground">
                            <RouteIcon class="text-2xl" />
                            <div>
                                <p class="mb-1 font-medium">No system selected</p>
                                <p class="text-xs">Select a system on the map to see routes</p>
                            </div>
                        </div>
                    </template>
                </TabsContent>

                <TabsContent value="route">
                    <NavigationRoute
                        :map="map"
                        :solarsystems="solarsystems"
                        :selected_map_solarsystem="selected_map_solarsystem"
                        :ignored_systems="ignored_systems"
                        :active_character="activeCharacter"
                        :character_status="characterStatus"
                        :destinations="map_navigation.destinations"
                    />
                </TabsContent>

                <TabsContent value="find-systems">
                    <NavigationFindSystems
                        :map="map"
                        :solarsystems="solarsystems"
                        :selected_map_solarsystem="selected_map_solarsystem"
                        :active_character="activeCharacter"
                        :character_status="characterStatus"
                        :destinations="map_navigation.destinations"
                        :ignored_systems="ignored_systems"
                    />
                </TabsContent>
            </Tabs>
        </MapPanelContent>
    </MapPanel>
</template>

<style scoped></style>

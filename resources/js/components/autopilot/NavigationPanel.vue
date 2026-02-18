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
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
import MapPanelHeader from '@/components/ui/map-panel/MapPanelHeader.vue';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import useHasWritePermission from '@/composables/useHasWritePermission';
import { useMapUserSettings } from '@/composables/useMapUserSettings';
import { useStaticData } from '@/composables/useStaticData';
import useUser from '@/composables/useUser';
import type { TMap, TResolvedMapNavigation, TResolvedSelectedMapSolarsystem } from '@/pages/maps';
import { TCharacter } from '@/types/models';
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

const mapUserSettings = useMapUserSettings();
const routePreferenceLabel = computed(() => {
    const labels: Record<string, string> = {
        shorter: 'Shortest',
        safer: 'Safer',
        less_secure: 'Less Secure',
    };
    return labels[mapUserSettings.value.route_preference] ?? 'Shortest';
});

const { staticData, loadStaticData } = useStaticData();

void loadStaticData();

const solarsystems = computed(() => staticData.value?.solarsystems ?? []);

const activeTab = ref('destinations');
</script>

<template>
    <MapPanel>
        <MapPanelHeader card-id="autopilot">
            Navigation
            <span class="ml-2 text-muted-foreground/60">{{ routePreferenceLabel }}</span>
            <template #actions>
                <AutopilotSettings />
                <MapRouteSolarsystemAdd :map :map_route_solarsystems="map_navigation.destinations" v-if="can_write" />
            </template>
        </MapPanelHeader>

        <MapPanelContent>
            <Tabs v-model="activeTab" default-value="destinations" class="flex h-full flex-col">
                <TabsList class="grid h-8 w-full shrink-0 grid-cols-3 rounded-none border-b border-border/50 bg-muted/20 p-0">
                    <TabsTrigger
                        value="destinations"
                        class="flex h-8 items-center justify-center gap-1 rounded-none border-r border-border/30 font-mono text-[10px] tracking-wider uppercase data-[state=active]:bg-muted/30 data-[state=active]:text-foreground data-[state=active]:shadow-none"
                    >
                        <RouteIcon class="size-3" />
                        <span>Watch</span>
                    </TabsTrigger>
                    <TabsTrigger
                        value="route"
                        class="flex h-8 items-center justify-center gap-1 rounded-none border-r border-border/30 font-mono text-[10px] tracking-wider uppercase data-[state=active]:bg-muted/30 data-[state=active]:text-foreground data-[state=active]:shadow-none"
                    >
                        <NetworkIcon class="size-3" />
                        <span>Route</span>
                    </TabsTrigger>
                    <TabsTrigger
                        value="find-systems"
                        class="flex h-8 items-center justify-center gap-1 rounded-none font-mono text-[10px] tracking-wider uppercase data-[state=active]:bg-muted/30 data-[state=active]:text-foreground data-[state=active]:shadow-none"
                    >
                        <MapIcon class="size-3" />
                        <span>Find</span>
                    </TabsTrigger>
                </TabsList>

                <TabsContent value="destinations" class="mt-0 flex-1 overflow-y-auto">
                    <template v-if="selected_map_solarsystem">
                        <ActiveCharacterLocation
                            :active-character="activeCharacter"
                            :character-status="characterStatus"
                            v-if="activeCharacter && characterStatus"
                        />
                        <NavigationDestinations :destinations="map_navigation.destinations" :ignored_systems="ignored_systems" />
                    </template>
                    <div v-else class="flex h-full flex-col items-center justify-center gap-2 p-4">
                        <p class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">Select a system</p>
                    </div>
                </TabsContent>

                <TabsContent value="route" class="mt-0 flex-1 overflow-y-auto">
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

                <TabsContent value="find-systems" class="mt-0 flex-1 overflow-y-auto">
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

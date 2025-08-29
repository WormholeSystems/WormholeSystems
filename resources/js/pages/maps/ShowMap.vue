<script setup lang="ts">
import MapAccessController from '@/actions/App/Http/Controllers/MapAccessController';
import MapPreferencesController from '@/actions/App/Http/Controllers/MapPreferencesController';
import MapSettingsController from '@/actions/App/Http/Controllers/MapSettingsController';
import Audits from '@/components/audits/Audits.vue';
import MapCharacters from '@/components/characters/MapCharacters.vue';
import QuestionIcon from '@/components/icons/QuestionIcon.vue';
import MapKillmails from '@/components/killmails/MapKillmails.vue';
import MapComponent from '@/components/map/MapComponent.vue';
import MapSearch from '@/components/map/MapSearch.vue';
import MapUserSetting from '@/components/map/MapUserSetting.vue';
import Tracker from '@/components/map/Tracker.vue';
import Autopilot from '@/components/routes/Autopilot.vue';
import SeoHead from '@/components/SeoHead.vue';
import ShipHistory from '@/components/ShipHistory.vue';
import Signatures from '@/components/signatures/Signatures.vue';
import SolarsystemDetails from '@/components/solarsystem/SolarsystemDetails.vue';
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useActiveMapCharacter } from '@/composables/useActiveMapCharacter';
import useHasWritePermission from '@/composables/useHasWritePermission';
import useIsMapOwner from '@/composables/useIsMapOwner';
import { useOnClient } from '@/composables/useOnClient';
import AppLayout from '@/layouts/AppLayout.vue';
import { TShowMapProps } from '@/pages/maps/index';
import { Link, router } from '@inertiajs/vue3';
import { echo } from '@laravel/echo-vue';
import { Settings } from 'lucide-vue-next';
import { computed } from 'vue';

const {
    map,
    selected_map_solarsystem,
    map_killmails,
    map_route_solarsystems,
    map_user_settings,
    shortest_path,
    ignored_systems,
    map_characters,
    closest_systems,
} = defineProps<TShowMapProps>();

const character = useActiveMapCharacter();
const hasWriteAccess = useHasWritePermission();
const isOwner = useIsMapOwner();

const settingsUrl = computed(() => {
    if (isOwner.value) {
        return MapSettingsController.show(map.slug).url;
    }

    if (hasWriteAccess.value) {
        return MapAccessController.show(map.slug).url;
    }

    return MapPreferencesController.show(map.slug).url;
});

useOnClient(() =>
    router.on('before', (event) => {
        const id = echo().socketId();
        if (!id) return;
        event.detail.visit.headers['X-Socket-ID'] = id;
    }),
);
</script>

<template>
    <AppLayout>
        <SeoHead
            :title="map.name"
            :description="`Explore the ${map.name} wormhole mapping network. Navigate dangerous wormhole space with real-time intel, signature tracking, and collaborative mapping tools.`"
            keywords="wormhole map, eve online navigation, wormhole signatures, space exploration, real-time intel"
        />

        <div class="space-y-4 p-4">
            <div class="">
                <div class="relative">
                    <MapComponent :map :config />
                    <MapSearch :map :search :solarsystems v-if="hasWriteAccess" />
                    <div class="absolute top-4 right-4 flex gap-2">
                        <Tracker :map_user_settings="map_user_settings" :character :map :key="character?.id" v-if="hasWriteAccess" />
                        <Tooltip>
                            <TooltipTrigger>
                                <Button variant="outline" size="icon" as-child>
                                    <Link :href="settingsUrl">
                                        <Settings class="h-4 w-4" />
                                    </Link>
                                </Button>
                            </TooltipTrigger>
                            <TooltipContent side="bottom">
                                <p class="text-sm">Settings</p>
                            </TooltipContent>
                        </Tooltip>
                    </div>
                </div>
            </div>
            <div class="grid grid-cols-10 content-start items-start gap-4">
                <div class="col-span-10 grid gap-4 lg:col-span-5 2xl:col-span-4">
                    <SolarsystemDetails v-if="selected_map_solarsystem" :map_solarsystem="selected_map_solarsystem" :map :map_route_solarsystems />
                    <div class="flex flex-col items-center justify-center gap-8 rounded-lg border border-dashed p-16 text-neutral-700" v-else>
                        <QuestionIcon class="text-4xl" />
                        <p class="text-center">Select a solarsystem to see more details</p>
                    </div>
                    <Signatures :map :map_solarsystem="selected_map_solarsystem" v-if="selected_map_solarsystem" />
                    <Audits :audits="selected_map_solarsystem?.audits" v-if="selected_map_solarsystem && selected_map_solarsystem.audits" />
                    <ShipHistory />
                </div>
                <div class="col-span-10 grid gap-4 lg:col-span-5 2xl:col-span-3">
                    <MapCharacters :map_characters />
                    <MapKillmails :map_killmails="map_killmails" :map_id="map.id" :map_user_settings="map_user_settings" />
                </div>
                <div class="col-span-10 2xl:col-span-3">
                    <Autopilot
                        :map_route_solarsystems
                        :map
                        :solarsystems
                        :selected_map_solarsystem
                        :map_characters
                        :map_user_settings
                        :shortest_path
                        :ignored_systems
                        :closest_systems
                    />
                </div>
            </div>
        </div>
        <MapUserSetting :map_user_setting="map.map_user_setting" v-if="map.map_user_setting" />
    </AppLayout>
</template>
<style></style>

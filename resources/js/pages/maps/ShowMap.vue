<script setup lang="ts">
import MapController from '@/actions/App/Http/Controllers/MapController';
import MapCharacters from '@/components/characters/MapCharacters.vue';
import LockIcon from '@/components/icons/LockIcon.vue';
import QuestionIcon from '@/components/icons/QuestionIcon.vue';
import TrashIcon from '@/components/icons/TrashIcon.vue';
import MapKillmails from '@/components/killmails/MapKillmails.vue';
import MapComponent from '@/components/map/MapComponent.vue';
import MapSearch from '@/components/map/MapSearch.vue';
import MapUserSetting from '@/components/map/MapUserSetting.vue';
import Tracker from '@/components/map/Tracker.vue';
import Autopilot from '@/components/routes/Autopilot.vue';
import ShipHistory from '@/components/ShipHistory.vue';
import Signatures from '@/components/signatures/Signatures.vue';
import SolarsystemDetails from '@/components/solarsystem/SolarsystemDetails.vue';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useOnClient } from '@/composables/useOnClient';
import useUser from '@/composables/useUser';
import AppLayout from '@/layouts/AppLayout.vue';
import { TShowMapProps } from '@/pages/maps/index';
import MapAccess from '@/routes/map-access';
import { Head, Link, router } from '@inertiajs/vue3';
import { echo } from '@laravel/echo-vue';
import { ref } from 'vue';

const { map, selected_map_solarsystem, map_killmails, map_route_solarsystems, has_write_access, allow_crit, allow_eol, allow_eve_scout } =
    defineProps<TShowMapProps>();

const user = useUser();

const confirmation = ref('');

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
        <Head :title="map.name" />

        <div class="space-y-4 p-4">
            <div class="">
                <div class="relative">
                    <MapComponent :map :config />
                    <MapSearch :map :search :solarsystems v-if="has_write_access" />
                    <div class="absolute top-4 right-4 flex gap-2" v-if="has_write_access">
                        <Tracker :map_characters v-if="map_characters" :map :key="user.active_character?.id" />
                        <Tooltip>
                            <TooltipTrigger>
                                <Button :variant="'outline'" as-child>
                                    <Link :href="MapAccess.show(map.slug)">
                                        <LockIcon />
                                    </Link>
                                </Button>
                            </TooltipTrigger>
                            <TooltipContent side="bottom">
                                <p class="text-sm">Map Access</p>
                                <p class="text-xs text-muted-foreground">Manage who can view and edit this map</p>
                            </TooltipContent>
                        </Tooltip>
                        <Dialog v-if="user.characters.find((c) => c.id === map.owner.id)">
                            <DialogTrigger>
                                <Button size="icon" variant="destructive">
                                    <TrashIcon />
                                </Button>
                            </DialogTrigger>
                            <DialogContent>
                                <DialogHeader>
                                    <DialogTitle> Are you sure you want to delete this map?</DialogTitle>
                                    <DialogDescription>
                                        This action cannot be undone. All data associated with this map will be permanently deleted. If you want to
                                        proceed, please type the name of the map ({{ map.name }}) below to confirm.
                                    </DialogDescription>
                                </DialogHeader>
                                <DialogFooter>
                                    <Input v-model="confirmation" />
                                    <Button as-child variant="destructive" :disabled="confirmation !== map.name">
                                        <Link :href="MapController.destroy(map.slug)" method="delete" data-confirm="Are you sure?">
                                            Permanently Delete Map
                                        </Link>
                                    </Button>
                                </DialogFooter>
                            </DialogContent>
                        </Dialog>
                    </div>
                </div>
            </div>
            <div class="grid grid-cols-12 content-start items-start gap-4">
                <div class="col-span-12 grid gap-4 lg:col-span-6 xl:col-span-5">
                    <SolarsystemDetails v-if="selected_map_solarsystem" :map_solarsystem="selected_map_solarsystem" :map :map_route_solarsystems />
                    <div class="flex flex-col items-center justify-center gap-8 rounded-lg border border-dashed p-16 text-neutral-700" v-else>
                        <QuestionIcon class="text-4xl" />
                        <p class="text-center">Select a solarsystem to see more details</p>
                    </div>
                    <Signatures :map :map_solarsystem="selected_map_solarsystem" v-if="selected_map_solarsystem" />
                    <ShipHistory />
                </div>
                <div class="col-span-12 grid gap-4 lg:col-span-6 xl:col-span-4">
                    <MapCharacters :map_characters />
                    <MapKillmails :map_killmails="map_killmails" :map_id="map.id" />
                </div>
                <div class="col-span-12 xl:col-span-3">
                    <Autopilot
                        :map_route_solarsystems
                        v-if="selected_map_solarsystem"
                        :map
                        :solarsystems
                        :selected_map_solarsystem
                        :allow_crit
                        :allow_eol
                        :allow_eve_scout
                        :map_characters
                    />
                    <div class="flex flex-col items-center justify-center gap-8 rounded-lg border border-dashed p-16 text-neutral-700" v-else>
                        <QuestionIcon class="text-4xl" />
                        <p class="text-center">Select a solarsystem to see autopilot</p>
                    </div>
                </div>
            </div>
        </div>
        <MapUserSetting :map_user_setting="map.map_user_setting" v-if="map.map_user_setting" />
    </AppLayout>
</template>
<style></style>

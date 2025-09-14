<script setup lang="ts">
import PingController from '@/actions/App/Http/Controllers/PingController';
import TrackingIcon from '@/components/icons/TrackingIcon.vue';
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { createMapSolarsystem, createTracking, updateMapUserSettings, useMapSolarsystems } from '@/composables/map';
import { TCharacter, TMap, TMapUserSetting } from '@/types/models';
import { useFetch, useIntervalFn } from '@vueuse/core';
import { onMounted, watch } from 'vue';

const { character, map, map_user_settings } = defineProps<{
    character: TCharacter | undefined;
    map: TMap;
    map_user_settings: TMapUserSetting;
}>();

const { map_solarsystems } = useMapSolarsystems();

const ping_interval_seconds = 60 * 5 * 1000;

const { execute } = useFetch(PingController.show(map.slug).url, {
    immediate: false,
});
useIntervalFn(execute, ping_interval_seconds, {
    immediateCallback: true,
});

watch(
    () => character?.status?.solarsystem_id,
    (new_solarsystem_id, old_solarsystem_id) => {
        if (!map_user_settings.is_tracking) return;
        if (!new_solarsystem_id || !old_solarsystem_id) return;
        if (new_solarsystem_id === old_solarsystem_id) return;

        requestConnectSolarsystem(old_solarsystem_id, new_solarsystem_id);
    },
);

onMounted(() => {
    if (!map_user_settings.is_tracking) return;

    addCurrentSolarsystem();
});

function requestConnectSolarsystem(old_solarsystem_id: number | null, new_solarsystem_id: number) {
    const old_map_solarsystem = map_solarsystems.value.find((s) => s.solarsystem_id === old_solarsystem_id);
    if (!old_map_solarsystem) return;
    if (old_map_solarsystem.solarsystem_id === new_solarsystem_id) return;

    createTracking(old_map_solarsystem.id, new_solarsystem_id);
}

function handleToggleTracking() {
    if (!map_user_settings.tracking_allowed) return;

    updateMapUserSettings(map_user_settings, {
        is_tracking: !map_user_settings.is_tracking,
    });
}

function addCurrentSolarsystem() {
    const active_solarsystem = character?.status?.solarsystem;
    if (!active_solarsystem) return;

    const existing_solarsystem = map_solarsystems.value.find((s) => s.solarsystem_id === active_solarsystem.id);
    if (existing_solarsystem) return;

    createMapSolarsystem(active_solarsystem.id);
}
</script>

<template>
    <Tooltip>
        <TooltipTrigger>
            <Button
                @click="handleToggleTracking"
                :variant="map_user_settings.is_tracking && character && map_user_settings.tracking_allowed ? 'default' : 'secondary'"
                :disabled="!character || !map_user_settings.tracking_allowed"
                size="icon"
            >
                <TrackingIcon />
            </Button>
        </TooltipTrigger>
        <TooltipContent side="bottom">
            <template v-if="!map_user_settings.tracking_allowed">
                <p class="text-sm font-medium text-red-500">Tracking Unavailable</p>
                <p class="text-xs text-muted-foreground">Location Monitoring must be enabled to use tracking</p>
                <p class="text-xs text-muted-foreground">Click the eye icon to enable location monitoring</p>
            </template>
            <template v-else-if="!character">
                <p class="text-sm font-medium text-red-500">Character Status Unknown</p>
                <p class="max-w-sm text-xs text-muted-foreground">You must have an active character that is online and has scopes granted</p>
                <p class="max-w-sm text-xs text-muted-foreground">Note: It might take a minute for the character to show up</p>
            </template>
            <template v-else>
                <p class="text-sm">Tracking</p>
                <p class="text-xs text-muted-foreground">
                    {{ map_user_settings.is_tracking ? 'Enabled' : 'Disabled' }} - Automatically track solarsystem changes
                </p>
                <p class="text-xs text-muted-foreground">Current Solarsystem: {{ character?.status?.solarsystem?.name || 'Unknown' }}</p>
            </template>
        </TooltipContent>
    </Tooltip>
</template>

<style scoped></style>

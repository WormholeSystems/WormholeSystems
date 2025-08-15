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
            <Button @click="handleToggleTracking" :variant="map_user_settings.is_tracking ? 'default' : 'secondary'" size="icon">
                <TrackingIcon />
            </Button>
        </TooltipTrigger>
        <TooltipContent side="bottom">
            <p class="text-sm">Tracking</p>
            <p class="text-xs text-muted-foreground">
                {{ map_user_settings.is_tracking ? 'Enabled' : 'Disabled' }} - Automatically track solarsystem changes
            </p>
            <p class="text-xs text-muted-foreground">Current Solarsystem: {{ character?.status?.solarsystem?.name || 'Unknown' }}</p>
        </TooltipContent>
    </Tooltip>
</template>

<style scoped></style>

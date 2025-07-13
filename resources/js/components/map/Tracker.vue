<script setup lang="ts">
import TrackingIcon from '@/components/icons/TrackingIcon.vue';
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useMapSolarsystems } from '@/composables/map';
import useUser from '@/composables/useUser';
import { TCharacter, TMap } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { computed, ref, watch } from 'vue';

const { map_characters, map } = defineProps<{ map_characters: TCharacter[]; map: TMap }>();

const user = useUser();

const active_map_character = computed(() => {
    return map_characters.find((character) => character.id === user.value.active_character.id);
});

const { map_solarsystems } = useMapSolarsystems();

const enabled = ref(true);

watch(
    () => active_map_character.value?.status?.solarsystem_id,
    (new_solarsystem_id, old_solarsystem_id) => {
        if (!enabled.value) return;
        if (!new_solarsystem_id) return;
        if (!old_solarsystem_id) return requestAddSolarsystem(new_solarsystem_id);
        if (new_solarsystem_id === old_solarsystem_id) return;

        requestConnectSolarsystem(old_solarsystem_id, new_solarsystem_id);
    },
    {
        immediate: true,
    },
);

function requestConnectSolarsystem(old_solarsystem_id: number | null, new_solarsystem_id: number) {
    const old_map_solarsystem = map_solarsystems.value.find((s) => s.solarsystem_id === old_solarsystem_id);
    if (!old_map_solarsystem) return;
    if (old_map_solarsystem.solarsystem_id === new_solarsystem_id) return;

    router.post(
        route('tracking.store'),
        {
            from_map_solarsystem_id: old_map_solarsystem.id,
            to_solarsystem_id: new_solarsystem_id,
        },
        {
            preserveState: true,
            preserveScroll: true,
        },
    );
}

function requestAddSolarsystem(solarsystem_id: number) {
    if (map_solarsystems.value.some((s) => s.solarsystem_id === solarsystem_id)) return;
    router.post(
        route('map-solarsystems.store'),
        {
            map_id: map.id,
            solarsystem_id,
        },
        {
            preserveState: true,
            preserveScroll: true,
        },
    );
}
</script>

<template>
    <Tooltip>
        <TooltipTrigger>
            <Button @click="enabled = !enabled" :variant="enabled ? 'default' : 'secondary'" size="icon">
                <TrackingIcon />
            </Button>
        </TooltipTrigger>
        <TooltipContent side="bottom">
            <p class="text-sm">Tracking</p>
            <p class="text-xs text-muted-foreground">{{ enabled ? 'Enabled' : 'Disabled' }} - Automatically track solarsystem changes</p>
            <p class="text-xs text-muted-foreground">Current Solarsystem: {{ active_map_character?.status?.solarsystem?.name || 'Unknown' }}</p>
        </TooltipContent>
    </Tooltip>
</template>

<style scoped></style>

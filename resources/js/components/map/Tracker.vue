<script setup lang="ts">
import TrackingIcon from '@/components/icons/TrackingIcon.vue';
import TrackingSignatureDialog from '@/components/map/TrackingSignatureDialog.vue';
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useTracking } from '@/composables/map/composables/useTracking';
import { usePing } from '@/composables/usePing';
import { useStaticSolarsystem } from '@/composables/useStaticSolarsystems';
import { TMap } from '@/pages/maps';
import { TCharacter } from '@/types/models';
import { computed } from 'vue';

const { character, map } = defineProps<{
    character: TCharacter | undefined;
    map: TMap;
}>();

usePing(map);

const {
    toggle,
    is_tracking,
    is_tracking_allowed,
    can_track,
    signatures,
    show_signature_modal,
    handleSelectSignature,
    origin_map_solarsystem,
    target_solarsystem,
} = useTracking();

const currentSolarsystem = useStaticSolarsystem(() => character?.status?.solarsystem_id ?? null);
const targetSolarsystemName = computed(() => target_solarsystem.value?.name || currentSolarsystem.value?.name || null);
</script>

<template>
    <Tooltip>
        <TooltipTrigger>
            <Button @click="toggle" :variant="is_tracking ? 'default' : 'ghost'" :disabled="!can_track" size="icon" class="size-7">
                <TrackingIcon class="size-4" />
            </Button>
        </TooltipTrigger>
        <TooltipContent side="bottom">
            <template v-if="!is_tracking_allowed">
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
                <p class="text-xs text-muted-foreground">{{ is_tracking ? 'Enabled' : 'Disabled' }} - Automatically track solarsystem changes</p>
                <p class="text-xs text-muted-foreground">Current Solarsystem: {{ currentSolarsystem?.name || 'Unknown' }}</p>
            </template>
        </TooltipContent>
    </Tooltip>

    <TrackingSignatureDialog
        v-model:open="show_signature_modal"
        :origin-map-solarsystem="origin_map_solarsystem"
        :target-solarsystem-name="targetSolarsystemName"
        :signatures="signatures"
        @select-signature="handleSelectSignature"
    />
</template>

<style scoped></style>

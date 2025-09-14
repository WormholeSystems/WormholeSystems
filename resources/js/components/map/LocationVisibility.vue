<script setup lang="ts">
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { updateMapUserSettings } from '@/composables/map';
import { TMapUserSetting } from '@/types/models';
import { Eye, EyeOff } from 'lucide-vue-next';

const { map_user_settings } = defineProps<{
    map_user_settings: TMapUserSetting;
}>();

function handleToggleVisibility() {
    updateMapUserSettings(map_user_settings, {
        tracking_allowed: !map_user_settings.tracking_allowed,
    });
}
</script>

<template>
    <Tooltip>
        <TooltipTrigger>
            <Button @click="handleToggleVisibility" :variant="map_user_settings.tracking_allowed ? 'default' : 'secondary'" size="icon">
                <Eye v-if="map_user_settings.tracking_allowed" class="h-4 w-4" />
                <EyeOff v-else class="h-4 w-4" />
            </Button>
        </TooltipTrigger>
        <TooltipContent side="bottom">
            <p class="text-sm">Location Monitoring</p>
            <p class="max-w-xs text-xs text-muted-foreground">
                {{ map_user_settings.tracking_allowed ? ' Enabled' : 'Disabled' }} - Allow the map to use your location for tracking and to share with
                other map users.
            </p>
        </TooltipContent>
    </Tooltip>
</template>

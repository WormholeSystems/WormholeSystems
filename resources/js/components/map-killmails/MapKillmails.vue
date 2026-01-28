<script setup lang="ts">
import SettingsIcon from '@/components/icons/SettingsIcon.vue';
import Killmail from '@/components/map-killmails/Killmail.vue';
import { DropdownMenu, DropdownMenuContent, DropdownMenuRadioGroup, DropdownMenuRadioItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
import MapPanelHeader from '@/components/ui/map-panel/MapPanelHeader.vue';
import MapPanelHeaderActionButton from '@/components/ui/map-panel/MapPanelHeaderActionButton.vue';
import { useOnClient } from '@/composables/useOnClient';
import { getMapChannelName } from '@/const/channels';
import { KillmailReceivedEvent } from '@/const/events';
import MapUserSettings from '@/routes/map-user-settings';
import { TKillmail, TMapUserSetting } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { useEcho } from '@laravel/echo-vue';

const { map_killmails, map_id, map_user_settings } = defineProps<{
    map_killmails?: TKillmail[];
    map_id: number;
    map_user_settings: TMapUserSetting;
}>();

type KillmailReceivedEvent = {
    killmail: TKillmail;
};

function handleFilterChange(value: 'all' | 'jspace' | 'kspace' | string) {
    router.put(
        MapUserSettings.update(map_user_settings.id).url,
        {
            killmail_filter: value,
        },
        {
            only: ['map_killmails', 'map_user_settings'],
            preserveState: true,
            preserveScroll: true,
        },
    );
}

useOnClient(() =>
    useEcho<KillmailReceivedEvent>(getMapChannelName(map_id), KillmailReceivedEvent, () => {
        router.reload({
            only: ['map_killmails'],
        });
    }),
);
</script>

<template>
    <MapPanel>
        <MapPanelHeader>
            Killmails
            <span v-if="map_killmails?.length" class="ml-1 text-amber-400">{{ map_killmails.length }}</span>
            <template #actions>
                <DropdownMenu>
                    <DropdownMenuTrigger as-child>
                        <MapPanelHeaderActionButton size="icon">
                            <SettingsIcon />
                        </MapPanelHeaderActionButton>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                        <DropdownMenuRadioGroup :model-value="map_user_settings.killmail_filter" @update:model-value="handleFilterChange">
                            <DropdownMenuRadioItem value="all"> All killmails</DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="jspace"> J-Space killmails</DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="kspace"> K-Space killmails</DropdownMenuRadioItem>
                        </DropdownMenuRadioGroup>
                    </DropdownMenuContent>
                </DropdownMenu>
            </template>
        </MapPanelHeader>
        <MapPanelContent>
            <div class="@container">
                <template v-if="map_killmails?.length">
                    <TransitionGroup name="list">
                        <Killmail v-for="killmail in map_killmails" :key="killmail.id" :killmail="killmail" />
                    </TransitionGroup>
                </template>
                <div v-else class="flex h-full flex-col items-center justify-center gap-2 p-4">
                    <p class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">No killmails</p>
                </div>
            </div>
        </MapPanelContent>
    </MapPanel>
</template>

<style scoped>
.list-move, /* apply transition to moving elements */
.list-enter-active,
.list-leave-active {
    transition: all 0.5s ease;
}

.list-enter-from,
.list-leave-to {
    opacity: 0;
    transform: translateX(30px);
}

/* ensure leaving items are taken out of layout flow so that moving
   animations can be calculated correctly. */
.list-leave-active {
    position: absolute;
    opacity: 0;
    transition-duration: 0ms;
}
</style>

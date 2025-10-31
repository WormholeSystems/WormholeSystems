<script setup lang="ts">
import SettingsIcon from '@/components/icons/SettingsIcon.vue';
import Killmail from '@/components/map-killmails/Killmail.vue';
import { Button } from '@/components/ui/button';
import { CardAction, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { DropdownMenu, DropdownMenuContent, DropdownMenuRadioGroup, DropdownMenuRadioItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
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
        <CardHeader>
            <CardTitle>Map killmails</CardTitle>
            <CardDescription>See what is happening in your chain</CardDescription>
            <CardAction>
                <DropdownMenu>
                    <DropdownMenuTrigger as-child>
                        <Button variant="secondary" size="icon">
                            <SettingsIcon />
                        </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                        <DropdownMenuRadioGroup :model-value="map_user_settings.killmail_filter" @update:model-value="handleFilterChange">
                            <DropdownMenuRadioItem value="all"> All killmails</DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="jspace"> J-Space killmails</DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="kspace"> K-Space killmails</DropdownMenuRadioItem>
                        </DropdownMenuRadioGroup>
                    </DropdownMenuContent>
                </DropdownMenu>
            </CardAction>
        </CardHeader>
        <MapPanelContent inner-class="border-0 bg-transparent">
            <div class="relative max-h-100 overflow-x-hidden overflow-y-scroll mask-b-from-90% mask-alpha pr-1">
                <div class="@container rounded-lg border bg-white dark:bg-neutral-900/40">
                    <div class="grid grid-cols-[auto_auto_auto_auto_auto_auto] gap-x-2 text-xs">
                        <div class="col-span-full grid grid-cols-subgrid border-b bg-muted/50 px-2 py-1.5 text-xs font-medium text-muted-foreground">
                            <span>Victim</span>
                            <span>Attacker</span>
                            <span>Location</span>
                            <span class="hidden @lg:block">Sov</span>
                            <span>Time</span>
                            <span class="hidden text-right @lg:block">Value</span>
                        </div>
                        <TransitionGroup name="list">
                            <Killmail v-for="killmail in map_killmails" :key="killmail.id" :killmail="killmail" />
                        </TransitionGroup>
                        <div v-if="!map_killmails?.length" class="col-span-full p-2 text-center text-muted-foreground">No killmails found</div>
                    </div>
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

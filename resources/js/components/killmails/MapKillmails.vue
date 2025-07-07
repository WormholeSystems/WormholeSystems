<script setup lang="ts">
import Killmail from '@/components/killmails/Killmail.vue';
import KillmailPlaceholder from '@/components/killmails/KillmailPlaceholder.vue';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { getMapChannelName } from '@/const/channels';
import { KillmailReceivedEvent } from '@/const/events';
import { TKillmail } from '@/types/models';
import { Deferred, router } from '@inertiajs/vue3';
import { useEchoPublic } from '@laravel/echo-vue';

const { map_killmails, map_id } = defineProps<{
    map_killmails?: TKillmail[];
    map_id: number;
}>();

type KillmailReceivedEvent = {
    killmail: TKillmail;
};

useEchoPublic<KillmailReceivedEvent>(getMapChannelName(map_id), KillmailReceivedEvent, () => {
    router.reload({
        only: ['map_killmails'],
    });
});
</script>

<template>
    <Card>
        <CardHeader>
            <CardTitle>Map killmails</CardTitle>
            <CardDescription>Recents killmails that happened in one of the map solarsystems</CardDescription>
        </CardHeader>
        <CardContent>
            <div
                class="relative grid h-100 grid-cols-[auto_1fr_auto_auto_auto] content-start overflow-x-hidden overflow-y-scroll mask-b-from-90% mask-alpha pr-4 pb-8"
            >
                <Deferred data="map_killmails">
                    <TransitionGroup name="list">
                        <Killmail v-for="killmail in map_killmails" :key="killmail.id" :killmail="killmail" />
                    </TransitionGroup>
                    <template #fallback>
                        <KillmailPlaceholder v-for="i in 20" :key="i" />
                    </template>
                </Deferred>
            </div>
        </CardContent>
    </Card>
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

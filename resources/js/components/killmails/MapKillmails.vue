<script setup lang="ts">
import Killmail from '@/components/killmails/Killmail.vue';
import KillmailPlaceholder from '@/components/killmails/KillmailPlaceholder.vue';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { getMapChannelName } from '@/const/channels';
import { KillmailReceivedEvent } from '@/const/events';
import { TKillmail } from '@/types/models';
import { Deferred, router } from '@inertiajs/vue3';
import { useEchoPublic } from '@laravel/echo-vue';
import { useLocalStorage } from '@vueuse/core';
import { computed } from 'vue';

const { map_killmails, map_id } = defineProps<{
    map_killmails?: TKillmail[];
    map_id: number;
}>();

const filter = useLocalStorage<'all' | 'jspace' | 'kspace'>('killmail_filter', 'all');

const filtered_killmails = computed(() => {
    if (filter.value === 'all') {
        return map_killmails;
    }
    return map_killmails?.filter((killmail) => {
        if (filter.value === 'jspace') {
            return !!killmail.solarsystem.class;
        }

        if (filter.value === 'kspace') {
            return !killmail.solarsystem.class;
        }

        return true;
    });
});

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
        <CardHeader class="flex justify-between">
            <div class="">
                <CardTitle>Map killmails</CardTitle>
                <CardDescription>Recents killmails that happened in one of the map solarsystems</CardDescription>
            </div>
            <Button
                variant="outline"
                size="sm"
                class="text-xs"
                @click="filter = filter === 'all' ? 'jspace' : filter === 'jspace' ? 'kspace' : 'all'"
            >
                {{ filter === 'all' ? 'Show J-Space' : filter === 'jspace' ? 'Show K-Space' : 'Show All' }}
            </Button>
        </CardHeader>
        <CardContent>
            <div
                class="relative grid h-100 grid-cols-[auto_1fr_auto_auto_auto] content-start overflow-x-hidden overflow-y-scroll mask-b-from-90% mask-alpha pr-4 pb-8"
            >
                <Deferred data="map_killmails">
                    <TransitionGroup name="list">
                        <Killmail v-for="killmail in filtered_killmails" :key="killmail.id" :killmail="killmail" />
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

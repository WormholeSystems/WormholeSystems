<script setup lang="ts">
import SettingsIcon from '@/components/icons/SettingsIcon.vue';
import Killmail from '@/components/killmails/Killmail.vue';
import KillmailPlaceholder from '@/components/killmails/KillmailPlaceholder.vue';
import { Button } from '@/components/ui/button';
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { DropdownMenu, DropdownMenuContent, DropdownMenuRadioGroup, DropdownMenuRadioItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { Table, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { getMapChannelName } from '@/const/channels';
import { KillmailReceivedEvent } from '@/const/events';
import { TKillmail } from '@/types/models';
import { Deferred, router } from '@inertiajs/vue3';
import { useEcho } from '@laravel/echo-vue';
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

useEcho<KillmailReceivedEvent>(getMapChannelName(map_id), KillmailReceivedEvent, () => {
    router.reload({
        only: ['map_killmails'],
    });
});
</script>

<template>
    <Card class="bg-neutral-50 pb-0 dark:bg-transparent">
        <CardHeader>
            <CardTitle>Map killmails</CardTitle>
            <CardDescription>Recents killmails that happened in one of the map solarsystems</CardDescription>
            <CardAction>
                <DropdownMenu>
                    <DropdownMenuTrigger as-child>
                        <Button variant="secondary" size="icon">
                            <SettingsIcon />
                        </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                        <DropdownMenuRadioGroup v-model="filter">
                            <DropdownMenuRadioItem value="all"> All killmails</DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="jspace"> J-Space killmails</DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="kspace"> K-Space killmails</DropdownMenuRadioItem>
                        </DropdownMenuRadioGroup>
                    </DropdownMenuContent>
                </DropdownMenu>
            </CardAction>
        </CardHeader>
        <CardContent class="px-1 pb-1">
            <div class="relative max-h-100 overflow-x-hidden overflow-y-scroll mask-b-from-90% mask-alpha pr-1">
                <div class="rounded-lg border bg-white dark:bg-neutral-900/40">
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead>Victim</TableHead>
                                <TableHead>Attacker</TableHead>
                                <TableHead>Details</TableHead>
                            </TableRow>
                        </TableHeader>
                        <Deferred data="map_killmails">
                            <TransitionGroup name="list">
                                <Killmail v-for="killmail in filtered_killmails" :key="killmail.id" :killmail="killmail" />
                            </TransitionGroup>
                            <TableRow v-if="!filtered_killmails?.length">
                                <TableCell colspan="3" class="text-center text-muted-foreground"> No killmails found </TableCell>
                            </TableRow>
                            <template #fallback>
                                <KillmailPlaceholder />
                            </template>
                        </Deferred>
                    </Table>
                </div>
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

<script setup lang="ts">
import KillmailView from '@/components/map-killmails/KillmailView.vue';
import type { TResolvedSolarsystem } from '@/pages/maps';
import { TKillmail } from '@/types/models';

export type TKillmailViewModel = {
    killmail: TKillmail;
    solarsystem: TResolvedSolarsystem;
    alias?: string | null;
};

const { items } = defineProps<{
    items: TKillmailViewModel[];
}>();
</script>

<template>
    <div class="@container">
        <template v-if="items.length">
            <div class="grid grid-cols-[1.25rem_1.25rem_1.25rem_auto_1.25rem_auto_auto_1.25rem_1.25rem_2rem_auto_auto] gap-x-2">
                <TransitionGroup name="list">
                    <KillmailView
                        v-for="item in items"
                        :key="item.killmail.id"
                        :killmail="item.killmail"
                        :solarsystem="item.solarsystem"
                        :alias="item.alias"
                    />
                </TransitionGroup>
            </div>
        </template>
        <div v-else class="flex h-full flex-col items-center justify-center gap-2 p-4">
            <p class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">No killmails</p>
        </div>
    </div>
</template>

<style scoped>
.list-move,
.list-enter-active,
.list-leave-active {
    transition: all 0.5s ease;
}

.list-enter-from,
.list-leave-to {
    opacity: 0;
    transform: translateX(30px);
}

.list-leave-active {
    position: absolute;
    opacity: 0;
    transition-duration: 0ms;
}
</style>

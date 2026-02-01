<script setup lang="ts">
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useNowUTC } from '@/composables/useNowUTC';
import { useServerStatus } from '@/composables/useServerStatus';
import { format } from 'date-fns';
import { computed } from 'vue';

const status = useServerStatus();

const now = useNowUTC();

const now_formatted = computed(() => format(now.value, 'MMM dd, HH:mm'));

const intl_compact = new Intl.NumberFormat('en-US', {
    notation: 'compact',
    compactDisplay: 'short',
});
</script>

<template>
    <div class="flex items-center gap-3 font-mono text-xs text-muted-foreground" v-if="status">
        <span
            :data-online="status.players > 0"
            :data-vip="status.vip"
            class="block size-1.5 rounded-full bg-red-500 data-[online=true]:bg-green-500 data-[vip=true]:bg-yellow-500"
        ></span>
        <span>{{ now_formatted }}</span>
        <Tooltip>
            <TooltipTrigger>
                <span class="text-muted-foreground/60">{{ intl_compact.format(status.players) }} </span>
            </TooltipTrigger>
            <TooltipContent>
                Players online: <span class="font-semibold">{{ status.players.toLocaleString() }}</span>
            </TooltipContent>
        </Tooltip>
    </div>
</template>

<style scoped></style>

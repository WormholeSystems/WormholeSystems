<script setup lang="ts">
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { TProcessedConnection } from '@/composables/map';
import { useNowUTC } from '@/composables/useNowUTC';
import { TSignature } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import { differenceInDays, differenceInHours, differenceInMinutes, format } from 'date-fns';
import { computed } from 'vue';

const { signature } = defineProps<{
    signature: TSignature;
}>();

const now = useNowUTC();

const updated_at = computed(() => signature.updated_at);
const created_at = computed(() => signature.created_at);

const updated_date = computed(() => new UTCDate(updated_at.value));
const created_date = computed(() => new UTCDate(created_at.value));

const updated_ago = computed(() => {
    const diff_in_days = differenceInDays(now.value, updated_date.value);
    if (diff_in_days > 0) {
        return `${diff_in_days}d`;
    }
    const diff_in_hours = differenceInHours(now.value, updated_date.value);
    if (diff_in_hours > 0) {
        return `${diff_in_hours}h`;
    }
    const diff_in_minutes = differenceInMinutes(now.value, updated_date.value);
    if (diff_in_minutes > 0) {
        return `${diff_in_minutes}m`;
    }

    return 'now';
});

const updated_date_formatted = computed(() => format(updated_date.value, 'MMM dd, HH:mm'));
const created_date_formatted = computed(() => format(created_date.value, 'MMM dd, HH:mm'));
</script>

<template>
    <Tooltip>
        <TooltipTrigger class="time font-mono text-xs whitespace-nowrap text-muted-foreground tabular-nums">
            <span>{{ updated_ago }}</span>
        </TooltipTrigger>
        <TooltipContent class="grid grid-cols-[auto_auto] gap-2">
            <span class="font-semibold">Last updated at</span>
            <p class="">{{ updated_date_formatted }}</p>
            <span class="font-semibold">Created at</span>
            <p class="">{{ created_date_formatted }}</p>
        </TooltipContent>
    </Tooltip>
</template>

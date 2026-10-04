<script setup lang="ts">
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { UTCDate } from '@date-fns/utc';
import { useNow } from '@vueuse/core';
import { differenceInDays, differenceInHours, differenceInMinutes, format } from 'date-fns';
import { Clock } from 'lucide-vue-next';
import { computed } from 'vue';

const { pasted_at } = defineProps<{
    pasted_at: string | null;
}>();

const now = useNow({ interval: 30_000 });

const pasted_date = computed(() => (pasted_at ? new UTCDate(pasted_at) : null));

const time_ago = computed(() => {
    if (!pasted_date.value) {
        return null;
    }

    const diff_in_days = differenceInDays(now.value, pasted_date.value);
    if (diff_in_days > 0) {
        return `${diff_in_days}d ago`;
    }
    const diff_in_hours = differenceInHours(now.value, pasted_date.value);
    if (diff_in_hours > 0) {
        return `${diff_in_hours}h ago`;
    }
    const diff_in_minutes = differenceInMinutes(now.value, pasted_date.value);
    if (diff_in_minutes > 0) {
        return `${diff_in_minutes}m ago`;
    }

    return 'just now';
});

const formatted_date = computed(() => (pasted_date.value ? format(pasted_date.value, 'MMM dd, HH:mm') : null));
</script>

<template>
    <Tooltip v-if="time_ago">
        <TooltipTrigger as-child>
            <span class="ml-1 inline-flex items-center gap-1 whitespace-nowrap text-muted-foreground/70 normal-case tabular-nums">
                <Clock class="size-3" />
                {{ time_ago }}
            </span>
        </TooltipTrigger>
        <TooltipContent>Signatures last pasted {{ formatted_date }}</TooltipContent>
    </Tooltip>
</template>

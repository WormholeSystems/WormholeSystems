<script setup lang="ts">
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useNowUTC } from '@/composables/useNowUTC';
import { TMapConnection } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import { differenceInDays, differenceInHours, differenceInMinutes, format, formatDistanceStrict, max, min } from 'date-fns';
import { computed } from 'vue';

const props = defineProps<{
    connection: TMapConnection;
}>();

const now = useNowUTC();

const massStatusDisplay = computed(() => props.connection.mass_status);

const createdDate = computed(() => {
    const dates = [props.connection.created_at];

    // Add signature created_at dates if they exist
    if (props.connection.signatures?.length) {
        props.connection.signatures.forEach((sig) => {
            if (sig.created_at) {
                dates.push(sig.created_at);
            }
        });
    }

    // Find the earliest date
    return min(dates.map((date) => new UTCDate(date)));
});

const updatedDate = computed(() => {
    const dates = [props.connection.updated_at];

    // Add signature updated_at dates if they exist
    if (props.connection.signatures?.length) {
        props.connection.signatures.forEach((sig) => {
            if (sig.updated_at) {
                dates.push(sig.updated_at);
            }
        });
    }

    // Find the latest date
    return max(dates.map((date) => new UTCDate(date)));
});

const createdAt = computed(() => {
    return format(createdDate.value, 'MMM dd, HH:mm');
});

const updatedAt = computed(() => {
    return format(updatedDate.value, 'MMM dd, HH:mm');
});

function getTimeAgo(date: Date): string {
    const diff_in_days = differenceInDays(now.value, date);
    if (diff_in_days > 0) {
        return `${diff_in_days}d ago`;
    }
    const diff_in_hours = differenceInHours(now.value, date);
    if (diff_in_hours > 0) {
        return `${diff_in_hours}h ago`;
    }
    const diff_in_minutes = differenceInMinutes(now.value, date);
    if (diff_in_minutes > 0) {
        return `${diff_in_minutes}m ago`;
    }
    return 'just now';
}

const createdAgo = computed(() => getTimeAgo(createdDate.value));
const updatedAgo = computed(() => getTimeAgo(updatedDate.value));

const eolDate = computed(() => {
    return props.connection.marked_as_eol_at ? new UTCDate(props.connection.marked_as_eol_at) : null;
});

const eolAt = computed(() => {
    return eolDate.value ? format(eolDate.value, 'MMM dd, HH:mm') : null;
});

const eolAgo = computed(() => {
    if (!eolDate.value) return null;
    return formatDistanceStrict(eolDate.value, now.value, {
        addSuffix: true,
    });
});
</script>

<template>
    <div class="space-y-1">
        <div class="border-b pb-1 text-xs font-medium text-foreground">Status</div>
        <div class="grid grid-cols-2 divide-y truncate text-xs text-muted-foreground *:py-1">
            <div class="col-span-full grid grid-cols-subgrid">
                <span>EOL</span>
                <span class="text-right" :class="{ 'text-purple-500': connection.marked_as_eol_at }">
                    <template v-if="connection.marked_as_eol_at && eolAt">
                        <Tooltip>
                            <TooltipTrigger as-child>
                                <span class="cursor-help">Yes</span>
                            </TooltipTrigger>
                            <TooltipContent> {{ eolAt }} ({{ eolAgo }}) </TooltipContent>
                        </Tooltip>
                    </template>
                    <span v-else>No</span>
                </span>
            </div>
            <div class="col-span-full grid grid-cols-subgrid">
                <span>Mass Status</span>
                <span
                    class="text-right capitalize"
                    :class="{
                        'text-green-500': connection.mass_status === 'fresh',
                        'text-yellow-500': connection.mass_status === 'reduced',
                        'text-red-500': connection.mass_status === 'critical',
                    }"
                >
                    {{ massStatusDisplay }}
                </span>
            </div>
            <div class="col-span-full grid grid-cols-subgrid">
                <span>Created</span>
                <Tooltip>
                    <TooltipTrigger as-child>
                        <span class="cursor-help text-right">{{ createdAt }}</span>
                    </TooltipTrigger>
                    <TooltipContent>{{ createdAgo }}</TooltipContent>
                </Tooltip>
            </div>
            <div class="col-span-full grid grid-cols-subgrid">
                <span>Updated</span>
                <Tooltip>
                    <TooltipTrigger as-child>
                        <span class="cursor-help text-right">{{ updatedAt }}</span>
                    </TooltipTrigger>
                    <TooltipContent>{{ updatedAgo }}</TooltipContent>
                </Tooltip>
            </div>
        </div>
    </div>
</template>

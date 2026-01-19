<script setup lang="ts">
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useNowUTC } from '@/composables/useNowUTC';
import { TMapConnection } from '@/pages/maps';
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

const lifetimeDate = computed(() => {
    return props.connection.lifetime_status_updated_at ? new UTCDate(props.connection.lifetime_status_updated_at) : null;
});

const lifetimeAt = computed(() => {
    return lifetimeDate.value ? format(lifetimeDate.value, 'MMM dd, HH:mm') : null;
});

const lifetimeAgo = computed(() => {
    if (!lifetimeDate.value) return null;
    return formatDistanceStrict(lifetimeDate.value, now.value, {
        addSuffix: true,
    });
});

const lifetimeDisplay = computed(() => {
    switch (props.connection.lifetime_status) {
        case 'healthy':
            return 'Healthy';
        case 'eol':
            return 'End of Life (<4h)';
        case 'critical':
            return 'Critical (<1h)';
        default:
            return 'Unknown';
    }
});
</script>

<template>
    <div class="space-y-1">
        <div class="border-b pb-1 text-xs font-medium text-foreground">Status</div>
        <div class="grid grid-cols-2 divide-y truncate text-xs text-muted-foreground *:py-1">
            <div class="col-span-full grid grid-cols-subgrid">
                <span>Lifetime</span>
                <span
                    class="data-lifetime-status=critical:text-red-500 data-lifetime-status=eol:text-purple-500 text-right"
                    :data-lifetime-status="connection.lifetime_status"
                >
                    <template v-if="lifetimeAt && connection.lifetime_status !== 'healthy'">
                        <Tooltip>
                            <TooltipTrigger as-child>
                                <span class="cursor-help">{{ lifetimeDisplay }}</span>
                            </TooltipTrigger>
                            <TooltipContent> {{ lifetimeAt }} ({{ lifetimeAgo }}) </TooltipContent>
                        </Tooltip>
                    </template>
                    <span v-else>{{ lifetimeDisplay }}</span>
                </span>
            </div>
            <div class="col-span-full grid grid-cols-subgrid">
                <span>Mass Status</span>
                <span
                    :data-mass-status="connection.mass_status"
                    class="data-mass-status=critical:text-red-500 data-mass-status=fresh:text-green-500 data-mass-status=reduced:text-yellow-500 text-right capitalize"
                >
                    {{ massStatusDisplay }}
                </span>
            </div>
            <div class="col-span-full grid grid-cols-subgrid">
                <span>Created</span>
                <Tooltip>
                    <TooltipTrigger as-child>
                        <span class="cursor-help text-right">{{ createdAgo }}</span>
                    </TooltipTrigger>
                    <TooltipContent>{{ createdAt }}</TooltipContent>
                </Tooltip>
            </div>
            <div class="col-span-full grid grid-cols-subgrid">
                <span>Updated</span>
                <Tooltip>
                    <TooltipTrigger as-child>
                        <span class="cursor-help text-right">{{ updatedAgo }}</span>
                    </TooltipTrigger>
                    <TooltipContent>{{ updatedAt }}</TooltipContent>
                </Tooltip>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useNowUTC } from '@/composables/useNowUTC';
import { TMapConnection } from '@/pages/maps';
import { UTCDate } from '@date-fns/utc';
import { differenceInDays, differenceInHours, differenceInMinutes, format, formatDistanceStrict, max, min } from 'date-fns';
import { Heart } from 'lucide-vue-next';
import { computed } from 'vue';

const props = defineProps<{
    connection: TMapConnection;
}>();

const now = useNowUTC();

const massMeta = computed(() => {
    switch (props.connection.mass_status) {
        case 'fresh':
            return { label: 'Fresh', text: 'text-green-500', dot: 'bg-green-500' };
        case 'reduced':
            return { label: 'Reduced', text: 'text-amber-500', dot: 'bg-amber-500' };
        case 'critical':
            return { label: 'Critical', text: 'text-red-500', dot: 'bg-red-500' };
        default:
            return { label: 'Unknown', text: 'text-muted-foreground', dot: 'bg-neutral-500' };
    }
});

const typeMeta = computed(() =>
    props.connection.type === 'stargate'
        ? { label: 'Stargate', text: 'text-sky-500', dot: 'bg-sky-500' }
        : { label: 'Wormhole', text: 'text-foreground', dot: 'bg-neutral-500' },
);

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

const lifetimeMeta = computed(() => {
    switch (props.connection.lifetime_status) {
        case 'healthy':
            return { label: 'Healthy', text: 'text-green-500', dot: 'bg-green-500' };
        case 'eol':
            return { label: 'End of Life (<4h)', text: 'text-purple-500', dot: 'bg-purple-500' };
        case 'critical':
            return { label: 'Critical (<1h)', text: 'text-red-500', dot: 'bg-red-500' };
        default:
            return { label: 'Unknown', text: 'text-muted-foreground', dot: 'bg-neutral-500' };
    }
});
</script>

<template>
    <div class="space-y-1">
        <div class="border-b pb-1 text-xs font-medium text-foreground">Status</div>
        <div class="grid grid-cols-2 divide-y truncate text-xs text-muted-foreground *:py-1">
            <div class="col-span-full grid grid-cols-subgrid">
                <span>Type</span>
                <span class="flex items-center justify-end gap-1.5 text-right" :class="typeMeta.text">
                    <span class="inline-block size-2 rounded-full" :class="typeMeta.dot" />
                    {{ typeMeta.label }}
                </span>
            </div>
            <div class="col-span-full grid grid-cols-subgrid">
                <span>Lifetime</span>
                <span class="flex items-center justify-end gap-1.5 text-right" :class="lifetimeMeta.text">
                    <span class="inline-block size-2 shrink-0 rounded-full" :class="lifetimeMeta.dot" />
                    <template v-if="lifetimeAt && connection.lifetime_status !== 'healthy'">
                        <Tooltip>
                            <TooltipTrigger as-child>
                                <span class="cursor-help">{{ lifetimeMeta.label }}</span>
                            </TooltipTrigger>
                            <TooltipContent> {{ lifetimeAt }} ({{ lifetimeAgo }}) </TooltipContent>
                        </Tooltip>
                    </template>
                    <span v-else>{{ lifetimeMeta.label }}</span>
                </span>
            </div>
            <div class="col-span-full grid grid-cols-subgrid">
                <span>Mass Status</span>
                <span class="flex items-center justify-end gap-1.5 text-right" :class="massMeta.text">
                    <span class="inline-block size-2 rounded-full" :class="massMeta.dot" />
                    {{ massMeta.label }}
                </span>
            </div>
            <div v-if="connection.preserve_mass" class="col-span-full grid grid-cols-subgrid">
                <span>Preserve mass</span>
                <span class="flex items-center justify-end gap-1.5 text-right text-emerald-500">
                    <Heart class="size-3" />
                    Yes
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

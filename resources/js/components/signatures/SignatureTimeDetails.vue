<script setup lang="ts">
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { TProcessedConnection } from '@/composables/map';
import { useNowUTC } from '@/composables/useNowUTC';
import { TSignatureCategory } from '@/lib/SignatureParser';
import { TSignature } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import { differenceInDays, differenceInHours, differenceInMinutes, format, formatDistanceStrict, min } from 'date-fns';
import { computed } from 'vue';

const { category, selected_connection, signature } = defineProps<{
    category: TSignatureCategory;
    selected_connection: TProcessedConnection | null;
    signature: TSignature;
}>();

const now = useNowUTC();

const created_at = computed(() => {
    if (!selected_connection) return signature.created_at;

    return min([signature.created_at, selected_connection.created_at]);
});

const updated_at = computed(() => signature.updated_at);

const modified_at = computed(() => {
    if (category === 'Wormhole') {
        return created_at.value;
    }

    return updated_at.value;
});

const created_date = computed(() => new UTCDate(created_at.value));
const updated_date = computed(() => new UTCDate(updated_at.value));
const modified_date = computed(() => new UTCDate(modified_at.value));

const modified_ago = computed(() => {
    const diff_in_days = differenceInDays(now.value, modified_date.value);
    if (diff_in_days > 0) {
        return `${diff_in_days}d`;
    }
    const diff_in_hours = differenceInHours(now.value, modified_date.value);
    if (diff_in_hours > 0) {
        return `${diff_in_hours}h`;
    }
    const diff_in_minutes = differenceInMinutes(now.value, modified_date.value);
    if (diff_in_minutes > 0) {
        return `${diff_in_minutes}m`;
    }

    return 'now';
});

const created_date_formatted = computed(() => {
    return format(created_date.value, 'MMM dd, HH:mm');
});
const updated_date_formatted = computed(() => {
    return format(updated_date.value, 'MMM dd, HH:mm');
});

const lifetime_date = computed(() => {
    const lifetimeTime = signature.lifetime_updated_at || selected_connection?.lifetime_updated_at;
    return lifetimeTime ? new UTCDate(lifetimeTime) : null;
});

const lifetime_ago = computed(() => {
    if (!lifetime_date.value) return null;
    return formatDistanceStrict(lifetime_date.value, now.value, {
        addSuffix: true,
    });
});

const lifetime_display = computed(() => {
    const lifetime = signature.lifetime !== 'healthy' ? signature.lifetime : selected_connection?.lifetime;
    switch (lifetime) {
        case 'eol':
            return 'End of Life (<4h)';
        case 'critical':
            return 'Critical (<1h)';
        default:
            return null;
    }
});

const current_lifetime = computed(() => {
    return signature.lifetime !== 'healthy' ? signature.lifetime : selected_connection?.lifetime || 'healthy';
});
</script>

<template>
    <Tooltip>
        <TooltipTrigger
            :data-lifetime="current_lifetime"
            :data-mass="selected_connection?.mass_status || signature.mass_status"
            class="time whitespace-nowrap text-neutral-500"
        >
            <span>
                {{ modified_ago }}
            </span>
        </TooltipTrigger>
        <TooltipContent class="grid grid-cols-[auto_auto] gap-2">
            <span class="font-semibold">Created at</span>
            <p class="">{{ created_date_formatted }}</p>
            <span class="font-semibold">Last modified at</span>
            <p class="">{{ updated_date_formatted }}</p>
            <template v-if="lifetime_ago && lifetime_display">
                <span class="font-semibold">{{ lifetime_display }}</span>
                <p class="">{{ lifetime_ago }}</p>
            </template>
        </TooltipContent>
    </Tooltip>
</template>

<style scoped>
/* Lifetime + Mass combinations with animations */
.time[data-lifetime='critical'][data-mass='critical'] {
    color: var(--color-red-500);
    animation: lifetime-critical-mass-critical 2s infinite;
}

@keyframes lifetime-critical-mass-critical {
    0%,
    100% {
        color: var(--color-red-500);
    }
    50% {
        color: var(--color-red-700);
    }
}

.time[data-lifetime='critical'][data-mass='reduced'] {
    color: var(--color-red-500);
    animation: lifetime-critical-mass-reduced 2s infinite;
}

@keyframes lifetime-critical-mass-reduced {
    0%,
    100% {
        color: var(--color-red-500);
    }
    50% {
        color: var(--color-orange-500);
    }
}

.time[data-lifetime='eol'][data-mass='critical'] {
    color: var(--color-red-500);
    animation: lifetime-eol-mass-critical 2s infinite;
}

@keyframes lifetime-eol-mass-critical {
    0%,
    100% {
        color: var(--color-red-500);
    }
    50% {
        color: var(--color-purple-500);
    }
}

.time[data-lifetime='eol'][data-mass='reduced'] {
    color: var(--color-orange-500);
    animation: lifetime-eol-mass-reduced 2s infinite;
}

@keyframes lifetime-eol-mass-reduced {
    0%,
    100% {
        color: var(--color-orange-500);
    }
    50% {
        color: var(--color-purple-500);
    }
}

/* Pure lifetime states */
.time[data-lifetime='critical'] {
    color: var(--color-red-500);
}

.time[data-lifetime='eol'] {
    color: var(--color-purple-500);
}

/* Pure mass states */
.time[data-mass='unknown'] {
    color: var(--color-neutral-500);
}

.time[data-mass='reduced'] {
    color: var(--color-orange-500);
}

.time[data-mass='critical'] {
    color: var(--color-red-500);
}
</style>

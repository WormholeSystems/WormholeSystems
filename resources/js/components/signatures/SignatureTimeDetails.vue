<script setup lang="ts">
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { TProcessedConnection } from '@/composables/map';
import { useNowUTC } from '@/composables/useNowUTC';
import { TSignatureCategory } from '@/lib/SignatureParser';
import { TSignature } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import { differenceInDays, differenceInHours, differenceInMinutes, format, min } from 'date-fns';
import { computed } from 'vue';

const { category, selected_connection, signature } = defineProps<{
    category: TSignatureCategory;
    selected_connection: TProcessedConnection | null;
    signature: TSignature;
}>();

const is_eol = computed(() => {
    return Boolean(signature.marked_as_eol_at) || Boolean(selected_connection?.marked_as_eol_at);
});

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
</script>

<template>
    <Tooltip>
        <TooltipTrigger
            :data-eol="is_eol"
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
        </TooltipContent>
    </Tooltip>
</template>

<style scoped>
.time[data-eol='true'][data-mass='critical'] {
    color: var(--color-red-500);
    animation: eol-critical 2s infinite;
}

@keyframes eol-critical {
    0%,
    100% {
        color: var(--color-red-500);
    }
    50% {
        color: var(--color-purple-500);
    }
}

.time[data-eol='true'][data-mass='reduced'] {
    color: var(--color-orange-500);
    animation: eol-heavy 2s infinite;
}

@keyframes eol-heavy {
    0%,
    100% {
        color: var(--color-orange-500);
    }
    50% {
        color: var(--color-purple-500);
    }
}

.time[data-mass='unknown'] {
    color: var(--color-neutral-500);
}

.time[data-mass='reduced'] {
    color: var(--color-orange-500);
}

.time[data-mass='critical'] {
    color: var(--color-red-500);
}

.time[data-eol='true'] {
    color: var(--color-purple-500);
}
</style>

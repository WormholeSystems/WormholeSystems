<script setup lang="ts">
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { TProcessedConnection } from '@/composables/map';
import { TSignatureCategory } from '@/lib/SignatureParser';
import { TSignature } from '@/types/models';
import { useNow } from '@vueuse/core';
import { differenceInHours, differenceInSeconds, format, min } from 'date-fns';
import { computed } from 'vue';

const { category, selected_connection, signature } = defineProps<{
    category: TSignatureCategory;
    selected_connection: TProcessedConnection | null;
    signature: TSignature;
}>();

const now = useNow();

const created_at = computed(() => {
    if (category === 'Wormhole') {
        if (selected_connection?.created_at) {
            const connection_created_at = new Date(selected_connection.created_at);
            const signature_created_at = new Date(signature.created_at);
            return min([connection_created_at, signature_created_at]);
        }
    }

    return new Date(signature.created_at);
});

const updated_at = computed(() => {
    return signature.updated_at ? new Date(signature.updated_at) : null;
});

const modified_date = computed(() => {
    if (signature.category === 'Wormhole') {
        return created_at.value;
    }

    return updated_at.value || created_at.value;
});

const modified_at = computed(() => {
    const seconds = differenceInSeconds(now.value, modified_date.value);
    if (seconds < 60) {
        return 'now';
    }

    if (seconds < 3600) {
        return `${Math.floor(seconds / 60)}m`;
    }

    if (seconds < 86400) {
        return `${Math.floor(seconds / 3600)}h`;
    }

    return `${Math.floor(seconds / 86400)}d`;
});

const modified_class = computed(() => {
    const hours = differenceInHours(now.value, modified_date.value);

    if (selected_connection?.is_eol) {
        return 'eol';
    }
    if (hours < 8) {
        return 'fresh';
    }
    if (hours < 24) {
        return 'old';
    }

    return 'very-old';
});
</script>

<template>
    <Tooltip>
        <TooltipTrigger
            :data-modified-class="modified_class"
            class="whitespace-nowrap data-[modified-class=eol]:text-purple-500 data-[modified-class=fresh]:text-neutral-500 data-[modified-class=old]:text-neutral-500 data-[modified-class=very-old]:text-neutral-500"
        >
            {{ modified_at }}
        </TooltipTrigger>
        <TooltipContent class="grid grid-cols-[auto_auto] gap-2">
            <span class="font-semibold">Created at</span>
            <p class="">{{ format(created_at, 'MMM dd, HH:ii') }}</p>
            <span class="font-semibold">Last modified at</span>
            <p class="">{{ modified_date ? format(modified_date, 'MMM dd, HH:ii') : 'N/A' }}</p>
        </TooltipContent>
    </Tooltip>
</template>

<style scoped></style>

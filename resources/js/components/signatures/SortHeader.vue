<script setup lang="ts">
import ChevronDownIcon from '@/components/icons/ChevronDownIcon.vue';
import ChevronUpIcon from '@/components/icons/ChevronUpIcon.vue';
import { computed } from 'vue';

type Props = {
    label: string;
    column: string;
    isCurrentColumn: boolean;
    currentDirection: 'asc' | 'desc';
};

const { label, column, isCurrentColumn, currentDirection } = defineProps<Props>();

const emit = defineEmits<{
    sort: [column: string];
}>();

const isAscending = computed(() => isCurrentColumn && currentDirection === 'asc');
const isDescending = computed(() => isCurrentColumn && currentDirection === 'desc');

function handleClick() {
    emit('sort', column);
}
</script>

<template>
    <button
        @click="handleClick"
        class="flex w-full cursor-pointer items-center justify-between gap-1 text-left transition-colors hover:text-foreground"
    >
        <span>{{ label }}</span>
        <ChevronUpIcon v-if="isAscending" class="h-3 w-3" />
        <ChevronDownIcon v-else-if="isDescending" class="h-3 w-3" />
    </button>
</template>

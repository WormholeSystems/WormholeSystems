<script setup lang="ts">
import ChevronDownIcon from '@/components/icons/ChevronDownIcon.vue';
import ChevronUpIcon from '@/components/icons/ChevronUpIcon.vue';
import { computed } from 'vue';

type Props = {
    label: string;
    column: string;
    isCurrentColumn: boolean;
    currentDirection: 'asc' | 'desc';
}

const {
    label,
    column,
    isCurrentColumn,
    currentDirection,
} = defineProps<Props>()

const emit = defineEmits<{
    sort: [column: string];
}>();

const isAscending = computed(() =>isCurrentColumn && currentDirection === 'asc');
const isDescending = computed(() =>isCurrentColumn && currentDirection === 'desc');

function handleClick() {
    emit('sort', column);
}
</script>

<template>
    <button
        @click="handleClick"
        class="
            flex items-center gap-1 text-left hover:text-foreground transition-colors cursor-pointer w-full justify-between
        "
    >
        <span>{{ label }}</span>
            <ChevronUpIcon
                v-if="isAscending"
                class="h-3 w-3"
            />
            <ChevronDownIcon
                v-else-if="isDescending"
                class="h-3 w-3"
            />
    </button>
</template>

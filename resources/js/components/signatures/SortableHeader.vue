<script setup lang="ts">
import ChevronDownIcon from '@/components/icons/ChevronDownIcon.vue';
import ChevronUpIcon from '@/components/icons/ChevronUpIcon.vue';
import { Button } from '@/components/ui/button';

defineProps<{
    direction?: 'asc' | 'desc';
}>();
</script>

<template>
    <Button variant="ghost" class="flex w-full justify-between">
        <slot />
        <div class="w-4">
            <ChevronUpIcon v-if="direction === 'asc'" />
            <ChevronDownIcon v-else-if="direction === 'desc'" />
        </div>
    </Button>
</template>

<style scoped></style>

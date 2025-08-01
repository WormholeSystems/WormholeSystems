<script setup lang="ts">
import { TCharacter } from '@/types/models';

defineProps<{
    pilots: TCharacter[];
}>();
</script>

<template>
    <span class="flex h-[20px] items-center gap-1 truncate border-t border-b-neutral-300 px-2 pt-0.5 text-[10px] leading-0 dark:border-neutral-700">
        <span class="size-1 animate-pulse rounded-full bg-green-500"></span>{{ pilots.at(0)?.name }}
        <span v-if="pilots.length > 1">and {{ pilots.length - 1 }} more</span>
    </span>
</template>

<style scoped></style>

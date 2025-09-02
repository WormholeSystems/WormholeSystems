<script setup lang="ts">

import { CardContent } from '@/components/ui/card';
import { cn } from '@/lib/utils';

defineProps<{
    innerClass?: string;
}>()
</script>

<template>
    <CardContent class="px-1 pb-1">
        <div :class="cn('rounded-lg border bg-white dark:bg-neutral-900/40', innerClass)">
        <slot />
        </div>
  </CardContent>
</template>

<style scoped>

</style>

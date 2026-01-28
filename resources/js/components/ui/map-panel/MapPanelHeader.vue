<script setup lang="ts">
import type { HTMLAttributes } from 'vue';
import { cn } from '@/lib/utils';

const props = defineProps<{
    class?: HTMLAttributes['class'];
}>();
</script>

<template>
    <div :class="cn('flex h-9 shrink-0 items-center justify-between border-b border-border/50 bg-muted/30 px-3', props.class)">
        <h3 class="font-mono text-[10px] uppercase tracking-wider text-muted-foreground">
            <slot />
        </h3>
        <div v-if="$slots.actions" class="flex items-center gap-2">
            <slot name="actions" />
        </div>
    </div>
</template>

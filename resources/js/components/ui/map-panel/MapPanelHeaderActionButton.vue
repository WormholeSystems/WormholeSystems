<script setup lang="ts">
import type { HTMLAttributes } from 'vue';
import { computed } from 'vue';
import { Button, type ButtonVariants } from '@/components/ui/button';
import { cn } from '@/lib/utils';

const props = withDefaults(
    defineProps<{
        variant?: ButtonVariants['variant'];
        size?: ButtonVariants['size'];
        class?: HTMLAttributes['class'];
        type?: 'button' | 'submit' | 'reset';
        asChild?: boolean;
        as?: string;
    }>(),
    {
        variant: 'ghost',
        size: 'xs',
    },
);

const sizingClass = computed(() => {
    if (props.size === 'icon') {
        return 'h-6 w-6 p-0';
    }

    return 'h-6 px-2 text-[11px] leading-none';
});
</script>

<template>
    <Button
        :variant="variant"
        :size="size"
        :type="type"
        :as-child="asChild"
        :as="as"
        :class="cn('gap-1 shadow-none', sizingClass, props.class)"
    >
        <slot />
    </Button>
</template>

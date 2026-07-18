<script setup lang="ts">
import type { ComboboxTriggerProps } from 'reka-ui';
import type { HTMLAttributes } from 'vue';
import { reactiveOmit } from '@vueuse/core';
import { ChevronDown } from 'lucide-vue-next';
import { ComboboxTrigger, useForwardProps } from 'reka-ui';
import { cn } from '@/lib/utils';

const props = withDefaults(defineProps<ComboboxTriggerProps & { class?: HTMLAttributes['class']; size?: 'sm' | 'default' }>(), {
    size: 'default',
});

const delegatedProps = reactiveOmit(props, 'class', 'size');

const forwarded = useForwardProps(delegatedProps);

/**
 * Size styles are plain classes (not data-[size] variants) so consumer class
 * overrides win via tailwind-merge instead of losing on selector specificity.
 */
const sizeClasses: Record<'sm' | 'default', string> = {
    default: 'h-9 px-3 text-sm',
    sm: 'h-5 px-3 text-xs',
};
</script>

<template>
    <ComboboxTrigger
        data-slot="combobox-trigger"
        :data-size="size"
        v-bind="forwarded"
        :class="
            cn(
                `border-input focus-visible:border-ring focus-visible:ring-ring/50 dark:bg-input/30 dark:hover:bg-input/50 flex w-full items-center justify-between gap-2 rounded-md border bg-transparent whitespace-nowrap shadow-xs transition-[color,box-shadow] outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0`,
                sizeClasses[size],
                props.class,
            )
        "
        tabindex="0"
    >
        <slot />
        <ChevronDown class="size-4 opacity-50" />
    </ComboboxTrigger>
</template>

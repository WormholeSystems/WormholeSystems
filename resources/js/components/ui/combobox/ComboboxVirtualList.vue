<script setup lang="ts" generic="T extends AcceptableValue">
import type { AcceptableValue } from 'reka-ui';
import type { HTMLAttributes } from 'vue';
import { ComboboxVirtualizer } from 'reka-ui';
import { cn } from '@/lib/utils';
import ComboboxList from './ComboboxList.vue';
import ComboboxViewport from './ComboboxViewport.vue';

defineOptions({
    inheritAttrs: false,
});

/**
 * The scaffold every virtualized combobox shares: a popup list whose scrolling
 * is delegated to the inner viewport (the virtualizer's scroll parent), an
 * empty state, and the virtualizer itself. The scroll-neutralizing classes on
 * ComboboxList live here and nowhere else — consumers only provide options and
 * an item slot (plus an optional #header, e.g. an in-popup search input).
 */
const props = withDefaults(
    defineProps<{
        options: T[];
        estimateSize?: number;
        textContent?: (option: T) => string;
        emptyText?: string;
        class?: HTMLAttributes['class'];
    }>(),
    {
        estimateSize: 28,
        textContent: undefined,
        emptyText: 'No results found',
        class: undefined,
    },
);

defineSlots<{
    default?(props: { option: T }): unknown;
    header?(): unknown;
    empty?(): unknown;
}>();
</script>

<template>
    <ComboboxList v-bind="$attrs" :class="cn('max-h-none overflow-y-visible p-0', props.class)">
        <slot name="header" />
        <ComboboxViewport class="max-h-72">
            <div v-if="options.length === 0" class="px-2 py-1.5 text-xs text-muted-foreground">
                <slot name="empty">{{ emptyText }}</slot>
            </div>
            <ComboboxVirtualizer v-slot="{ option }" :options="options" :estimate-size="estimateSize" :text-content="textContent">
                <!-- The virtualizer clones its slot's first vnode to position the row, and a
                     forwarded slot is a Fragment it cannot clone — so the row root must be a
                     concrete element owned by this component. -->
                <div class="w-full">
                    <slot :option="option as T" />
                </div>
            </ComboboxVirtualizer>
        </ComboboxViewport>
    </ComboboxList>
</template>

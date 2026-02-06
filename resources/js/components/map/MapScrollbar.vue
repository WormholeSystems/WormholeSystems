<script setup lang="ts">
defineProps<{
    orientation: 'vertical' | 'horizontal';
    thumb_size: number;
    thumb_offset: number;
    visible: boolean;
    track_size: number;
    scrollbar_size: number;
}>();

const emit = defineEmits<{
    'track-mousedown': [event: MouseEvent];
    'thumb-mousedown': [event: MouseEvent];
}>();
</script>

<template>
    <div
        v-if="orientation === 'vertical'"
        class="absolute top-0 right-0 z-40 transition-opacity duration-300"
        :class="visible ? 'opacity-100' : 'pointer-events-none opacity-0'"
        :style="{ width: scrollbar_size + 'px', height: track_size + 'px' }"
        @mousedown.prevent="emit('track-mousedown', $event)"
    >
        <div
            class="absolute left-0 w-full rounded-full bg-neutral-400/60 transition-colors hover:bg-neutral-400/80 dark:bg-neutral-600/60 dark:hover:bg-neutral-600/80"
            :style="{
                height: thumb_size + 'px',
                top: thumb_offset + 'px',
            }"
            @mousedown.prevent.stop="emit('thumb-mousedown', $event)"
        />
    </div>
    <div
        v-else
        class="absolute bottom-0 left-0 z-40 transition-opacity duration-300"
        :class="visible ? 'opacity-100' : 'pointer-events-none opacity-0'"
        :style="{ height: scrollbar_size + 'px', width: track_size + 'px' }"
        @mousedown.prevent="emit('track-mousedown', $event)"
    >
        <div
            class="absolute top-0 h-full rounded-full bg-neutral-400/60 transition-colors hover:bg-neutral-400/80 dark:bg-neutral-600/60 dark:hover:bg-neutral-600/80"
            :style="{
                width: thumb_size + 'px',
                left: thumb_offset + 'px',
            }"
            @mousedown.prevent.stop="emit('thumb-mousedown', $event)"
        />
    </div>
</template>

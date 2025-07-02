<script setup lang="ts">
import { useMap } from '@/composables/useMap';
import { useMapMouse } from '@/composables/useMapMouse';
import { useEventListener } from '@vueuse/core';
import { ref } from 'vue';

type Coords = { x: number; y: number };

const map = useMap();

const mouse = useMapMouse();

const start_position = ref<Coords>();

useEventListener('mouseup', handleDragEnd);

function handleDragStart(event: MouseEvent) {
    const rect = map.value.container!.getBoundingClientRect();
    start_position.value = {
        x: event.clientX - rect.left,
        y: event.clientY - rect.top,
    };
}

function handleDragEnd() {
    start_position.value = undefined;
}
</script>

<template>
    <div class="absolute inset-0" @mousedown="handleDragStart" />
    <div
        v-if="start_position"
        class="pointer-events-none absolute border-2 border-blue-500"
        :style="{
            left: `${Math.min(start_position.x, mouse.x)}px`,
            top: `${Math.min(start_position.y, mouse.y)}px`,
            width: `${Math.abs(start_position.x - mouse.x)}px`,
            height: `${Math.abs(start_position.y - mouse.y)}px`,
        }"
    ></div>
</template>

<style scoped></style>

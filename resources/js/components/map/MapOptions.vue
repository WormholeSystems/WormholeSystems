<script setup lang="ts">
import GripLines from '@/components/icons/GripLines.vue';
import MinusIcon from '@/components/icons/MinusIcon.vue';
import PlusIcon from '@/components/icons/PlusIcon.vue';
import { Button } from '@/components/ui/button';
import { useMapContainer, useMapScale } from '@/composables/map';
import { useLayout } from '@/composables/useLayout';
import { TMapConfig } from '@/types/map';
import { useEventListener } from '@vueuse/core';
import { ref } from 'vue';

const { config } = defineProps<{
    config: TMapConfig;
}>();

const { scale, setScale } = useMapScale();

const container = useMapContainer();

const { layout, setLayout } = useLayout();

const resizing = ref(false);

function onResizeStart(event: MouseEvent) {
    event.preventDefault();
    resizing.value = true;
}

useEventListener('pointermove', (event) => {
    if (!resizing.value) {
        return;
    }

    event.preventDefault();

    const map_container = container.value;
    if (!map_container) {
        return;
    }

    const rect = map_container.getBoundingClientRect();
    const new_height = Math.max(300, event.clientY - rect.top);

    if (new_height < 300 || new_height > config.max_size.y + 4) {
        return;
    }

    setLayout({
        ...layout.value,
        map_height: new_height,
    });
});
useEventListener('pointerup', () => {
    if (!resizing.value) {
        return;
    }
    resizing.value = false;
});
</script>

<template>
    <div class="absolute right-0 bottom-0 z-30 flex overflow-hidden rounded-tl-lg border bg-neutral-100 dark:bg-neutral-900">
        <span class="flex h-8 items-center px-2 text-muted-foreground">{{ (scale * 100).toFixed(0) + '%' }}</span>
        <Button variant="ghost" size="icon" class="h-8 w-8" @click="setScale(scale - 0.1)">
            <MinusIcon />
        </Button>
        <Button variant="ghost" size="icon" class="h-8 w-8" @click="setScale(scale + 0.1)">
            <PlusIcon />
        </Button>
        <div @pointerdown="onResizeStart" id="map-resize-handle" class="flex size-8 cursor-ns-resize items-center justify-center">
            <GripLines class="text-muted-foreground" />
        </div>
    </div>
</template>

<style scoped></style>

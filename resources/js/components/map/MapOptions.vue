<script setup lang="ts">
import BackgroundImageIcon from '@/components/icons/BackgroundImageIcon.vue';
import CheckIcon from '@/components/icons/CheckIcon.vue';
import GripLines from '@/components/icons/GripLines.vue';
import MinusIcon from '@/components/icons/MinusIcon.vue';
import PlusIcon from '@/components/icons/PlusIcon.vue';
import TimesIcon from '@/components/icons/TimesIcon.vue';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { useMapContainer, useMapScale } from '@/composables/map';
import { useLayout } from '@/composables/useLayout';
import { useMapBackground } from '@/composables/useMapBackground';
import { TMapConfig } from '@/types/map';
import { useEventListener } from '@vueuse/core';
import { ref } from 'vue';

const { config } = defineProps<{
    config: TMapConfig;
}>();

const { scale, setScale } = useMapScale();

const container = useMapContainer();

const { layout, setLayout } = useLayout();

const { backgroundImageUrl, setBackgroundImageUrl, clearBackgroundImage } = useMapBackground();

const resizing = ref(false);
const backgroundInputUrl = ref('');

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

function applyBackgroundImage() {
    if (backgroundInputUrl.value.trim()) {
        setBackgroundImageUrl(backgroundInputUrl.value.trim());
        backgroundInputUrl.value = '';
    }
}

function removeBackgroundImage() {
    clearBackgroundImage();
}
</script>

<template>
    <div class="absolute right-0 bottom-0 z-30 flex overflow-hidden rounded-tl-lg border bg-neutral-100 dark:bg-neutral-900">
        <Popover>
            <PopoverTrigger as-child>
                <Button variant="ghost" size="icon" class="h-8 w-8 text-muted-foreground" title="Background Image">
                    <BackgroundImageIcon />
                </Button>
            </PopoverTrigger>
            <PopoverContent class="w-80" side="top" align="end">
                <div class="flex gap-2">
                    <Input v-model="backgroundInputUrl" placeholder="Enter image URL..." class="flex-1" @keydown.enter="applyBackgroundImage" />
                    <Button variant="ghost" size="icon" @click="applyBackgroundImage" :disabled="!backgroundInputUrl.trim()" title="Apply">
                        <CheckIcon />
                    </Button>
                    <Button variant="ghost" size="icon" @click="removeBackgroundImage" :disabled="!backgroundImageUrl" title="Clear">
                        <TimesIcon />
                    </Button>
                </div>
            </PopoverContent>
        </Popover>
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

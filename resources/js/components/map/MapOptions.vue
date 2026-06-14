<script setup lang="ts">
import BackgroundImageIcon from '@/components/icons/BackgroundImageIcon.vue';
import MinusIcon from '@/components/icons/MinusIcon.vue';
import PlusIcon from '@/components/icons/PlusIcon.vue';
import { Button } from '@/components/ui/button';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { useMapScale } from '@/composables/map';
import { TMapBackgroundMode, useMapBackground } from '@/composables/useMapBackground';
import { ImageUp, Trash2 } from 'lucide-vue-next';
import { ref, useTemplateRef } from 'vue';

const { scale, setScale } = useMapScale();

const { backgroundImageUrl, backgroundMode, setBackgroundImageUrl, clearBackgroundImage, setBackgroundMode } = useMapBackground();

const fileInput = useTemplateRef<HTMLInputElement>('file-input');
const is_dragging = ref(false);

const modeOptions: { value: TMapBackgroundMode; label: string; hint: string }[] = [
    { value: 'grid', label: 'Move with map', hint: 'Spans the grid, pans and zooms with the systems' },
    { value: 'viewport', label: 'Fit to view', hint: 'Fills the panel and stays put while you navigate' },
];

function triggerUpload() {
    fileInput.value?.click();
}

function loadFile(file: File) {
    const reader = new FileReader();
    reader.onload = () => {
        if (typeof reader.result === 'string') {
            setBackgroundImageUrl(reader.result);
        }
    };
    reader.readAsDataURL(file);
}

function onFileSelected(event: Event) {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    // Reset so re-selecting the same file fires change again.
    input.value = '';
    if (file) loadFile(file);
}

function onDrop(event: DragEvent) {
    is_dragging.value = false;
    const file = event.dataTransfer?.files?.[0];
    if (file?.type.startsWith('image/')) loadFile(file);
}
</script>

<template>
    <div class="absolute right-3 bottom-3 z-30 flex items-center gap-0.5 rounded-full bg-white/60 px-1 dark:bg-neutral-800/60">
        <Popover>
            <PopoverTrigger as-child>
                <Button variant="ghost" size="icon" class="h-8 w-8 rounded-full text-neutral-600 dark:text-neutral-400" title="Background Image">
                    <BackgroundImageIcon />
                </Button>
            </PopoverTrigger>
            <PopoverContent class="w-72 p-3" side="top" align="end">
                <div class="flex flex-col gap-3">
                    <div class="flex items-center justify-between">
                        <span class="text-sm font-medium">Background</span>
                        <Button
                            v-if="backgroundImageUrl"
                            variant="ghost"
                            size="sm"
                            class="h-6 gap-1 px-2 text-xs text-muted-foreground hover:text-destructive"
                            @click="clearBackgroundImage"
                        >
                            <Trash2 class="size-3.5" />
                            Remove
                        </Button>
                    </div>

                    <input ref="file-input" type="file" accept="image/*" class="hidden" @change="onFileSelected" />
                    <div
                        class="group relative flex aspect-video cursor-pointer items-center justify-center overflow-hidden rounded-xl border-2 border-dashed transition-colors"
                        :class="is_dragging ? 'border-primary bg-primary/10' : 'border-border hover:border-primary/50 hover:bg-muted/40'"
                        @click="triggerUpload"
                        @dragover.prevent="is_dragging = true"
                        @dragleave="is_dragging = false"
                        @drop.prevent="onDrop"
                    >
                        <template v-if="backgroundImageUrl">
                            <img :src="backgroundImageUrl" alt="Map background preview" class="absolute inset-0 h-full w-full object-cover" />
                            <div
                                class="absolute inset-0 flex flex-col items-center justify-center gap-1 bg-black/55 text-white opacity-0 transition-opacity group-hover:opacity-100"
                            >
                                <ImageUp class="size-5" />
                                <span class="text-xs font-medium">Replace image</span>
                            </div>
                        </template>
                        <div v-else class="flex flex-col items-center gap-1.5 text-muted-foreground">
                            <div class="flex size-9 items-center justify-center rounded-full bg-muted transition-colors group-hover:bg-primary/10">
                                <ImageUp class="size-4" />
                            </div>
                            <span class="text-xs font-medium text-foreground">Drag &amp; drop or click</span>
                            <span class="text-[10px]">PNG, JPG, GIF or WebP</span>
                        </div>
                    </div>

                    <div v-if="backgroundImageUrl" class="flex flex-col gap-1.5">
                        <span class="text-xs font-medium text-muted-foreground">Position</span>
                        <div class="grid grid-cols-2 gap-1 rounded-lg bg-muted p-1">
                            <button
                                v-for="option in modeOptions"
                                :key="option.value"
                                type="button"
                                class="rounded-md px-2 py-1.5 text-xs font-medium transition-colors"
                                :class="
                                    backgroundMode === option.value
                                        ? 'bg-background text-foreground shadow-sm'
                                        : 'text-muted-foreground hover:text-foreground'
                                "
                                :title="option.hint"
                                @click="setBackgroundMode(option.value)"
                            >
                                {{ option.label }}
                            </button>
                        </div>
                    </div>
                </div>
            </PopoverContent>
        </Popover>
        <span class="flex h-8 items-center px-1 text-neutral-600 dark:text-neutral-400">{{ (scale * 100).toFixed(0) + '%' }}</span>
        <Button variant="ghost" size="icon" class="h-8 w-8 rounded-full text-neutral-600 dark:text-neutral-400" @click="setScale(scale - 0.1)">
            <MinusIcon />
        </Button>
        <Button variant="ghost" size="icon" class="h-8 w-8 rounded-full text-neutral-600 dark:text-neutral-400" @click="setScale(scale + 0.1)">
            <PlusIcon />
        </Button>
    </div>
</template>

<style scoped></style>

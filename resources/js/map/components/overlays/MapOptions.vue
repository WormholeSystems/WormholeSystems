<script setup lang="ts">
import MapBackgroundImageController from '@/actions/App/Http/Controllers/MapBackgroundImageController';
import BackgroundImageIcon from '@/components/icons/BackgroundImageIcon.vue';
import MinusIcon from '@/components/icons/MinusIcon.vue';
import PlusIcon from '@/components/icons/PlusIcon.vue';
import { Button } from '@/components/ui/button';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useLayout } from '@/composables/useLayout';
import { TMapBackgroundMode, useMapBackground } from '@/composables/useMapBackground';
import { updateMapUserSettings } from '@/map/actions/updateMapUserSettings';
import { useMapStore } from '@/map/store/mapStore';
import { router } from '@inertiajs/vue3';
import { ImageUp, Loader2, Trash2, Waypoints, Workflow } from 'lucide-vue-next';
import { computed, ref, useTemplateRef, type Component } from 'vue';
import { toast } from 'vue-sonner';

/**
 * The bottom-right map options cluster (placement mode, background image,
 * zoom). Ported from the old tree onto the store: the effective layout and map
 * metadata come from the store, and zoom writes go through useLayout's
 * setLayout (cookie + debounced server sync) — MapRoot mirrors the layout's
 * scale back into the store, exactly like the old mapState.scale flow.
 */
type TMapLayoutMode = 'manual' | 'tree';

const store = useMapStore();

const { layout, setLayout } = useLayout();

const scale = computed(() => store.scale.value);

function setScale(newScale: number) {
    const roundedScale = Math.round(newScale * 10) / 10;
    if (roundedScale < 0.5 || roundedScale > 2) return;
    setLayout({
        ...layout.value,
        scale: roundedScale,
    });
}

const effective_layout = computed(() => store.effectiveLayout.value);

const layoutModes: { value: TMapLayoutMode; icon: Component; label: string }[] = [
    { value: 'manual', icon: Waypoints, label: 'Custom placement' },
    { value: 'tree', icon: Workflow, label: 'Automatic placement' },
];

// A personal override of the map's default placement. Picking the map default clears the
// override so the viewer follows the map again.
function setPlacement(mode: TMapLayoutMode) {
    const meta = store.meta.value;
    if (!meta) return;
    updateMapUserSettings(meta.slug, { layout_override: mode === meta.layout ? null : mode }, ['map_user_settings']);
}

const allowLayoutOverride = computed(() => store.meta.value?.allow_layout_override ?? false);
const mapSlug = computed(() => store.meta.value?.slug ?? null);

const { backgroundImageUrl, backgroundMode, canUpload } = useMapBackground();

const fileInput = useTemplateRef<HTMLInputElement>('file-input');
const is_dragging = ref(false);
const is_uploading = ref(false);

const modeOptions: { value: TMapBackgroundMode; label: string; hint: string }[] = [
    { value: 'grid', label: 'Move with map', hint: 'Spans the grid, pans and zooms with the systems' },
    { value: 'viewport', label: 'Fit to view', hint: 'Fills the panel and stays put while you navigate' },
];

function triggerUpload() {
    if (!is_uploading.value) fileInput.value?.click();
}

function uploadFile(file: File) {
    if (!mapSlug.value) return;
    router.post(
        MapBackgroundImageController.store(mapSlug.value).url,
        { background_image: file },
        {
            forceFormData: true,
            preserveScroll: true,
            preserveState: true,
            only: ['map_user_settings'],
            onStart: () => (is_uploading.value = true),
            onFinish: () => (is_uploading.value = false),
            onError: (errors) => toast.error(errors.background_image ?? 'Failed to upload background image'),
        },
    );
}

function removeBackground() {
    if (!mapSlug.value) return;
    router.delete(MapBackgroundImageController.destroy(mapSlug.value).url, {
        preserveScroll: true,
        preserveState: true,
        only: ['map_user_settings'],
    });
}

function setMode(mode: TMapBackgroundMode) {
    if (!mapSlug.value) return;
    updateMapUserSettings(mapSlug.value, { background_image_mode: mode }, ['map_user_settings']);
}

function onFileSelected(event: Event) {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    // Reset so re-selecting the same file fires change again.
    input.value = '';
    if (file) uploadFile(file);
}

function onDrop(event: DragEvent) {
    is_dragging.value = false;
    if (is_uploading.value) return;
    const file = event.dataTransfer?.files?.[0];
    if (file?.type.startsWith('image/')) uploadFile(file);
}
</script>

<template>
    <div class="absolute right-3 bottom-3 z-30 flex items-center gap-0.5 rounded-full bg-white/60 px-1 dark:bg-neutral-800/60">
        <template v-if="allowLayoutOverride">
            <Tooltip v-for="mode in layoutModes" :key="mode.value" :delay-duration="300">
                <TooltipTrigger as-child>
                    <Button
                        variant="ghost"
                        size="icon"
                        class="h-8 w-8 rounded-full"
                        :class="
                            effective_layout === mode.value
                                ? 'bg-white text-foreground shadow-sm dark:bg-neutral-700'
                                : 'text-neutral-600 dark:text-neutral-400'
                        "
                        @click="setPlacement(mode.value)"
                    >
                        <component :is="mode.icon" class="size-4" />
                    </Button>
                </TooltipTrigger>
                <TooltipContent>{{ mode.label }}</TooltipContent>
            </Tooltip>
            <span class="mx-0.5 h-5 w-px bg-neutral-300/70 dark:bg-neutral-600/70" />
        </template>
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
                            :disabled="is_uploading"
                            @click="removeBackground"
                        >
                            <Trash2 class="size-3.5" />
                            Remove
                        </Button>
                    </div>

                    <input ref="file-input" type="file" accept="image/*" class="hidden" @change="onFileSelected" />
                    <div
                        v-if="canUpload"
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
                            <span class="text-[10px]">PNG, JPG, GIF or WebP · max 8 MB</span>
                        </div>

                        <div v-if="is_uploading" class="absolute inset-0 flex items-center justify-center bg-background/70">
                            <Loader2 class="size-5 animate-spin text-primary" />
                        </div>
                    </div>
                    <p v-else class="rounded-xl border border-dashed border-border px-3 py-4 text-center text-xs text-muted-foreground">
                        Sign in to set a background image for this map.
                    </p>

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
                                @click="setMode(option.value)"
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

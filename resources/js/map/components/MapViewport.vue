<script setup lang="ts">
import { ContextMenu, ContextMenuTrigger } from '@/components/ui/context-menu';
import { useMapBackground } from '@/composables/useMapBackground';
import MapScrollbar from '@/map/components/MapScrollbar.vue';
import { clientToBase } from '@/map/core/coords';
import type { Vec2 } from '@/map/core/types';
import { resolveNodeId, usePointerGestures, type Gesture } from '@/map/interactions/gestures';
import { useMapScrollbars } from '@/map/interactions/useMapScrollbars';
import { useMapStore } from '@/map/store/mapStore';
import { computed, useTemplateRef } from 'vue';

/**
 * The scroll container plus its visual chrome (grid, background image modes,
 * custom scrollbars, marquee box), ported from the old MapComponent template.
 * The container stays overflow-hidden like the old one — that's what hides the
 * native scrollbars while keeping programmatic scrolling (panning, thumb drags)
 * working — and the pointer-gesture arbiter is mounted on it.
 *
 * Slots: default = canvas content (edges + nodes), #context-menu = the menu
 * content of the canvas-wide ContextMenu, #overlays = panel-fixed chrome
 * (rally badge, options, …) rendered above the scroll surface.
 */
const { gestures } = defineProps<{
    gestures: Gesture[];
}>();

const emit = defineEmits<{
    (e: 'contextMenuOpenChange', open: boolean): void;
    (e: 'surfaceContextMenu', payload: { nodeId: number | null; basePoint: Vec2 }): void;
}>();

const store = useMapStore();

const surface = useTemplateRef<HTMLElement>('surface');

defineExpose({ surface });

usePointerGestures(surface, gestures, store);

const { backgroundImageUrl, backgroundMode } = useMapBackground();

// The manual map uses the full configured canvas. The tree layout instead sizes the
// canvas to its own content (plus padding) so the SVG viewBox covers every connection
// without leaving a large empty scroll area below a small tree. The container's
// `h-full w-full` keeps it at least viewport-sized.
const contentSize = computed(() => {
    const scale = store.scale.value;
    // Padding leaves room for a node's body/handles past its anchor, so edge nodes
    // aren't clipped by the canvas's overflow-hidden.
    const padding = 240 * scale;
    if (!store.isTreeLayout.value) {
        const maxSize = store.config.value.max_size;
        return { x: maxSize.x * scale + padding, y: maxSize.y * scale + padding };
    }
    let maxX = 0;
    let maxY = 0;
    for (const id of store.systems.keys()) {
        const position = store.renderPosition(id);
        if (!position) continue;
        maxX = Math.max(maxX, position.x * scale);
        maxY = Math.max(maxY, position.y * scale);
    }
    return { x: maxX + padding, y: maxY + padding };
});

const mapContainerStyle = computed(() => {
    const cell = `${store.config.value.grid_size * store.scale.value}px`;
    const baseStyle = {
        backgroundSize: `${cell} ${cell}`,
        minHeight: `${contentSize.value.y}px`,
        minWidth: `${contentSize.value.x}px`,
    };

    // In "grid" mode the image is painted onto the scaled map content, so it
    // spans the whole grid and pans / zooms together with the systems.
    if (backgroundImageUrl.value && backgroundMode.value === 'grid') {
        return {
            ...baseStyle,
            backgroundImage: `linear-gradient(to right, rgba(0, 0, 0, 0.3) 1px, transparent 1px), linear-gradient(to bottom, rgba(0, 0, 0, 0.3) 1px, transparent 1px), url(${backgroundImageUrl.value})`,
            backgroundSize: `${cell} ${cell}, ${cell} ${cell}, cover`,
            backgroundRepeat: 'repeat, repeat, no-repeat',
            backgroundPosition: '0 0, 0 0, center center',
            backgroundAttachment: 'scroll, scroll, scroll',
        };
    }

    return baseStyle;
});

// In "viewport" mode the image lives on the (non-scrolling) scrollable container,
// so it stays fixed to the visible panel regardless of panning or zoom.
const scrollableContainerStyle = computed(() => {
    if (!backgroundImageUrl.value || backgroundMode.value !== 'viewport') {
        return undefined;
    }

    return {
        backgroundImage: `url(${backgroundImageUrl.value})`,
        backgroundSize: 'cover',
        backgroundPosition: 'center center',
        backgroundRepeat: 'no-repeat',
    };
});

const {
    scrollbars_visible,
    has_vertical,
    has_horizontal,
    v_thumb_size,
    v_thumb_offset,
    v_track_height,
    h_thumb_size,
    h_thumb_offset,
    h_track_width,
    scrollbar_size,
    onThumbMousedown,
    onTrackMousedown,
    onScrollAreaEnter,
    onScrollAreaMousemove,
} = useMapScrollbars(
    surface,
    () => contentSize.value,
    store.scale,
);

/** The live marquee box in screen pixels within the canvas. */
const marqueeRect = computed(() => {
    const box = store.marquee.value;
    if (!box) return null;
    const scale = store.scale.value;
    return {
        x: Math.min(box.start.x, box.end.x) * scale,
        y: Math.min(box.start.y, box.end.y) * scale,
        width: Math.abs(box.start.x - box.end.x) * scale,
        height: Math.abs(box.start.y - box.end.y) * scale,
    };
});

function toBasePoint(event: MouseEvent): Vec2 {
    const element = surface.value;
    if (!element) {
        return { x: 0, y: 0 };
    }
    const rect = element.getBoundingClientRect();
    return clientToBase(event.clientX, event.clientY, {
        rectLeft: rect.left,
        rectTop: rect.top,
        scrollLeft: element.scrollLeft,
        scrollTop: element.scrollTop,
        scale: store.scale.value,
    });
}

function handleContextMenu(event: MouseEvent): void {
    // Prevent the default context menu on middle click (the pan button).
    if (event.button === 1) {
        event.preventDefault();
        return;
    }
    emit('surfaceContextMenu', { nodeId: resolveNodeId(event.target), basePoint: toBasePoint(event) });
}
</script>

<template>
    <div
        class="relative h-full w-full overflow-hidden bg-card ring-1 ring-border ring-offset-[-0.5px]"
        @mouseenter="onScrollAreaEnter"
        @mousemove="onScrollAreaMousemove"
    >
        <div
            ref="surface"
            class="relative h-full w-full overflow-hidden bg-neutral-100 dark:bg-neutral-950"
            :class="{ 'cursor-grab': store.isTreeLayout.value }"
            :style="scrollableContainerStyle"
            @contextmenu="handleContextMenu"
        >
            <ContextMenu @update:open="(open) => emit('contextMenuOpenChange', open)">
                <ContextMenuTrigger>
                    <div
                        class="relative grid h-full w-full overflow-hidden"
                        :class="{ 'bg-grid': !store.isTreeLayout.value }"
                        :style="mapContainerStyle"
                        @dragover.prevent
                    >
                        <slot />
                        <svg v-if="marqueeRect" class="pointer-events-none absolute inset-0 h-full w-full" xmlns="http://www.w3.org/2000/svg">
                            <rect
                                :x="marqueeRect.x"
                                :y="marqueeRect.y"
                                :width="marqueeRect.width"
                                :height="marqueeRect.height"
                                class="fill-amber-500/10 stroke-amber-500 stroke-1"
                                :rx="4"
                                :ry="4"
                                stroke-dasharray="2,2"
                            />
                        </svg>
                    </div>
                </ContextMenuTrigger>
                <slot name="context-menu" />
            </ContextMenu>
        </div>
        <MapScrollbar
            v-if="has_vertical"
            orientation="vertical"
            :thumb_size="v_thumb_size"
            :thumb_offset="v_thumb_offset"
            :visible="scrollbars_visible"
            :track_size="v_track_height"
            :scrollbar_size="scrollbar_size"
            @track-mousedown="onTrackMousedown('vertical', $event)"
            @thumb-mousedown="onThumbMousedown('vertical', $event)"
        />
        <MapScrollbar
            v-if="has_horizontal"
            orientation="horizontal"
            :thumb_size="h_thumb_size"
            :thumb_offset="h_thumb_offset"
            :visible="scrollbars_visible"
            :track_size="h_track_width"
            :scrollbar_size="scrollbar_size"
            @track-mousedown="onTrackMousedown('horizontal', $event)"
            @thumb-mousedown="onThumbMousedown('horizontal', $event)"
        />
        <slot name="overlays" />
    </div>
</template>

<style scoped>
.bg-grid {
    background-image: linear-gradient(to right, var(--grid) 1px, transparent 1px), linear-gradient(to bottom, var(--grid) 1px, transparent 1px);
}

html.dark .bg-grid {
    background-image: linear-gradient(to right, var(--grid) 1px, transparent 1px), linear-gradient(to bottom, var(--grid) 1px, transparent 1px);
}

/* Ensure custom background images work properly with grid overlay */
.bg-grid[style*='background-image: url'] {
    background-image: inherit; /* Use the inline style from the computed property */
}
</style>

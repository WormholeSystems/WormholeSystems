<script setup lang="ts">
import MapConnectionContextMenu from '@/components/map/MapConnectionContextMenu.vue';
import MapConnectionDetails from '@/components/map/MapConnectionDetails.vue';
import MapConnections from '@/components/map/MapConnections.vue';
import MapContextMenu from '@/components/map/MapContextMenu.vue';
import MapOptions from '@/components/map/MapOptions.vue';
import MapRallyBadge from '@/components/map/MapRallyBadge.vue';
import MapScrollbar from '@/components/map/MapScrollbar.vue';
import MapSolarsystem from '@/components/map/MapSolarsystem.vue';
import { ContextMenu, ContextMenuTrigger } from '@/components/ui/context-menu';
import { Popover, PopoverAnchor } from '@/components/ui/popover';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import {
    deleteSelectedMapSolarsystems,
    useCreateMap,
    useMapGrid,
    useMapMouse,
    useMapPanning,
    useMapScale,
    useMapScrollbars,
    useMapSolarsystems,
    useMapViewMode,
} from '@/composables/map';
import { useConnectionInteraction } from '@/composables/map/composables/useConnectionInteraction';
import { useMapEvents } from '@/composables/map/composables/useMapEvents';
import { useLayout } from '@/composables/useLayout';
import { useMapBackground } from '@/composables/useMapBackground';
import usePermission from '@/composables/usePermission';
import { useUserEvents } from '@/composables/useUserEvents';
import { TMap } from '@/pages/maps';
import { TMapConfig } from '@/types/map';
import { Position, useMagicKeys, whenever } from '@vueuse/core';
import { computed, ref, useTemplateRef } from 'vue';

const { map, config } = defineProps<{
    map: TMap;
    config: TMapConfig;
}>();

const { viewMode, is_tree_layout } = useMapViewMode();

const container = useTemplateRef('map-container');
const scrollable_container = useTemplateRef('scrollable-container');

const { layout } = useLayout();

useCreateMap(
    () => map,
    () => container.value!,
    () => config,
    layout,
    () => viewMode.value,
);

const { map_solarsystems } = useMapSolarsystems();

const { Delete } = useMagicKeys();

const { grid_size } = useMapGrid();

const { canEdit: can_write } = usePermission();

const {
    selected_connection,
    selected_connection_id,
    handleConnectionContextMenu,
    handleConnectionClick,
    connection_popover_open,
    connection_popover_position,
} = useConnectionInteraction();

const context_menu_type = computed(() => (selected_connection.value ? 'connection' : 'map'));

const { scale } = useMapScale();

const mouse = useMapMouse();

const { backgroundImageUrl, backgroundMode } = useMapBackground();

const { handleMouseDown, handleMouseMove, handleMouseUp, handleMouseLeave, handleContextMenu } = useMapPanning(scrollable_container);

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
} = useMapScrollbars(scrollable_container, () => contentSize.value);

// The manual map uses the full configured canvas. The tree layout instead sizes the
// canvas to its own content (plus padding) so the SVG viewBox covers every connection
// without leaving a large empty scroll area below a small tree. The container's
// `h-full w-full` keeps it at least viewport-sized.
const contentSize = computed(() => {
    // Padding leaves room for a node's body/handles past its anchor, so edge nodes
    // aren't clipped by the canvas's overflow-hidden.
    const padding = 240 * scale.value;
    if (!is_tree_layout.value) {
        return { x: config.max_size.x * scale.value + padding, y: config.max_size.y * scale.value + padding };
    }
    let maxX = 0;
    let maxY = 0;
    for (const solarsystem of map_solarsystems.value) {
        if (!solarsystem.position) continue;
        maxX = Math.max(maxX, solarsystem.position.x);
        maxY = Math.max(maxY, solarsystem.position.y);
    }
    return { x: maxX + padding, y: maxY + padding };
});

const mapContainerStyle = computed(() => {
    const cell = `${grid_size.value * scale.value}px`;
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

useMapEvents(map);
useUserEvents();

const opened_at = ref<Position | null>(null);

whenever(Delete, () => deleteSelectedMapSolarsystems());

function onOpenChange(open: boolean) {
    if (!open) {
        selected_connection_id.value = null;
        return;
    }

    opened_at.value = mouse.value;
}
</script>

<template>
    <div
        class="relative h-full w-full overflow-hidden bg-card ring-1 ring-border ring-offset-[-0.5px]"
        @mouseenter="onScrollAreaEnter"
        @mousemove="onScrollAreaMousemove"
    >
        <div
            ref="scrollable-container"
            class="relative h-full w-full overflow-hidden bg-neutral-100 dark:bg-neutral-950"
            :class="{ 'cursor-grab': is_tree_layout }"
            :style="scrollableContainerStyle"
            @mousedown="handleMouseDown"
            @mousemove="handleMouseMove"
            @mouseup="handleMouseUp"
            @mouseleave="handleMouseLeave"
            @contextmenu="handleContextMenu"
        >
            <ContextMenu @update:open="onOpenChange">
                <ContextMenuTrigger>
                    <div
                        class="relative grid h-full w-full overflow-hidden"
                        :class="{ 'bg-grid': !is_tree_layout }"
                        @dragover.prevent
                        ref="map-container"
                        :style="mapContainerStyle"
                    >
                        <MapConnections @connection-context-menu="handleConnectionContextMenu" @connection-click="handleConnectionClick" />
                        <MapSolarsystem v-for="solarsystem in map_solarsystems" :key="solarsystem.id" :map_solarsystem="solarsystem" />
                    </div>
                </ContextMenuTrigger>
                <MapContextMenu v-if="context_menu_type === 'map' && can_write" :position="opened_at!" />
                <MapConnectionContextMenu
                    v-else-if="context_menu_type === 'connection' && selected_connection && can_write"
                    :map_connection="selected_connection"
                />
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
        <MapRallyBadge />
        <Tooltip v-if="is_tree_layout" :delay-duration="300">
            <TooltipTrigger as-child>
                <span
                    class="pointer-events-auto absolute bottom-3 left-3 z-20 rounded-full bg-amber-500 px-2 py-0.5 text-[10px] font-bold tracking-wider text-white uppercase shadow-sm select-none dark:bg-amber-600"
                >
                    Beta
                </span>
            </TooltipTrigger>
            <TooltipContent>The tree layout is experimental and still a work in progress.</TooltipContent>
        </Tooltip>
        <MapOptions :config />
    </div>
    <Popover v-model:open="connection_popover_open" :key="selected_connection?.id" v-if="selected_connection">
        <PopoverAnchor
            :style="{
                position: 'absolute',
                left: connection_popover_position?.x + 'px',
                top: connection_popover_position?.y + 'px',
            }"
        />
        <MapConnectionDetails :connection="selected_connection" />
    </Popover>
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

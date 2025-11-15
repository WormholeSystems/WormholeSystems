<script setup lang="ts">
import MapConnectionContextMenu from '@/components/map/MapConnectionContextMenu.vue';
import MapConnectionDetails from '@/components/map/MapConnectionDetails.vue';
import MapConnections from '@/components/map/MapConnections.vue';
import MapContextMenu from '@/components/map/MapContextMenu.vue';
import MapOptions from '@/components/map/MapOptions.vue';
import MapSolarsystem from '@/components/map/MapSolarsystem.vue';
import { ContextMenu, ContextMenuTrigger } from '@/components/ui/context-menu';
import { Popover, PopoverAnchor } from '@/components/ui/popover';
import {
    deleteSelectedMapSolarsystems,
    useCreateMap,
    useMapGrid,
    useMapMouse,
    useMapPanning,
    useMapScale,
    useMapSolarsystems,
} from '@/composables/map';
import { useConnectionInteraction } from '@/composables/map/composables/useConnectionInteraction';
import { useMapEvents } from '@/composables/map/composables/useMapEvents';
import useHasWritePermission from '@/composables/useHasWritePermission';
import { useLayout } from '@/composables/useLayout';
import { useMapBackground } from '@/composables/useMapBackground';
import { TMap } from '@/pages/maps';
import { TMapConfig } from '@/types/map';
import { Position, useMagicKeys, whenever } from '@vueuse/core';
import { computed, ref, useTemplateRef } from 'vue';

const { map, config } = defineProps<{
    map: TMap;
    config: TMapConfig;
}>();

const container = useTemplateRef('map-container');
const scrollable_container = useTemplateRef('scrollable-container');

const scroll_locked = ref(false);

const { layout } = useLayout();

useCreateMap(
    () => map,
    () => container.value!,
    () => config,
    layout,
);

const { map_solarsystems } = useMapSolarsystems();

const { Delete } = useMagicKeys();

const { grid_size } = useMapGrid();

const can_write = useHasWritePermission();

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

const { backgroundImageUrl } = useMapBackground();

const { handleMouseDown, handleMouseMove, handleMouseUp, handleMouseLeave, handleContextMenu } = useMapPanning(scrollable_container);

const mapContainerStyle = computed(() => {
    const baseStyle = {
        backgroundSize: `${grid_size.value * scale.value}px ${grid_size.value * scale.value}px`,
        minHeight: `${config.max_size.y * scale.value}px`,
        minWidth: `${config.max_size.x * scale.value}px`,
    };

    if (backgroundImageUrl.value) {
        return {
            ...baseStyle,
            backgroundImage: `linear-gradient(to right, rgba(0, 0, 0, 0.3) 1px, transparent 1px), linear-gradient(to bottom, rgba(0, 0, 0, 0.3) 1px, transparent 1px), url(${backgroundImageUrl.value})`,
            backgroundSize: `${grid_size.value * scale.value}px ${grid_size.value * scale.value}px, ${grid_size.value * scale.value}px ${grid_size.value * scale.value}px, cover`,
            backgroundRepeat: 'repeat, repeat, no-repeat',
            backgroundPosition: '0 0, 0 0, center center',
            backgroundAttachment: 'scroll, scroll, fixed',
        };
    }

    return baseStyle;
});

useMapEvents(map);

const opened_at = ref<Position | null>(null);

whenever(Delete, () => deleteSelectedMapSolarsystems());

function onOpenChange(open: boolean) {
    if (!open) {
        selected_connection_id.value = null;
        return;
    }

    opened_at.value = mouse.value;
}

let timeout: ReturnType<typeof setTimeout> | null = null;

function onScroll(event: WheelEvent) {
    if (event.ctrlKey || event.metaKey || event.altKey || event.shiftKey) {
        return;
    }
    scroll_locked.value = true;
    if (timeout) {
        clearTimeout(timeout);
    }
    timeout = setTimeout(() => {
        scroll_locked.value = false;
        timeout = null;
    }, 500);
}
</script>

<template>
    <div class="relative h-full w-full overflow-hidden rounded-lg border bg-card">
        <div
            ref="scrollable-container"
            :data-scroll-locked="scroll_locked"
            class="relative h-full w-full overflow-scroll bg-neutral-50 data-[scroll-locked=true]:overflow-hidden dark:bg-neutral-900/50"
            @wheel="onScroll"
            @mousedown="handleMouseDown"
            @mousemove="handleMouseMove"
            @mouseup="handleMouseUp"
            @mouseleave="handleMouseLeave"
            @contextmenu="handleContextMenu"
        >
            <ContextMenu @update:open="onOpenChange">
                <ContextMenuTrigger>
                    <div class="bg-grid relative grid h-full w-full" @dragover.prevent ref="map-container" :style="mapContainerStyle">
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

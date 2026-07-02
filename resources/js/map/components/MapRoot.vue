<script setup lang="ts">
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useLayout } from '@/composables/useLayout';
import { useMapUserSettings } from '@/composables/useMapUserSettings';
import { usePath } from '@/composables/usePath';
import usePermission from '@/composables/usePermission';
import { useRallyRoute } from '@/composables/useRallyRoute';
import { useUserEvents } from '@/composables/useUserEvents';
import { deleteSelectedMapSolarsystems } from '@/map/actions/deleteSelectedMapSolarsystems';
import EdgeLayer from '@/map/components/edges/EdgeLayer.vue';
import MapViewport from '@/map/components/MapViewport.vue';
import MapNode from '@/map/components/nodes/MapNode.vue';
import ConnectionPopover from '@/map/components/overlays/ConnectionPopover.vue';
import MapAddConnectionDialog from '@/map/components/overlays/MapAddConnectionDialog.vue';
import MapConnectionContextMenu from '@/map/components/overlays/MapConnectionContextMenu.vue';
import MapContextMenu from '@/map/components/overlays/MapContextMenu.vue';
import MapOptions from '@/map/components/overlays/MapOptions.vue';
import MapRallyBadge from '@/map/components/overlays/MapRallyBadge.vue';
import MapSolarsystemContextMenu from '@/map/components/overlays/MapSolarsystemContextMenu.vue';
import type { Vec2 } from '@/map/core/types';
import type { Gesture } from '@/map/interactions/gestures';
import { createLinkDragGesture } from '@/map/interactions/linkDrag';
import { createMarqueeGesture } from '@/map/interactions/marquee';
import { createNodeDragGesture } from '@/map/interactions/nodeDrag';
import { createPanGesture } from '@/map/interactions/pan';
import { useIsUsingInput } from '@/map/interactions/useIsUsingInput';
import { createMapStore, provideMapStore } from '@/map/store/mapStore';
import { useMapSync } from '@/map/sync/useMapSync';
import { TMap, TMapConnection, TSolarsystem } from '@/pages/maps';
import { TMapConfig } from '@/types/map';
import { useMagicKeys, whenever } from '@vueuse/core';
import { computed, ref, useTemplateRef, watch, watchEffect } from 'vue';

/**
 * The root of the rewritten map canvas — same external contract as the old
 * MapComponent: it takes the resolved map and the canvas config, everything
 * else comes from page props. It owns the store, the gestures, and all the
 * canvas-level overlays (context menus, connection popover, options, dialogs).
 */
const { map, config } = defineProps<{
    map: TMap;
    config: TMapConfig;
}>();

const { path } = usePath();
const { getRallyRouteInfo } = useRallyRoute();

// usePath deep-freezes its ref; unwrap it into the shallow-readonly shape the
// store's RouteDeps expects (the store only ever reads it).
const routePath = computed(() => path.value as readonly TSolarsystem[] | null);

const store = createMapStore({ path: routePath, getRallyRouteInfo });
provideMapStore(store);

watch(
    () => map,
    (value) => store.reconcileMap(value),
    { immediate: true },
);

watch(
    () => config,
    (value) => {
        store.config.value = value;
    },
    { immediate: true },
);

// The zoom level lives in the shared layout prop (cookie-persisted via
// useLayout); the old createLayout mirrored it into mapState.scale the same way.
const { layout } = useLayout();
watchEffect(() => {
    store.scale.value = layout.value?.scale ?? 1;
});

// Mirror the viewer's personal layout override (map_user_settings.layout_override,
// written by MapOptions) into the store so the effective layout derives from it.
const mapUserSettings = useMapUserSettings();
watchEffect(() => {
    store.userLayoutOverride.value = mapUserSettings.value?.layout_override ?? null;
});

useMapSync(store, () => map.id);
useUserEvents();

const { canEdit: canWrite } = usePermission();

const systemIds = computed(() => [...store.systems.keys()]);

// --- Gestures ------------------------------------------------------------

const viewport = useTemplateRef('viewport');
const surface = computed<HTMLElement | null>(() => viewport.value?.surface ?? null);

const { gesture: linkDragGesture, pendingTo } = createLinkDragGesture(store);
const gestures: Gesture[] = [linkDragGesture, createNodeDragGesture(store), createPanGesture(store, surface), createMarqueeGesture(store)];

/** The ghost edge's fixed end: the drag-origin node's rendered anchor. */
const pendingFrom = computed<Vec2 | null>(() => {
    const originId = store.linkDragOriginId.value;
    return originId === null ? null : store.renderPosition(originId);
});

// --- Connection popover / context menus ----------------------------------

const selectedConnectionId = ref<number | null>(null);

const selectedConnection = computed(() => {
    const id = selectedConnectionId.value;
    if (id === null) return null;
    const resolved = store.resolveConnection(id);
    if (!resolved) return null;
    return { ...resolved.connection, source: resolved.source, target: resolved.target };
});

const connectionPopoverOpen = ref(false);
const connectionPopoverPosition = ref<Vec2 | null>(null);

function handleConnectionContextMenu(_event: MouseEvent, connection: TMapConnection): void {
    selectedConnectionId.value = connection.id;
}

function handleConnectionClick(event: MouseEvent, connection: TMapConnection): void {
    connectionPopoverOpen.value = true;
    // Store the click in viewport coordinates.
    connectionPopoverPosition.value = { x: event.clientX, y: event.clientY };
    selectedConnectionId.value = connection.id;
}

// A virtual anchor at the click point. Positioning the popover off a real element fails
// when the map (which scrolls, and whose ancestors may form a containing block) shifts
// it; a virtual reference is read in viewport coordinates, like the context menu.
const connectionPopoverReference = computed(() => {
    const position = connectionPopoverPosition.value;
    if (!position) {
        return undefined;
    }

    return { getBoundingClientRect: () => new DOMRect(position.x, position.y, 0, 0) };
});

/** The node under the last right-click and where it landed, in base units. */
const contextMenuNodeId = ref<number | null>(null);
const contextMenuBasePoint = ref<Vec2 | null>(null);

function handleSurfaceContextMenu(payload: { nodeId: number | null; basePoint: Vec2 }): void {
    contextMenuNodeId.value = payload.nodeId;
    contextMenuBasePoint.value = payload.basePoint;
}

const contextMenuSystem = computed(() => {
    const id = contextMenuNodeId.value;
    return id === null ? null : (store.systems.get(id) ?? null);
});

// Connection wins like in the old root (selected_connection drove the type);
// the node menu replaces the old per-node ContextMenu wrapper, which the new
// MapNode no longer renders.
const contextMenuType = computed<'connection' | 'node' | 'map'>(() => {
    if (selectedConnection.value) return 'connection';
    if (contextMenuSystem.value) return 'node';
    return 'map';
});

function handleContextMenuOpenChange(open: boolean): void {
    if (!open) {
        selectedConnectionId.value = null;
        contextMenuNodeId.value = null;
    }
}

// --- Keyboard ------------------------------------------------------------

const { Delete } = useMagicKeys();
const isUsingInput = useIsUsingInput();

whenever(Delete, () => {
    if (!isUsingInput.value) {
        deleteSelectedMapSolarsystems();
    }
});
</script>

<template>
    <MapViewport
        ref="viewport"
        :gestures="gestures"
        @context-menu-open-change="handleContextMenuOpenChange"
        @surface-context-menu="handleSurfaceContextMenu"
    >
        <EdgeLayer
            :pending-from="pendingFrom"
            :pending-to="pendingTo"
            @connection-click="handleConnectionClick"
            @connection-context-menu="handleConnectionContextMenu"
        />
        <MapNode v-for="id in systemIds" :id="id" :key="id" />
        <template #context-menu>
            <MapContextMenu v-if="contextMenuType === 'map' && canWrite" :position="contextMenuBasePoint" />
            <MapSolarsystemContextMenu v-else-if="contextMenuType === 'node' && contextMenuSystem" :map_solarsystem="contextMenuSystem" />
            <MapConnectionContextMenu
                v-else-if="contextMenuType === 'connection' && selectedConnection && canWrite"
                :map_connection="selectedConnection"
            />
        </template>
        <template #overlays>
            <MapRallyBadge />
            <Tooltip v-if="store.isTreeLayout.value" :delay-duration="300">
                <TooltipTrigger as-child>
                    <span
                        class="pointer-events-auto absolute bottom-3 left-3 z-20 rounded-full bg-amber-500 px-2 py-0.5 text-[10px] font-bold tracking-wider text-white uppercase shadow-sm select-none dark:bg-amber-600"
                    >
                        Beta
                    </span>
                </TooltipTrigger>
                <TooltipContent>Automatic placement is experimental and still a work in progress.</TooltipContent>
            </Tooltip>
            <MapOptions />
        </template>
    </MapViewport>
    <ConnectionPopover
        v-if="selectedConnection"
        :key="selectedConnection.id"
        v-model:open="connectionPopoverOpen"
        :connection="selectedConnection"
        :reference="connectionPopoverReference"
    />
    <MapAddConnectionDialog />
</template>

<style scoped></style>

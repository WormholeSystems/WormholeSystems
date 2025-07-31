<script setup lang="ts">
import MapConnectionContextMenu from '@/components/map/MapConnectionContextMenu.vue';
import MapConnections from '@/components/map/MapConnections.vue';
import MapContextMenu from '@/components/map/MapContextMenu.vue';
import MapSolarsystem from '@/components/map/MapSolarsystem.vue';
import { ContextMenu, ContextMenuTrigger } from '@/components/ui/context-menu';
import { useMapAction, useMapConnections, useMapGrid, useMapSolarsystems, useMap as useNewMap } from '@/composables/map';
import { useHasWritePermission } from '@/composables/useHasPermission';
import { getMapChannelName } from '@/const/channels';
import {
    CharacterStatusUpdatedEvent,
    MapConnectionCreatedEvent,
    MapConnectionDeletedEvent,
    MapConnectionUpdatedEvent,
    MapRouteSolarsystemsUpdatedEvent,
    MapSolarsystemCreatedEvent,
    MapSolarsystemDeletedEvent,
    MapSolarsystemsDeletedEvent,
    MapSolarsystemsUpdatedEvent,
    MapSolarsystemUpdatedEvent,
    MapUpdatedEvent,
    SignatureCreatedEvent,
    SignatureDeletedEvent,
    SignatureUpdatedEvent,
} from '@/const/events';
import { TMapConfig } from '@/types/map';
import { TMap } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { useEcho } from '@laravel/echo-vue';
import { useMagicKeys, whenever } from '@vueuse/core';
import { computed, ref, useTemplateRef } from 'vue';

const { map, config } = defineProps<{
    map: TMap;
    config: TMapConfig;
}>();

const container = useTemplateRef('map-container');

const scroll_locked = ref(false);

useNewMap(
    () => map,
    () => container.value!,
    () => config,
);

const { map_solarsystems } = useMapSolarsystems();

const connections = useMapConnections();

const { Delete } = useMagicKeys();

const { grid_size } = useMapGrid();

const { removeSelectedMapSolarsystems } = useMapAction();

const can_write = useHasWritePermission();

const selected_connection_id = ref<number | null>(null);
const selected_connection = computed(() => connections.value.find((con) => con.id === selected_connection_id.value));
const context_menu_type = computed(() => (selected_connection.value ? 'connection' : 'map'));

whenever(Delete, () => removeSelectedMapSolarsystems());

function onOpenChange(open: boolean) {
    if (!open) {
        selected_connection_id.value = null;
    }
}

useEcho(
    getMapChannelName(map.id),
    [
        MapUpdatedEvent,
        MapSolarsystemUpdatedEvent,
        MapSolarsystemsUpdatedEvent,
        MapConnectionCreatedEvent,
        MapConnectionUpdatedEvent,
        MapConnectionDeletedEvent,
    ],
    () => {
        router.reload({
            only: ['map'],
        });
    },
);

useEcho(getMapChannelName(map.id), [MapSolarsystemCreatedEvent, MapSolarsystemDeletedEvent, MapSolarsystemsDeletedEvent], () => {
    router.reload({
        only: ['map', 'map_killmails'],
    });
});

useEcho(getMapChannelName(map.id), [MapRouteSolarsystemsUpdatedEvent], () => {
    router.reload({
        only: ['map_route_solarsystems'],
    });
});

useEcho(getMapChannelName(map.id), CharacterStatusUpdatedEvent, () => {
    router.reload({
        only: ['map_characters', 'ship_history'],
    });
});

useEcho(getMapChannelName(map.id), [SignatureCreatedEvent, SignatureUpdatedEvent, SignatureDeletedEvent], () => {
    router.reload({
        only: ['selected_map_solarsystem'],
    });
});
useEcho(getMapChannelName(map.id), [MapConnectionCreatedEvent, MapConnectionDeletedEvent, MapConnectionUpdatedEvent], () => {
    router.reload({
        only: ['selected_map_solarsystem', 'map_route_solarsystems'],
    });
});

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
    <div
        :data-scroll-locked="scroll_locked"
        class="relative max-h-[1000px] w-full resize-y overflow-y-scroll rounded-lg border bg-neutral-50 data-[scroll-locked=true]:overflow-hidden dark:bg-neutral-900/50"
        :style="{
            height: config.max_size.y > 1000 ? `${config.max_size.y}px` : 'auto',
        }"
        @wheel="onScroll"
    >
        <ContextMenu @update:open="onOpenChange">
            <ContextMenuTrigger>
                <div
                    class="bg-grid relative grid"
                    @dragover.prevent
                    ref="map-container"
                    :style="{
                        backgroundSize: `${grid_size}px ${grid_size}px`,
                        width: `${config.max_size.x}px`,
                        height: `${config.max_size.y}px`,
                    }"
                >
                    <MapConnections @connection-contextmenu="(e, con) => (selected_connection_id = con.id)" />
                    <MapSolarsystem v-for="solarsystem in map_solarsystems" :key="solarsystem.id" :map_solarsystem="solarsystem" />
                </div>
            </ContextMenuTrigger>
            <MapContextMenu v-if="context_menu_type === 'map' && can_write" />
            <MapConnectionContextMenu
                v-else-if="context_menu_type === 'connection' && selected_connection && can_write"
                :map_connection="selected_connection"
            />
        </ContextMenu>
    </div>
</template>

<style scoped>
.bg-grid {
    background-image: linear-gradient(to right, var(--grid) 1px, transparent 1px), linear-gradient(to bottom, var(--grid) 1px, transparent 1px);
}

html.dark .bg-grid {
    background-image: linear-gradient(to right, var(--grid) 1px, transparent 1px), linear-gradient(to bottom, var(--grid) 1px, transparent 1px);
}
</style>

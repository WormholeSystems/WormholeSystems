<script setup lang="ts">
import MapConnectionContextMenu from '@/components/map/MapConnectionContextMenu.vue';
import MapConnections from '@/components/map/MapConnections.vue';
import MapContextMenu from '@/components/map/MapContextMenu.vue';
import MapSolarsystem from '@/components/map/MapSolarsystem.vue';
import { ContextMenu, ContextMenuTrigger } from '@/components/ui/context-menu';
import { useMap as useNewMap, useMapAction, useMapConnections, useMapGrid, useMapSolarsystems } from '@/composables/map';
import { getMapChannelName } from '@/const/channels';
import {
    MapConnectionCreatedEvent,
    MapConnectionDeletedEvent,
    MapConnectionUpdatedEvent,
    MapSolarsystemCreatedEvent,
    MapSolarsystemDeletedEvent,
    MapSolarsystemsDeletedEvent,
    MapSolarsystemsUpdatedEvent,
    MapSolarsystemUpdatedEvent,
    MapUpdatedEvent,
} from '@/const/events';
import { TMapConfig } from '@/types/map';
import { TMap } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { useEchoPublic } from '@laravel/echo-vue';
import { useMagicKeys, whenever } from '@vueuse/core';
import { computed, ref, useTemplateRef } from 'vue';

const { map, config } = defineProps<{
    map: TMap;
    config: TMapConfig;
}>();

const container = useTemplateRef('map-container');

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

const selected_connection_id = ref<number | null>(null);
const selected_connection = computed(() => connections.value.find((con) => con.id === selected_connection_id.value));
const context_menu_type = computed(() => (selected_connection.value ? 'connection' : 'map'));

whenever(Delete, () => removeSelectedMapSolarsystems());

function onOpenChange(open: boolean) {
    if (!open) {
        selected_connection_id.value = null;
    }
}

useEchoPublic(
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

useEchoPublic(getMapChannelName(map.id), [MapSolarsystemCreatedEvent, MapSolarsystemDeletedEvent, MapSolarsystemsDeletedEvent], () => {
    router.reload({
        only: ['map', 'map_killmails'],
    });
});
</script>

<template>
    <div class="relative h-240 w-full overflow-scroll border-b bg-neutral-900/50">
        <ContextMenu @update:open="onOpenChange">
            <ContextMenuTrigger>
                <div
                    class="bg-grid relative grid h-400 w-1000"
                    @dragover.prevent
                    ref="map-container"
                    :style="{
                        backgroundSize: `${grid_size}px ${grid_size}px`,
                    }"
                >
                    <MapConnections @connection-contextmenu="(e, con) => (selected_connection_id = con.id)" />
                    <MapSolarsystem v-for="solarsystem in map_solarsystems" :key="solarsystem.id" :map_solarsystem="solarsystem" />
                </div>
            </ContextMenuTrigger>
            <MapContextMenu v-if="context_menu_type === 'map'" />
            <MapConnectionContextMenu v-else-if="context_menu_type === 'connection' && selected_connection" :map_connection="selected_connection" />
        </ContextMenu>
    </div>
</template>

<style scoped>
.bg-grid {
    background-image:
        linear-gradient(to right, var(--color-neutral-900) 1px, transparent 1px),
        linear-gradient(to bottom, var(--color-neutral-900) 1px, transparent 1px);
}
</style>

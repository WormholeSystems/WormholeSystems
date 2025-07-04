<script setup lang="ts">
import MapConnection from '@/components/map/MapConnection.vue';
import { useMapConnections, useMapMouse, useSelection } from '@/composables/map';
import { useNewConnection } from '@/composables/useNewConnection';
import { TMapConnection } from '@/types/models';
import { useEventListener } from '@vueuse/core';
import { ref, useTemplateRef } from 'vue';

const container = useTemplateRef('container');

const connections = useMapConnections();

const { origin } = useNewConnection();

const mouse = useMapMouse();

const { selection, setSelectionStart, setSelectionEnd } = useSelection();

const dragging = ref(false);

function handleDragStart(event: PointerEvent) {
    if (event.button !== 0) return;
    dragging.value = true;
    setSelectionStart(event.offsetX, event.offsetY);
}

function handleDragMove() {
    if (!dragging.value) return;

    setSelectionEnd(mouse.value.x, mouse.value.y);
}

function handleDragEnd(event: PointerEvent) {
    if (!dragging.value) return;
    dragging.value = false;
    const bounds = container.value?.getBoundingClientRect();
    if (!bounds) return;

    const x = event.clientX - bounds.left;
    const y = event.clientY - bounds.top;

    setSelectionEnd(x, y);
}

useEventListener('pointerup', handleDragEnd);

function getConnectionExtra(connection: TMapConnection): string {
    if (!connection.ship_size) return '';
    switch (connection.ship_size) {
        case 'frigate':
            return 'SM';
        case 'medium':
            return 'MD';
        default:
            return '';
    }
}
</script>

<template>
    <div class="" ref="container" @pointerdown="handleDragStart" @pointermove="handleDragMove">
        <svg
            class="h-full w-full text-neutral-700"
            xmlns="http://www.w3.org/2000/svg"
            :viewBox="`0 0 ${container?.clientWidth ?? 0} ${container?.clientHeight ?? 0}`"
        >
            <MapConnection
                v-for="map_connection in connections"
                :key="map_connection.id"
                :from="map_connection.source.position!"
                :to="map_connection.target.position!"
                :extra="getConnectionExtra(map_connection)"
                :mass_status="map_connection.mass_status"
                :is_eol="map_connection.is_eol"
                @contextmenu="$emit('connection-contextmenu', $event, map_connection)"
            />
            <MapConnection v-if="origin" :from="origin.position!" :to="mouse" />
            <rect
                v-if="dragging && selection?.start"
                :x="Math.min(selection.start.x, mouse.x)"
                :y="Math.min(selection.start.y, mouse.y)"
                :width="Math.abs(selection.start.x - mouse.x)"
                :height="Math.abs(selection.start.y - mouse.y)"
                class="fill-amber-500/10 stroke-amber-500 stroke-1"
                :rx="4"
                :ry="4"
                stroke-dasharray="2,2"
            />
        </svg>
    </div>
</template>

<style scoped></style>

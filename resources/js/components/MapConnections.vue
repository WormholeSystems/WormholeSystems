<script setup lang="ts">
import MapConnection from '@/components/MapConnection.vue';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Popover, PopoverAnchor, PopoverContent } from '@/components/ui/popover';
import { TConnectionWithSourceAndTarget, useMapConnections, useMapMouse, useSelection } from '@/composables/map';
import { useNewConnection } from '@/composables/useNewConnection';
import { router } from '@inertiajs/vue3';
import { useEventListener } from '@vueuse/core';
import { computed, ref, useTemplateRef } from 'vue';

const container = useTemplateRef('container');

const connections = useMapConnections();

const { origin } = useNewConnection();

const mouse = useMapMouse();

const selected_connection = ref<TConnectionWithSourceAndTarget | null>(null);
const center = computed(() => selection.value && getConnectionCenter(selected_connection.value!));

const { selection, setSelectionStart, setSelectionEnd } = useSelection();

const dragging = ref(false);

function handleDragStart(event: PointerEvent) {
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

function handleConnectionClick(connection: TConnectionWithSourceAndTarget) {
    selected_connection.value = connection;
}

function getConnectionCenter(connection: TConnectionWithSourceAndTarget): { x: number; y: number } {
    const from = connection.source.position!;
    const to = connection.target.position!;

    return {
        x: (from.x + to.x) / 2,
        y: (from.y + to.y) / 2,
    };
}

function handleClosePopover() {
    selected_connection.value = null;
}

function handleConnectionDelete() {
    if (!selected_connection.value) return;
    router.delete(route('map-connections.destroy', selected_connection.value.id), {
        onSuccess: () => {
            selected_connection.value = null;
        },
    });
}
</script>

<template>
    <Popover :open="selected_connection !== null" @update:open="handleClosePopover">
        <PopoverAnchor
            v-if="selected_connection"
            :style="{
                left: `${center?.x}px`,
                top: `${center?.y}px`,
            }"
            class="absolute"
        />
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
                    @click="() => handleConnectionClick(map_connection)"
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
        <PopoverContent as-child>
            <Card class="text-sm">
                <CardHeader>
                    <CardTitle> Connection details</CardTitle>
                </CardHeader>
                <CardContent>
                    From: <span class="font-semibold">{{ selected_connection?.source.name }}</span
                    ><br />
                    To: <span class="font-semibold">{{ selected_connection?.target.name }}</span
                    ><br />
                </CardContent>
                <CardFooter>
                    <Button variant="destructive" size="sm" @click="handleConnectionDelete"> Delete</Button>
                </CardFooter>
            </Card>
        </PopoverContent>
    </Popover>
</template>

<style scoped></style>

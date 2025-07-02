<script setup lang="ts">
import MapConnection from '@/components/MapConnection.vue';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Popover, PopoverAnchor, PopoverContent } from '@/components/ui/popover';
import { useMapMouse } from '@/composables/useMapMouse';
import { useNewConnection } from '@/composables/useNewConnection';
import { useSelection } from '@/composables/useSelection';
import { useSystemDrag } from '@/composables/useSystemDrag';
import { TMapConnection, TMapSolarSystem } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { computed, ref, useTemplateRef } from 'vue';

type TConnectionWithSourceAndTarget = TMapConnection & {
    source: TMapSolarSystem;
    target: TMapSolarSystem;
};

const { map_connections, map_solarsystems } = defineProps<{
    map_connections: TMapConnection[];
    map_solarsystems: TMapSolarSystem[];
}>();

const container = useTemplateRef('container');

const { origin } = useNewConnection();

const mouse = useMapMouse();

const { dragged_solarsystem, position } = useSystemDrag();

const { start, handleDragStart, handleDragMove, handleDragEnd, area } = useSelection(() => container.value);

const selected_connection = ref<TConnectionWithSourceAndTarget | null>(null);

const connections = computed(() => map_connections.map(getConnectionWithSourceAndTarget).map(updateConnectionForDraggedSystem));

function getConnectionWithSourceAndTarget(connection: TMapConnection): TConnectionWithSourceAndTarget {
    const source = map_solarsystems.find((s) => s.id === connection.from_map_solarsystem_id)!;
    const target = map_solarsystems.find((s) => s.id === connection.to_map_solarsystem_id)!;

    return {
        ...connection,
        source,
        target,
    };
}

function updateConnectionForDraggedSystem(connection: TConnectionWithSourceAndTarget): TConnectionWithSourceAndTarget {
    if (!dragged_solarsystem.value) return connection;
    if (connection.from_map_solarsystem_id !== dragged_solarsystem.value.id && connection.to_map_solarsystem_id !== dragged_solarsystem.value.id)
        return connection;

    const other_system =
        connection.from_map_solarsystem_id === dragged_solarsystem.value.id ? connection.to_map_solarsystem_id : connection.from_map_solarsystem_id;

    const other_system_position = map_solarsystems.find((s) => s.id === other_system)!.position!;

    if (!other_system_position) return connection;

    return {
        ...connection,
        from_map_solarsystem_id: dragged_solarsystem.value.id,
        to_map_solarsystem_id: other_system,
        source: {
            ...dragged_solarsystem.value,
            position: position.value,
        },
        target: {
            ...map_solarsystems.find((s) => s.id === other_system)!,
            position: other_system_position,
        },
    };
}

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
                left: `${getConnectionCenter(selected_connection!).x}px`,
                top: `${getConnectionCenter(selected_connection!).y}px`,
            }"
            class="absolute"
        />
        <div class="" ref="container" @dragstart.prevent="handleDragStart" draggable="true">
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
                    v-if="start"
                    :x="Math.min(start.x, mouse.x)"
                    :y="Math.min(start.y, mouse.y)"
                    :width="Math.abs(start.x - mouse.x)"
                    :height="Math.abs(start.y - mouse.y)"
                    class="fill-transparent stroke-amber-500 stroke-1"
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

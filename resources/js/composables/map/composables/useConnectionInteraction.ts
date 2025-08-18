import { useMapConnections, useMapMouse } from '@/composables/map';
import { TMapConnection } from '@/types/models';
import { Position } from '@vueuse/core';
import { computed, ref, watchEffect } from 'vue';

export function useConnectionInteraction() {
    const connection_popover_open = ref(false);
    const connection_popover_position = ref<Position | null>(null);

    const connections = useMapConnections();

    const selected_connection_id = ref<number | null>(null);
    const selected_connection = computed(() => connections.value.find((con) => con.id === selected_connection_id.value));

    const mouse = useMapMouse();

    function handleConnectionContextMenu(_: MouseEvent, connection: TMapConnection) {
        selected_connection_id.value = connection.id;
    }

    function handleConnectionClick(_: MouseEvent, connection: TMapConnection) {
        connection_popover_open.value = true;
        connection_popover_position.value = {
            x: mouse.value.x,
            y: mouse.value.y,
        };
        selected_connection_id.value = connection.id;
    }

    watchEffect(() => console.log(selected_connection.value));

    return {
        connection_popover_open,
        connection_popover_position,
        selected_connection_id,
        selected_connection,
        handleConnectionContextMenu,
        handleConnectionClick,
    };
}

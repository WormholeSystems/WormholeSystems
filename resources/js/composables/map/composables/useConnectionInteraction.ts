import { useMapConnections } from '@/composables/map';
import { TMapConnection } from '@/pages/maps';
import { Position } from '@vueuse/core';
import { computed, ref } from 'vue';

export function useConnectionInteraction() {
    const connection_popover_open = ref(false);
    const connection_popover_position = ref<Position | null>(null);

    const connections = useMapConnections();

    const selected_connection_id = ref<number | null>(null);
    const selected_connection = computed(() => connections.value.find((con) => con.id === selected_connection_id.value));

    function handleConnectionContextMenu(_: MouseEvent, connection: TMapConnection) {
        selected_connection_id.value = connection.id;
    }

    function handleConnectionClick(event: MouseEvent, connection: TMapConnection) {
        connection_popover_open.value = true;
        // Store the click in viewport coordinates.
        connection_popover_position.value = {
            x: event.clientX,
            y: event.clientY,
        };
        selected_connection_id.value = connection.id;
    }

    // A virtual anchor at the click point. Positioning the popover off a real element fails
    // when the map (which scrolls, and whose ancestors may form a containing block) shifts
    // it; a virtual reference is read in viewport coordinates, like the context menu.
    const connection_popover_reference = computed(() => {
        const position = connection_popover_position.value;
        if (!position) {
            return undefined;
        }

        return { getBoundingClientRect: () => new DOMRect(position.x, position.y, 0, 0) };
    });

    return {
        connection_popover_open,
        connection_popover_position,
        connection_popover_reference,
        selected_connection_id,
        selected_connection,
        handleConnectionContextMenu,
        handleConnectionClick,
    };
}

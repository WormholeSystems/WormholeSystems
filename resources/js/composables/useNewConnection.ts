import { TMapSolarSystem } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { MaybeRefOrGetter, useEventListener } from '@vueuse/core';
import { computed, reactive, toValue } from 'vue';

const store = reactive<{
    origin: TMapSolarSystem | null;
}>({
    origin: null,
});

export function useNewConnection(
    handle?: MaybeRefOrGetter<HTMLElement>,
    map_solarsystem?: MaybeRefOrGetter<TMapSolarSystem>,
    container?: MaybeRefOrGetter<HTMLElement>,
) {
    const origin = computed(() => store.origin);

    /**
     * Handles the drag start event for creating a new connection.
     */
    function handleDragStart() {
        const value = toValue(map_solarsystem)!;

        store.origin = value;
    }

    /**
     * Handles the drag end event for creating a new connection.
     */
    function handleDragEnd() {
        store.origin = null;
    }

    function handleConnectionCreation() {
        const system = toValue(map_solarsystem);
        if (!system || !store.origin) return;
        router.post(
            route('map-connections.store'),
            {
                from_map_solarsystem_id: store.origin.id,
                to_map_solarsystem_id: system.id,
            },
            {
                preserveState: true,
                preserveScroll: true,
                onSuccess: () => {
                    store.origin = null; // Reset origin after successful connection creation
                },
            },
        );
    }

    useEventListener(handle, 'pointerdown', handleDragStart);

    useEventListener('pointerup', handleDragEnd);

    useEventListener(container, 'pointerup', handleConnectionCreation);

    return {
        origin,
    };
}

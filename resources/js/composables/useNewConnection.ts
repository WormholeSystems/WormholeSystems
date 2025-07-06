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
        store.origin = toValue(map_solarsystem)!;
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
                ship_size: getMaximumShipSizeForConnection(store.origin, system),
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

    function getMaximumShipSizeForConnection(from: TMapSolarSystem, to: TMapSolarSystem): string | undefined {
        const classes = [from.class, to.class].filter((c) => c !== null && c !== undefined);
        if (classes.some((c) => c >= 13 && c <= 18)) return 'frigate';
        if (classes.includes(1)) return 'medium';

        // Check if Turnur connects to JSpace
        const names = [from.name, to.name];
        if (names.includes('Turnur') && classes.length) return 'medium';

        // Check if Thera connects to Highsec
        const highsec = [from.solarsystem?.security, to.solarsystem?.security].filter((s) => s && s >= 0.5);
        if (names.includes('Thera') && highsec.length) return 'medium';

        return undefined;
    }

    useEventListener(handle, 'pointerdown', handleDragStart);

    useEventListener('pointerup', handleDragEnd);

    useEventListener(container, 'pointerup', handleConnectionCreation);
    useEventListener(handle, 'drag', (e) => e.preventDefault());

    return {
        origin,
    };
}

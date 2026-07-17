import { useOnClient } from '@/composables/useOnClient';
import { useStaticSolarsystems } from '@/composables/useStaticSolarsystems';
import { getMapChannelName } from '@/const/channels';
import type { MapStore } from '@/map/store/mapStore';
import { TShowMapProps } from '@/pages/maps';
import type { AppPageProps } from '@/types';
import { usePage } from '@inertiajs/vue3';
import { echo, useEcho } from '@laravel/echo-vue';
import { computed, MaybeRefOrGetter, onMounted, onUnmounted, toValue } from 'vue';
import { ALL_MAP_PROPS, mapEventHandlers, type SyncEffect } from './handlers';
import { type ConnectionStateChange, createReconnectListener } from './reconnect';
import { createReloadCoalescer } from './reloadCoalescer';

type PusherConnection = {
    bind(event: 'state_change', callback: (states: ConnectionStateChange) => void): void;
    unbind(event: 'state_change', callback: (states: ConnectionStateChange) => void): void;
};

/**
 * Owns all realtime traffic for the map: one subscription per broadcast event,
 * each mapped through the handler table onto store operations plus coalesced
 * per-user prop reloads. Payload events are broadcast to everyone (including
 * the originator), so optimistic local writes are simply confirmed by the echo.
 *
 * Reconnects (sleep, network blip, Reverb restart) trigger one coalesced
 * full-prop reload that flows through reconcileMap — no missed-event tracking.
 */
export function useMapSync(store: MapStore, mapId: MaybeRefOrGetter<number>): void {
    const page = usePage<AppPageProps<TShowMapProps>>();
    const { resolveSolarsystem } = useStaticSolarsystems();
    const coalescer = createReloadCoalescer();

    const channelName = computed(() => getMapChannelName(toValue(mapId)));

    function applyEffect(effect: SyncEffect): void {
        const props = [...(effect.reload ?? [])];
        if (effect.reloadIfSelected?.length) {
            const selectedId = page.props.selected_map_solarsystem?.id;
            if (selectedId !== undefined && effect.reloadIfSelected.includes(selectedId)) {
                props.push('selected_map_solarsystem');
            }
        }
        coalescer.schedule(props);
    }

    for (const [eventName, handler] of Object.entries(mapEventHandlers)) {
        useOnClient(() =>
            useEcho(channelName.value, [eventName], (payload: unknown) => {
                applyEffect(handler(store, payload, resolveSolarsystem));
            }),
        );
    }

    const handleConnectionStateChange = createReconnectListener(() => coalescer.schedule(ALL_MAP_PROPS));

    onMounted(() => {
        connection()?.bind('state_change', handleConnectionStateChange);
    });

    onUnmounted(() => {
        connection()?.unbind('state_change', handleConnectionStateChange);
    });
}

function connection(): PusherConnection | null {
    try {
        const connector = echo().connector as { pusher?: { connection?: PusherConnection } };
        return connector.pusher?.connection ?? null;
    } catch {
        return null;
    }
}

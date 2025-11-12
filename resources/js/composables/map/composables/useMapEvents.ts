import { useOnClient } from '@/composables/useOnClient';
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
import { TMap } from '@/pages/maps';
import { router } from '@inertiajs/vue3';
import { useEcho } from '@laravel/echo-vue';
import { computed, MaybeRefOrGetter, toRef } from 'vue';

export function useMapEvents(map: MaybeRefOrGetter<TMap>) {
    const map_ref = toRef(map);

    const map_channel_name = computed(() => getMapChannelName(map_ref.value.id));

    useOnClient(() =>
        useEcho(
            map_channel_name.value,
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
        ),
    );

    useOnClient(() =>
        useEcho(map_channel_name.value, [MapSolarsystemCreatedEvent, MapSolarsystemDeletedEvent, MapSolarsystemsDeletedEvent], () => {
            router.reload({
                only: ['map', 'map_killmails', 'selected_map_solarsystem'],
            });
        }),
    );

    useOnClient(() =>
        useEcho(getMapChannelName(map_ref.value.id), [MapRouteSolarsystemsUpdatedEvent], () => {
            router.reload({
                only: ['map_navigation'],
            });
        }),
    );

    useOnClient(() =>
        useEcho(map_channel_name.value, CharacterStatusUpdatedEvent, () => {
            router.reload({
                only: ['map_characters', 'ship_history'],
            });
        }),
    );

    useOnClient(() =>
        useEcho(map_channel_name.value, [SignatureCreatedEvent, SignatureUpdatedEvent, SignatureDeletedEvent], () => {
            router.reload({
                only: ['selected_map_solarsystem'],
            });
        }),
    );
    useOnClient(() =>
        useEcho(map_channel_name.value, [MapConnectionCreatedEvent, MapConnectionDeletedEvent, MapConnectionUpdatedEvent], () => {
            router.reload({
                only: ['selected_map_solarsystem', 'map_navigation', 'eve_scout_connections'],
            });
        }),
    );
}

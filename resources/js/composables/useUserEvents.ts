import { useOnClient } from '@/composables/useOnClient';
import useUser from '@/composables/useUser';
import { getUserChannelName } from '@/const/channels';
import { MapUserRouteSolarsystemsUpdatedEvent, UserCharacterStatusUpdatedEvent } from '@/const/events';
import { router } from '@inertiajs/vue3';
import { useEcho } from '@laravel/echo-vue';

/**
 * Subscribe to the authenticated user's private channel and refresh the shared
 * `auth` prop whenever one of their characters' status changes. This keeps the
 * character list (e.g. online state in context menus) current even for
 * characters that are not on the map being viewed.
 */
export function useUserEvents() {
    useUserChannel(UserCharacterStatusUpdatedEvent, () => {
        router.reload({ only: ['auth'] });
    });
}

/** A user's personal watchlist changed on a map, from another of their tabs. */
export type TMapUserRouteSolarsystemsUpdatedEvent = {
    user_id: number;
    map_id: number;
};

/**
 * Run the callback whenever the user's personal watchlist changes, on any map.
 */
export function useMapUserRouteSolarsystemsEvents(callback: (event: TMapUserRouteSolarsystemsUpdatedEvent) => void) {
    useUserChannel(MapUserRouteSolarsystemsUpdatedEvent, callback);
}

function useUserChannel<T>(event: string, callback: (payload: T) => void) {
    const user = useUser();

    useOnClient(() => {
        const userId = user.value?.id;
        if (!userId) {
            return;
        }

        useEcho<T>(getUserChannelName(userId), event, callback);
    });
}

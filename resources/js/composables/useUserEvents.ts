import { useOnClient } from '@/composables/useOnClient';
import useUser from '@/composables/useUser';
import { getUserChannelName } from '@/const/channels';
import { UserCharacterStatusUpdatedEvent } from '@/const/events';
import { router } from '@inertiajs/vue3';
import { useEcho } from '@laravel/echo-vue';

/**
 * Subscribe to the authenticated user's private channel and refresh the shared
 * `auth` prop whenever one of their characters' status changes. This keeps the
 * character list (e.g. online state in context menus) current even for
 * characters that are not on the map being viewed.
 */
export function useUserEvents() {
    const user = useUser();

    useOnClient(() => {
        const userId = user.value?.id;
        if (!userId) {
            return;
        }

        useEcho(getUserChannelName(userId), UserCharacterStatusUpdatedEvent, () => {
            router.reload({ only: ['auth'] });
        });
    });
}

import MapUserSettings from '@/routes/map-user-settings';
import { TMapUserSetting } from '@/types/models';
import { router } from '@inertiajs/vue3';

export function updateMapUserSettings(map_user_settings: TMapUserSetting, data: Partial<TMapUserSetting>, only?: string[]) {
    return router.put(MapUserSettings.update(map_user_settings.id).url, data, {
        preserveScroll: true,
        preserveState: true,
        only: only ?? ['map_user_settings', 'map_navigation'],
    });
}

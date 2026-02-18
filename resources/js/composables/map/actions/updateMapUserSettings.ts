import MapUserSettingController from '@/actions/App/Http/Controllers/MapUserSettingController';
import { TMapUserSetting } from '@/types/models';
import { router } from '@inertiajs/vue3';

export function updateMapUserSettings(mapSlug: string, data: Partial<TMapUserSetting>, only?: string[]) {
    return router.put(MapUserSettingController.update(mapSlug).url, data, {
        preserveScroll: true,
        preserveState: true,
        only: only ?? ['map_user_settings', 'map_navigation'],
    });
}

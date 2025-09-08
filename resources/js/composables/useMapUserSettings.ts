import { AppPageProps } from '@/types';
import { TMapUserSetting } from '@/types/models';
import { usePage } from '@inertiajs/vue3';
import { computed } from 'vue';

export function useMapUserSettings() {
    const page = usePage<
        AppPageProps<{
            map_user_settings: TMapUserSetting;
        }>
    >();

    return computed(() => {
        if (!page.props.map_user_settings) {
            throw new Error('Could not find map user settings in page props');
        }

        return page.props.map_user_settings;
    });
}

import { useShowMap } from '@/composables/useShowMap';
import { router } from '@inertiajs/vue3';
import { computed } from 'vue';

export function useOriginMapSolarsystem() {
    const page = useShowMap();

    const origin_map_solarsystem = computed(() => page.props.tracking_origin || null);

    function load() {
        return page.props.tracking_origin || null;
    }

    function update(map_solarsystem_id: number | null, callback?: () => void) {
        router.reload({
            data: {
                origin_map_solarsystem_id: map_solarsystem_id,
            },
            only: ['tracking_origin'],
            onSuccess: () => {
                if (callback) callback();
            },
        });
    }

    return {
        origin_map_solarsystem,
        load,
        update,
    };
}

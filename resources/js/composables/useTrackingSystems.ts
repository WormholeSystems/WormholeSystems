import { useShowMap } from '@/composables/useShowMap';
import { router } from '@inertiajs/vue3';
import { computed } from 'vue';

export function useTrackingSystems() {
    const page = useShowMap();

    const origin_map_solarsystem = computed(() => page.props.tracking_origin || null);
    const target_solarsystem = computed(() => page.props.tracking_target || null);

    function load() {
        return {
            origin: page.props.tracking_origin || null,
            target: page.props.tracking_target || null,
        };
    }

    function update(map_solarsystem_id: number | null, target_solarsystem_id: number | null = null, callback?: () => void) {
        router.reload({
            data: {
                origin_map_solarsystem_id: map_solarsystem_id,
                target_solarsystem_id: target_solarsystem_id,
            },
            only: ['tracking_origin', 'tracking_target'],
            onSuccess: () => {
                if (callback) callback();
            },
        });
    }

    return {
        origin_map_solarsystem,
        target_solarsystem,
        load,
        update,
    };
}

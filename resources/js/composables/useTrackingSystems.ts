import { useShowMap } from '@/composables/useShowMap';
import { useStaticSolarsystems } from '@/composables/useStaticSolarsystems';
import { router } from '@inertiajs/vue3';
import { computed } from 'vue';

export function useTrackingSystems() {
    const page = useShowMap();
    const { resolveSolarsystem } = useStaticSolarsystems();

    const origin_map_solarsystem = computed(() => {
        const origin = page.props.tracking_origin;
        if (!origin) {
            return null;
        }

        return {
            ...origin,
            solarsystem: resolveSolarsystem(origin.solarsystem_id),
        };
    });
    const target_solarsystem = computed(() => {
        const target = page.props.tracking_target;
        if (!target) {
            return null;
        }

        return resolveSolarsystem(target.solarsystem_id);
    });

    function load() {
        return {
            origin: origin_map_solarsystem.value,
            target: target_solarsystem.value,
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

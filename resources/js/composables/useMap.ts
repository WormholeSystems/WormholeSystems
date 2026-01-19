import { useShowMap } from '@/composables/useShowMap';
import { useStaticSolarsystems } from '@/composables/useStaticSolarsystems';
import type { TMap } from '@/pages/maps';
import { computed } from 'vue';

export function useMap() {
    const page = useShowMap();
    const { resolveSolarsystem } = useStaticSolarsystems();

    return computed<TMap>(() => {
        const map = page.props.map;
        const map_solarsystems = map.map_solarsystems?.map((mapSolarsystem) => ({
            ...mapSolarsystem,
            solarsystem: resolveSolarsystem(mapSolarsystem.solarsystem_id),
        }));

        return {
            ...map,
            map_solarsystems: map_solarsystems ?? map.map_solarsystems,
        };
    });
}

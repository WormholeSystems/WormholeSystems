import { useShowMap } from '@/composables/useShowMap';
import { useStaticSolarsystems } from '@/composables/useStaticSolarsystems';
import type { TResolvedSelectedMapSolarsystem } from '@/pages/maps';
import type { ComputedRef } from 'vue';
import { computed } from 'vue';

export function useSelectedMapSolarsystem(): ComputedRef<TResolvedSelectedMapSolarsystem | null> {
    const page = useShowMap();
    const { resolveSolarsystem } = useStaticSolarsystems();

    return computed(() => {
        const selected = page.props.selected_map_solarsystem;
        if (!selected) {
            return null;
        }

        return {
            ...selected,
            solarsystem: resolveSolarsystem(selected.solarsystem_id),
        };
    });
}

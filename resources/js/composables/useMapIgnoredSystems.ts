import MapIgnoredSolarsystems from '@/routes/map-ignored-solarsystems';
import { AppPageProps } from '@/types';
import { VisitHelperOptions } from '@inertiajs/core';
import { router, usePage } from '@inertiajs/vue3';
import { computed } from 'vue';

/**
 * Map-wide list of solar systems that should never be auto-mapped while tracking.
 * Distinct from the session-based route-avoidance `ignored_systems` list.
 */
export function useMapIgnoredSystems() {
    const page = usePage<AppPageProps<{ map: { id: number; slug: string }; map_ignored_systems?: number[] }>>();

    const map_ignored_systems = computed(() => page.props.map_ignored_systems ?? []);

    function isIgnored(solarsystem_id: number | null | undefined): boolean {
        if (!solarsystem_id) return false;
        return map_ignored_systems.value.includes(solarsystem_id);
    }

    function ignoreSolarsystem(solarsystem_id: number, options: VisitHelperOptions = {}): void {
        router.post(
            MapIgnoredSolarsystems.store().url,
            { map_id: page.props.map.id, solarsystem_id },
            {
                preserveScroll: true,
                preserveState: true,
                only: ['map', 'map_ignored_systems'],
                ...options,
            },
        );
    }

    function unignoreSolarsystem(solarsystem_id: number, options: VisitHelperOptions = {}): void {
        router.delete(MapIgnoredSolarsystems.destroy({ map: page.props.map.slug, solarsystem_id }).url, {
            preserveScroll: true,
            preserveState: true,
            only: ['map', 'map_ignored_systems'],
            ...options,
        });
    }

    function clearIgnoreList(options: VisitHelperOptions = {}): void {
        router.delete(MapIgnoredSolarsystems.destroyAll(page.props.map.slug).url, {
            preserveScroll: true,
            preserveState: true,
            only: ['map', 'map_ignored_systems'],
            ...options,
        });
    }

    return {
        map_ignored_systems,
        isIgnored,
        ignoreSolarsystem,
        unignoreSolarsystem,
        clearIgnoreList,
    };
}

import IgnoreSystems from '@/routes/ignore-systems';
import { VisitHelperOptions } from '@inertiajs/core';
import { router, usePage } from '@inertiajs/vue3';
import { computed } from 'vue';

export function useIgnoreList() {
    const page = usePage();

    // Get ignored systems reactively from page props
    const ignored_systems = computed(() => (page.props.ignored_systems as number[]) || []);

    function ignoreSolarsystem(solarsystem_id: number, options: VisitHelperOptions = {}): void {
        router.post(
            IgnoreSystems.store().url,
            { solarsystem_id: solarsystem_id },
            {
                ...options,
                preserveScroll: true,
                preserveState: true,
                only: ['map_characters', 'map_navigation', 'ignored_systems', 'shortest_path'],
            },
        );
    }

    function clearIgnoreList(options: VisitHelperOptions = {}): void {
        router.delete(IgnoreSystems.destroyAll().url, {
            ...options,
            preserveScroll: true,
            preserveState: true,
            only: ['map_characters', 'map_navigation', 'ignored_systems', 'shortest_path'],
        });
    }

    return {
        ignored_systems,
        ignoreSolarsystem,
        clearIgnoreList,
    };
}

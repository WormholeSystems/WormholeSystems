import IgnoreSystems from '@/routes/ignore-systems';
import { router, usePage } from '@inertiajs/vue3';
import { computed } from 'vue';

export function useIgnoreList() {
    const page = usePage();

    // Get ignored systems reactively from page props
    const ignoredSystems = computed(() => (page.props.ignored_systems as number[]) || []);
    const isIgnored = (systemId: number): boolean => ignoredSystems.value.includes(systemId);

    const ignoreSystem = (systemId: number): void => {
        router.post(
            IgnoreSystems.store().url,
            { solarsystem_id: systemId },
            {
                preserveScroll: true,
                preserveState: true,
                only: ['map_characters', 'map_route_solarsystems', 'ignored_systems'],
            },
        );
    };

    const clearIgnoreList = (): void => {
        router.delete(IgnoreSystems.destroyAll().url, {
            preserveScroll: true,
            preserveState: true,
            only: ['map_characters', 'map_route_solarsystems', 'ignored_systems'],
        });
    };

    return {
        ignoredSystems,
        isIgnored,
        ignoreSystem,
        clearIgnoreList,
    };
}

import { AppPageProps } from '@/types';
import { router, usePage } from '@inertiajs/vue3';
import { computed } from 'vue';

export function useNavigationSystems() {
    const page = usePage<
        AppPageProps<{
            from_solarsystem_id?: number;
            to_solarsystem_id?: number;
        }>
    >();

    const fromSystemId = computed(() => page.props.from_solarsystem_id);
    const toSystemId = computed(() => page.props.to_solarsystem_id);

    function setFromSystem(solarsystemId: number) {
        router.reload({
            data: {
                from_solarsystem_id: solarsystemId,
            },
            only: ['map_navigation'],
        });
    }

    function setToSystem(solarsystemId: number) {
        router.reload({
            data: {
                to_solarsystem_id: solarsystemId,
            },
            only: ['map_navigation'],
        });
    }

    function clearFromSystem() {
        router.reload({
            data: {
                from_solarsystem_id: null,
            },
            only: ['map_navigation'],
        });
    }

    function clearToSystem() {
        router.reload({
            data: {
                to_solarsystem_id: null,
            },
            only: ['map_navigation'],
        });
    }

    function setBothSystems(fromId: number, toId: number) {
        router.reload({
            data: {
                from_solarsystem_id: fromId,
                to_solarsystem_id: toId,
            },
            only: ['map_navigation'],
        });
    }

    function clearBothSystems() {
        router.reload({
            data: {
                from_solarsystem_id: null,
                to_solarsystem_id: null,
            },
            only: ['map_navigation'],
        });
    }

    return {
        fromSystemId,
        toSystemId,
        setFromSystem,
        setToSystem,
        clearFromSystem,
        clearToSystem,
        setBothSystems,
        clearBothSystems,
    };
}

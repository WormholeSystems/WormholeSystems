import HomeSystemController from '@/actions/App/Http/Controllers/HomeSystemController';
import { useMap } from '@/composables/useMap';
import { router } from '@inertiajs/vue3';
import { computed, type MaybeRefOrGetter, toValue } from 'vue';

export function useHomeSystem(mapSolarsystemId: MaybeRefOrGetter<number>) {
    const map = useMap();

    const isHome = computed(() => map.value.home_solarsystem_id === toValue(mapSolarsystemId));

    function toggleHomeSystem() {
        const id = toValue(mapSolarsystemId);
        router.post(
            HomeSystemController.store(map.value.slug).url,
            { map_solarsystem_id: isHome.value ? null : id },
            { preserveScroll: true, preserveState: true },
        );
    }

    return {
        isHome,
        toggleHomeSystem,
    };
}

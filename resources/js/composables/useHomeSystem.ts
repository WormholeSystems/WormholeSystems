import HomeSystemController from '@/actions/App/Http/Controllers/HomeSystemController';
import { useMap } from '@/composables/useMap';
import { router } from '@inertiajs/vue3';
import { computed, type MaybeRefOrGetter, toValue } from 'vue';

export function useHomeSystem(solarsystemId: MaybeRefOrGetter<number>) {
    const map = useMap();

    const isHome = computed(() => map.value.home_solarsystem_id === toValue(solarsystemId));

    function toggleHomeSystem() {
        const solarsystem_id = toValue(solarsystemId);
        router.post(
            HomeSystemController.store(map.value.slug).url,
            { solarsystem_id: isHome.value ? null : solarsystem_id },
            { preserveScroll: true, preserveState: true },
        );
    }

    return {
        isHome,
        toggleHomeSystem,
    };
}

import RallyPointController from '@/actions/App/Http/Controllers/RallyPointController';
import { useMap } from '@/composables/useMap';
import { router } from '@inertiajs/vue3';
import { computed, type MaybeRefOrGetter, toValue } from 'vue';

export function useRallyPoint(solarsystemId: MaybeRefOrGetter<number>) {
    const map = useMap();

    const isRally = computed(() => map.value.rally_solarsystem_ids.includes(toValue(solarsystemId)));

    function toggleRallyPoint() {
        const action = isRally.value ? RallyPointController.destroy(map.value.slug) : RallyPointController.store(map.value.slug);
        router.visit(action.url, {
            method: action.method,
            data: { solarsystem_id: toValue(solarsystemId) },
            preserveScroll: true,
            preserveState: true,
        });
    }

    return {
        isRally,
        toggleRallyPoint,
    };
}

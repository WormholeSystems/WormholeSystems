import RallyPointController from '@/actions/App/Http/Controllers/RallyPointController';
import { useMap } from '@/composables/useMap';
import { router } from '@inertiajs/vue3';
import { computed, type MaybeRefOrGetter, toValue } from 'vue';

export function useRallyPoint(solarsystemId: MaybeRefOrGetter<number>) {
    const map = useMap();

    const isRally = computed(() => map.value.rally_solarsystem_id === toValue(solarsystemId));

    function toggleRallyPoint() {
        const id = toValue(solarsystemId);
        router.post(
            RallyPointController.store(map.value.slug).url,
            { solarsystem_id: isRally.value ? null : id },
            { preserveScroll: true, preserveState: true },
        );
    }

    return {
        isRally,
        toggleRallyPoint,
    };
}

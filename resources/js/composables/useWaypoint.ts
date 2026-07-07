import BulkWaypointController from '@/actions/App/Http/Controllers/BulkWaypointController';
import useUser from '@/composables/useUser';
import Waypoints from '@/routes/waypoints';
import { router } from '@inertiajs/vue3';
import { computed } from 'vue';

export function useWaypoint() {
    const user = useUser();

    /** Waypoints can only be set for characters that are online in-game. */
    const onlineCharacters = computed(() => user.value?.characters.filter((character) => character.status?.is_online) ?? []);

    function setWaypoint(character_id: number, solarsystem_id: number, clear_other_waypoints: boolean = true, add_to_beginning: boolean = false) {
        router.post(
            Waypoints.store().url,
            {
                character_id,
                destination_id: solarsystem_id,
                clear_other_waypoints,
                add_to_beginning,
            },
            {
                preserveState: true,
                preserveScroll: true,
                only: ['notification'],
            },
        );
    }

    function setWaypointAll(solarsystem_id: number, clear_other_waypoints: boolean = true, add_to_beginning: boolean = false) {
        router.post(
            BulkWaypointController.store().url,
            {
                destination_id: solarsystem_id,
                clear_other_waypoints,
                add_to_beginning,
            },
            {
                preserveState: true,
                preserveScroll: true,
                only: ['notification'],
            },
        );
    }

    return { setWaypoint, setWaypointAll, onlineCharacters };
}

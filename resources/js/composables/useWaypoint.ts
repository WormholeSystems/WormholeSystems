import { router } from '@inertiajs/vue3';

export function useWaypoint() {
    function setWaypoint(character_id: number, solarsystem_id: number, clear_other_waypoints: boolean = true, add_to_beginning: boolean = false) {
        router.post(
            route('waypoints.store'),
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

    return setWaypoint;
}

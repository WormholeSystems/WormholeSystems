import { useShowMap } from '@/composables/useShowMap';
import useUser from '@/composables/useUser';
import { computed } from 'vue';

/**
 * Returns the active map character for the user that hold
 * the status of the user (solarsystem, ship, etc.).
 */
export function useActiveMapCharacter() {
    const user = useUser();
    const page = useShowMap();

    return computed(() => page.props.map_characters.find((c) => c.id === user.value.active_character.id));
}

import type { TMapSolarsystem } from '@/pages/maps';
import { ref } from 'vue';

// The system the "Add connection" dialog should link a newly added system to. Kept at
// module scope so a node's context menu can open one shared, map-level dialog instead of
// rendering a dialog per node.
const origin = ref<TMapSolarsystem | null>(null);

export function useAddConnectionDialog() {
    function openAddConnection(system: TMapSolarsystem): void {
        origin.value = system;
    }

    function closeAddConnection(): void {
        origin.value = null;
    }

    return { origin, openAddConnection, closeAddConnection };
}

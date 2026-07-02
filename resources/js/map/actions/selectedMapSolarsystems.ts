import type { MapStore } from '@/map/store/mapStore';
import type { TMapSolarsystem } from '@/pages/maps';

/**
 * The systems the bulk actions operate on: the current selection minus pinned
 * systems and the map's home system, which are never bulk-deleted or organized.
 */
export function getSelectedMapSolarsystems(store: MapStore): TMapSolarsystem[] {
    const homeSolarsystemId = store.meta.value?.home_solarsystem_id ?? null;
    return [...store.selectedIds.value]
        .map((id) => store.systems.get(id))
        .filter((system): system is TMapSolarsystem => system !== undefined && !system.pinned && system.solarsystem_id !== homeSolarsystemId);
}

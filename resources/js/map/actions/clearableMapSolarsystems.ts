import type { MapStore } from '@/map/store/mapStore';
import type { TMapSolarsystem } from '@/pages/maps';

/**
 * The systems "Clear map" removes: everything except pinned systems and the
 * map's home system.
 */
export function getClearableMapSolarsystems(store: MapStore): TMapSolarsystem[] {
    const homeSolarsystemId = store.meta.value?.home_solarsystem_id ?? null;
    return [...store.systems.values()].filter((system) => !system.pinned && system.solarsystem_id !== homeSolarsystemId);
}

import { TMap } from '@/pages/maps';
import { getCurrentInstance, inject, provide, shallowRef, type InjectionKey, type ShallowRef } from 'vue';
import { createDerivedState, type DerivedState, type RouteDeps, type TMapMeta } from './derived';
import { createEntityState, type EntityState } from './entities';
import { createViewState, type ViewState } from './viewState';

export type MapStore = EntityState &
    ViewState &
    DerivedState & {
        meta: ShallowRef<TMapMeta | null>;
        patchMapMeta(meta: TMapMeta): void;
        /** Full hydration/resync: keyed diff of both collections plus the metadata. */
        reconcileMap(map: TMap): void;
    };

export function createMapStore(routeDeps: RouteDeps = {}): MapStore {
    const entities = createEntityState();
    const view = createViewState();
    const meta = shallowRef<TMapMeta | null>(null);
    const derived = createDerivedState(entities, view, meta, routeDeps);

    function patchMapMeta(next: TMapMeta): void {
        meta.value = next;
    }

    function reconcileMap(map: TMap): void {
        const { map_solarsystems, map_connections, ...rest } = map;
        patchMapMeta(rest);
        entities.reconcile({ map_solarsystems, map_connections });
        for (const id of [...view.selectedIds.value]) {
            if (!entities.systems.has(id)) {
                view.pruneSelection(id);
            }
        }
    }

    const removeSystem = (id: number): void => {
        entities.removeSystem(id);
        view.pruneSelection(id);
    };

    return {
        ...entities,
        ...view,
        ...derived,
        removeSystem,
        meta,
        patchMapMeta,
        reconcileMap,
    };
}

const MAP_STORE_KEY: InjectionKey<MapStore> = Symbol('mapStore');

/**
 * The store is provided for the canvas subtree and registered as the page-level
 * current store, so sidebar panels living outside the subtree (characters,
 * killmails, …) reach the same instance through useMapStore(). A shallow ref so
 * lazy readers (computeds created before the canvas mounts, e.g. the status bar)
 * re-evaluate once the store appears.
 */
const currentStore: ShallowRef<MapStore | null> = shallowRef(null);

export function provideMapStore(store: MapStore): void {
    provide(MAP_STORE_KEY, store);
    currentStore.value = store;
}

export function useMapStore(): MapStore {
    const store = (getCurrentInstance() ? inject(MAP_STORE_KEY, null) : null) ?? currentStore.value;
    if (!store) {
        throw new Error('useMapStore() called before a map store was provided');
    }
    return store;
}

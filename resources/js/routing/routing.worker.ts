import { findClosestSystems, findRoute, setStaticData } from '@/routing/algorithm';
import * as Comlink from 'comlink';

const api = {
    setStaticData,
    findRoute,
    findClosestSystems,
};

export type RoutingWorkerApi = typeof api;

Comlink.expose(api);

import { useStaticData } from '@/composables/useStaticData';
import type { RoutingWorkerApi } from '@/routing/routing.worker';
import type { ClosestSystem, RouteResult, RoutingConnection, RoutingSettings } from '@/routing/types';
import * as Comlink from 'comlink';

let initPromise: Promise<void> | null = null;
let workerProxy: Comlink.Remote<RoutingWorkerApi> | null = null;

export async function initializeRouting(): Promise<void> {
    if (typeof Worker === 'undefined') {
        return;
    }

    if (!initPromise) {
        initPromise = (async () => {
            const worker = new Worker('/workers/routing.worker.js', {
                type: 'module',
                name: 'routing-worker',
            });
            workerProxy = Comlink.wrap<RoutingWorkerApi>(worker);

            const { loadStaticData } = useStaticData();
            const data = await loadStaticData();

            await workerProxy.setStaticData({
                solarsystems: data.solarsystems,
                connections: data.connections,
            });
        })();
    }

    await initPromise;
}

export async function findRoute(
    settings: RoutingSettings,
    fromId: number,
    toId: number,
    dynamicConnections: RoutingConnection[],
    eveScoutConnections: RoutingConnection[],
    ignoredSystems: number[],
): Promise<RouteResult> {
    await initializeRouting();
    if (!workerProxy) {
        return { route: [], jumps: 0, cost: 0 };
    }
    return workerProxy.findRoute(settings, fromId, toId, dynamicConnections, eveScoutConnections, ignoredSystems);
}

export async function findClosestSystems(
    settings: RoutingSettings,
    fromId: number,
    condition: string,
    limit: number,
    dynamicConnections: RoutingConnection[],
    eveScoutConnections: RoutingConnection[],
    ignoredSystems: number[],
): Promise<ClosestSystem[]> {
    await initializeRouting();
    if (!workerProxy) {
        return [];
    }
    return workerProxy.findClosestSystems(settings, fromId, condition, limit, dynamicConnections, eveScoutConnections, ignoredSystems);
}

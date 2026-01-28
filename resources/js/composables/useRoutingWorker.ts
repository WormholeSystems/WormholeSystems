import { useStaticData } from '@/composables/useStaticData';
import type {
    ConnectionType,
    RoutingSettings,
    WorkerClosestResult,
    WorkerClosestSystem,
    WorkerComputePayload,
    WorkerConnection,
    WorkerInitPayload,
    WorkerRequest,
    WorkerRouteResult,
    WorkerRouteStep,
} from '@/routing/types';
import type { TMassStatus } from '@/types/models';
import type { TStaticSolarsystem } from '@/types/static-data';

let routingEngine: RoutingEngine | null = null;
let initPromise: Promise<void> | null = null;

type GraphEdge = WorkerConnection;

const massStatusAllowList: Record<TMassStatus, Set<TMassStatus>> = {
    fresh: new Set(['fresh']),
    reduced: new Set(['fresh', 'reduced']),
    critical: new Set(['fresh', 'reduced', 'critical']),
};

// Zarzakh uses Jovian stargates that require special access - don't route THROUGH it
const ZARZAKH_SYSTEM_ID = 30100000;

let staticSolarsystems: Map<number, TStaticSolarsystem> = new Map();
let staticAdjacency: Map<number, GraphEdge[]> = new Map();

type RoutingEngine = {
    initialize: (payload: WorkerInitPayload) => Promise<void>;
    compute: (payload: Omit<WorkerComputePayload, 'callId'>) => Promise<(WorkerRouteResult | WorkerClosestResult)[]>;
};

const engine: RoutingEngine = {
    async initialize(payload) {
        staticSolarsystems = new Map(payload.solarsystems.map((system) => [system.id, system]));
        staticAdjacency = buildAdjacency(payload.connections, 'stargate');
    },
    async compute(payload) {
        if (!staticSolarsystems.size) {
            return [];
        }

        const dynamicAdjacency = buildAdjacency(payload.dynamicConnections, 'wormhole');
        const eveScoutAdjacency = payload.settings.useEveScout
            ? buildAdjacency(payload.eveScoutConnections, 'evescout')
            : new Map<number, GraphEdge[]>();
        const ignoredSet = new Set<number>(payload.ignoredSystems.filter(Boolean));
        // Zarzakh uses Jovian stargates - don't route THROUGH it (but TO/FROM is OK)
        ignoredSet.add(ZARZAKH_SYSTEM_ID);

        return payload.requests.map((request) => {
            if (request.type === 'route') {
                return calculateRoute(request, payload.settings, dynamicAdjacency, eveScoutAdjacency, ignoredSet);
            }

            return calculateClosestSystems(request, payload.settings, dynamicAdjacency, eveScoutAdjacency, ignoredSet);
        });
    },
};

function buildAdjacency(connections: Record<number, number[]> | WorkerConnection[], typeOverride: ConnectionType): Map<number, GraphEdge[]> {
    const adjacency = new Map<number, GraphEdge[]>();

    if (Array.isArray(connections)) {
        for (const edge of connections) {
            addEdge(adjacency, edge.from, edge.to, edge);
            addEdge(adjacency, edge.to, edge.from, { ...edge, from: edge.to, to: edge.from });
        }

        return adjacency;
    }

    for (const [fromKey, neighbors] of Object.entries(connections)) {
        const fromId = Number(fromKey);
        for (const neighbor of neighbors) {
            const toId = Number(neighbor);
            if (!Number.isFinite(toId)) {
                continue;
            }

            const edge: GraphEdge = {
                from: fromId,
                to: toId,
                type: typeOverride,
            };
            addEdge(adjacency, fromId, toId, edge);
            addEdge(adjacency, toId, fromId, { ...edge, from: toId, to: fromId });
        }
    }

    return adjacency;
}

function addEdge(map: Map<number, GraphEdge[]>, from: number, to: number, edge: GraphEdge): void {
    const existing = map.get(from);
    if (existing) {
        existing.push(edge);
        return;
    }

    map.set(from, [edge]);
}

function calculateRoute(
    request: WorkerRequest & { type: 'route' },
    settings: RoutingSettings,
    dynamicAdj: Map<number, GraphEdge[]>,
    eveScoutAdj: Map<number, GraphEdge[]>,
    ignored: Set<number>,
): WorkerRouteResult {
    const distances = new Map<number, number>();
    const previous = new Map<number, number | null>();
    const via = new Map<number, ConnectionType | null>();
    const queue = new PriorityQueue<number>();

    distances.set(request.fromId, 0);
    previous.set(request.fromId, null);
    via.set(request.fromId, null);
    queue.push(request.fromId, 0);

    while (!queue.isEmpty()) {
        const current = queue.pop();
        if (!current) {
            break;
        }

        const currentId = current.value;
        const currentCost = current.priority;

        if (currentId === request.toId) {
            break;
        }

        if (currentCost > (distances.get(currentId) ?? Infinity)) {
            continue;
        }

        const neighbors = getNeighbors(currentId, dynamicAdj, eveScoutAdj);
        for (const neighbor of neighbors) {
            if (neighbor.type === 'evescout' && !settings.useEveScout) {
                continue;
            }

            if (isIgnoredNode(neighbor.to, request, ignored)) {
                continue;
            }

            const targetNode = staticSolarsystems.get(neighbor.to);
            if (!targetNode) {
                continue;
            }

            if (!isEdgeAllowed(neighbor, settings)) {
                continue;
            }

            const edgeCost = getEdgeCost(settings, targetNode.security, neighbor.type);
            const newCost = currentCost + edgeCost;

            if (newCost < (distances.get(neighbor.to) ?? Infinity)) {
                distances.set(neighbor.to, newCost);
                previous.set(neighbor.to, currentId);
                via.set(neighbor.to, neighbor.type);
                queue.push(neighbor.to, newCost);
            }
        }
    }

    if (!distances.has(request.toId)) {
        return { id: request.id, type: 'route', route: [], jumps: 0, cost: 0 };
    }

    const path: WorkerRouteStep[] = [];
    let node: number | null = request.toId;

    while (node !== null) {
        path.push({ id: node, via: via.get(node) ?? null });
        node = previous.get(node) ?? null;
    }

    path.reverse();

    return {
        id: request.id,
        type: 'route',
        route: path,
        jumps: Math.max(0, path.length - 1),
        cost: Number(distances.get(request.toId) ?? 0),
    };
}

function calculateClosestSystems(
    request: WorkerRequest & { type: 'closest' },
    settings: RoutingSettings,
    dynamicAdj: Map<number, GraphEdge[]>,
    eveScoutAdj: Map<number, GraphEdge[]>,
    ignored: Set<number>,
): WorkerClosestResult {
    const visited = new Set<number>(ignored);
    const queue = new PriorityQueue<{ id: number; jumps: number }>();
    const results: WorkerClosestSystem[] = [];

    visited.add(request.fromId);
    queue.push({ id: request.fromId, jumps: 0 }, 0);

    while (!queue.isEmpty() && results.length < request.limit) {
        const current = queue.pop();
        if (!current) {
            break;
        }

        const { id: currentId, jumps } = current.value;
        const currentCost = current.priority;

        const neighbors = getNeighbors(currentId, dynamicAdj, eveScoutAdj);
        for (const neighbor of neighbors) {
            if (neighbor.type === 'evescout' && !settings.useEveScout) {
                continue;
            }

            if (isIgnoredNode(neighbor.to, request, ignored)) {
                continue;
            }

            if (!isEdgeAllowed(neighbor, settings)) {
                continue;
            }

            const targetNode = staticSolarsystems.get(neighbor.to);
            if (!targetNode) {
                continue;
            }

            const nextCost = currentCost + getEdgeCost(settings, targetNode.security, neighbor.type);
            const nextJumps = jumps + 1;

            if (!visited.has(neighbor.to)) {
                visited.add(neighbor.to);
                queue.push({ id: neighbor.to, jumps: nextJumps }, nextCost);

                if (matchesCondition(targetNode, request.condition)) {
                    results.push({
                        solarsystem_id: neighbor.to,
                        jumps: nextJumps,
                        cost: nextCost,
                    });

                    if (results.length >= request.limit) {
                        break;
                    }
                }
            }
        }
    }

    return { id: request.id, type: 'closest', results };
}

function getNeighbors(nodeId: number, dynamicAdj: Map<number, GraphEdge[]>, eveScoutAdj: Map<number, GraphEdge[]>): GraphEdge[] {
    const neighbors: GraphEdge[] = [];

    const staticNeighbors = staticAdjacency.get(nodeId);
    if (staticNeighbors) {
        neighbors.push(...staticNeighbors);
    }

    const dynamicNeighbors = dynamicAdj.get(nodeId);
    if (dynamicNeighbors) {
        neighbors.push(...dynamicNeighbors);
    }

    const scoutNeighbors = eveScoutAdj.get(nodeId);
    if (scoutNeighbors) {
        neighbors.push(...scoutNeighbors);
    }

    return neighbors;
}

function isEdgeAllowed(edge: GraphEdge, settings: RoutingSettings): boolean {
    if (edge.type === 'stargate') {
        return true;
    }

    if (!settings.allowEol && edge.lifetimeStatus === 'eol') {
        return false;
    }

    if (edge.massStatus && !massStatusAllowList[settings.massStatus].has(edge.massStatus)) {
        return false;
    }

    return true;
}

function isIgnoredNode(nodeId: number, request: { fromId: number; toId?: number }, ignored: Set<number>): boolean {
    if (nodeId === request.fromId || (request.toId !== undefined && nodeId === request.toId)) {
        return false;
    }

    return ignored.has(nodeId);
}

function getEdgeCost(settings: RoutingSettings, security: number, _type: ConnectionType): number {
    let cost = 1;

    if (settings.routePreference === 'shorter') {
        return cost;
    }

    const penalty = Math.max(0, Math.min(100, settings.securityPenalty)) / 100;
    const clampedSecurity = Math.max(0, Math.min(1, security));
    const directional = settings.routePreference === 'safer' ? 1 - clampedSecurity : clampedSecurity;

    cost += directional * penalty;

    return cost;
}

function matchesCondition(system: TStaticSolarsystem, condition: string): boolean {
    switch (condition) {
        case 'observatories':
            return Boolean(system.has_jove_observatory);
        case 'npc_stations':
            return Boolean(system.has_stations);
        case 'highsec':
            return system.security >= 0.5;
        case 'lowsec':
            return system.security >= 0.1 && system.security <= 0.4;
        case 'nullsec':
            return system.security <= 0.0;
        default:
            return false;
    }
}

class PriorityQueue<T> {
    private items: Array<{ priority: number; value: T }> = [];

    public push(value: T, priority: number): void {
        this.items.push({ priority, value });
        this.heapifyUp(this.items.length - 1);
    }

    public pop(): { priority: number; value: T } | null {
        if (!this.items.length) {
            return null;
        }

        this.swap(0, this.items.length - 1);
        const item = this.items.pop()!;
        this.heapifyDown(0);
        return item;
    }

    public isEmpty(): boolean {
        return this.items.length === 0;
    }

    private heapifyUp(index: number): void {
        let current = index;

        while (current > 0) {
            const parentIndex = Math.floor((current - 1) / 2);
            if (this.items[parentIndex].priority <= this.items[current].priority) {
                break;
            }

            this.swap(current, parentIndex);
            current = parentIndex;
        }
    }

    private heapifyDown(index: number): void {
        let current = index;

        while (true) {
            const left = 2 * current + 1;
            const right = 2 * current + 2;
            let smallest = current;

            if (left < this.items.length && this.items[left].priority < this.items[smallest].priority) {
                smallest = left;
            }

            if (right < this.items.length && this.items[right].priority < this.items[smallest].priority) {
                smallest = right;
            }

            if (smallest === current) {
                break;
            }

            this.swap(current, smallest);
            current = smallest;
        }
    }

    private swap(a: number, b: number): void {
        const temp = this.items[a];
        this.items[a] = this.items[b];
        this.items[b] = temp;
    }
}

export async function initializeRoutingWorker(): Promise<void> {
    if (!routingEngine) {
        routingEngine = engine;
    }

    if (!initPromise) {
        initPromise = (async () => {
            const { loadStaticData } = useStaticData();
            const data = await loadStaticData();
            await routingEngine!.initialize({ solarsystems: data.solarsystems, connections: data.connections });
        })();
    }

    await initPromise;
}

export async function getRoutingWorkerClient(): Promise<RoutingEngine> {
    await initializeRoutingWorker();

    if (!routingEngine) {
        throw new Error('Routing engine is not initialized');
    }

    return routingEngine;
}

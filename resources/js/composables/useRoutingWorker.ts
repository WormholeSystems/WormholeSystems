import { useStaticData } from '@/composables/useStaticData';
import type { ClosestSystem, ConnectionType, RouteResult, RouteStep, RoutingConnection, RoutingSettings } from '@/routing/types';
import type { TLifetimeStatus, TMassStatus } from '@/types/models';
import type { TStaticSolarsystem } from '@/types/static-data';

let initPromise: Promise<void> | null = null;

let staticSolarsystems: Map<number, TStaticSolarsystem> = new Map();
let staticAdjacency: Map<number, RoutingConnection[]> = new Map();

const massStatusAllowList: Record<TMassStatus, Set<TMassStatus>> = {
    fresh: new Set(['fresh']),
    reduced: new Set(['fresh', 'reduced']),
    critical: new Set(['fresh', 'reduced', 'critical']),
};

const lifetimeStatusAllowList: Record<TLifetimeStatus, Set<TLifetimeStatus>> = {
    healthy: new Set(['healthy']),
    eol: new Set(['healthy', 'eol']),
    critical: new Set(['healthy', 'eol', 'critical']),
};

// Zarzakh uses Jovian stargates that require special access - don't route THROUGH it
const ZARZAKH_SYSTEM_ID = 30100000;

export async function initializeRouting(): Promise<void> {
    if (!initPromise) {
        initPromise = (async () => {
            const { loadStaticData } = useStaticData();
            const data = await loadStaticData();
            staticSolarsystems = new Map(data.solarsystems.map((system) => [system.id, system]));
            staticAdjacency = buildAdjacency(data.connections, 'stargate');
        })();
    }

    await initPromise;
}

export function findRoute(
    settings: RoutingSettings,
    fromId: number,
    toId: number,
    dynamicConnections: RoutingConnection[],
    eveScoutConnections: RoutingConnection[],
    ignoredSystems: number[],
): RouteResult {
    if (!staticSolarsystems.size) {
        return { route: [], jumps: 0, cost: 0 };
    }

    const dynamicAdj = buildAdjacency(dynamicConnections, 'wormhole');
    const eveScoutAdj = settings.useEveScout ? buildAdjacency(eveScoutConnections, 'evescout') : new Map<number, RoutingConnection[]>();
    const ignored = buildIgnoredSet(ignoredSystems);

    const distances = new Map<number, number>();
    const previous = new Map<number, number | null>();
    const via = new Map<number, ConnectionType | null>();
    const queue = new PriorityQueue<number>();

    distances.set(fromId, 0);
    previous.set(fromId, null);
    via.set(fromId, null);
    queue.push(fromId, 0);

    while (!queue.isEmpty()) {
        const current = queue.pop();
        if (!current) {
            break;
        }

        const currentId = current.value;
        const currentCost = current.priority;

        if (currentId === toId) {
            break;
        }

        if (currentCost > (distances.get(currentId) ?? Infinity)) {
            continue;
        }

        for (const neighbor of getNeighbors(currentId, dynamicAdj, eveScoutAdj)) {
            if (neighbor.type === 'evescout' && !settings.useEveScout) {
                continue;
            }

            if (isIgnoredNode(neighbor.to, fromId, toId, ignored)) {
                continue;
            }

            if (!isEdgeAllowed(neighbor, settings)) {
                continue;
            }

            const targetNode = staticSolarsystems.get(neighbor.to);
            if (!targetNode) {
                continue;
            }

            const edgeCost = getEdgeCost(settings, targetNode);
            const newCost = currentCost + edgeCost;

            if (newCost < (distances.get(neighbor.to) ?? Infinity)) {
                distances.set(neighbor.to, newCost);
                previous.set(neighbor.to, currentId);
                via.set(neighbor.to, neighbor.type);
                queue.push(neighbor.to, newCost);
            }
        }
    }

    if (!distances.has(toId)) {
        return { route: [], jumps: 0, cost: 0 };
    }

    const path = reconstructRoute(toId, previous, via);

    return {
        route: path,
        jumps: Math.max(0, path.length - 1),
        cost: Number(distances.get(toId) ?? 0),
    };
}

export function findClosestSystems(
    settings: RoutingSettings,
    fromId: number,
    condition: string,
    limit: number,
    dynamicConnections: RoutingConnection[],
    eveScoutConnections: RoutingConnection[],
    ignoredSystems: number[],
): ClosestSystem[] {
    if (!staticSolarsystems.size) {
        return [];
    }

    const dynamicAdj = buildAdjacency(dynamicConnections, 'wormhole');
    const eveScoutAdj = settings.useEveScout ? buildAdjacency(eveScoutConnections, 'evescout') : new Map<number, RoutingConnection[]>();
    const ignored = buildIgnoredSet(ignoredSystems);

    const visited = new Set<number>(ignored);
    const previous = new Map<number, number | null>();
    const viaMap = new Map<number, ConnectionType | null>();
    const queue = new PriorityQueue<{ id: number; jumps: number }>();
    const results: ClosestSystem[] = [];

    visited.add(fromId);
    previous.set(fromId, null);
    viaMap.set(fromId, null);
    queue.push({ id: fromId, jumps: 0 }, 0);

    while (!queue.isEmpty() && results.length < limit) {
        const current = queue.pop();
        if (!current) {
            break;
        }

        const { id: currentId, jumps } = current.value;
        const currentCost = current.priority;

        for (const neighbor of getNeighbors(currentId, dynamicAdj, eveScoutAdj)) {
            if (neighbor.type === 'evescout' && !settings.useEveScout) {
                continue;
            }

            if (isIgnoredNode(neighbor.to, fromId, undefined, ignored)) {
                continue;
            }

            if (!isEdgeAllowed(neighbor, settings)) {
                continue;
            }

            const targetNode = staticSolarsystems.get(neighbor.to);
            if (!targetNode) {
                continue;
            }

            const nextCost = currentCost + getEdgeCost(settings, targetNode);
            const nextJumps = jumps + 1;

            if (!visited.has(neighbor.to)) {
                visited.add(neighbor.to);
                previous.set(neighbor.to, currentId);
                viaMap.set(neighbor.to, neighbor.type);
                queue.push({ id: neighbor.to, jumps: nextJumps }, nextCost);

                if (matchesCondition(targetNode, condition)) {
                    results.push({
                        solarsystem_id: neighbor.to,
                        jumps: nextJumps,
                        cost: nextCost,
                        route: reconstructRoute(neighbor.to, previous, viaMap),
                    });

                    if (results.length >= limit) {
                        break;
                    }
                }
            }
        }
    }

    return results;
}

function buildAdjacency(connections: Record<number, number[]> | RoutingConnection[], typeOverride: ConnectionType): Map<number, RoutingConnection[]> {
    const adjacency = new Map<number, RoutingConnection[]>();

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

            const edge: RoutingConnection = {
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

function addEdge(map: Map<number, RoutingConnection[]>, from: number, to: number, edge: RoutingConnection): void {
    const existing = map.get(from);
    if (existing) {
        existing.push(edge);
        return;
    }

    map.set(from, [edge]);
}

function buildIgnoredSet(ignoredSystems: number[]): Set<number> {
    const ignored = new Set<number>(ignoredSystems.filter(Boolean));
    ignored.add(ZARZAKH_SYSTEM_ID);
    return ignored;
}

function getNeighbors(
    nodeId: number,
    dynamicAdj: Map<number, RoutingConnection[]>,
    eveScoutAdj: Map<number, RoutingConnection[]>,
): RoutingConnection[] {
    const neighbors: RoutingConnection[] = [];

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

function isEdgeAllowed(edge: RoutingConnection, settings: RoutingSettings): boolean {
    if (edge.type === 'stargate') {
        return true;
    }

    if (edge.lifetimeStatus && !lifetimeStatusAllowList[settings.lifetimeStatus].has(edge.lifetimeStatus)) {
        return false;
    }

    if (edge.massStatus && !massStatusAllowList[settings.massStatus].has(edge.massStatus)) {
        return false;
    }

    return true;
}

function isIgnoredNode(nodeId: number, fromId: number, toId: number | undefined, ignored: Set<number>): boolean {
    if (nodeId === fromId || (toId !== undefined && nodeId === toId)) {
        return false;
    }

    return ignored.has(nodeId);
}

function getEdgeCost(settings: RoutingSettings, targetSystem: TStaticSolarsystem): number {
    if (settings.routePreference === 'shorter') {
        return 1;
    }

    const penaltyCost = Math.exp(0.15 * settings.securityPenalty);
    const security = targetSystem.security;

    if (settings.routePreference === 'safer') {
        if (security <= 0.0) {
            return 2 * penaltyCost;
        }
        if (security < 0.45) {
            return penaltyCost;
        }
        return 0.9;
    }

    // less_secure
    if (security <= 0.0) {
        return 2 * penaltyCost;
    }
    if (security < 0.45) {
        return 0.9;
    }
    return penaltyCost;
}

function reconstructRoute(targetId: number, previous: Map<number, number | null>, viaMap: Map<number, ConnectionType | null>): RouteStep[] {
    const path: RouteStep[] = [];
    let node: number | null = targetId;

    while (node !== null) {
        path.push({ id: node, via: viaMap.get(node) ?? null });
        node = previous.get(node) ?? null;
    }

    path.reverse();
    return path;
}

function matchesCondition(system: TStaticSolarsystem, condition: string): boolean {
    if (condition.startsWith('service_')) {
        const serviceId = parseInt(condition.substring(8), 10);
        if (!Number.isNaN(serviceId)) {
            return system.services?.includes(serviceId) ?? false;
        }
        return false;
    }

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

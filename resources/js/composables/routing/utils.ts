import type { TMapConnection, TMapSolarsystem } from '@/pages/maps';
import type { RoutingConnection } from '@/routing/types';
import type { TEveScoutConnection } from '@/types/eve-scout';
import type { TLifetimeStatus, TMassStatus } from '@/types/models';

export function convertMapConnectionsToWorkerEdges(
    mapConnections: TMapConnection[] | undefined | null,
    mapSolarsystems: TMapSolarsystem[] | undefined | null,
): RoutingConnection[] {
    if (!mapConnections?.length || !mapSolarsystems?.length) {
        return [];
    }

    const lookup = new Map<number, number>();
    mapSolarsystems.forEach((system) => lookup.set(system.id, system.solarsystem_id));

    const edges: RoutingConnection[] = [];

    for (const connection of mapConnections) {
        const from = lookup.get(connection.from_map_solarsystem_id);
        const to = lookup.get(connection.to_map_solarsystem_id);

        if (!from || !to || from === to) {
            continue;
        }

        edges.push({
            from,
            to,
            type: 'wormhole',
            massStatus: connection.mass_status,
            lifetimeStatus: connection.lifetime_status,
        });
    }

    return edges;
}

export function convertEveScoutConnections(connections: TEveScoutConnection[] | undefined, include: boolean): RoutingConnection[] {
    if (!include || !connections?.length) {
        return [];
    }

    const edges: RoutingConnection[] = [];

    for (const connection of connections) {
        if (!connection.in_system_id || !connection.out_system_id) {
            continue;
        }

        edges.push({
            from: connection.in_system_id,
            to: connection.out_system_id,
            type: 'evescout',
            massStatus: normalizeMassStatus(connection.mass),
            lifetimeStatus: normalizeLifetimeStatus(connection.life),
        });
    }

    return edges;
}

function normalizeMassStatus(raw?: string | null): TMassStatus {
    const normalized = raw?.toLowerCase();
    if (!normalized || normalized === 'unknown' || normalized === 'fresh' || normalized === 'stable') {
        return 'fresh';
    }

    if (normalized === 'reduced' || normalized === 'destab') {
        return 'reduced';
    }

    return 'critical';
}

function normalizeLifetimeStatus(raw?: string | null): TLifetimeStatus {
    if (!raw) {
        return 'healthy';
    }

    const normalized = raw.toLowerCase();

    if (normalized.includes('eol')) {
        return 'eol';
    }

    if (normalized.includes('critical')) {
        return 'critical';
    }

    return 'healthy';
}

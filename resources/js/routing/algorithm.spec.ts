import { findClosestSystems, findRoute, setStaticData } from '@/routing/algorithm';
import type { RoutingConnection, RoutingSettings } from '@/routing/types';
import type { TStaticSolarsystem } from '@/types/static-data';
import { beforeEach, describe, expect, it } from 'vitest';

const HOME = 31000001;
const EXIT = 30000001;
const NEIGHBOUR = 30000002;
const MIDPOINT = 30000003;
const HUB = 30000004;

function system(id: number, name: string, security: number): TStaticSolarsystem {
    return { id, name, security, has_stations: true } as unknown as TStaticSolarsystem;
}

/**
 * A highsec gate chain of three jumps from EXIT to HUB, shortcut by a pair of
 * wormholes through home that covers the same ground in two. The chain only
 * ever wins while wormholes are allowed.
 */
const solarsystems = [
    system(HOME, 'J123456', -0.99),
    system(EXIT, 'Exit', 0.9),
    system(NEIGHBOUR, 'Neighbour', 0.9),
    system(MIDPOINT, 'Midpoint', 0.9),
    system(HUB, 'Hub', 0.9),
];

const wormholes: RoutingConnection[] = [
    { from: HOME, to: EXIT, type: 'wormhole', massStatus: 'fresh', lifetimeStatus: 'healthy' },
    { from: HOME, to: HUB, type: 'wormhole', massStatus: 'fresh', lifetimeStatus: 'healthy' },
];

function settings(overrides: Partial<RoutingSettings> = {}): RoutingSettings {
    return {
        routePreference: 'shorter',
        securityPenalty: 50,
        lifetimeStatus: 'critical',
        massStatus: 'critical',
        useEveScout: false,
        useWormholes: true,
        ...overrides,
    };
}

beforeEach(() => {
    setStaticData({
        solarsystems,
        connections: { [EXIT]: [NEIGHBOUR], [NEIGHBOUR]: [MIDPOINT], [MIDPOINT]: [HUB] },
    });
});

describe('findRoute with wormholes excluded', () => {
    it('takes the chain shortcut while wormholes are allowed', () => {
        const route = findRoute(settings(), EXIT, HUB, wormholes, [], []);

        expect(route.jumps).toBe(2);
        expect(route.route.map((step) => step.id)).toEqual([EXIT, HOME, HUB]);
    });

    it('counts only gate jumps once wormholes are excluded', () => {
        const route = findRoute(settings({ useWormholes: false }), EXIT, HUB, wormholes, [], []);

        expect(route.jumps).toBe(3);
        expect(route.route.map((step) => step.id)).toEqual([EXIT, NEIGHBOUR, MIDPOINT, HUB]);
        expect(route.route.every((step) => step.via !== 'wormhole')).toBe(true);
    });

    it('leaves a system unreachable when only a wormhole led there', () => {
        const route = findRoute(settings({ useWormholes: false }), EXIT, HOME, wormholes, [], []);

        expect(route.route).toEqual([]);
        expect(route.jumps).toBe(0);
    });

    it('keeps EVE Scout on its own switch', () => {
        const thera: RoutingConnection[] = [{ from: EXIT, to: HUB, type: 'evescout', massStatus: 'fresh', lifetimeStatus: 'healthy' }];

        const route = findRoute(settings({ useWormholes: false, useEveScout: true }), EXIT, HUB, wormholes, thera, []);

        expect(route.route.map((step) => step.via)).toEqual([null, 'evescout']);
    });
});

describe('findClosestSystems with wormholes excluded', () => {
    it('does not reach systems that only the chain connects', () => {
        const found = findClosestSystems(settings({ useWormholes: false }), EXIT, 'npc_stations', 10, wormholes, [], []);

        expect(found.map((entry) => entry.solarsystem_id)).not.toContain(HOME);
    });
});

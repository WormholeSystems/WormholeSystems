import { describe, expect, it } from 'vitest';

import type { StationGroup } from '../../../state/route-planner.svelte';
import { byFaction, factionOptions, matchesOwner, stationFilter } from './find-stations';

const CALDARI = { id: 500001, name: 'Caldari State' };
const GALLENTE = { id: 500004, name: 'Gallente Federation' };

function corp(
	id: number,
	name: string,
	faction: { id: number; name: string } | null,
	stations: [number, number, string][],
): StationGroup {
	const stationsBySystem = new Map<number, { id: number; name: string }[]>();
	for (const [system, stationId, stationName] of stations) {
		stationsBySystem.set(system, [
			...(stationsBySystem.get(system) ?? []),
			{ id: stationId, name: stationName },
		]);
	}
	return { id, name, faction, systems: new Set(stationsBySystem.keys()), stationsBySystem };
}

const navy = corp(1000035, 'Caldari Navy', CALDARI, [
	[30000142, 1, 'Jita 4-4'],
	[30000144, 2, 'Perimeter Navy'],
]);
const lai = corp(1000020, 'Lai Dai', CALDARI, [[30000142, 3, 'Jita Lai Dai']]);
const roden = corp(1000102, 'Roden Shipyards', GALLENTE, [[30002659, 4, 'Dodixie Roden']]);
const pirates = corp(1000127, 'Guristas', null, [[30004978, 5, 'Hideout']]);
const CORPS = [roden, navy, lai, pirates];

const repair = corp(7, 'Repair Facilities', null, [
	[30000142, 1, 'Jita 4-4'],
	[30002659, 4, 'Dodixie Roden'],
]);

describe('factionOptions', () => {
	it('names each faction once, alphabetically', () => {
		expect(factionOptions(CORPS)).toEqual([CALDARI, GALLENTE]);
	});
});

describe('byFaction', () => {
	it('sorts by faction, then name, factionless last', () => {
		expect(byFaction(CORPS).map((c) => c.name)).toEqual([
			'Caldari Navy',
			'Lai Dai',
			'Roden Shipyards',
			'Guristas',
		]);
	});
});

describe('matchesOwner', () => {
	it('matches the corporation or its faction by name', () => {
		expect(matchesOwner(navy, 'navy')).toBe(true);
		expect(matchesOwner(lai, 'caldari')).toBe(true);
		expect(matchesOwner(pirates, 'caldari')).toBe(false);
	});
});

describe('stationFilter', () => {
	it('is null with nothing picked: every station matches, nothing to expand', () => {
		expect(stationFilter(null, null, CORPS, [repair])).toBeNull();
	});

	it('answers a lone service or corporation with its own stations', () => {
		expect(stationFilter(null, 7, CORPS, [repair])).toBe(repair);
		const alone = stationFilter({ kind: 'corp', id: navy.id }, null, CORPS, [repair]);
		expect(alone?.systems).toEqual(navy.systems);
		expect(alone?.stationsBySystem).toEqual(navy.stationsBySystem);
	});

	it('matches nothing for a dangling owner pick, corp or faction alike', () => {
		expect(stationFilter({ kind: 'corp', id: 999 }, null, CORPS, [repair])?.systems.size).toBe(0);
		expect(stationFilter({ kind: 'faction', id: 999 }, null, CORPS, [repair])?.systems.size).toBe(
			0,
		);
	});

	it('merges a faction from its member corporations', () => {
		const caldari = stationFilter({ kind: 'faction', id: CALDARI.id }, null, CORPS, [repair]);
		expect(caldari?.name).toBe('Caldari State');
		expect(caldari?.systems).toEqual(new Set([30000142, 30000144]));
		expect(caldari?.stationsBySystem.get(30000142)?.map((s) => s.name)).toEqual([
			'Jita 4-4',
			'Jita Lai Dai',
		]);
	});

	it('intersects owner and service at the station level', () => {
		const both = stationFilter({ kind: 'corp', id: navy.id }, 7, CORPS, [repair]);
		expect(both?.systems).toEqual(new Set([30000142]));
		expect(both?.stationsBySystem.get(30000142)?.map((s) => s.name)).toEqual(['Jita 4-4']);
	});
});

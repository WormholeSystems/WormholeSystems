// The map the landing page's screenshots are taken from: a C5 home with a chain hanging off
// it the way a real evening looks. Everything is written through the API (as the people on
// the map) or straight into the tables the background jobs would fill, so nothing here
// needs a live ESI.
import { request, type APIRequestContext, type APIResponse } from '@playwright/test';

import { addSystem, connect, createMap } from '../arrange';
import {
	clearKillmails,
	ensureIdentity,
	grantAccess,
	seedAggressor,
	seedKillmail,
	setThreat,
	withDb,
} from '../db';

const API_BASE = 'http://127.0.0.1:3000';

export const MAP_NAME = 'Nightfall Home Chain';

// Systems, by their SDE ids.
export const SYS = {
	home: 31002238, // J115405, C5, static H296
	s1: 31001882, // J122515, C5 Wolf-Rayet
	s1a: 31001725, // J123746, C4
	thera: 31000005,
	b: 31002515, // J005482, C2 Black Hole, statics A239 / B274
	ikuchi: 30000138, // high-sec, one gate from Jita
	amamake: 30002537, // low-sec
	c: 31001018, // J104351, C3, static U210
	nullsec: 30001161, // HED-GP
} as const;

export const JITA = 30000142;

const SHIP = {
	tengu: 29984,
	loki: 29990,
	proteus: 29988,
	legion: 29986,
	sabre: 22456,
	stratios: 33470,
	buzzard: 11192,
	astero: 33468,
	gila: 17715,
	orca: 28606,
	heron: 605,
	nestor: 33472,
	vindicator: 17740,
	cheetah: 11182,
	capsule: 670,
	moros: 19724,
	ikitursa: 52252,
} as const;

// Everyone here is made up. Their ids sit in a range of their own, clear of the e2e
// suite's slots, but ids like these can belong to real players on Tranquility, so the shots
// never show the image server's picture for them: `PORTRAITS` swaps in NPC agents, and
// `LOGOS` a plain emblem drawn on the spot.
const CORP = { id: 98999701, name: 'Nightfall Survey Group', ticker: 'NSRVY' };
const ALLIANCE = { id: 99999701, name: 'Quiet Horizon Collective', ticker: 'QHZN' };
const FRIENDLY_CORP = { id: 98999702, name: 'Lantern Deep Industries', ticker: 'LNTRN' };
const FRIENDLY_ALLIANCE = { id: 99999702, name: 'Outer Drift Compact', ticker: 'DRIFT' };
const HOSTILE_CORP = { id: 98999711, name: 'Sable Meridian', ticker: 'SBLM' };
const HOSTILE_ALLIANCE = { id: 99999711, name: 'Blackwater Recursion', ticker: 'BWREC' };

const PILOTS = [
	{ characterId: 91999701, name: 'Kaelen Voss', portrait: 3016120 },
	{ characterId: 91999702, name: 'Mira Okonkwo-Hale', portrait: 3015214 },
	{ characterId: 91999703, name: 'Tavish Rendal', portrait: 3012345 },
	{ characterId: 91999704, name: 'Ysolde Varkonen', portrait: 3011560 },
	{ characterId: 91999705, name: 'Daro Teskin', portrait: 3013777 },
	{ characterId: 91999706, name: 'Ione Castellan', portrait: 3009841 },
	{ characterId: 91999707, name: 'Brannoc Hale', portrait: 3017800 },
] as const;

// People the access screen names who are not flying tonight.
const OUTSIDERS = [
	{ characterId: 91999711, name: 'Sefa Lindqvist', portrait: 3018690 },
	{ characterId: 91999712, name: 'Corvin Ashgrove', portrait: 3008416 },
] as const;

const HOSTILES = [
	{ characterId: 91999721, name: 'Vex Anduran' },
	{ characterId: 91999722, name: 'Halric Mour' },
] as const;

/** Demo character id → an NPC agent's portrait. Anything else gets the blank one. */
export const PORTRAITS = new Map<number, number>(
	[...PILOTS, ...OUTSIDERS].map((p) => [p.characterId, p.portrait]),
);

/** Demo corporation and alliance ids → the colour of the plain emblem drawn for them. */
export const LOGOS = new Map<number, string>([
	[CORP.id, '#f59e0b'],
	[ALLIANCE.id, '#38bdf8'],
	[FRIENDLY_CORP.id, '#34d399'],
	[FRIENDLY_ALLIANCE.id, '#a78bfa'],
	[HOSTILE_CORP.id, '#f43f5e'],
	[HOSTILE_ALLIANCE.id, '#fb7185'],
]);

// Kills the threat card and the killmail card show, in ids nothing real will use.
const KILL_IDS = Array.from({ length: 9 }, (_, i) => 990000101 + i);

// Home's scanner, with an age for each row. JYC-883 is only half scanned.
const HOME_SIGNATURES: { id: string; group: string; name?: string; age: number }[] = [
	{ id: 'HKT-412', group: 'wormhole', age: 310 },
	{ id: 'QWE-208', group: 'wormhole', age: 205 },
	{ id: 'ZNA-771', group: 'wormhole', age: 96 },
	{ id: 'LRP-530', group: 'data', name: 'Unsecured Frontier Enclave Relay', age: 1450 },
	{ id: 'MVF-114', group: 'gas', name: 'Sizeable Perimeter Reservoir', age: 2210 },
	{ id: 'OUX-962', group: 'relic', name: 'Forgotten Frontier Recursive Depot', age: 830 },
	{ id: 'TBD-047', group: 'combat', name: 'Core Garrison', age: 2900 },
	{ id: 'JYC-883', group: 'unknown', age: 41 },
];

export interface Pilot {
	characterId: number;
	name: string;
	session: string;
	userId: number;
	api: APIRequestContext;
}

export interface DemoMap {
	mapId: number;
	/** The calm map the hero is taken from; the busy one above is for the feature shots. */
	heroMapId: number;
	viewer: Pilot;
	pilots: Pilot[];
	nodes: Record<keyof typeof SYS, number>;
	connections: Record<string, number>;
}

async function ok(res: APIResponse, what: string) {
	if (!res.ok()) throw new Error(`${what}: ${res.status()} ${await res.text()}`);
	return res;
}

async function seedOrganisations() {
	await withDb(async (db) => {
		for (const a of [ALLIANCE, FRIENDLY_ALLIANCE]) {
			await db.query(
				`insert into alliances (id, name, ticker) values ($1, $2, $3)
				 on conflict (id) do update set name = excluded.name, ticker = excluded.ticker`,
				[a.id, a.name, a.ticker],
			);
		}
		for (const [c, a] of [
			[CORP, ALLIANCE],
			[FRIENDLY_CORP, FRIENDLY_ALLIANCE],
		] as const) {
			await db.query(
				`insert into corporations (id, name, ticker, alliance_id) values ($1, $2, $3, $4)
				 on conflict (id) do update set name = excluded.name, ticker = excluded.ticker,
				     alliance_id = excluded.alliance_id`,
				[c.id, c.name, c.ticker, a.id],
			);
		}
	});
}

async function seedPilot(p: { characterId: number; name: string }): Promise<Pilot> {
	const session = `demo-shots-${p.characterId}`;
	const identity = await ensureIdentity({
		characterId: p.characterId,
		session,
		name: p.name,
		ownerHash: session,
	});
	await withDb((db) =>
		db.query(
			'update characters set name = $2, corporation_id = $3, alliance_id = $4 where id = $1',
			[p.characterId, p.name, CORP.id, ALLIANCE.id],
		),
	);
	const api = await request.newContext({
		baseURL: API_BASE,
		extraHTTPHeaders: { cookie: `ws_session=${session}` },
	});
	return { ...identity, api };
}

/** Put a pilot in a system, flying something, and sharing it with the map. */
async function place(mapId: number, pilot: Pilot, systemId: number, shipTypeId: number) {
	await withDb(async (db) => {
		const scope = await db.query(
			`insert into esi_scopes (name) values ('esi-location.read_location.v1')
			 on conflict (name) do update set name = excluded.name returning id`,
		);
		await db.query('delete from esi_tokens where character_id = $1', [pilot.characterId]);
		const token = await db.query(
			`insert into esi_tokens (character_id, refresh_token) values ($1, 'demo-token') returning id`,
			[pilot.characterId],
		);
		await db.query(
			'insert into esi_token_scopes (token_id, scope_id) values ($1, $2) on conflict do nothing',
			[token.rows[0].id, scope.rows[0].id],
		);
		await db.query(
			`insert into character_status (character_id, online, solar_system_id, ship_type_id,
			     ship_name, station_id, last_online_at)
			 values ($1, true, $2, $3, $4, null, now())
			 on conflict (character_id) do update set
			     online = true, solar_system_id = excluded.solar_system_id,
			     ship_type_id = excluded.ship_type_id, ship_name = excluded.ship_name,
			     station_id = null, updated_at = now()`,
			[pilot.characterId, systemId, shipTypeId, `${pilot.name.split(' ')[0]}'s ship`],
		);
		await db.query(
			`insert into map_user_settings (map_id, user_id, tracking_allowed, introduction_confirmed_at)
			 values ($1, $2, true, now())
			 on conflict (map_id, user_id) do update set tracking_allowed = true,
			     introduction_confirmed_at = now()`,
			[mapId, pilot.userId],
		);
	});
}

/** Remove what an earlier run left behind, so every run starts from the same place. */
export async function clearDemo() {
	const ids = [...PILOTS, ...OUTSIDERS].map((p) => p.characterId);
	await withDb(async (db) => {
		await db.query(
			`delete from maps where id in (
				 select map_id from map_access
				 where subject_type = 'character' and subject_id = any($1) and role = 'owner'
			 )`,
			[ids],
		);
		// Nobody is flying them between runs, so nothing should poll for them either.
		await db.query('delete from esi_tokens where character_id = any($1)', [ids]);
		await db.query('delete from character_status where character_id = any($1)', [ids]);
	});
	await clearKillmails(KILL_IDS);
	await setThreat(SYS.s1, 'unknown', []);
}

export async function seedDemoMap(): Promise<DemoMap> {
	await clearDemo();
	await seedOrganisations();

	const pilots: Pilot[] = [];
	for (const p of PILOTS) pilots.push(await seedPilot(p));
	for (const p of OUTSIDERS) await seedPilot(p);
	const [viewer, scout, tackle, hunter, explorer, hauler, brawler] = pilots;
	const api = viewer.api;

	const mapId = await createMap(api, MAP_NAME);
	for (const p of pilots.slice(1)) await grantAccess(mapId, p.characterId, 'member');

	const post = async (path: string, data: object) =>
		ok(await api.post(`/api/maps/${mapId}/${path}`, { data: { map_id: mapId, ...data } }), path);

	// A tree that reads left to right: home, its holes, theirs.
	// Sized for the map tile in the shots, clear of the rally banner in its top right.
	const col = (n: number) => 30 + n * 270;
	const nodes: Record<keyof typeof SYS, number> = {
		home: await addSystem(api, mapId, SYS.home, { x: col(0), y: 280, alias: 'Home' }),
		s1: await addSystem(api, mapId, SYS.s1, { x: col(1), y: 120, alias: 'S1' }),
		b: await addSystem(api, mapId, SYS.b, { x: col(1), y: 300, alias: 'B' }),
		c: await addSystem(api, mapId, SYS.c, { x: col(1), y: 435, alias: 'C' }),
		s1a: await addSystem(api, mapId, SYS.s1a, { x: col(2), y: 75, alias: 'S1a' }),
		thera: await addSystem(api, mapId, SYS.thera, { x: col(2), y: 165 }),
		ikuchi: await addSystem(api, mapId, SYS.ikuchi, { x: col(2), y: 255 }),
		amamake: await addSystem(api, mapId, SYS.amamake, { x: col(2), y: 345 }),
		nullsec: await addSystem(api, mapId, SYS.nullsec, { x: col(2), y: 435 }),
	};

	const link = (from: number, to: number, size: 'xl' | 'large' | 'medium' | 'small') =>
		connect(api, mapId, from, to, { size });
	const connections = {
		homeS1: await link(nodes.home, nodes.s1, 'xl'),
		homeB: await link(nodes.home, nodes.b, 'large'),
		homeC: await link(nodes.home, nodes.c, 'large'),
		s1S1a: await link(nodes.s1, nodes.s1a, 'large'),
		s1Thera: await link(nodes.s1, nodes.thera, 'medium'),
		bIkuchi: await link(nodes.b, nodes.ikuchi, 'large'),
		bAmamake: await link(nodes.b, nodes.amamake, 'large'),
		cNull: await link(nodes.c, nodes.nullsec, 'large'),
	};

	// Who went through home's static, each as themselves. First, so the history ends on the
	// scanning and naming that came after.
	const jumps: [Pilot, 'outbound' | 'inbound', number][] = [
		[scout, 'outbound', SHIP.buzzard],
		[brawler, 'outbound', SHIP.loki],
		[brawler, 'inbound', SHIP.loki],
		[viewer, 'outbound', SHIP.tengu],
		[hauler, 'outbound', SHIP.orca],
		// Rolling it: out and back in a dread.
		[brawler, 'outbound', SHIP.moros],
		[brawler, 'inbound', SHIP.moros],
		[hauler, 'inbound', SHIP.orca],
		[viewer, 'inbound', SHIP.tengu],
	];
	for (const [who, direction, ship] of jumps) {
		await ok(
			await who.api.post(`/api/maps/${mapId}/connections/jumps/add`, {
				data: { map_id: mapId, connection_id: connections.homeS1, direction, ship_type_id: ship },
			}),
			'jump',
		);
	}

	const setConn = (connection_id: number, state: object) =>
		post('connections/set-status', { connection_id, ...state });
	await setConn(connections.homeS1, { mass_status: 'critical' });
	await setConn(connections.homeC, { mass_status: 'reduced' });
	await setConn(connections.s1Thera, { time_status: 'eol' });
	await setConn(connections.bAmamake, { time_status: 'eol' });
	await setConn(connections.cNull, { time_status: 'critical', mass_status: 'critical' });

	// Home's signatures, and the far side of its static.
	await post('signatures/paste', {
		solar_system_id: SYS.home,
		signatures: HOME_SIGNATURES.map(({ id, group, name }) => ({ signature_id: id, group, name })),
	});
	await post('signatures/paste', {
		solar_system_id: SYS.s1,
		signatures: [{ signature_id: 'VKD-301', group: 'wormhole' }],
	});
	const catalog: { types: { id: number; name: string }[] } = await (
		await api.get('/api/signature-types')
	).json();
	const sigs: { id: number; signature_id: string }[] = await (
		await api.get(`/api/maps/${mapId}/signatures`)
	).json();
	const typed: [string, string, number][] = [
		['HKT-412', 'H296 - C5', connections.homeS1],
		['VKD-301', 'K162 - C5', connections.homeS1],
		['QWE-208', 'K162 - C2', connections.homeB],
		['ZNA-771', 'K162 - C3', connections.homeC],
	];
	for (const [code, type, connection_id] of typed) {
		const signature_pk = sigs.find((s) => s.signature_id === code)!.id;
		const signature_type_id = catalog.types.find((t) => t.name === type)!.id;
		await post('signatures/update', { signature_pk, signature_type_id });
		await post('signatures/link', { signature_pk, connection_id });
	}

	// What the others called out as they went, so the history has more than one name in it.
	const as = async (who: Pilot, path: string, data: object) =>
		ok(
			await who.api.post(`/api/maps/${mapId}/${path}`, { data: { map_id: mapId, ...data } }),
			path,
		);
	const status = (who: Pilot, node: number, s: string) =>
		as(who, 'systems/set-status', { map_solar_system_id: node, status: s });
	await post('systems/set-home', { map_solar_system_id: nodes.home, value: true });
	await post('systems/set-pinned', { map_solar_system_id: nodes.home, value: true });
	await status(viewer, nodes.home, 'friendly');
	await status(explorer, nodes.s1a, 'empty');
	await status(hunter, nodes.ikuchi, 'empty');
	await status(viewer, nodes.c, 'unscanned');
	await status(hunter, nodes.amamake, 'hostile');
	await status(tackle, nodes.b, 'active');
	await as(tackle, 'systems/set-rally', { map_solar_system_id: nodes.b, value: true });
	await status(scout, nodes.s1, 'hostile');
	await as(scout, 'systems/set-occupier', {
		map_solar_system_id: nodes.s1,
		occupier: HOSTILE_CORP.name,
	});
	await as(scout, 'systems/set-notes', {
		map_solar_system_id: nodes.s1,
		notes: [
			'**Sable Meridian** live here. Fortizar on P4-M2, two Astrahus on P1.',
			'',
			'- Active 18:00 to 23:00, Legions and a Loki scout',
			'- Roll their static when they see us on d-scan',
			'- Bring nothing you cannot afford to lose',
		].join('\n'),
	});

	// Spread the evening out, instead of all of it having happened in the last second.
	await withDb(async (db) => {
		await db.query(
			`with ordered as (
				 select id, row_number() over (order by id desc) as n from map_events where map_id = $1
			 )
			 update map_events e set created_at = now() - make_interval(mins => (o.n * 3)::int)
			 from ordered o where o.id = e.id`,
			[mapId],
		);
		await db.query(
			`with ordered as (
				 select id, row_number() over (order by id desc) as n
				 from map_connection_jumps where map_id = $1
			 )
			 update map_connection_jumps j
			 set created_at = now() - make_interval(mins => (o.n * 11)::int)
			 from ordered o where o.id = j.id`,
			[mapId],
		);
		for (const s of HOME_SIGNATURES) {
			await db.query(
				`update signatures set created_at = now() - make_interval(mins => $3),
				     updated_at = now() - make_interval(mins => $3)
				 where map_id = $1 and signature_id = $2`,
				[mapId, s.id, s.age],
			);
		}
	});

	await place(mapId, viewer, SYS.home, SHIP.tengu);
	await place(mapId, scout, SYS.s1a, SHIP.buzzard);
	await place(mapId, tackle, SYS.b, SHIP.sabre);
	await place(mapId, hunter, SYS.ikuchi, SHIP.proteus);
	await place(mapId, explorer, SYS.thera, SHIP.astero);
	await place(mapId, hauler, SYS.home, SHIP.orca);
	await place(mapId, brawler, SYS.home, SHIP.loki);

	// What has been dying next door.
	for (const h of HOSTILES) {
		await seedAggressor({
			characterId: h.characterId,
			name: h.name,
			corporationId: HOSTILE_CORP.id,
			corporationName: HOSTILE_CORP.name,
			corporationTicker: HOSTILE_CORP.ticker,
			allianceId: HOSTILE_ALLIANCE.id,
			allianceName: HOSTILE_ALLIANCE.name,
			allianceTicker: HOSTILE_ALLIANCE.ticker,
		});
	}
	const kills = [
		{ ship: SHIP.gila, value: 412_000_000, attackers: 6, mins: 18, at: SYS.s1 },
		{ ship: SHIP.stratios, value: 386_000_000, attackers: 1, mins: 47, at: SYS.s1, solo: true },
		{ ship: SHIP.legion, value: 1_240_000_000, attackers: 9, mins: 95, at: SYS.s1 },
		{ ship: SHIP.heron, value: 21_000_000, attackers: 1, mins: 130, at: SYS.s1, solo: true },
		{ ship: SHIP.capsule, value: 260_000, attackers: 3, mins: 132, at: SYS.s1 },
		{ ship: SHIP.vindicator, value: 3_100_000_000, attackers: 14, mins: 240, at: SYS.amamake },
		{ ship: SHIP.nestor, value: 640_000_000, attackers: 5, mins: 300, at: SYS.s1 },
		{ ship: SHIP.cheetah, value: 64_000_000, attackers: 2, mins: 420, at: SYS.b },
		{ ship: SHIP.ikitursa, value: 980_000_000, attackers: 7, mins: 610, at: SYS.s1 },
	];
	for (const [i, k] of kills.entries()) {
		const aggressor = HOSTILES[i % HOSTILES.length];
		await seedKillmail({
			id: KILL_IDS[i],
			solarSystemId: k.at,
			minutesAgo: k.mins,
			victimShipTypeId: k.ship,
			totalValue: k.value,
			attackerCount: k.attackers,
			isSolo: k.solo ?? false,
			finalBlowShipTypeId: i % 2 ? SHIP.legion : SHIP.loki,
			finalBlowCharacterId: aggressor.characterId,
			finalBlowCorporationId: HOSTILE_CORP.id,
			finalBlowAllianceId: HOSTILE_ALLIANCE.id,
		});
	}
	await setThreat(SYS.s1, 'critical', [
		{ id: HOSTILE_ALLIANCE.id, type: 'alliance', name: HOSTILE_ALLIANCE.name, kills: 74 },
		{ id: HOSTILE_CORP.id, type: 'corporation', name: HOSTILE_CORP.name, kills: 51 },
	]);

	// Access: the corp, a friendly alliance, and two outsiders, one of them for a few days.
	const access = (subject_type: string, subject_id: number, role: string, expires_at?: string) =>
		post('access/set', { subject_type, subject_id, role, expires_at });
	await access('corporation', CORP.id, 'member');
	await access('alliance', FRIENDLY_ALLIANCE.id, 'viewer');
	await access('character', OUTSIDERS[0].characterId, 'manager');
	await access(
		'character',
		OUTSIDERS[1].characterId,
		'viewer',
		new Date(Date.now() + 3 * 24 * 3600_000).toISOString(),
	);

	// Discord: two channels and the alerts that post to them.
	const webhook = async (name: string) => {
		const res = await ok(
			await api.post(`/api/maps/${mapId}/webhooks`, {
				data: { name, url: 'https://discord.com/api/webhooks/1/demo-shots' },
			}),
			'webhook',
		);
		return ((await res.json()) as { id: number }).id;
	};
	const intel = await webhook('#chain-intel');
	const defense = await webhook('#home-defense');
	const alert = async (body: object) =>
		ok(
			await api.post(`/api/maps/${mapId}/alerts`, {
				data: { mention: 'none', filters: [], filter_match: 'any', delivery: 'webhook', ...body },
			}),
			'alert',
		);
	await alert({
		name: 'Kills next door',
		kind: 'killmail',
		map_webhook_id: intel,
		target_solar_system_id: SYS.home,
		max_jumps: 2,
	});
	await alert({
		name: 'Hostiles in the chain',
		kind: 'proximity',
		map_webhook_id: defense,
		mention: 'everyone',
		target_solar_system_id: SYS.home,
		max_jumps: 3,
	});
	await alert({
		name: 'Dreads in range of Amamake',
		kind: 'jump_range',
		map_webhook_id: intel,
		target_solar_system_id: SYS.amamake,
		max_jumps: 0,
		ship_type: 'dreadnought',
		jdc_level: 5,
	});

	const heroMapId = await seedHeroMap(pilots);
	return { mapId, heroMapId, viewer, pilots, nodes, connections };
}

/**
 * A quiet evening for the hero: the same home and the same people, a tidy chain of seven
 * with one hostile next door and one hole at reduced mass, and nothing else shouting. The
 * busy map carries the states the feature shots are about. None of these systems is the
 * one with seeded kills and a threat rating, so no ring shows up here either.
 */
async function seedHeroMap(pilots: Pilot[]): Promise<number> {
	const [viewer] = pilots;
	const api = viewer.api;
	const mapId = await createMap(api, 'Home Chain');
	for (const p of pilots.slice(1)) await grantAccess(mapId, p.characterId, 'member');
	const post = async (path: string, data: object) =>
		ok(await api.post(`/api/maps/${mapId}/${path}`, { data: { map_id: mapId, ...data } }), path);

	const col = (n: number) => 90 + n * 300;
	const home = await addSystem(api, mapId, SYS.home, { x: col(0), y: 280, alias: 'Home' });
	const a = await addSystem(api, mapId, SYS.s1a, { x: col(1), y: 130, alias: 'A' });
	const b = await addSystem(api, mapId, SYS.b, { x: col(1), y: 280, alias: 'B' });
	const c = await addSystem(api, mapId, SYS.c, { x: col(1), y: 430, alias: 'C' });
	const thera = await addSystem(api, mapId, SYS.thera, { x: col(2), y: 130 });
	const ikuchi = await addSystem(api, mapId, SYS.ikuchi, { x: col(2), y: 280 });
	const nullsec = await addSystem(api, mapId, SYS.nullsec, { x: col(2), y: 430 });

	const link = (from: number, to: number, size: 'xl' | 'large' | 'medium') =>
		connect(api, mapId, from, to, { size });
	const homeA = await link(home, a, 'large');
	const homeB = await link(home, b, 'large');
	const homeC = await link(home, c, 'large');
	await link(a, thera, 'medium');
	await link(b, ikuchi, 'large');
	await link(c, nullsec, 'large');
	await post('connections/set-status', { connection_id: homeB, mass_status: 'reduced' });

	await post('systems/set-home', { map_solar_system_id: home, value: true });
	await post('systems/set-status', { map_solar_system_id: a, status: 'hostile' });

	await post('signatures/paste', {
		solar_system_id: SYS.home,
		signatures: HOME_SIGNATURES.filter((s) => s.group !== 'unknown').map(({ id, group, name }) => ({
			signature_id: id,
			group,
			name,
		})),
	});
	const catalog: { types: { id: number; name: string }[] } = await (
		await api.get('/api/signature-types')
	).json();
	const sigs: { id: number; signature_id: string }[] = await (
		await api.get(`/api/maps/${mapId}/signatures`)
	).json();
	const typed: [string, string, number][] = [
		['HKT-412', 'K162 - C4', homeA],
		['QWE-208', 'K162 - C2', homeB],
		['ZNA-771', 'K162 - C3', homeC],
	];
	for (const [code, type, connection_id] of typed) {
		const signature_pk = sigs.find((s) => s.signature_id === code)!.id;
		const signature_type_id = catalog.types.find((t) => t.name === type)!.id;
		await post('signatures/update', { signature_pk, signature_type_id });
		await post('signatures/link', { signature_pk, connection_id });
	}

	await withDb(async (db) => {
		for (const s of HOME_SIGNATURES) {
			await db.query(
				`update signatures set created_at = now() - make_interval(mins => $3),
				     updated_at = now() - make_interval(mins => $3)
				 where map_id = $1 and signature_id = $2`,
				[mapId, s.id, s.age],
			);
		}
		await db.query(
			`update map_connections set created_at = now() - interval '3 hours' where map_id = $1`,
			[mapId],
		);
		// Everyone shares their position on this map too; where they are was set above.
		for (const p of pilots) {
			await db.query(
				`insert into map_user_settings (map_id, user_id, tracking_allowed, introduction_confirmed_at)
				 values ($1, $2, true, now())
				 on conflict (map_id, user_id) do update set tracking_allowed = true,
				     introduction_confirmed_at = now()`,
				[mapId, p.userId],
			);
		}
	});
	return mapId;
}

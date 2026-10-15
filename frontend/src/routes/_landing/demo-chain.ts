import type { MapConnection } from '$lib/api/types/MapConnection';
import type { MappedSystem } from '$lib/map/system';
import type { Signature } from '$lib/api/types/Signature';

/**
 * A small chain for the signature demo, in the same shapes the API serves, so the landing
 * page renders the real signature rows instead of look-alikes that drift away from them.
 *
 * Everything here is invented, but it is invented in the type the server would send: if a
 * field changes shape, this stops compiling rather than quietly rendering something the
 * product no longer looks like.
 */

const MAP_ID = 0;

/**
 * Ages are rendered relative to now, so the fixtures carry offsets rather than fixed dates
 * that would drift into "three years ago". Computed once per render, which is also what
 * keeps the server's markup and the client's first paint agreeing.
 */
const hoursAgo = (h: number) => new Date(Date.now() - h * 3_600_000).toISOString();

function system(
	id: number,
	fields: Partial<MappedSystem> &
		Pick<MappedSystem, 'position_x' | 'position_y' | 'name' | 'region'>,
): MappedSystem {
	return {
		kind: 'system',
		id,
		map_id: MAP_ID,
		solar_system_id: 30000000 + id,
		alias: null,
		is_home: false,
		is_rally: false,
		is_pinned: false,
		signatures_pasted_at: null,
		status: 'unknown',
		occupying_group: null,
		// What the SDE gives every wormhole; the k-space entries below say otherwise.
		security_status: -0.99,
		wormhole_class_id: null,
		region_id: 10000000 + id,
		constellation_id: 20000000 + id,
		constellation: `${fields.region} Constellation`,
		effect_name: null,
		is_shattered: false,
		threat_level: null,
		statics: [],
		sovereignty: null,
		...fields,
	};
}

function statik(name: string, destClass: number): MappedSystem['statics'][number] {
	return {
		code: name,
		dest_class: destClass,
		max_jump_mass: null,
		total_mass: null,
		signature_strength: null,
		lifetime_hours: null,
	} satisfies MappedSystem['statics'][number];
}

const HOME = 1;

/** The system the demo's signatures were scanned in. */
export const HOME_SYSTEM = system(HOME, {
	position_x: 0,
	position_y: 160,
	name: 'Turnur',
	region: 'Metropolis',
	security_status: 0.35,
	is_home: true,
	status: 'friendly',
});

export const DEMO_SYSTEMS: MappedSystem[] = [
	HOME_SYSTEM,
	system(2, {
		position_x: 260,
		position_y: 40,
		name: 'J123746',
		region: 'E-R00028',
		wormhole_class_id: 5,
		status: 'hostile',
		statics: [statik('H296', 5), statik('C140', 8)],
	}),
	system(3, {
		position_x: 260,
		position_y: 300,
		name: 'Korasen',
		region: 'Black Rise',
		security_status: 0.28,
		status: 'active',
	}),
];

function connection(id: number, from: number, to: number, fields: Partial<MapConnection> = {}) {
	return {
		id,
		map_id: MAP_ID,
		from_system: from,
		to_system: to,
		kind: 'wormhole',
		mass_status: null,
		time_status: null,
		size: null,
		preserve_mass: false,
		jumps_count: 0,
		jumps_mass_sum: 0,
		time_status_updated_at: null,
		lifetime_hours: 24,
		created_at: hoursAgo(4),
		updated_at: hoursAgo(1),
		...fields,
	} satisfies MapConnection;
}

export const DEMO_CONNECTIONS: MapConnection[] = [
	connection(12, HOME, 2, { mass_status: 'reduced', jumps_count: 14 }),
	connection(13, HOME, 3, { time_status: 'eol', jumps_count: 3 }),
];

let nextId = 100;

/** A signature in Turnur, as the API would serve it. */
export function demoSignature(fields: Partial<Signature> & Pick<Signature, 'signature_id'>) {
	return {
		id: nextId++,
		map_id: MAP_ID,
		solar_system_id: HOME_SYSTEM.solar_system_id,
		group: 'unknown',
		signature_type_id: null,
		name: null,
		size: null,
		mass_status: null,
		time_status: null,
		connection_id: null,
		time_status_updated_at: null,
		created_at: hoursAgo(0),
		updated_at: hoursAgo(0),
		...fields,
	} satisfies Signature;
}

/**
 * The wormhole types somebody picked for Turnur's holes, by catalog name: catalog ids
 * belong to the install, so they are looked up once the catalog has loaded.
 */
export const DEMO_WORMHOLE_TYPES: Record<string, string> = {
	'ABC-123': 'K162 - C5',
	'DEF-456': 'K162 - Unknown',
};

/** What Turnur's panel held before the scan the demo pastes. */
export function demoSignatures(): Signature[] {
	return [
		demoSignature({
			signature_id: 'ABC-123',
			group: 'wormhole',
			connection_id: 12,
			created_at: hoursAgo(3),
			updated_at: hoursAgo(3),
		}),
		demoSignature({
			signature_id: 'DEF-456',
			group: 'wormhole',
			time_status: 'eol',
			connection_id: 13,
			created_at: hoursAgo(15),
			updated_at: hoursAgo(2),
		}),
		demoSignature({
			signature_id: 'GHI-789',
			created_at: hoursAgo(0.2),
			updated_at: hoursAgo(0.2),
		}),
		demoSignature({
			signature_id: 'JKL-012',
			group: 'data',
			name: 'Unsecured Frontier Enclave Relay',
			created_at: hoursAgo(6),
			updated_at: hoursAgo(6),
		}),
		demoSignature({
			signature_id: 'MNO-345',
			group: 'combat',
			name: 'Core Garrison',
			created_at: hoursAgo(26),
			updated_at: hoursAgo(26),
		}),
	];
}

/**
 * The probe scanner's copy of Turnur an hour later: GHI-789 resolved, the combat site run
 * and gone, a new gas site, and a new signature nobody has scanned down yet. In the
 * client's own tab-separated format, so the demo
 * goes through the same parser a real paste does.
 */
export const DEMO_SCAN = [
	'ABC-123\tCosmic Signature\tWormhole\tUnstable Wormhole\t100.0%\t4.12 AU',
	'DEF-456\tCosmic Signature\tWormhole\tUnstable Wormhole\t100.0%\t11.8 AU',
	'GHI-789\tCosmic Signature\tRelic Site\tForgotten Frontier Quarantine Outpost\t100.0%\t7.30 AU',
	'JKL-012\tCosmic Signature\tData Site\tUnsecured Frontier Enclave Relay\t100.0%\t2.95 AU',
	'PQR-678\tCosmic Signature\tGas Site\tBountiful Frontier Reservoir\t100.0%\t16.4 AU',
	'STU-901\tCosmic Signature\t\t\t21.4%\t9.02 AU',
].join('\n');

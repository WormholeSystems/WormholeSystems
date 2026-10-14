import type { AccessEntry } from '$lib/api/types/AccessEntry';
import type { MapCharacter } from '$lib/api/types/MapCharacter';
import type { MapConnection } from '$lib/api/types/MapConnection';
import type { MappedSystem } from '$lib/map/system';
import type { Signature } from '$lib/api/types/Signature';

/**
 * A small chain, in the same shapes the API serves, so the landing page can render the
 * real map components instead of look-alikes that drift away from them.
 *
 * Everything here is invented, but it is invented in the type the server would send: if a
 * field changes shape, this stops compiling rather than quietly rendering something the
 * product no longer looks like.
 */

const MAP_ID = 0;

// Real entities, so every portrait and logo on the page is the one EVE serves for them.
const OWNER_ID = 95569887; // Nicolas Kion
const CORP_ID = 98630705; // Strix Ridens [OWL.]
const CORP_TICKER = 'OWL.';

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

export const HOME = 1;

/** The chain's home, named so the signature demo can render it without hunting for it. */
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
		name: 'J103512',
		region: 'B-R00004',
		wormhole_class_id: 2,
		status: 'empty',
		statics: [statik('D382', 7), statik('Z647', 1)],
	}),
	system(3, {
		position_x: 260,
		position_y: 160,
		name: 'J123746',
		region: 'E-R00028',
		wormhole_class_id: 5,
		status: 'hostile',
		threat_level: 'high',
		occupying_group: 'OWL.',
		statics: [statik('H296', 5), statik('C140', 8)],
	}),
	system(4, {
		position_x: 260,
		position_y: 300,
		name: 'Korasen',
		region: 'Black Rise',
		security_status: 0.28,
		status: 'active',
	}),
	system(5, {
		position_x: 520,
		position_y: 100,
		name: 'J104351',
		region: 'D-R00016',
		wormhole_class_id: 3,
		status: 'unscanned',
		statics: [statik('X702', 7)],
	}),
	system(6, {
		position_x: 520,
		position_y: 240,
		name: 'Q-XEB3',
		region: 'Fountain',
		security_status: -0.42,
		status: 'unknown',
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
	connection(11, HOME, 2),
	connection(12, HOME, 3, { mass_status: 'reduced', jumps_count: 14, jumps_mass_sum: 620_000_000 }),
	connection(13, HOME, 4, { time_status: 'critical', jumps_count: 3 }),
	connection(14, 3, 5),
	connection(15, 3, 6, { mass_status: 'critical', jumps_count: 22 }),
];

export const DEMO_PILOTS: Record<number, MapCharacter[]> = {
	[HOME]: [
		{
			character_id: OWNER_ID,
			name: 'Nicolas Kion',
			corporation_ticker: CORP_TICKER,
			solar_system_id: 30000001,
			ship_type_id: 11192,
			ship_name: null,
			ship_type: 'Buzzard',
			ship_group_id: null,
			is_docked: false,
			is_mine: true,
		} satisfies MapCharacter,
	],
	3: [
		{
			character_id: 95042605,
			name: 'Tovan Khev',
			corporation_ticker: 'CONC',
			solar_system_id: 30000003,
			ship_type_id: 17738,
			ship_name: null,
			ship_type: 'Machariel',
			ship_group_id: null,
			is_docked: false,
			is_mine: false,
		} satisfies MapCharacter,
	],
};

/** Signatures scanned in Turnur, in the states the panel colours differently. */
export const DEMO_SIGNATURES: Signature[] = [
	{
		id: 101,
		map_id: MAP_ID,
		solar_system_id: 30000001,
		signature_id: 'ABC-123',
		group: 'wormhole',
		signature_type_id: null,
		name: null,
		size: null,
		mass_status: null,
		time_status: null,
		connection_id: 12,
		time_status_updated_at: null,
		created_at: hoursAgo(3),
		updated_at: hoursAgo(3),
	} satisfies Signature,
	{
		id: 102,
		map_id: MAP_ID,
		solar_system_id: 30000001,
		signature_id: 'DEF-456',
		group: 'wormhole',
		signature_type_id: null,
		name: null,
		size: null,
		mass_status: null,
		time_status: 'eol',
		connection_id: 13,
		time_status_updated_at: null,
		created_at: hoursAgo(15),
		updated_at: hoursAgo(2),
	} satisfies Signature,
	{
		id: 103,
		map_id: MAP_ID,
		solar_system_id: 30000001,
		signature_id: 'GHI-789',
		group: 'unknown',
		signature_type_id: null,
		name: null,
		size: null,
		mass_status: null,
		time_status: null,
		connection_id: null,
		time_status_updated_at: null,
		created_at: hoursAgo(0.2),
		updated_at: hoursAgo(0.2),
	} satisfies Signature,
	{
		id: 104,
		map_id: MAP_ID,
		solar_system_id: 30000001,
		signature_id: 'JKL-012',
		group: 'data',
		signature_type_id: null,
		name: null,
		size: null,
		mass_status: null,
		time_status: null,
		connection_id: null,
		time_status_updated_at: null,
		created_at: hoursAgo(6),
		updated_at: hoursAgo(6),
	} satisfies Signature,
];

/**
 * Grants on the demo map. Real ids from ESI, so the portraits and logos are the ones the
 * image server actually serves for these entities.
 */
export const DEMO_ACCESS: AccessEntry[] = [
	{ subject_type: 'character', subject_id: OWNER_ID, name: 'Nicolas Kion', role: 'owner' },
	{ subject_type: 'alliance', subject_id: 99014466, name: 'Strix Ridens.', role: 'manager' },
	{ subject_type: 'corporation', subject_id: CORP_ID, name: 'Strix Ridens', role: 'member' },
	{
		subject_type: 'character',
		subject_id: 95042605,
		name: 'Tovan Khev',
		role: 'viewer',
		expires_at: new Date(Date.now() + 7 * 86_400_000).toISOString(),
	},
];

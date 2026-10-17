// Direct database access for test fixtures. The app has no test-only endpoints, so the
// e2e identity (corp + user + character + session) is created straight in Postgres.
import { Client } from 'pg';

export const E2E_CORPORATION_ID = 98999998;
export const E2E_CHARACTER_ID = 91999998;
export const E2E_SESSION = 'e2e-session';

export async function withDb<T>(f: (client: Client) => Promise<T>): Promise<T> {
	const client = new Client({
		connectionString:
			process.env.DATABASE_URL ??
			'postgres://wormholesystems:wormholesystems@localhost:5432/wormholesystems',
	});
	await client.connect();
	try {
		return await f(client);
	} finally {
		await client.end();
	}
}

export interface Identity {
	characterId: number;
	session: string;
	userId: number;
	name: string;
}

/**
 * Create (or refresh) an identity: corp, user, character and a session. Idempotent, so
 * repeated runs reuse the same rows. The `users` id comes from the identity sequence, never
 * forced, so real logins are unaffected.
 */
export async function ensureIdentity(spec: {
	characterId: number;
	session: string;
	name: string;
	ownerHash: string;
}): Promise<Identity> {
	const userId = await withDb(async (db) => {
		await db.query(
			`insert into corporations (id, name, ticker) values ($1, 'E2E Corp', 'E2E')
			 on conflict do nothing`,
			[E2E_CORPORATION_ID],
		);
		const existing = await db.query('select user_id from characters where id = $1', [
			spec.characterId,
		]);
		let userId: number = existing.rows[0]?.user_id;
		if (userId === undefined) {
			const user = await db.query('insert into users default values returning id');
			userId = user.rows[0].id;
			await db.query(
				`insert into characters (id, user_id, name, owner_hash, corporation_id)
				 values ($1, $2, $3, $4, $5)`,
				[spec.characterId, userId, spec.name, spec.ownerHash, E2E_CORPORATION_ID],
			);
		}
		await db.query(
			`insert into sessions (id, user_id, active_character_id, expires_at)
			 values ($1, $2, $3, now() + interval '2 hours')
			 on conflict (id) do update set expires_at = now() + interval '2 hours'`,
			[spec.session, userId, spec.characterId],
		);
		return userId;
	});
	return { characterId: spec.characterId, session: spec.session, userId, name: spec.name };
}

/** The first character id of the extra identities; `slot` counts up from here. */
export const EXTRA_CHARACTER_BASE = 91999900;

/**
 * Where the per-worker identities of `ownAccountTest` start, clear of the slots specs pick
 * for themselves. Ten of them, one per parallel worker.
 */
export const OWN_ACCOUNT_SLOT = 60;

/**
 * An extra throwaway identity for multi-user tests. Ids are derived from `slot` (0-99);
 * global teardown does not know about most of these, so callers should clean up maps they
 * create, and the rows themselves are reused across runs.
 */
export function createIdentity(slot: number): Promise<Identity> {
	return ensureIdentity({
		characterId: EXTRA_CHARACTER_BASE + slot,
		session: `e2e-extra-${slot}`,
		name: `E2E Extra ${slot}`,
		ownerHash: `e2e-extra-hash-${slot}`,
	});
}

/** Grant a character access to a map directly (no invite flow exists yet). */
export async function grantAccess(
	mapId: number,
	characterId: number,
	role: 'viewer' | 'member' | 'manager' | 'owner',
) {
	await withDb(async (db) => {
		await db.query(
			`insert into map_access (map_id, subject_type, subject_id, role)
			 values ($1, 'character', $2, $3)
			 on conflict (map_id, subject_id) do update set role = excluded.role`,
			[mapId, characterId, role],
		);
	});
	await skipIntroduction(mapId, characterId);
}

/**
 * Mark the map's introduction as already seen by this character's user.
 *
 * It opens as a modal over the whole map for anyone who has not been through it, so every
 * test that drives a map as a second identity needs it out of the way first. `gotoApp`
 * does the same for the main one.
 */
export async function skipIntroduction(mapId: number, characterId: number) {
	await withDb(async (db) => {
		await db.query(
			`insert into map_user_settings (map_id, user_id, introduction_confirmed_at)
			 select $1, c.user_id, now() from characters c where c.id = $2 and c.user_id is not null
			 on conflict (map_id, user_id) do update set introduction_confirmed_at = now()`,
			[mapId, characterId],
		);
	});
}

/** Give a character a tracking-scoped token and an online status in a system. */
export async function setCharacterPresence(
	characterId: number,
	solarSystemId: number,
	online = true,
) {
	await withDb(async (db) => {
		const scope = await db.query(
			`insert into esi_scopes (name) values ('esi-location.read_location.v1')
			 on conflict (name) do update set name = excluded.name returning id`,
		);
		const token = await db.query(
			`insert into esi_tokens (character_id, refresh_token) values ($1, 'e2e-token')
			 returning id`,
			[characterId],
		);
		await db.query(
			`insert into esi_token_scopes (token_id, scope_id) values ($1, $2)
			 on conflict do nothing`,
			[token.rows[0].id, scope.rows[0].id],
		);
		await db.query(
			`insert into character_status (character_id, online, solar_system_id, last_online_at)
			 values ($1, $2, $3, now())
			 on conflict (character_id) do update set
			     online = excluded.online, solar_system_id = excluded.solar_system_id,
			     updated_at = now()`,
			[characterId, online, solarSystemId],
		);
		// Online only lasts while the owner has the app open: the poller marks an idle user's
		// characters offline on its next tick, which would pull the pilot mid-test.
		if (online) {
			await db.query(
				`update users set last_active_at = now()
				 where id = (select user_id from characters where id = $1)`,
				[characterId],
			);
		}
	});
}

/** Set a user's per-map tracking opt-in directly. */
export async function setTrackingAllowed(mapId: number, userId: number, allowed: boolean) {
	await withDb(async (db) => {
		await db.query(
			`insert into map_user_settings (map_id, user_id, tracking_allowed)
			 values ($1, $2, $3)
			 on conflict (map_id, user_id) do update set tracking_allowed = excluded.tracking_allowed`,
			[mapId, userId, allowed],
		);
	});
}

/** Write a wormhole system's threat analysis directly (as the daily batch would). */
export async function setThreat(
	solarSystemId: number,
	level: 'unknown' | 'high' | 'critical',
	entities: { id: number; type: 'alliance' | 'corporation'; name: string; kills: number }[],
) {
	await withDb(async (db) => {
		await db.query(
			`update wormhole_systems set threat_level = $2, threat_analyzed_at = now()
			 where solar_system_id = $1`,
			[solarSystemId, level],
		);
		await db.query('delete from wormhole_system_threats where solar_system_id = $1', [
			solarSystemId,
		]);
		for (const e of entities) {
			await db.query(
				`insert into wormhole_system_threats (solar_system_id, entity_id, entity_type, name, kills)
				 values ($1, $2, $3, $4, $5)`,
				[solarSystemId, e.id, e.type, e.name, e.kills],
			);
		}
	});
}

/** Backdate a map's critical connections so they count as stale. */
export async function ageStaleConnections(mapId: number) {
	await withDb(async (db) => {
		await db.query(
			`update map_connections set time_status_updated_at = now() - interval '2 hours'
			 where map_id = $1 and time_status = 'critical'`,
			[mapId],
		);
	});
}

/** Make one of a user's characters the acting one, as the character switcher would. */
export async function setActiveCharacter(characterId: number, session = E2E_SESSION) {
	await withDb((db) =>
		db.query('update sessions set active_character_id = $1 where id = $2', [characterId, session]),
	);
}

const LOCATION_SCOPES = [
	'esi-location.read_location.v1',
	'esi-location.read_ship_type.v1',
	'esi-location.read_online.v1',
];

/**
 * Put an organisation on a wormhole system's threat list, as the killmail analysis would.
 *
 * Seeded rather than analysed: the analysis needs a quarter of real killmails to say
 * anything, and a test should not depend on who happened to be shooting whom.
 */
export async function seedThreat(threat: {
	solarSystemId: number;
	entityId: number;
	entityType: 'alliance' | 'corporation';
	name: string;
	kills: number;
}) {
	await withDb((db) =>
		db.query(
			`insert into wormhole_system_threats (solar_system_id, entity_id, entity_type, name, kills)
			 values ($1, $2, $3, $4, $5)`,
			[threat.solarSystemId, threat.entityId, threat.entityType, threat.name, threat.kills],
		),
	);
}

export async function clearThreats(entityIds: number[]) {
	await withDb((db) =>
		db.query('delete from wormhole_system_threats where entity_id = any($1)', [entityIds]),
	);
}

/** Replace a character's granted ESI scopes with exactly `names`. */
export async function setScopes(characterId: number, names: string[]) {
	await withDb(async (db) => {
		await db.query('delete from esi_tokens where character_id = $1', [characterId]);
		const token = await db.query(
			`insert into esi_tokens (character_id, refresh_token) values ($1, 'e2e-token')
			 returning id`,
			[characterId],
		);
		for (const name of names) {
			const scope = await db.query(
				`insert into esi_scopes (name) values ($1)
				 on conflict (name) do update set name = excluded.name returning id`,
				[name],
			);
			await db.query(
				`insert into esi_token_scopes (token_id, scope_id) values ($1, $2)
				 on conflict do nothing`,
				[token.rows[0].id, scope.rows[0].id],
			);
		}
	});
}

/** Undo `skipIntroduction`, so a granted identity meets the walkthrough after all. */
export async function showIntroduction(mapId: number, characterId: number) {
	await withDb(async (db) => {
		await db.query(
			`update map_user_settings set introduction_confirmed_at = null
			 where map_id = $1
			   and user_id = (select user_id from characters where id = $2)`,
			[mapId, characterId],
		);
	});
}

/**
 * Give a character a token the tracking poller will actually use: unexpired, and carrying
 * the three location scopes. Without the far-future expiry the poller tries to refresh it
 * against the real SSO and gives up on the character.
 */
export async function grantLocationScopes(characterId: number) {
	await withDb(async (db) => {
		await db.query('delete from esi_tokens where character_id = $1', [characterId]);
		const token = await db.query(
			`insert into esi_tokens (character_id, access_token, token_expires_at, refresh_token)
			 values ($1, 'e2e-access-token', now() + interval '1 day', 'e2e-refresh-token')
			 returning id`,
			[characterId],
		);
		for (const name of LOCATION_SCOPES) {
			const scope = await db.query(
				`insert into esi_scopes (name) values ($1)
				 on conflict (name) do update set name = excluded.name returning id`,
				[name],
			);
			await db.query(
				'insert into esi_token_scopes (token_id, scope_id) values ($1, $2) on conflict do nothing',
				[token.rows[0].id, scope.rows[0].id],
			);
		}
	});
}

/**
 * Flag a character online without giving them a location, which is what the 60s tier-1
 * poll does. Tier 2 (every 5s) only looks at characters already flagged online, so this is
 * how a test opts into the fast loop without waiting a minute for the slow one.
 */
export async function setCharacterOnline(characterId: number) {
	await withDb((db) =>
		db.query(
			`insert into character_status (character_id, online, solar_system_id, last_online_at)
			 values ($1, true, null, now())
			 on conflict (character_id) do update set
			     online = true, solar_system_id = null, last_online_at = now(), updated_at = now()`,
			[characterId],
		),
	);
}

/**
 * Mark a user as recently active, as their own open tab would via the socket heartbeat.
 * The tracking poller ignores users who have not been seen in five minutes, so a pilot
 * nobody is watching from is never polled.
 */
export async function markUserActive(userId: number) {
	await withDb((db) => db.query('update users set last_active_at = now() where id = $1', [userId]));
}

/** A killmail as the ingest would have stored it, for tests that need one to exist. */
export async function seedKillmail(kill: {
	id: number;
	solarSystemId: number;
	minutesAgo: number;
	victimShipTypeId: number;
	totalValue: number;
	attackerCount: number;
	isSolo?: boolean;
	isNpc?: boolean;
	victimCharacterId?: number;
	finalBlowShipTypeId?: number;
	/** The aggressor, for the columns the card shows when it has the room. */
	finalBlowCharacterId?: number;
	finalBlowCorporationId?: number;
	finalBlowAllianceId?: number;
}) {
	await withDb((db) =>
		db.query(
			`insert into killmails (
				 id, hash, solar_system_id, time, orgs,
				 victim_character_id, victim_ship_type_id, total_value, attacker_count,
				 is_solo, is_npc, final_blow_ship_type_id,
				 final_blow_character_id, final_blow_corporation_id, final_blow_alliance_id
			 )
			 values ($1, 'e2e-hash', $2, now() - make_interval(mins => $3), '[]'::jsonb,
			         $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
			 on conflict (id) do update set
			     solar_system_id = excluded.solar_system_id,
			     time = excluded.time,
			     victim_ship_type_id = excluded.victim_ship_type_id,
			     total_value = excluded.total_value,
			     attacker_count = excluded.attacker_count,
			     is_solo = excluded.is_solo,
			     is_npc = excluded.is_npc,
			     final_blow_character_id = excluded.final_blow_character_id,
			     final_blow_corporation_id = excluded.final_blow_corporation_id,
			     final_blow_alliance_id = excluded.final_blow_alliance_id`,
			[
				kill.id,
				kill.solarSystemId,
				kill.minutesAgo,
				kill.victimCharacterId ?? null,
				kill.victimShipTypeId,
				kill.totalValue,
				kill.attackerCount,
				kill.isSolo ?? false,
				kill.isNpc ?? false,
				kill.finalBlowShipTypeId ?? null,
				kill.finalBlowCharacterId ?? null,
				kill.finalBlowCorporationId ?? null,
				kill.finalBlowAllianceId ?? null,
			],
		),
	);
}

/**
 * A pilot nobody signed in as, with a corp and an alliance: what the killmail resolver
 * writes for an aggressor, and what the card names when it has room for it.
 */
export async function seedAggressor(who: {
	characterId: number;
	name: string;
	corporationId: number;
	corporationName: string;
	corporationTicker: string;
	allianceId: number;
	allianceName: string;
	allianceTicker: string;
}) {
	await withDb(async (db) => {
		await db.query(
			`insert into alliances (id, name, ticker) values ($1, $2, $3)
			 on conflict (id) do update set name = excluded.name, ticker = excluded.ticker`,
			[who.allianceId, who.allianceName, who.allianceTicker],
		);
		await db.query(
			`insert into corporations (id, name, ticker, alliance_id) values ($1, $2, $3, $4)
			 on conflict (id) do update set name = excluded.name, ticker = excluded.ticker`,
			[who.corporationId, who.corporationName, who.corporationTicker, who.allianceId],
		);
		await db.query(
			`insert into characters (id, name, corporation_id, alliance_id)
			 values ($1, $2, $3, $4)
			 on conflict (id) do update set name = excluded.name`,
			[who.characterId, who.name, who.corporationId, who.allianceId],
		);
	});
}

/**
 * Delete a map straight from the database. For the tests that hand ownership away: the
 * API only lets an owner delete, so the identity that made the map cannot tidy it up, and
 * a fixed-name map left behind breaks the next run.
 */
export async function deleteMapRow(mapId: number) {
	await withDb((db) => db.query('delete from maps where id = $1', [mapId]));
}

/** Remove seeded killmails so a rerun starts clean. */
export async function clearKillmails(ids: number[]) {
	await withDb((db) => db.query('delete from killmails where id = any($1)', [ids]));
}

import {
	E2E_CHARACTER_ID,
	E2E_CORPORATION_ID,
	E2E_SESSION,
	EXTRA_CHARACTER_BASE,
	OWN_ACCOUNT_SLOT,
	withDb,
} from './db';

// Remove everything the e2e identity and the per-worker accounts own, then the main
// identity itself, so test runs leave no residue in the dev database.
export default async function globalTeardown() {
	const ownAccounts = Array.from(
		{ length: 10 },
		(_, i) => EXTRA_CHARACTER_BASE + OWN_ACCOUNT_SLOT + i,
	);
	await withDb(async (db) => {
		await db.query(
			`delete from maps where id in (
				 select map_id from map_access
				 where subject_type = 'character' and subject_id = any($1) and role = 'owner'
			 )`,
			[[E2E_CHARACTER_ID, ...ownAccounts]],
		);
		await db.query('delete from sessions where id = $1', [E2E_SESSION]);
		const user = await db.query('select user_id from characters where id = $1', [E2E_CHARACTER_ID]);
		await db.query('delete from characters where id = $1', [E2E_CHARACTER_ID]);
		if (user.rows[0]) {
			await db.query(
				'delete from users where id = $1 and not exists (select 1 from characters where user_id = $1)',
				[user.rows[0].user_id],
			);
		}
		await db.query(
			'delete from corporations where id = $1 and not exists (select 1 from characters where corporation_id = $1)',
			[E2E_CORPORATION_ID],
		);
	});
}

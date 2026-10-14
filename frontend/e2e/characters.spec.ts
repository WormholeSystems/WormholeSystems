import { expect, gotoApp, ownAccountTest as test } from './fixtures';
import { E2E_CORPORATION_ID, withDb } from './db';

// The account's characters: which one acts now, and which one new sessions start as. It
// adds an alt and moves the star, so it runs as the worker's own account rather than the
// one every other spec shares.

const ALT_ID = 91999981;

/** Add an alt to the account and star its main, so the test starts from one place. */
async function seedAlt(mainId: number, userId: number) {
	await withDb(async (db) => {
		await db.query(
			`insert into characters (id, user_id, name, owner_hash, corporation_id)
			 values ($1, $2, 'E2E Star Alt', 'e2e-star-alt-hash', $3)
			 on conflict (id) do update set user_id = excluded.user_id`,
			[ALT_ID, userId, E2E_CORPORATION_ID],
		);
		// Two statements rather than one: at most one preferred character per user is a
		// unique index, and a single update would trip over it mid-statement.
		await db.query('update characters set is_preferred = false where user_id = $1', [userId]);
		await db.query('update characters set is_preferred = true where id = $1', [mainId]);
	});
}

async function removeAlt(mainId: number) {
	await withDb(async (db) => {
		await db.query('delete from characters where id = $1', [ALT_ID]);
		await db.query('update characters set is_preferred = true where id = $1', [mainId]);
	});
}

test('starring a character makes it the one new sessions start as', async ({
	page,
	api,
	account,
}) => {
	const mainId = account.characterId;
	await seedAlt(mainId, account.userId);
	try {
		await gotoApp(page, '/settings/characters');

		const main = page.locator(`[data-character="${mainId}"]`);
		const alt = page.locator(`[data-character="${ALT_ID}"]`);
		await expect(main.getByTestId('preferred-character')).toBeVisible();

		await alt.getByTestId('prefer-character').click();
		await expect(alt.getByTestId('preferred-character')).toBeVisible();
		await expect(main.getByTestId('prefer-character')).toBeVisible();

		// The star is the account's default, not this session's character: acting is unchanged.
		const characters = await (await api.get('/api/me/characters')).json();
		expect(characters.find((c) => c.character_id === ALT_ID).is_preferred).toBe(true);
		expect(characters.find((c) => c.character_id === mainId).is_active).toBe(true);
	} finally {
		await removeAlt(mainId);
	}
});

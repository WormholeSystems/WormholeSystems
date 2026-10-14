import { expect, gotoApp, test } from './fixtures';
import { addSystem, createMap } from './arrange';
import { createIdentity, grantAccess, setCharacterPresence, withDb } from './db';

// The pilots card: who is sharing their location on this map, ordered by who can act, and
// how far each of them is from where you are looking.

const J122515 = 31001882; // C5
const J005482 = 31002515; // C2
const JITA = 30000142;

const CAPSULE = 670;
const MACHARIEL = 17738;
const BUZZARD = 11192; // Covert Ops

type Api = import('@playwright/test').APIRequestContext;

/** Another member of the map, sharing their location, flying a named hull. */
async function addPilot(
	api: Api,
	mapId: number,
	slot: number,
	system: number,
	ship: { typeId: number; docked?: boolean },
) {
	const identity = await createIdentity(slot);
	await grantAccess(mapId, identity.characterId, 'member');
	await setCharacterPresence(identity.characterId, system);
	await withDb((db) =>
		db.query(
			`insert into map_user_settings (map_id, user_id, tracking_allowed) values ($1, $2, true)
			 on conflict (map_id, user_id) do update set tracking_allowed = true`,
			[mapId, identity.userId],
		),
	);
	await withDb((db) =>
		db.query(
			`update character_status set ship_type_id = $2, ship_name = 'A ship',
			     station_id = $3
			 where character_id = $1`,
			[identity.characterId, ship.typeId, ship.docked ? 60003760 : null],
		),
	);
	return identity;
}

test('pilots are listed with the ones who can act first', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Pilots');
	await addSystem(api, mapId, J122515, { alias: 'HOME' });

	await addPilot(api, mapId, 20, J122515, { typeId: CAPSULE });
	await addPilot(api, mapId, 21, J122515, { typeId: MACHARIEL, docked: true });
	await addPilot(api, mapId, 22, J122515, { typeId: BUZZARD });
	await addPilot(api, mapId, 23, J122515, { typeId: MACHARIEL });

	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);
	const card = page.getByTestId('characters-card');
	await expect(card).toBeVisible();

	// Ready first, then the scanner, then the docked pilot, then the pod.
	await expect.poll(async () => card.getByTestId('pilot-row').count(), { timeout: 10_000 }).toBe(4);
	const order = await card
		.getByTestId('pilot-row')
		.evaluateAll((rows) => rows.map((r: HTMLElement) => r.dataset.pilot));
	expect(order).toEqual([
		'E2E Extra 23', // flying something
		'E2E Extra 22', // scanner
		'E2E Extra 21', // docked
		'E2E Extra 20', // pod
	]);

	// Nobody is hidden, and the ones who cannot act are dimmed rather than dropped.
	const dimmed = await card
		.getByTestId('pilot-row')
		.evaluateAll((rows) => rows.map((r: HTMLElement) => r.className.includes('opacity-50')));
	expect(dimmed).toEqual([false, false, true, true]);
});

test('distance is measured from the system you are looking at', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E PilotsDistance');
	const home = await addSystem(api, mapId, J122515, { alias: 'HOME' });
	const next = await addSystem(api, mapId, J005482, { alias: '1', x: 500 });
	await api.post(`/api/maps/${mapId}/connections/add`, {
		data: { map_id: mapId, from_system: home, to_system: next, kind: 'wormhole' },
	});

	await addPilot(api, mapId, 24, J005482, { typeId: MACHARIEL });

	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);
	const row = page.getByTestId('characters-card').getByTestId('pilot-row');
	await expect(row).toHaveCount(1);
	// One wormhole away from the system in focus.
	await expect(row.getByTestId('pilot-jumps')).toHaveText('1j', { timeout: 10_000 });

	// Look at the pilot's own system and the same pilot is now zero jumps away, because the
	// card measures from wherever the map is focused, like the watchlist does.
	await page.getByTestId('system-node').filter({ hasText: 'J005482' }).click();
	await expect(row.getByTestId('pilot-jumps')).toHaveText('0j');
});

test('a pilot outside the chain still shows where they are', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E PilotsOffMap');
	await addSystem(api, mapId, J122515, { alias: 'HOME' });
	await addPilot(api, mapId, 25, JITA, { typeId: MACHARIEL });

	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);
	const row = page.getByTestId('characters-card').getByTestId('pilot-row');
	await expect(row).toHaveCount(1);
	// Not a placement on the map, so it has to be resolved by name rather than looked up.
	await expect(row).toContainText('Jita');
});

test('a pilot row right-clicks to the system menu', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E PilotsMenu');
	await addSystem(api, mapId, J122515, { alias: 'HOME' });
	// In known space, so the row's system is one the map does not hold: the menu still has
	// to work, which means the system was resolved rather than read off a placement.
	await addPilot(api, mapId, 28, JITA, { typeId: MACHARIEL });

	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);
	const row = page.getByTestId('characters-card').getByTestId('pilot-row');
	await expect(row).toHaveCount(1);
	// The menu appears once the system behind the row has been resolved, which is also when
	// the row stops saying "Unknown".
	await expect(row).toContainText('Jita');

	await row.click({ button: 'right' });
	const menu = page.getByRole('menu');
	await expect(menu).toBeVisible();
	await expect(menu.getByText('Set destination')).toBeVisible();
	await expect(menu.getByText('Add to map')).toBeVisible();
});

test('viewers do not get a pilot list at all', async ({ api, browser }) => {
	const mapId = await createMap(api, 'E2E PilotsViewer');
	await addSystem(api, mapId, J122515, { alias: 'HOME' });
	await addPilot(api, mapId, 26, J122515, { typeId: MACHARIEL });

	const viewer = await createIdentity(27);
	await grantAccess(mapId, viewer.characterId, 'viewer');
	const ctx = await browser.newContext();
	await ctx.addCookies([
		{ name: 'ws_session', value: viewer.session, domain: 'localhost', path: '/' },
	]);
	const page = await ctx.newPage();
	await page.goto(`http://localhost:5173/maps/${mapId}?system=${J122515}`);
	await page.waitForSelector('[data-testid="panel-grid"]');

	// Presence is member-gated on the server, so the card is there but empty rather than
	// leaking who is online.
	await expect(page.getByTestId('characters-card')).toBeVisible();
	await expect(page.getByTestId('pilots-empty')).toBeVisible();
	await ctx.close();
});

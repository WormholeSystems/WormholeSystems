import { createdId, expect, gotoApp, reloadApp, test } from './fixtures';

// Watching a map without an account: a share link, and a public map.

const J122515 = 31001882;
const JITA = 30000142;

async function seedMap(api: import('@playwright/test').APIRequestContext, name: string) {
	const res = await api.post('/api/maps', { data: { name } });
	const mapId = await createdId(res);
	const add = async (id: number, x: number) =>
		(
			await (
				await api.post(`/api/maps/${mapId}/systems/add`, {
					data: { map_id: mapId, solar_system_id: id, x, y: 300, alias: null },
				})
			).json()
		).id as number;
	const home = await add(J122515, 200);
	const far = await add(JITA, 600);
	await api.post(`/api/maps/${mapId}/connections/add`, {
		data: { map_id: mapId, from_system: home, to_system: far, kind: 'wormhole' },
	});
	return mapId;
}

test('a share link shows the chain to somebody with no account', async ({ page, browser, api }) => {
	const mapId = await seedMap(api, 'E2E Shared');

	await gotoApp(page, `/maps/${mapId}/settings/access`);
	await page.getByTestId('share-create').click();
	const url = await page.getByTestId('share-url').inputValue();
	expect(url).toContain('/share/');

	// A browser that has never signed in follows the link.
	const guest = await browser.newContext();
	const guestPage = await guest.newPage();
	await guestPage.goto(url);
	await guestPage.waitForSelector('html[data-hydrated="true"]');

	// The link is a way in, not a place: it hands over the token and drops them on the map
	// everyone else is looking at.
	await expect(guestPage).toHaveURL(new RegExp(`/maps/${mapId}$`));
	await expect(guestPage.getByTestId('map-canvas')).toBeVisible();
	await expect(guestPage.getByTestId('system-node')).toHaveCount(2);
	await expect(guestPage.getByTestId('status-bar-name')).toHaveText('E2E Shared');

	// Watching, not flying: nothing that writes, and nothing that needs an account.
	await expect(guestPage.getByText('Watching')).toBeVisible();
	await expect(guestPage.getByTestId('undo-button')).toHaveCount(0);
	await expect(guestPage.getByTestId('settings-link')).toHaveCount(0);
	// The chain is there to read, not to rearrange: no handle to drag a node by and none
	// to pull a connection out of.
	await expect(guestPage.getByTestId('drag-handle')).toHaveCount(0);
	await expect(guestPage.getByTestId('connection-handle')).toHaveCount(0);

	// The token is remembered, so the map is an ordinary address from here on.
	await guestPage.goto(new URL(`/maps/${mapId}`, url).toString());
	await guestPage.waitForSelector('html[data-hydrated="true"]');
	await expect(guestPage.getByTestId('system-node')).toHaveCount(2);

	// Withdrawing the link shuts the door, remembered token or not.
	await page.getByTestId('share-revoke').click();
	await page.getByTestId('confirm-accept').click();
	await expect(page.getByTestId('share-create')).toBeVisible();

	await reloadApp(guestPage);
	await expect(guestPage.getByTestId('map-error')).toBeVisible();

	const shut = await guest.newPage();
	const response = await shut.goto(url);
	expect(response?.status()).toBe(404);

	await guest.close();
	await api.delete(`/api/maps/${mapId}`);
});

test('a private map is not readable by a link that was never made', async ({ browser, api }) => {
	const mapId = await seedMap(api, 'E2E NotShared');
	const guest = await browser.newContext();
	const guestPage = await guest.newPage();

	// The map itself, asked for directly by a stranger: "not found", not "not allowed".
	// A private map does not confirm its own existence to somebody with no way in.
	const direct = await guestPage.goto(`http://127.0.0.1:3000/api/maps/${mapId}`);
	expect(direct?.status()).toBe(404);

	// And a guessed token.
	const guessed = await guestPage.goto('http://localhost:5173/share/not-a-real-token');
	expect(guessed?.status()).toBe(404);

	await guest.close();
	await api.delete(`/api/maps/${mapId}`);
});

test('a public map needs no link at all', async ({ browser, api }) => {
	const mapId = await seedMap(api, 'E2E Public');
	await api.post(`/api/maps/${mapId}/update`, {
		data: { map_id: mapId, is_public: true },
	});

	const guest = await browser.newContext();
	const guestPage = await guest.newPage();
	const view = await guestPage.goto(`http://127.0.0.1:3000/api/maps/${mapId}`);
	expect(view?.status()).toBe(200);
	const body = await view!.json();
	expect(body.role).toBe('viewer');
	// The key to the map never travels to somebody who could not mint one.
	expect(body.map.share_token ?? null).toBeNull();

	await guestPage.goto(`http://localhost:5173/maps/${mapId}`);
	await guestPage.waitForSelector('html[data-hydrated="true"]');
	await expect(guestPage.getByTestId('system-node')).toHaveCount(2);

	await guest.close();
	await api.delete(`/api/maps/${mapId}`);
});

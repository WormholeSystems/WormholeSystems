import { expect, gotoApp, ownAccountTest as test, reloadApp } from './fixtures';
import { createMap } from './arrange';
import { setCharacterPresence } from './db';

// ESI waypoints: menu gating and endpoint validation. The real ESI call is never made in
// tests (no valid refresh token), so the API-level checks assert validation behavior. The
// pilot is put online and offline here, so it is the worker's own account, not the shared one.

const JITA = 30000142;
const J122515 = 31001882;

test('waypoint submenus: k-space only, disabled without online characters', async ({
	page,
	api,
	account,
}) => {
	const mapId = await createMap(api, 'E2E Waypoints');
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: JITA, x: 200, y: 200, alias: null },
	});
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: J122515, x: 200, y: 400, alias: null },
	});

	// Offline: submenu shows the disabled hint.
	await setCharacterPresence(account.characterId, JITA, false);
	await gotoApp(page, `/maps/${mapId}`);
	const jita = page.getByTestId('system-node').filter({ hasText: 'Jita' });
	await jita.click({ button: 'right' });
	await page.getByTestId('menu-set-destination').hover();
	await expect(
		page.getByTestId('destination-submenu').getByText('No characters online'),
	).toBeVisible();
	await page.keyboard.press('Escape');

	// W-space node: no waypoint submenus at all.
	const wh = page.getByTestId('system-node').filter({ hasText: 'J122515' });
	await wh.click({ button: 'right' });
	await expect(page.getByTestId('menu-set-destination')).toHaveCount(0);
	await page.keyboard.press('Escape');

	// Online: the character appears as a submenu entry.
	await setCharacterPresence(account.characterId, JITA, true);
	await reloadApp(page);
	await jita.click({ button: 'right' });
	await page.getByTestId('menu-set-destination').hover();
	await expect(
		page.getByTestId('destination-submenu').getByRole('menuitem', { name: account.name }),
	).toBeVisible();
});

test('waypoint endpoint validation', async ({ api, account }) => {
	// Foreign character rejected.
	const foreign = await api.post('/api/waypoints', {
		data: { character_id: 1, destination_id: JITA },
	});
	expect(foreign.status()).toBe(400);

	// Unknown destination rejected.
	const badDest = await api.post('/api/waypoints', {
		data: { character_id: account.characterId, destination_id: 42 },
	});
	expect(badDest.status()).toBe(400);

	// Stations are valid ESI destinations too (Jita 4-4), so they pass validation and
	// fail later on the missing scope rather than being rejected as unknown.
	const stationDest = await api.post('/api/waypoints', {
		data: { character_id: account.characterId, destination_id: 60003760 },
	});
	expect(stationDest.status()).not.toBe(400);

	// Valid request fails on the token/scope step (no real ESI token in tests): the
	// fixture token has no scopes, so the API answers 409 "missing scope".
	await setCharacterPresence(account.characterId, JITA, true);
	const noScope = await api.post('/api/waypoints', {
		data: { character_id: account.characterId, destination_id: JITA },
	});
	expect([409, 502]).toContain(noScope.status());
});

import { expect, gotoApp, reloadApp, test } from './fixtures';
import { createMap } from './arrange';
import { createIdentity, grantAccess, setCharacterPresence, setTrackingAllowed } from './db';

// Pilot presence: online, tracking-scoped, opted-in characters of member+ users show on
// the node; opting out hides them; viewers get nothing.

const JITA = 30000142;

test('pilots row shows opted-in online characters and hides on opt-out', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Presence');
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: JITA, x: 200, y: 200, alias: null },
	});

	const pilot = await createIdentity(3);
	await grantAccess(mapId, pilot.characterId, 'member');
	await setCharacterPresence(pilot.characterId, JITA);
	await setTrackingAllowed(mapId, pilot.userId, true);

	await gotoApp(page, `/maps/${mapId}`);
	const node = page.getByTestId('system-node').filter({ hasText: 'Jita' });
	const row = node.getByTestId('pilots-row');
	await expect(row).toBeVisible();
	await expect(row.getByText('E2E Extra 3')).toBeVisible();

	// Pilot tooltip lists name and corp ticker.
	await row.hover();
	await expect(page.getByText('[E2E]')).toBeVisible();

	// Opt-out hides the pilot after the next fetch (reload forces it).
	await setTrackingAllowed(mapId, pilot.userId, false);
	await reloadApp(page);
	await expect(node.getByTestId('pilots-row')).toHaveCount(0);
});

test('viewers see no pilots and offline characters stay hidden', async ({ page, api, browser }) => {
	const mapId = await createMap(api, 'E2E PresenceGate');
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: JITA, x: 200, y: 200, alias: null },
	});

	const pilot = await createIdentity(4);
	await grantAccess(mapId, pilot.characterId, 'member');
	await setCharacterPresence(pilot.characterId, JITA);
	await setTrackingAllowed(mapId, pilot.userId, true);

	// The characters endpoint refuses viewers.
	const viewer = await createIdentity(5);
	await grantAccess(mapId, viewer.characterId, 'viewer');
	const viewerCtx = await browser.newContext();
	await viewerCtx.addCookies([
		{ name: 'ws_session', value: viewer.session, domain: 'localhost', path: '/' },
	]);
	const viewerPage = await viewerCtx.newPage();
	await viewerPage.goto(`http://localhost:5173/maps/${mapId}`);
	await viewerPage.waitForSelector('html[data-hydrated="true"]');
	await expect(viewerPage.getByTestId('system-node').filter({ hasText: 'Jita' })).toBeVisible();
	await expect(viewerPage.getByTestId('pilots-row')).toHaveCount(0);
	await viewerCtx.close();

	// Offline characters disappear from presence.
	await setCharacterPresence(pilot.characterId, JITA, false);
	await gotoApp(page, `/maps/${mapId}`);
	await expect(page.getByTestId('pilots-row')).toHaveCount(0);
});

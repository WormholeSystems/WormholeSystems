import { expect, gotoApp, reloadApp, test } from './fixtures';
import { createMap } from './arrange';
import { createIdentity, grantAccess } from './db';

// The active-system model and the side panels (System Info, Signatures, Notes).

const J122515 = 31001882; // C5, Wolf-Rayet Star, static H296

test('clicking a node activates it: ring, URL, and System Info', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Active');
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: J122515, x: 200, y: 200, alias: 'Staging' },
	});

	await gotoApp(page, `/maps/${mapId}`);
	await expect(page.getByTestId('system-info-empty')).toBeVisible();

	const node = page.getByTestId('system-node').filter({ hasText: 'J122515' });
	await node.click();

	// Amber ring + deep link.
	await expect(node).toHaveClass(/ring-amber-500/);
	await expect(page).toHaveURL(new RegExp(`system=${J122515}`));

	// System Info content.
	const info = page.getByTestId('system-info');
	await expect(info.getByText('Staging')).toBeVisible();
	await expect(info.getByText('(J122515)')).toBeVisible();
	await expect(info.getByText('Wolf-Rayet Star')).toBeVisible();
	await expect(info.getByRole('link', { name: 'zKill' })).toHaveAttribute(
		'href',
		`https://zkillboard.com/system/${J122515}/`,
	);
	await expect(info.getByRole('link', { name: 'Anoik' })).toHaveAttribute(
		'href',
		'https://anoik.is/systems/J122515',
	);
	await expect(info.getByText('H296')).toBeVisible();

	// The signatures card follows the active system.
	await expect(page.getByTestId('signatures-card')).toBeVisible();
});

test('deep link ?system= restores the active system on load', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E DeepLink');
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: J122515, x: 200, y: 200, alias: null },
	});

	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);
	const node = page.getByTestId('system-node').filter({ hasText: 'J122515' });
	await expect(node).toHaveClass(/ring-amber-500/);
	await expect(page.getByTestId('system-info')).toBeVisible();
});

test('notes round-trip with markdown rendering', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Notes');
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: J122515, x: 200, y: 200, alias: null },
	});

	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);
	const card = page.getByTestId('notes-card');
	await expect(card.getByText('No notes')).toBeVisible();

	await card.getByLabel('Edit notes').click();
	await card.getByPlaceholder('Add notes...').fill('**Danger**: hostile Astrahus');
	// The card renders the new notes optimistically, so wait for the write itself before
	// reloading; otherwise the navigation can abort the request in flight.
	const saved = page.waitForResponse(
		(r) => r.url().includes('/systems/set-notes') && r.request().method() === 'POST',
	);
	await card.getByRole('button', { name: 'Save' }).click();
	await expect(card.locator('strong', { hasText: 'Danger' })).toBeVisible();
	await saved;

	// Survives a reload (persisted server-side).
	await reloadApp(page);
	await expect(card.locator('strong', { hasText: 'Danger' })).toBeVisible();
});

test('viewers cannot see notes', async ({ page, api, browser }) => {
	const mapId = await createMap(api, 'E2E ViewerNotes');
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: J122515, x: 200, y: 200, alias: null },
	});
	const viewer = await createIdentity(1);
	await grantAccess(mapId, viewer.characterId, 'viewer');

	// The details endpoint refuses viewers outright.
	const viewerCtx = await browser.newContext();
	await viewerCtx.addCookies([
		{ name: 'ws_session', value: viewer.session, domain: 'localhost', path: '/' },
	]);
	const viewerPage = await viewerCtx.newPage();
	await viewerPage.goto(`http://localhost:5173/maps/${mapId}?system=${J122515}`);
	await viewerPage.waitForSelector('html[data-hydrated="true"]');
	await expect(viewerPage.getByTestId('system-info')).toBeVisible();
	await expect(viewerPage.getByTestId('notes-card')).toHaveCount(0);
	await viewerCtx.close();
	void page;
});

import { expect, gotoApp, test } from './fixtures';
import { addSystem, createMap } from './arrange';
import { createIdentity, grantAccess } from './db';

// The app-wide solar-system context menu: right-click any system reference (route
// rows, search results) for add-to-map, external links, waypoints, route planner,
// and the rally toggle.

const JITA = 30000142;
const PERIMETER = 30000144;

test('route rows open the system menu with external links', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E SysMenuRoute');
	const a = await addSystem(api, mapId, JITA, { x: 200, y: 200 });
	const b = await addSystem(api, mapId, PERIMETER, { x: 560, y: 200 });
	await api.post(`/api/maps/${mapId}/connections/add`, {
		data: { map_id: mapId, from_system: a, to_system: b, kind: 'wormhole' },
	});

	await gotoApp(page, `/maps/${mapId}`);
	// Route via the pickers (keyboard flow).
	await page.getByTestId('system-picker-origin').click();
	await page.getByPlaceholder('Search…').fill('jita');
	await expect(page.getByRole('option', { name: /Jita/ })).toBeVisible();
	await page.getByPlaceholder('Search…').press('Enter');
	await page.getByTestId('system-picker-destination').click();
	await page.getByPlaceholder('Search…').last().fill('perimeter');
	await expect(page.getByRole('option', { name: /Perimeter/ })).toBeVisible();
	await page.getByPlaceholder('Search…').last().press('Enter');

	const list = page.getByTestId('route-list');
	await expect(list.getByText('Jita')).toBeVisible();

	// Right-click the Jita hop.
	await list.getByText('Jita').click({ button: 'right' });
	const menu = page.getByTestId('system-menu');
	await expect(menu).toBeVisible();
	// Placed system → no Add to map; rally toggle present instead.
	await expect(menu.getByTestId('menu-add-to-map')).toHaveCount(0);
	await expect(menu.getByTestId('menu-rally')).toBeVisible();

	// External: k-space gets the Jump Range entry; zKill links carry the ids.
	await menu.getByTestId('menu-external').hover();
	await expect(page.getByRole('menuitem', { name: 'Jump Range' })).toHaveAttribute(
		'href',
		'https://evemaps.dotlan.net/range/Revelation,5/Jita',
	);
	await expect(page.getByRole('menuitem', { name: 'Constellation' })).toHaveAttribute(
		'href',
		/zkillboard\.com\/constellation\/\d+\//,
	);
	await expect(page.getByRole('menuitem', { name: 'Region', exact: true })).toHaveAttribute(
		'href',
		/zkillboard\.com\/region\/\d+\//,
	);
	await page.keyboard.press('Escape');
});

test('search rows: add to map, then rally toggle once placed', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E SysMenuAdd');
	await addSystem(api, mapId, JITA, { x: 200, y: 200 });

	await gotoApp(page, `/maps/${mapId}`);
	await page.getByTestId('map-canvas').click({ button: 'right', position: { x: 500, y: 400 } });
	await page.getByRole('menuitem', { name: 'Add solar system' }).click();
	await page.getByPlaceholder('System, alias, occupier or notes…').fill('amarr');
	const row = page.locator('[data-slot="command-item"]', { hasText: 'Amarr' }).first();
	await expect(row).toBeVisible();

	// Unplaced → Add to map places it at a free spot.
	await row.click({ button: 'right' });
	const menu = page.getByTestId('system-menu');
	await menu.getByTestId('menu-add-to-map').click();
	await expect(page.getByTestId('system-node').filter({ hasText: 'Amarr' })).toBeVisible();

	// Reopen: the entry is gone, the rally toggle works now.
	await page.getByPlaceholder('System, alias, occupier or notes…').fill('amarr');
	await row.click({ button: 'right' });
	await expect(menu).toBeVisible();
	await expect(menu.getByTestId('menu-add-to-map')).toHaveCount(0);
	await menu.getByTestId('menu-rally').click();
	await expect
		.poll(async () => {
			const view = await (await api.get(`/api/maps/${mapId}`)).json();
			return view.systems.find((s: { name: string }) => s.name === 'Amarr')?.is_rally;
		})
		.toBe(true);
	await page.keyboard.press('Escape');
});

test('wormhole rows hide Jump Range; route planner sets the origin', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E SysMenuWh');
	await gotoApp(page, `/maps/${mapId}`);
	await page.getByTestId('map-canvas').click({ button: 'right', position: { x: 500, y: 400 } });
	await page.getByRole('menuitem', { name: 'Add solar system' }).click();
	await page.getByPlaceholder('System, alias, occupier or notes…').fill('J122515');
	const row = page.locator('[data-slot="command-item"]', { hasText: 'J122515' });
	await expect(row).toBeVisible();

	await row.click({ button: 'right' });
	const menu = page.getByTestId('system-menu');
	await menu.getByTestId('menu-external').hover();
	await expect(page.getByRole('menuitem', { name: 'System', exact: true }).first()).toBeVisible();
	await expect(page.getByRole('menuitem', { name: 'Jump Range' })).toHaveCount(0);

	// Route planner from the menu fills the origin picker.
	await menu.getByTestId('menu-route').hover();
	await page.getByRole('menuitem', { name: 'Set as origin' }).click();
	await page.keyboard.press('Escape'); // close the search dialog
	await expect(page.getByTestId('system-picker-origin')).toContainText('J122515');
});

test('waypoint submenu is disabled without online characters', async ({ api, browser }) => {
	const mapId = await createMap(api, 'E2E SysMenuWp');
	await addSystem(api, mapId, JITA, { x: 200, y: 200 });
	// A dedicated identity: the shared main character's presence is toggled by other
	// spec files.
	const member = await createIdentity(8);
	await grantAccess(mapId, member.characterId, 'member');

	const ctx = await browser.newContext();
	await ctx.addCookies([
		{ name: 'ws_session', value: member.session, domain: 'localhost', path: '/' },
	]);
	const memberPage = await ctx.newPage();
	await memberPage.goto(`http://localhost:5173/maps/${mapId}`);
	await memberPage.waitForSelector('html[data-hydrated="true"]');

	await memberPage
		.getByTestId('map-canvas')
		.click({ button: 'right', position: { x: 500, y: 400 } });
	await memberPage.getByRole('menuitem', { name: 'Add solar system' }).click();
	await memberPage.getByPlaceholder('System, alias, occupier or notes…').fill('perimeter');
	const row = memberPage.locator('[data-slot="command-item"]', { hasText: 'Perimeter' }).first();
	await row.click({ button: 'right' });
	await memberPage.getByTestId('menu-set-destination').hover();
	await expect(memberPage.getByText('No characters online')).toBeVisible();
	await ctx.close();
});

test('viewers get no write items', async ({ page, api, browser }) => {
	const mapId = await createMap(api, 'E2E SysMenuViewer');
	await addSystem(api, mapId, JITA, { x: 200, y: 200 });
	const viewer = await createIdentity(9);
	await grantAccess(mapId, viewer.characterId, 'viewer');

	const ctx = await browser.newContext();
	await ctx.addCookies([
		{ name: 'ws_session', value: viewer.session, domain: 'localhost', path: '/' },
	]);
	const viewerPage = await ctx.newPage();
	await viewerPage.goto(`http://localhost:5173/maps/${mapId}`);
	await viewerPage.waitForSelector('html[data-hydrated="true"]');

	// Viewers can still search (route pickers), and the menu shows only read items.
	await viewerPage.getByTestId('system-picker-origin').click();
	await viewerPage.getByPlaceholder('Search…').fill('amarr');
	const row = viewerPage.getByRole('option', { name: /Amarr/ }).first();
	await expect(row).toBeVisible();
	await row.click({ button: 'right' });
	const menu = viewerPage.getByTestId('system-menu');
	await expect(menu).toBeVisible();
	await expect(menu.getByTestId('menu-external')).toBeVisible();
	await expect(menu.getByTestId('menu-add-to-map')).toHaveCount(0);
	await expect(menu.getByTestId('menu-rally')).toHaveCount(0);
	await ctx.close();
	void page;
});

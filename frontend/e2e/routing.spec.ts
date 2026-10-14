import { createdId, expect, gotoApp, test } from './fixtures';

// Route planner: origin/destination via the context menu, jump counts, wormhole
// shortcuts, and the on-route connection highlight.

const JITA = 30000142;
const PERIMETER = 30000144; // one gate from Jita
const AMARR = 30002187; // many gates from Jita

async function createMap(api: import('@playwright/test').APIRequestContext, name: string) {
	const res = await api.post('/api/maps', { data: { name } });
	expect(res.ok()).toBe(true);
	return await createdId(res);
}

async function addSystem(
	api: import('@playwright/test').APIRequestContext,
	mapId: number,
	solarSystemId: number,
	x: number,
	y: number,
) {
	const res = await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: solarSystemId, x, y, alias: null },
	});
	expect(res.ok()).toBe(true);
	return await createdId(res);
}

test('gate route between adjacent systems via the context menu', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Route');
	await addSystem(api, mapId, JITA, 200, 200);
	await addSystem(api, mapId, PERIMETER, 500, 200);

	await gotoApp(page, `/maps/${mapId}`);

	const jita = page.getByTestId('system-node').filter({ hasText: 'Jita' });
	await jita.click({ button: 'right' });
	await page.getByTestId('menu-route').hover();
	await page.getByTestId('route-submenu').getByRole('menuitem', { name: 'Set as origin' }).click();

	const peri = page.getByTestId('system-node').filter({ hasText: 'Perimeter' });
	await peri.click({ button: 'right' });
	await page.getByTestId('menu-route').hover();
	await page
		.getByTestId('route-submenu')
		.getByRole('menuitem', { name: 'Set as destination' })
		.click();

	await expect(page.getByTestId('route-jumps')).toHaveText('1 jumps');
	// The same tone every other jump badge uses; they share one function.
	await expect(page.getByTestId('route-jumps')).toHaveClass(/text-green-400/);
	const list = page.getByTestId('route-list');
	await expect(list.getByText('Jita')).toBeVisible();
	await expect(list.getByText('Perimeter')).toBeVisible();
});

test('wormhole connection shortcuts the route and highlights the edge', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E WH Route');
	const a = await addSystem(api, mapId, JITA, 200, 200);
	const b = await addSystem(api, mapId, AMARR, 600, 200);
	await api.post(`/api/maps/${mapId}/connections/add`, {
		data: { map_id: mapId, from_system: a, to_system: b, kind: 'wormhole' },
	});

	await gotoApp(page, `/maps/${mapId}`);

	// Set origin/destination through the pickers' quick path: context menu again.
	const jita = page.getByTestId('system-node').filter({ hasText: 'Jita' });
	await jita.click({ button: 'right' });
	await page.getByTestId('menu-route').hover();
	await page.getByTestId('route-submenu').getByRole('menuitem', { name: 'Set as origin' }).click();
	const amarr = page.getByTestId('system-node').filter({ hasText: 'Amarr' });
	await amarr.click({ button: 'right' });
	await page.getByTestId('menu-route').hover();
	await page
		.getByTestId('route-submenu')
		.getByRole('menuitem', { name: 'Set as destination' })
		.click();

	// The wormhole makes it a single jump, marked as WH in the list.
	await expect(page.getByTestId('route-jumps')).toHaveText('1 jumps');
	await expect(page.getByTestId('route-list').getByText('WH')).toBeVisible();

	// The connection edge is highlighted.
	await expect(page.locator('path[data-on-route="true"]')).toHaveCount(1);

	// Clearing the route removes the highlight.
	await page.getByRole('button', { name: 'Clear route' }).click();
	await expect(page.locator('path[data-on-route="true"]')).toHaveCount(0);
});

test('navigation pickers support keyboard selection', async ({ page, api }) => {
	const res = await api.post('/api/maps', { data: { name: 'E2E PickerKeys' } });
	const mapId = await createdId(res);
	await gotoApp(page, `/maps/${mapId}`);

	const origin = page.getByTestId('system-picker-origin');
	await origin.click();
	await page.getByPlaceholder('Search…').fill('jita');
	await expect(page.getByRole('option', { name: /Jita/ })).toBeVisible();
	await page.getByPlaceholder('Search…').press('Enter');
	await expect(origin).toContainText('Jita');
});

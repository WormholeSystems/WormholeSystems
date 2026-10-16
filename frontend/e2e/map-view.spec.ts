import { expect, gotoApp, reloadApp, test } from './fixtures';
import { createMap } from './arrange';

// The interactive map canvas. Each test arranges its own map through the API and
// navigates straight to it; global teardown removes everything the e2e user owns.

test('search dialog adds a system to the map', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Search Map');
	await gotoApp(page, `/maps/${mapId}`);

	const canvas = page.getByTestId('map-canvas');
	await canvas.click({ button: 'right', position: { x: 500, y: 500 } });
	await page.getByRole('menuitem', { name: 'Add solar system' }).click();

	const input = page.getByPlaceholder('System, alias, occupier or notes…');
	await expect(input).toBeVisible();
	await input.fill('jita');
	const result = page.locator('[data-slot="command-item"]', { hasText: 'Jita' });
	await expect(result).toBeVisible();
	await result.click();

	// The node appears on the canvas with its name and region, centered on the
	// right-click spot (grid-snapped).
	const node = page.getByTestId('system-node').filter({ hasText: 'Jita' });
	await expect(node).toBeVisible();
	await expect(node.getByText('The Forge')).toBeVisible();
	const pos = await node.evaluate((el: HTMLElement) => ({
		left: parseFloat(el.style.left),
		top: parseFloat(el.style.top),
	}));
	// Click at (500, 500), node 180 wide / 40 tall → top-left near (410, 480).
	expect(Math.abs(pos.left - 410)).toBeLessThanOrEqual(40);
	expect(Math.abs(pos.top - 480)).toBeLessThanOrEqual(40);
});

test('search rows show class letters, statics and the effect badge', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E SearchRows');
	await gotoApp(page, `/maps/${mapId}`);

	const canvas = page.getByTestId('map-canvas');
	await canvas.click({ button: 'right', position: { x: 400, y: 400 } });
	await page.getByRole('menuitem', { name: 'Add solar system' }).click();
	const input = page.getByPlaceholder('System, alias, occupier or notes…');

	// A wormhole row says what a map node says: its class, where its statics lead, and the
	// effect as the same icon rather than a column of prose.
	await input.fill('J122515');
	const whRow = page.locator('[data-slot="command-item"]', { hasText: 'J122515' });
	await expect(whRow).toBeVisible();
	await expect(whRow.getByTestId('class-badge')).toHaveText('C5');
	// J122515 is a C5 whose one static (H296) leads to another C5.
	await expect(whRow.getByTestId('row-static')).toHaveText(['C5']);
	await expect(whRow.getByLabel('Wolf-Rayet Star').locator('img')).toBeVisible();

	// A k-space row: the class letter, not the raw security number.
	await input.fill('jita');
	const jitaRow = page.locator('[data-slot="command-item"]', { hasText: 'Jita' }).first();
	await expect(jitaRow.getByTestId('class-badge')).toHaveText('H');
	await expect(jitaRow.getByText('0.9')).toHaveCount(0);
	// k-space has no statics, so that cell carries the sovereignty logo instead.
	await expect(jitaRow.getByTestId('row-static')).toHaveCount(0);
	await page.keyboard.press('Escape');
});

test('node context menu sets the system status', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Status Map');
	const add = await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: 30000142, x: 200, y: 200, alias: null },
	});
	expect(add.ok()).toBe(true);

	await gotoApp(page, `/maps/${mapId}`);
	const node = page.getByTestId('system-node').filter({ hasText: 'Jita' });
	await node.click({ button: 'right' });
	await page.getByTestId('status-subtrigger').hover();
	await page.getByTestId('status-submenu').getByRole('menuitem', { name: 'Friendly' }).click();

	// The friendly status recolors the node border via its data-status channel.
	await expect(node).toHaveAttribute('data-status', 'friendly');
});

test('add connection searches from the palette and links what it places', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E MenuConnect');
	await gotoApp(page, `/maps/${mapId}`);

	// Place Jita by right-clicking the canvas, which is now the palette too.
	await page.getByTestId('map-canvas').click({ button: 'right', position: { x: 400, y: 300 } });
	await page.getByRole('menuitem', { name: 'Add solar system' }).click();
	await page.getByPlaceholder('System, alias, occupier or notes…').fill('jita');
	await page.locator('[data-slot="command-item"]', { hasText: 'Jita' }).first().click();
	const jita = page.getByTestId('system-node').filter({ hasText: 'Jita' });
	await expect(jita).toBeVisible();

	// "Add connection" opens the same palette, saying what it is for.
	await jita.click({ button: 'right' });
	await page.getByRole('menuitem', { name: 'Add connection' }).click();
	const input = page.getByPlaceholder('Connect to…');
	await expect(input).toBeVisible();

	// Picking an unplaced system both places it and joins the two.
	await input.fill('perimeter');
	await page.locator('[data-slot="command-item"]', { hasText: 'Perimeter' }).first().click();
	await expect(page.getByTestId('system-node').filter({ hasText: 'Perimeter' })).toBeVisible();
	await expect
		.poll(async () => (await (await api.get(`/api/maps/${mapId}`)).json()).connections.length)
		.toBe(1);

	// And the next Cmd+K is a plain search again, not still linking.
	await page.keyboard.press('Meta+k');
	await expect(page.getByPlaceholder('System, alias, occupier or notes…')).toBeVisible();
});

test('zoom steps in tenths, sticks to the map, and answers the wheel', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Zoom');
	await gotoApp(page, `/maps/${mapId}`);

	const level = page.getByTestId('zoom-level');
	const zoomIn = page.getByRole('button', { name: 'Zoom in' });
	const zoomOut = page.getByRole('button', { name: 'Zoom out' });
	await expect(level).toHaveText('100%');

	await zoomIn.click();
	await expect(level).toHaveText('110%');

	// Clamped at double size: the map is designed to stop there.
	for (let i = 0; i < 12; i++) await zoomIn.click();
	await expect(level).toHaveText('200%');
	// And at half.
	for (let i = 0; i < 20; i++) await zoomOut.click();
	await expect(level).toHaveText('50%');

	// It is where you left it after a reload: zoom belongs to the screen you are at.
	await zoomIn.click();
	await reloadApp(page);
	await expect(page.getByTestId('zoom-level')).toHaveText('60%');

	// Shift+wheel pans rather than scrolling the page; the wheel never zooms.
	const canvas = page.getByTestId('map-canvas');
	await canvas.hover();
	await page.mouse.wheel(0, 200);
	await expect(page.getByTestId('zoom-level')).toHaveText('60%');
});

import { expect, gotoApp, reloadApp, test } from './fixtures';
import { addSystem, connect, createMap } from './arrange';

// Automatic placement: the chain drawn as a tree from its connections, instead of from
// wherever people dragged the systems.

const J122515 = 31001882;
const JITA = 30000142;
const PERIMETER = 30000144;

/** A chain of three, deliberately placed in a heap so the layout has work to do. */
async function seedChain(api: import('@playwright/test').APIRequestContext, name: string) {
	const mapId = await createMap(api, name);
	const home = await addSystem(api, mapId, J122515, { x: 900, y: 700 });
	const second = await addSystem(api, mapId, JITA, { x: 900, y: 800 });
	const third = await addSystem(api, mapId, PERIMETER, { x: 900, y: 900 });
	await connect(api, mapId, home, second);
	await connect(api, mapId, home, third);
	await api.post(`/api/maps/${mapId}/systems/set-home`, {
		data: { map_id: mapId, map_solar_system_id: home, value: true },
	});
	return { mapId, home, second, third };
}

function nodePosition(page: import('@playwright/test').Page, text: string) {
	return page
		.getByTestId('system-node')
		.filter({ hasText: text })
		.evaluate((el: HTMLElement) => ({
			left: parseFloat(el.style.left),
			top: parseFloat(el.style.top),
		}));
}

test('a map set to automatic placement draws the chain as a tree', async ({ page, api }) => {
	const { mapId } = await seedChain(api, 'E2E Tree');
	await api.post(`/api/maps/${mapId}/update`, {
		data: { map_id: mapId, layout: 'tree' },
	});

	await gotoApp(page, `/maps/${mapId}`);
	await expect(page.getByTestId('system-node')).toHaveCount(3);

	// The home system roots the tree in the first column; its two holes share the next.
	const home = await nodePosition(page, 'J122515');
	const jita = await nodePosition(page, 'Jita');
	const perimeter = await nodePosition(page, 'Perimeter');
	expect(home.left).toBe(20);
	expect(jita.left).toBe(340);
	expect(perimeter.left).toBe(340);
	expect(jita.top).not.toBe(perimeter.top);

	// Nothing to drag: the layout owns the positions.
	await expect(page.getByTestId('drag-handle')).toHaveCount(0);
});

test('a viewer can pick their own placement when the map allows it', async ({ page, api }) => {
	const { mapId } = await seedChain(api, 'E2E TreeOverride');
	await api.post(`/api/maps/${mapId}/update`, {
		data: { map_id: mapId, allow_layout_override: true },
	});

	await gotoApp(page, `/maps/${mapId}`);
	await expect(page.getByTestId('system-node')).toHaveCount(3);
	// The map is still on custom placement, so the systems sit where they were put.
	expect((await nodePosition(page, 'Jita')).left).toBe(900);

	await page.getByTestId('placement-tree').click();
	await expect.poll(async () => (await nodePosition(page, 'Jita')).left).toBe(340);

	// It is a preference of this viewer's, so it survives a reload. The switch paints
	// before the write lands, so wait for the stored value: reloading first would cancel
	// the request in flight.
	await expect
		.poll(
			async () =>
				(await (await api.get(`/api/maps/${mapId}/settings/user`)).json()).layout_override,
		)
		.toBe('tree');
	await reloadApp(page);
	await expect(page.getByTestId('system-node')).toHaveCount(3);
	await expect.poll(async () => (await nodePosition(page, 'Jita')).left).toBe(340);

	// And going back to the map's own mode restores the dragged positions.
	await page.getByTestId('placement-manual').click();
	await expect.poll(async () => (await nodePosition(page, 'Jita')).left).toBe(900);
});

test('the switcher is hidden unless the map hands the choice over', async ({ page, api }) => {
	const { mapId } = await seedChain(api, 'E2E TreeNoOverride');
	await gotoApp(page, `/maps/${mapId}`);
	await expect(page.getByTestId('placement-controls')).toHaveCount(0);
});

test('shift-drag still rubber-bands while the layout places the systems', async ({ page, api }) => {
	const { mapId } = await seedChain(api, 'E2E TreeMarquee');
	await api.post(`/api/maps/${mapId}/update`, {
		data: { map_id: mapId, layout: 'tree' },
	});

	await gotoApp(page, `/maps/${mapId}`);
	await expect(page.getByTestId('system-node')).toHaveCount(3);
	const canvas = (await page.getByTestId('map-canvas').boundingBox())!;

	// A plain drag pans: the systems keep their laid-out positions and nothing is picked.
	await page.mouse.move(canvas.x + 600, canvas.y + 420);
	await page.mouse.down();
	await page.mouse.move(canvas.x + 500, canvas.y + 380, { steps: 8 });
	await page.mouse.up();
	await expect(page.locator('[data-testid="system-node"].bg-amber-100')).toHaveCount(0);

	// With shift held, the same drag draws a box and takes what it covers.
	await page.keyboard.down('Shift');
	await page.mouse.move(canvas.x + 20, canvas.y + 20);
	await page.mouse.down();
	await page.mouse.move(canvas.x + 900, canvas.y + 600, { steps: 12 });
	await page.mouse.up();
	await page.keyboard.up('Shift');

	await expect
		.poll(
			async () =>
				await page
					.locator('[data-testid="system-node"]')
					.evaluateAll((nodes) => nodes.filter((n) => n.className.includes('amber')).length),
		)
		.toBeGreaterThan(0);
});

test("the map's setup asks for the placement, and the map takes it", async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E TreeSetup');
	// Straight in without the usual introduction skip: setting it up is the point.
	await page.goto(`http://localhost:5173/maps/${mapId}`);
	await page.waitForSelector('html[data-hydrated="true"]');

	// Welcome and permissions share the first page; the next decides what the map does.
	await page.getByTestId('introduction-next').click();

	const placement = page.getByTestId('setup-placement');
	await expect(placement).toContainText('Changeable later');
	await expect(placement).toContainText('drag the systems into shape');
	await expect(placement).toContainText('draws the chain as a tree');

	await page.getByTestId('setup-placement-tree').click();
	await expect
		.poll(async () => (await (await api.get(`/api/maps/${mapId}`)).json()).map.layout)
		.toBe('tree');

	await api.delete(`/api/maps/${mapId}`);
});

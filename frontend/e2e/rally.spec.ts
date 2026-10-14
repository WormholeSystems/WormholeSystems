import { expect, gotoApp, test } from './fixtures';
import { addSystem, createMap } from './arrange';

// The rally badges: where the fleet is forming up, and how far that is from the staging
// system. The count is the map's own, home to rally, so everyone reads the same number.

const JITA = 30000142;
const PERIMETER = 30000144; // one gate from Jita
const AMARR = 30002187;
const DODIXIE = 30002659;

test('the rally badge names the system and counts the jumps home', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Rally');
	const home = await addSystem(api, mapId, JITA, { x: 200, y: 200 });
	const rally = await addSystem(api, mapId, PERIMETER, { x: 500, y: 200 });

	await gotoApp(page, `/maps/${mapId}`);
	// Nothing to form up on yet, so nothing is drawn.
	await expect(page.getByTestId('rally-badge')).toHaveCount(0);

	for (const [path, id] of [
		['set-home', home],
		['set-rally', rally],
	] as const) {
		const res = await api.post(`/api/maps/${mapId}/systems/${path}`, {
			data: { map_id: mapId, map_solar_system_id: id, value: true },
		});
		expect(res.ok()).toBe(true);
	}

	const badge = page.getByTestId('rally-badge');
	await expect(badge).toBeVisible();
	await expect(badge).toContainText('Rally point');
	await expect(badge).toContainText('Perimeter');

	// Jita and Perimeter are one gate apart, and the route is reachable without the map
	// having an edge for it: k-space routes through the stargate graph.
	const jumps = page.getByTestId('rally-jumps');
	await expect(jumps).toHaveText('1j');

	// The count opens the route it counted.
	await jumps.click();
	const route = page.getByTestId('route-list');
	await expect(route).toBeVisible();
	await expect(route).toContainText('Perimeter');

	await api.delete(`/api/maps/${mapId}`);
});

test('two rally points each get a badge, and a third replaces the oldest', async ({
	page,
	api,
}) => {
	const mapId = await createMap(api, 'E2E Two Rallies');
	const perimeter = await addSystem(api, mapId, PERIMETER, { x: 200, y: 200 });
	const amarr = await addSystem(api, mapId, AMARR, { x: 500, y: 200 });
	const dodixie = await addSystem(api, mapId, DODIXIE, { x: 800, y: 200 });

	async function rally(id: number, value: boolean) {
		const res = await api.post(`/api/maps/${mapId}/systems/set-rally`, {
			data: { map_id: mapId, map_solar_system_id: id, value },
		});
		expect(res.ok()).toBe(true);
	}

	await gotoApp(page, `/maps/${mapId}`);
	const badges = page.getByTestId('rally-badge');
	const rallyNodes = page.locator('[data-testid="system-node"][data-rally="true"]');

	await rally(perimeter, true);
	await rally(amarr, true);
	await expect(badges).toHaveCount(2);
	await expect(rallyNodes).toHaveCount(2);
	await expect(badges.filter({ hasText: 'Perimeter' })).toBeVisible();
	await expect(badges.filter({ hasText: 'Amarr' })).toBeVisible();

	// Perimeter was first, so it makes way.
	await rally(dodixie, true);
	await expect(badges).toHaveCount(2);
	await expect(badges.filter({ hasText: 'Perimeter' })).toHaveCount(0);
	await expect(badges.filter({ hasText: 'Dodixie' })).toBeVisible();

	// Clearing one leaves the other standing.
	await rally(amarr, false);
	await expect(badges).toHaveCount(1);
	await expect(badges).toContainText('Dodixie');
	await expect(rallyNodes).toHaveCount(1);

	await api.delete(`/api/maps/${mapId}`);
});

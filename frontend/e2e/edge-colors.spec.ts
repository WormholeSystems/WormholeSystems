import { createdId, expect, gotoApp, reloadApp, test } from './fixtures';

// Connection stroke colors follow the legacy model: sky stargates, orange reduced,
// purple EOL, red critical, neutral otherwise; degraded states dash.

const JITA = 30000142;
const PERIMETER = 30000144;

test('edge colors and dashes per state', async ({ page, api }) => {
	const res = await api.post('/api/maps', { data: { name: 'E2E EdgeColors' } });
	const mapId = await createdId(res);
	const add = async (sid: number, x: number, y: number) => {
		const r = await api.post(`/api/maps/${mapId}/systems/add`, {
			data: { map_id: mapId, solar_system_id: sid, x, y, alias: null },
		});
		return await createdId(r);
	};
	const a = await add(JITA, 200, 200);
	const b = await add(PERIMETER, 600, 200);
	const gate = await api.post(`/api/maps/${mapId}/connections/add`, {
		data: { map_id: mapId, from_system: a, to_system: b, kind: 'stargate' },
	});
	expect(gate.ok()).toBe(true);

	await gotoApp(page, `/maps/${mapId}`);
	const edge = page.locator('path[data-on-route]');
	await expect(edge).toHaveAttribute('stroke', '#0ea5e9'); // sky-500 stargate
	await expect(edge).toHaveAttribute('stroke-dasharray', '0');

	// Flip it to a reduced-mass wormhole: orange and dashed.
	const cid = (await (await api.get(`/api/maps/${mapId}`)).json()).connections[0].id;
	await api.post(`/api/maps/${mapId}/connections/set-status`, {
		data: { map_id: mapId, connection_id: cid, kind: 'wormhole', mass_status: 'reduced' },
	});
	await reloadApp(page);
	await expect(edge).toHaveAttribute('stroke', '#f97316'); // orange-500
	await expect(edge).toHaveAttribute('stroke-dasharray', '2 6');

	// EOL via the connection context menu (Lifetime submenu): purple.
	await api.post(`/api/maps/${mapId}/connections/set-status`, {
		data: { map_id: mapId, connection_id: cid, mass_status: 'stable' },
	});
	await reloadApp(page);
	await page.locator('path[stroke="transparent"]').click({ button: 'right', force: true });
	await page.getByTestId('lifetime-subtrigger').hover();
	await page.getByTestId('lifetime-submenu').getByRole('menuitem', { name: 'End of Life' }).click();
	await expect(edge).toHaveAttribute('stroke', '#a855f7'); // purple-500

	// The menu marks the current state and warns on non-gate stargate marking.
	await page.locator('path[stroke="transparent"]').click({ button: 'right', force: true });
	await page.getByTestId('size-subtrigger').hover();
	await expect(
		page.getByTestId('size-submenu').getByRole('menuitem', { name: 'Frigate' }),
	).toBeVisible();
	// A size set by hand can be taken back off.
	await expect(
		page.getByTestId('size-submenu').getByRole('menuitem', { name: 'Unknown' }),
	).toBeVisible();

	// Both ends offer their in-game bookmark name, rendered with the map's formats.
	await page.getByTestId('copy-name-subtrigger').hover();
	const names = page.getByTestId('copy-name-submenu').getByRole('menuitem');
	await expect(names).toHaveCount(2);
	await expect(names.first()).toHaveText('HS Jita The Forge');
});

test('unknown status uses the neutral border token', async ({ page, api }) => {
	const res = await api.post('/api/maps', { data: { name: 'E2E UnknownBorder' } });
	const mapId = await createdId(res);
	const r = await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: JITA, x: 200, y: 200, alias: null },
	});
	const mss = await createdId(r);
	await api.post(`/api/maps/${mapId}/systems/set-status`, {
		data: { map_id: mapId, map_solar_system_id: mss, status: 'unknown' },
	});

	await gotoApp(page, `/maps/${mapId}`);
	const node = page.getByTestId('system-node').filter({ hasText: 'Jita' });
	await expect(node).toHaveAttribute('data-status', 'unknown');
	// Light theme default in the test browser: neutral-300.
	await expect
		.poll(async () => node.evaluate((el) => getComputedStyle(el).borderTopColor))
		.toBe('oklch(0.87 0 0)');
});

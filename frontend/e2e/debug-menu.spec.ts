import { createdId, expect, gotoApp, test } from './fixtures';

// The development-only debug menu, and the map it builds: three pinned roots, a branching
// tree, and the loops that a tree layout cannot draw as a tree.

test('the debug menu builds a chain worth stress-testing the canvas with', async ({
	page,
	api,
}) => {
	const res = await api.post('/api/maps', { data: { name: 'E2E Debug' } });
	const mapId = await createdId(res);

	await gotoApp(page, `/maps/${mapId}`);
	await page.getByTestId('map-canvas').click({ button: 'right', position: { x: 400, y: 300 } });
	await page.getByTestId('debug-subtrigger').hover();
	await page.getByRole('menuitem', { name: 'Add a stress-test chain' }).click();

	// 3 roots, 2 children each, three levels deep. It builds a request at a time, so wait
	// for the finished shape rather than for the first paint.
	await expect(page.getByTestId('system-node')).toHaveCount(45, { timeout: 60_000 });
	await expect
		.poll(async () => (await (await api.get(`/api/maps/${mapId}`)).json()).connections.length, {
			timeout: 60_000,
		})
		// The 42 tree edges, plus the three loops back into it.
		.toBe(45);

	const view = await (await api.get(`/api/maps/${mapId}`)).json();
	expect(view.systems.filter((s: { is_pinned: boolean }) => s.is_pinned)).toHaveLength(3);

	await api.delete(`/api/maps/${mapId}`);
});

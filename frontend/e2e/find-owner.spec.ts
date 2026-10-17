import { createdId, expect, gotoApp, test } from './fixtures';

// Finding the nearest station of a particular NPC corporation. Same machinery as the
// "nearest repair shop" search: a named set of stations, and one relaxation over the graph.

const JITA = 30000142;

test('the navigation card finds the nearest station of a chosen owner', async ({ page, api }) => {
	const res = await api.post('/api/maps', { data: { name: 'E2E FindOwner' } });
	const mapId = await createdId(res);
	const add = await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: JITA, x: 200, y: 200, alias: null },
	});
	expect(add.ok()).toBe(true);
	const jita = await createdId(add);

	await gotoApp(page, `/maps/${mapId}`);
	// The search routes from somewhere, and the selected system is that somewhere.
	await page.getByTestId('system-node').filter({ hasText: 'Jita' }).click();

	await page.getByTestId('navigation-tab-find').click();
	await page.getByTestId('find-condition').click();
	await page.getByRole('option', { name: /station/i }).click();

	// Typing narrows 185 owners down to the one being looked for.
	await page.getByTestId('find-owner').click();
	await page.getByTestId('find-owner-search').fill('hyasyoda');
	await page.getByRole('option', { name: 'Hyasyoda Corporation' }).click();

	// Hyasyoda has a station in Jita itself, so the nearest is nought jumps away.
	const first = page.getByTestId('find-row').first();
	await expect(first).toContainText('Jita');

	// And it names the station it found, the way a service search does.
	await first.click();
	await expect(page.getByTestId('find-station').first()).toContainText('Hyasyoda');

	await api.delete(`/api/maps/${mapId}`);
	void jita;
});

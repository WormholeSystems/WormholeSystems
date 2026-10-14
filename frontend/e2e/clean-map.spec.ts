import { expect, gotoApp, test } from './fixtures';
import { addSystem, createMap } from './arrange';

// Cleaning the map: the branches nothing reaches any more, once a hole has collapsed and
// taken the way to them with it.

const J122515 = 31001882;
const JITA = 30000142;
const PERIMETER = 30000144;

test('the map says when a branch is adrift, and says what cleaning takes', async ({
	page,
	api,
}) => {
	const mapId = await createMap(api, 'E2E Clean');
	const home = await addSystem(api, mapId, J122515, { x: 200, y: 300 });
	const attached = await addSystem(api, mapId, JITA, { x: 500, y: 300 });
	await addSystem(api, mapId, PERIMETER, { x: 800, y: 300 });
	await api.post(`/api/maps/${mapId}/connections/add`, {
		data: { map_id: mapId, from_system: home, to_system: attached, kind: 'wormhole' },
	});
	await api.post(`/api/maps/${mapId}/systems/set-home`, {
		data: { map_id: mapId, map_solar_system_id: home, value: true },
	});

	await gotoApp(page, `/maps/${mapId}`);
	await expect(page.getByTestId('system-node')).toHaveCount(3);

	// Perimeter hangs off nothing, so it is the one adrift.
	const badge = page.getByTestId('orphaned-badge');
	await expect(badge).toHaveText('1 adrift');
	await badge.click();

	// The dialog names it before doing anything.
	const dialog = page.getByTestId('clean-map-dialog');
	await expect(dialog.getByTestId('clean-list')).toContainText('Perimeter');
	await expect(dialog.getByTestId('clean-list')).not.toContainText('Jita');

	await dialog.getByTestId('clean-map-confirm').click();
	await expect(page.getByTestId('system-node')).toHaveCount(2);
	await expect(page.getByTestId('orphaned-badge')).toHaveCount(0);
	// And it says so, because the branch it took was somewhere you were not looking.
	await expect(page.locator('[data-sonner-toast]')).toContainText('Map cleaned');

	await api.delete(`/api/maps/${mapId}`);
});

test('a map with nothing anchored is never adrift', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E CleanNoAnchor');
	await addSystem(api, mapId, J122515, { x: 200, y: 300 });
	await addSystem(api, mapId, JITA, { x: 500, y: 300 });

	await gotoApp(page, `/maps/${mapId}`);
	await expect(page.getByTestId('system-node')).toHaveCount(2);
	// Without a pinned or home system there is nothing to measure "reachable" from, and
	// the answer must not be "all of it".
	await expect(page.getByTestId('orphaned-badge')).toHaveCount(0);

	await api.delete(`/api/maps/${mapId}`);
});

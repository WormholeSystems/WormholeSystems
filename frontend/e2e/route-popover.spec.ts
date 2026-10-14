import { createdId, expect, gotoApp, test } from './fixtures';
import { staysHidden } from './arrange';
import { setCharacterPresence } from './db';

const JITA = 30000142;
const AMARR = 30002187;

// The route popover: a header over a list of hops, each with the holder of the system it
// passes through.

async function openRoute(
	page: import('@playwright/test').Page,
	api: import('@playwright/test').APIRequestContext,
) {
	const res = await api.post('/api/maps', { data: { name: 'E2E RoutePopover' } });
	const mapId = await createdId(res);
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: JITA, x: 200, y: 200, alias: null },
	});
	const me = await (await api.get('/api/me')).json();
	await setCharacterPresence(me.character_id ?? me.id, JITA);
	await api.post(`/api/maps/${mapId}/settings/user`, { data: { tracking_allowed: true } });
	await api.post(`/api/maps/${mapId}/watchlist/add`, {
		data: { map_id: mapId, solar_system_id: AMARR },
	});

	await gotoApp(page, `/maps/${mapId}`);
	await page.getByTestId('map-loading').waitFor({ state: 'detached', timeout: 15000 });
	const badge = page.getByTestId('jump-badge').last();
	await badge.waitFor({ timeout: 15000 });
	await badge.click();
	await page.getByTestId('route-popover').waitFor();
}

// Popover.Content defaults to a flex column with a gap. The popover lays its own sections
// out, so the gap showed up as a blank band the height of a row above the first hop.
test('the list starts right under the header', async ({ page, api }) => {
	await openRoute(page, api);
	const gap = await page.evaluate(() => {
		const pop = document.querySelector('[data-testid="route-popover"]')!;
		const list = document.querySelector('[data-testid="route-list"]')!;
		return list.getBoundingClientRect().top - pop.firstElementChild!.getBoundingClientRect().bottom;
	});
	expect(gap).toBeLessThanOrEqual(8);
});

// Opening a popover moves focus into it, and that used to land on the first tooltip
// trigger in the list and pop a tooltip nobody asked for.
test('opening it pops no tooltip', async ({ page, api }) => {
	await openRoute(page, api);
	await staysHidden(page.locator('[data-slot="tooltip-content"]'), 1_000);
});

test('a hop names the holder of the system on hover', async ({ page, api }) => {
	await openRoute(page, api);
	const badge = page.getByTestId('sovereignty-badge').first();
	await badge.scrollIntoViewIfNeeded();
	await badge.hover();
	await expect(page.locator('[data-slot="tooltip-content"]').first()).toBeVisible();
});

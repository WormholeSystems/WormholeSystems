import { createdId, expect, gotoApp, reloadApp, test } from './fixtures';
import { addSystem, createMap } from './arrange';
import { createIdentity, grantAccess } from './db';

// The redesigned navigation panel: always-visible A→B route, the shared watchlist
// with unified-origin jump counts, per-hop ignore, route settings, and Find.

const JITA = 30000142;
const AMARR = 30002187;
const PERIMETER = 30000144;
const J122515 = 31001882;

async function pickSystem(
	page: import('@playwright/test').Page,
	picker: string,
	query: string,
	name: string,
) {
	await page.getByTestId(picker).click();
	const input = page.getByPlaceholder('Search…').last();
	await input.fill(query);
	await expect(page.getByRole('option', { name: new RegExp(name) }).last()).toBeVisible();
	await input.press('Enter');
	// Confirm the pick landed before moving on (guards against stale dropdown races).
	await expect(page.getByTestId(picker)).toContainText(name);
}

/** Pick from a watchlist row's ⋮ menu. */
async function rowAction(
	page: import('@playwright/test').Page,
	row: import('@playwright/test').Locator,
	item: RegExp,
) {
	await row.getByTestId('watchlist-row-menu').click();
	await page.getByRole('menuitem', { name: item }).click();
}

async function setRoute(page: import('@playwright/test').Page, from: string, to: string) {
	await pickSystem(page, 'system-picker-origin', from.toLowerCase(), from);
	await pickSystem(page, 'system-picker-destination', to.toLowerCase(), to);
}

test('watchlist: defaults, add, unified origin badge, pin, remove, reload persistence', async ({
	page,
	api,
}) => {
	const mapId = await createMap(api, 'E2E NavWatch');
	await addSystem(api, mapId, JITA, { x: 200, y: 200 });
	await gotoApp(page, `/maps/${mapId}`);

	// Every new map arrives with the five trade hubs watched.
	const rows = page.getByTestId('watchlist-row');
	await expect(rows).toHaveCount(5);

	// Add one that is not a hub, via the header plus.
	await page.getByTestId('watchlist-add').click();
	await page.getByPlaceholder('Watch a system…').fill('ashab');
	await page.getByRole('option', { name: /Ashab/ }).first().click();
	await expect(rows).toHaveCount(6);

	const row = rows.filter({ hasText: 'Amarr' });
	// No origin yet → no jump count.
	await expect(row.getByText('--')).toBeVisible();

	// Activating Jita on the map becomes the origin: header + 11j badge.
	await page.getByTestId('system-node').filter({ hasText: 'Jita' }).click();
	await expect(page.getByTestId('navigation-card').getByTestId('route-origin')).toHaveText(
		'from Jita',
	);
	await expect(row.getByText('11j')).toBeVisible();

	// The jump badge opens the hop popover.
	await row.getByTestId('jump-badge').click();
	const popover = page.getByTestId('route-popover');
	await expect(popover).toBeVisible();
	await expect(popover.getByText('11 jumps')).toBeVisible();
	const lastHop = popover.getByTestId('route-list').getByText('Amarr');
	await lastHop.scrollIntoViewIfNeeded();
	await expect(lastHop).toBeVisible();
	await page.keyboard.press('Escape');

	// The hubs come pinned; pinning the one we added survives a reload (server-side rows).
	const ashab = rows.filter({ hasText: 'Ashab' });
	await expect(ashab.getByTestId('watchlist-pinned')).toHaveCount(0);
	await rowAction(page, ashab, /Pin as/);
	await expect(ashab.getByTestId('watchlist-pinned')).toBeVisible();
	await expect
		.poll(async () => {
			const list = await (await api.get(`/api/maps/${mapId}/watchlist`)).json();
			return list.every((e: { is_pinned: boolean }) => e.is_pinned);
		})
		.toBe(true);
	await reloadApp(page);
	await expect(rows).toHaveCount(6);

	// Removing every row shows the empty state.
	for (const name of ['Ashab', 'Jita', 'Amarr', 'Dodixie', 'Rens', 'Hek']) {
		await rowAction(page, rows.filter({ hasText: name }), /Remove/);
	}
	await expect(rows).toHaveCount(0);
	await expect(page.getByText('Watchlist empty')).toBeVisible();
});

test('route ignore: per-hop X reroutes, Clear restores', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E NavIgnore');
	await gotoApp(page, `/maps/${mapId}`);
	await setRoute(page, 'Jita', 'Amarr');
	await expect(page.getByTestId('route-jumps')).toHaveText('11 jumps');

	// Ignore a middle hop: the route now avoids it and the indicator appears.
	await page.getByTestId('route-list').getByLabel('Ignore Ashab').click();
	await expect(page.getByTestId('route-list').getByText('Ashab')).toHaveCount(0);
	await expect(page.getByTestId('clear-ignored')).toContainText('1 ignored');
	await page.getByTestId('clear-ignored').click();
	await expect(page.getByTestId('clear-ignored')).toHaveCount(0);
	await expect(page.getByTestId('route-jumps')).toHaveText('11 jumps');
});

test('route settings: preference persists; lifetime tolerance drops EOL holes', async ({
	page,
	api,
}) => {
	const mapId = await createMap(api, 'E2E NavSettings');
	const a = await addSystem(api, mapId, JITA, { x: 200, y: 200 });
	const b = await addSystem(api, mapId, AMARR, { x: 560, y: 200 });
	const conn = await api.post(`/api/maps/${mapId}/connections/add`, {
		data: { map_id: mapId, from_system: a, to_system: b, kind: 'wormhole' },
	});
	const connId = await createdId(conn);
	await api.post(`/api/maps/${mapId}/connections/set-status`, {
		data: { map_id: mapId, connection_id: connId, time_status: 'eol' },
	});

	await gotoApp(page, `/maps/${mapId}`);
	await setRoute(page, 'Jita', 'Amarr');
	// Default tolerance allows the EOL hole: one jump.
	await expect(page.getByTestId('route-jumps')).toHaveText('1 jumps');

	// Healthy Only drops the edge → the 11-jump gate route.
	await page.getByTestId('route-settings').click();
	await page.getByTestId('setting-route_allow_time_status').click();
	await page.getByRole('option', { name: 'Healthy Only' }).click();
	await expect(page.getByTestId('route-jumps')).toHaveText('11 jumps');

	// Preference persists server-side across reloads.
	await page.getByTestId('setting-route_preference').click();
	await page.getByRole('option', { name: 'Safer' }).click();
	await page.keyboard.press('Escape');
	// The header only reads 'Safer' once the server has confirmed the setting, so this is
	// what makes the reload below a real test of persistence rather than a race with it.
	await expect(page.getByTestId('navigation-card').getByText('Safer')).toBeVisible();
	await reloadApp(page);
	await expect(page.getByTestId('navigation-card').getByText('Safer')).toBeVisible();
});

test('find: closest systems from the unified origin', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E NavFind');
	const jita = await addSystem(api, mapId, JITA, { x: 200, y: 200 });
	// A mapped neighbour, so route highlighting has an edge to light up.
	const perimeter = await addSystem(api, mapId, PERIMETER, { x: 560, y: 200 });
	await api.post(`/api/maps/${mapId}/connections/add`, {
		data: { map_id: mapId, from_system: jita, to_system: perimeter, kind: 'wormhole' },
	});
	await gotoApp(page, `/maps/${mapId}`);
	await page.getByTestId('system-node').filter({ hasText: 'Jita' }).click();

	await page.getByTestId('navigation-tab-find').click();
	// Jove observatories: results with jump badges appear.
	await expect(page.getByTestId('find-row').first()).toBeVisible();

	// Stations with no filters: Jita itself matches at zero jumps, nothing to expand.
	await page.getByTestId('find-condition').click();
	await page.getByRole('option', { name: 'Stations' }).click();
	await expect(page.getByTestId('find-row').first().getByText('Jita')).toBeVisible();
	await expect(page.getByTestId('find-row').first().getByText('0j')).toBeVisible();
	await expect(page.getByTestId('find-stations-indicator')).toHaveCount(0);

	// A service filter (seeded from the SDE) names the concrete stations, collapsed
	// behind a per-row toggle so long lists stay scannable.
	await page.getByTestId('find-service').click();
	await page.getByRole('option', { name: 'Repair Facilities' }).click();
	await expect(page.getByTestId('find-row').first().getByText('Jita')).toBeVisible();
	await expect(page.getByTestId('find-row').first().getByText('0j')).toBeVisible();
	await expect(page.getByTestId('find-station')).toHaveCount(0);
	// Clicking anywhere on the row expands it (the count chevron is only an indicator).
	await page.getByTestId('find-row').first().click();
	const stations = page.getByTestId('find-station');
	await expect(stations.first()).toBeVisible();
	await expect(stations.first()).toContainText('Jita');
	await expect(page.getByTestId('find-row').first()).toHaveAttribute('aria-expanded', 'true');
	// The jump badge opens the hop popover without collapsing the row.
	await page.getByTestId('find-row').first().getByTestId('jump-badge').click();
	await expect(page.getByTestId('route-popover')).toBeVisible();
	await expect(page.getByTestId('find-row').first()).toHaveAttribute('aria-expanded', 'true');
	await page.keyboard.press('Escape');
	// Clicking the row again collapses it.
	await page.getByTestId('find-row').first().click();
	await expect(page.getByTestId('find-station')).toHaveCount(0);

	// Security Offices only exist on CONCORD lowsec stations (in-game quirk), so
	// highsec Jita is never a match and every result names a CONCORD station.
	await page.getByTestId('find-service').click();
	await page.getByRole('option', { name: 'Security Office' }).click();
	await expect(page.getByTestId('find-row').first()).toBeVisible();
	await expect(page.getByTestId('find-row').first().getByText('Jita')).toHaveCount(0);
	await page.getByTestId('find-row').first().click();
	await expect(page.getByTestId('find-station').first()).toContainText('CONCORD');

	// The owner filter searches corporations and whole factions, faction-sorted; picking
	// Caldari Navy narrows Jita's stations to its own, intersected with the service.
	await page.getByTestId('find-service').click();
	await page.getByRole('option', { name: 'Repair Facilities' }).click();
	await page.getByTestId('find-owner').click();
	await page.getByTestId('find-owner-search').fill('caldari navy');
	await page.getByRole('option', { name: 'Caldari Navy' }).click();
	await expect(page.getByTestId('find-row').first().getByText('Jita')).toBeVisible();
	await page.getByTestId('find-row').first().click();
	await expect(page.getByTestId('find-station').first()).toContainText('Caldari Navy');
	// A faction pick matches every member corporation's stations: in Jita that leaves
	// exactly Quafe's embassy (the one Gallente-owned station with repair).
	await page.getByTestId('find-owner').click();
	await page.getByTestId('find-owner-search').fill('gallente fed');
	await page.getByRole('option', { name: 'Gallente Federation' }).click();
	await page.getByTestId('find-row').first().click();
	await expect(page.getByTestId('find-station')).toHaveCount(1);
	await expect(page.getByTestId('find-station')).toContainText('Quafe');
	await page.getByTestId('find-owner').click();
	await page.getByRole('option', { name: 'Any owner' }).click();

	// Hovering a station keeps its system's route highlighted on the canvas (the
	// station belongs to that system, so the highlight must not drop). Perimeter is
	// one mapped jump away, so its route lights up the connection.
	const perimeterRow = page.getByTestId('find-row').filter({ hasText: 'Perimeter' });
	await perimeterRow.hover();
	const onRoute = page.locator('path[data-on-route="true"]');
	await expect(onRoute).toHaveCount(1);
	await perimeterRow.click();
	// Only this row is expanded, so its stations are the visible ones.
	await page.getByTestId('find-station').first().hover();
	await expect(onRoute).toHaveCount(1);
	// Leaving the list entirely clears it again.
	await page.getByTestId('find-condition').hover();
	await expect(onRoute).toHaveCount(0);

	// A station row can be right-clicked to set it as the in-game destination.
	await page.getByTestId('find-station').first().click({ button: 'right' });
	await expect(page.getByTestId('destination-menu')).toBeVisible();
	await expect(
		page.getByTestId('destination-menu').getByTestId('menu-set-destination'),
	).toBeVisible();
	await page.keyboard.press('Escape');
});

test('the pickers suggest systems already in play, before anything is typed', async ({
	page,
	api,
}) => {
	const mapId = await createMap(api, 'E2E NavPicks');
	await addSystem(api, mapId, J122515, { x: 200, y: 200 });
	await gotoApp(page, `/maps/${mapId}`);
	await page.getByTestId('system-node').filter({ hasText: 'J122515' }).click();

	// Opening a picker offers the selected system straight away, with why it is offered.
	await page.getByTestId('system-picker-origin').click();
	const suggestion = page.getByTestId('picker-suggestion').filter({ hasText: 'J122515' });
	// A chip, not a result row: it says why it is offered on hover rather than in a column.
	await suggestion.hover();
	await expect(page.locator('[data-slot="tooltip-content"]')).toHaveText('Selected system');
	await suggestion.click();
	await expect(page.getByTestId('system-picker-origin')).toContainText('J122515');

	// The other end offers it too, and a suggestion already chosen here drops out of its own
	// picker rather than offering a no-op.
	await page.getByTestId('system-picker-destination').click();
	await expect(page.getByTestId('picker-suggestion').filter({ hasText: 'J122515' })).toBeVisible();
	await page.keyboard.press('Escape');
	await page.getByTestId('system-picker-origin').click();
	await expect(page.getByTestId('picker-suggestion').filter({ hasText: 'J122515' })).toHaveCount(0);
});

test('typing in a picker replaces the suggestions with search results', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E NavSearch');
	await addSystem(api, mapId, J122515, { x: 200, y: 200 });
	await gotoApp(page, `/maps/${mapId}`);
	await page.getByTestId('system-node').filter({ hasText: 'J122515' }).click();

	await page.getByTestId('system-picker-origin').click();
	await expect(page.getByTestId('picker-suggestion').first()).toBeVisible();
	await page.getByPlaceholder('Search…').fill('Amarr');
	await expect(page.getByTestId('picker-suggestion')).toHaveCount(0);
	await expect(
		page.getByTestId('picker-result').filter({ hasText: 'Amarr' }).first(),
	).toBeVisible();
});

test('unreachable watchlist rows stay the same height as reachable ones', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E WatchHeight');
	await addSystem(api, mapId, JITA, { x: 200, y: 200 });
	// Amarr is gate-reachable from Jita (and already there as a default hub); the wormhole
	// system is not reachable at all.
	for (const sys of [AMARR, J122515]) {
		await api.post(`/api/maps/${mapId}/watchlist/add`, {
			data: { map_id: mapId, solar_system_id: sys },
		});
	}

	// With an origin, one row shows jumps and the other shows the no-route dashes.
	await gotoApp(page, `/maps/${mapId}?system=${JITA}`);
	await expect(page.getByTestId('watchlist-row')).toHaveCount(6);
	await expect(page.getByTestId('route-jumps-badge').first()).toBeVisible();
	const mixed = await page
		.getByTestId('watchlist-row')
		.evaluateAll((els) => els.map((e) => Math.round(e.getBoundingClientRect().height)));

	// With no origin at all, every row shows the dashes. A hyphen is a break opportunity,
	// so without care the column collapses to one dash and every row grows a line.
	await gotoApp(page, `/maps/${mapId}`);
	await expect(page.getByTestId('watchlist-row')).toHaveCount(6);
	const none = await page
		.getByTestId('watchlist-row')
		.evaluateAll((els) => els.map((e) => Math.round(e.getBoundingClientRect().height)));

	expect(new Set([...mixed, ...none]).size).toBe(1);
});

test('a wormhole hop names the signature to warp to', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E RouteWH');
	const a = await addSystem(api, mapId, J122515, { x: 200, y: 200 });
	const b = await addSystem(api, mapId, JITA, { x: 560, y: 200 });
	const conn = await api.post(`/api/maps/${mapId}/connections/add`, {
		data: { map_id: mapId, from_system: a, to_system: b, kind: 'wormhole' },
	});
	const connId = await createdId(conn);

	await gotoApp(page, `/maps/${mapId}`);
	await setRoute(page, 'J122515', 'Jita');

	// The marker stays the same width whatever it knows; hovering is what tells you.
	const marker = page.getByTestId('route-wormhole').first();
	await expect(marker).toHaveText('WH');
	await marker.hover();
	await expect(page.locator('[data-slot="tooltip-content"]')).toContainText('not scanned');

	// Move off, so the next hover opens a fresh tooltip rather than reading the open one.
	await page.mouse.move(0, 0);

	const sig = await api.post(`/api/maps/${mapId}/signatures/add`, {
		data: { map_id: mapId, solar_system_id: J122515, signature_id: 'QRS-481', group: 'wormhole' },
	});
	const sigId = await createdId(sig);
	await api.post(`/api/maps/${mapId}/signatures/link`, {
		data: { map_id: mapId, signature_pk: sigId, connection_id: connId },
	});

	// Linked, the tooltip names the signature: that is what you look for in the scanner.
	await expect(marker).toHaveText('WH');
	await marker.hover();
	await expect(page.locator('[data-slot="tooltip-content"]')).toContainText(
		'Take wormhole QRS-481',
	);
});

test('viewers see the watchlist read-only', async ({ page, api, browser }) => {
	const mapId = await createMap(api, 'E2E NavViewer');
	await addSystem(api, mapId, JITA, { x: 200, y: 200 });
	const owner = await api.post(`/api/maps/${mapId}/watchlist/add`, {
		data: { map_id: mapId, solar_system_id: AMARR },
	});
	expect(owner.ok()).toBe(true);
	const viewer = await createIdentity(3);
	await grantAccess(mapId, viewer.characterId, 'viewer');

	const ctx = await browser.newContext();
	await ctx.addCookies([
		{ name: 'ws_session', value: viewer.session, domain: 'localhost', path: '/' },
	]);
	const viewerPage = await ctx.newPage();
	await viewerPage.goto(`http://localhost:5173/maps/${mapId}`);
	await viewerPage.waitForSelector('html[data-hydrated="true"]');

	const row = viewerPage.getByTestId('watchlist-row');
	await expect(row).toHaveCount(5);
	await expect(viewerPage.getByTestId('watchlist-add')).toBeDisabled();
	// No menu to change anything, but the pin still shows.
	await expect(row.getByTestId('watchlist-row-menu')).toHaveCount(0);
	await expect(row.first().getByTestId('watchlist-pinned')).toBeVisible();
	await ctx.close();
	void page;
});

test('watchlist, personal and find are tabs, and the card remembers which one was open', async ({
	page,
	api,
}) => {
	const mapId = await createMap(api, 'E2E NavTabs');
	await addSystem(api, mapId, JITA, { x: 200, y: 200 });
	await gotoApp(page, `/maps/${mapId}`);
	const card = page.getByTestId('navigation-card');
	const shared = card.getByTestId('watchlist-shared').getByTestId('watchlist-row');

	await expect(shared).toHaveCount(5);
	await expect(card.getByTestId('find-condition')).toBeHidden();

	// Find keeps the origin but has no destination, and measures from the selected system.
	await card.getByTestId('navigation-tab-find').click();
	await expect(card.getByTestId('find-condition')).toBeVisible();
	await expect(shared.first()).toBeHidden();
	await expect(card.getByTestId('system-picker-origin')).toBeVisible();
	await expect(card.getByTestId('system-picker-destination')).toHaveCount(0);
	// The header holds still: the + stays, just not usable here.
	await expect(card.getByTestId('watchlist-add')).toBeDisabled();
	await expect(card.getByTestId('route-settings')).toBeVisible();
	await page.getByTestId('system-node').filter({ hasText: 'Jita' }).click();
	await expect(card.getByTestId('route-origin')).toHaveText('from Jita');
	await expect(card.getByTestId('find-row').first()).toBeVisible();

	await reloadApp(page);
	await expect(card.getByTestId('find-condition')).toBeVisible();
	await card.getByTestId('navigation-tab-watchlist').click();
	await expect(shared).toHaveCount(5);
	await expect(card.getByTestId('system-picker-destination')).toBeVisible();
});

test('a personal watchlist is yours alone and leaves the shared one alone', async ({
	page,
	api,
}) => {
	const mapId = await createMap(api, 'E2E NavPersonal');
	await gotoApp(page, `/maps/${mapId}`);
	const card = page.getByTestId('navigation-card');
	const personal = card.getByTestId('watchlist-personal').getByTestId('watchlist-row');

	await card.getByTestId('navigation-tab-personal').click();
	await expect(personal).toHaveCount(0);
	await card.getByTestId('watchlist-add').click();
	await page.getByPlaceholder('Watch a system…').fill('ashab');
	await page.getByRole('option', { name: /Ashab/ }).first().click();
	await expect(personal).toHaveCount(1);
	await expect(card.getByTestId('navigation-tab-personal')).toContainText('1');

	// The map's list did not grow, and the server kept it apart.
	const sharedList = await (await api.get(`/api/maps/${mapId}/watchlist`)).json();
	expect(sharedList).toHaveLength(5);
	const mine = await (await api.get(`/api/maps/${mapId}/watchlist/personal`)).json();
	expect(mine).toHaveLength(1);

	// Pinned, it is offered in the route pickers.
	await rowAction(page, personal, /Pin as/);
	await card.getByTestId('system-picker-destination').click();
	await expect(page.getByTestId('picker-suggestion').filter({ hasText: 'Ashab' })).toBeVisible();
	await page.keyboard.press('Escape');

	await reloadApp(page);
	await expect(personal).toHaveCount(1);
	await rowAction(page, personal, /Remove/);
	await expect(personal).toHaveCount(0);
});

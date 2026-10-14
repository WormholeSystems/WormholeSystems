import { createdId, expect, gotoApp, test } from './fixtures';
import { createMap } from './arrange';
import { clearThreats, createIdentity, grantAccess, seedThreat } from './db';

// The Cmd+K palette: jump to a system already on the map, or add one that is not.

const J122515 = 31001882; // C5, Wolf-Rayet Star
const JITA = 30000142;
const AMARR = 30002187;

test('Cmd+K opens the palette and jumps to a placed system', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Palette');
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: J122515, x: 200, y: 200, alias: 'Staging' },
	});
	await gotoApp(page, `/maps/${mapId}`);

	await page.keyboard.press('ControlOrMeta+k');
	await expect(page.getByTestId('palette-list')).toBeVisible();

	// The alias matches, not just the system name.
	await page.getByPlaceholder('System, alias, occupier or notes…').fill('Stag');
	const hit = page.getByTestId('palette-hit').first();
	await expect(hit).toContainText('J122515');
	await hit.click();

	// Picking a hit activates the system, which the deep link reflects.
	await expect(page).toHaveURL(new RegExp(`system=${J122515}`));
});

test('an off-map system can be added straight from the palette', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E PaletteAdd');
	await gotoApp(page, `/maps/${mapId}`);

	await page.getByTestId('palette-trigger').click();
	await page.getByPlaceholder('System, alias, occupier or notes…').fill('Jita');
	const add = page.getByTestId('palette-add').first();
	await expect(add).toContainText('Jita');
	await add.click();

	await expect(page.getByTestId('system-node').filter({ hasText: 'Jita' })).toBeVisible();
});

test('notes match for members and stay hidden from viewers', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E PaletteNotes');
	const added = await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: JITA, x: 200, y: 200, alias: null },
	});
	const placementId = await createdId(added);
	const noted = await api.post(`/api/maps/${mapId}/systems/set-notes`, {
		data: {
			map_id: mapId,
			map_solar_system_id: placementId,
			notes: 'Bookmarked the hauler perch here',
		},
	});
	expect(noted.ok()).toBe(true);

	await gotoApp(page, `/maps/${mapId}`);
	await page.getByTestId('palette-trigger').click();
	await page.getByPlaceholder('System, alias, occupier or notes…').fill('hauler perch');
	await expect(page.getByTestId('palette-hit').first()).toContainText('hauler perch');

	// A viewer never sees notes, so the same query finds nothing on the map.
	const viewer = await createIdentity(4);
	await grantAccess(mapId, viewer.characterId, 'viewer');
	const ctx = await page.context().browser()!.newContext();
	await ctx.addCookies([
		{ name: 'ws_session', value: viewer.session, domain: 'localhost', path: '/' },
	]);
	const viewerPage = await ctx.newPage();
	await viewerPage.goto(`http://localhost:5173/maps/${mapId}`);
	await viewerPage.waitForSelector('html[data-hydrated="true"]');
	await viewerPage.getByTestId('palette-trigger').click();
	await viewerPage.getByPlaceholder('System, alias, occupier or notes…').fill('hauler perch');
	await expect(viewerPage.getByTestId('palette-hit')).toHaveCount(0);
	await ctx.close();
});

test('columns line up across both groups, not just within a row', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E PaletteGrid');
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: AMARR, x: 200, y: 200, alias: null },
	});
	await gotoApp(page, `/maps/${mapId}`);

	await page.getByTestId('palette-trigger').click();
	await page.getByPlaceholder('System, alias, occupier or notes…').fill('amar');
	// One placed system and several unplaced ones, so both groups are on screen.
	await expect(page.getByTestId('palette-hit')).toHaveCount(1);
	await expect(page.getByTestId('palette-add').first()).toBeVisible();

	// The list owns the tracks and rows are subgrids, so every row's cells start at the
	// same x. A grid declared per row would size each row from its own content instead.
	const columns = await page.getByTestId('palette-list').evaluate((el) =>
		[...el.querySelectorAll('[data-slot="command-item"]')].map((row) =>
			[...row.querySelectorAll(':scope > * > *')]
				.slice(0, 4)
				.map((cell) => Math.round(cell.getBoundingClientRect().left))
				.join(','),
		),
	);
	expect(columns.length).toBeGreaterThan(1);
	expect(new Set(columns).size).toBe(1);
});

test('the system itself outranks intel that merely mentions it', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E PaletteRank');
	const added = await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: J122515, x: 200, y: 200, alias: null },
	});
	const placementId = await createdId(added);
	await api.post(`/api/maps/${mapId}/systems/set-notes`, {
		data: {
			map_id: mapId,
			map_solar_system_id: placementId,
			notes: 'Haul the loot to Jita when the chain closes',
		},
	});

	await gotoApp(page, `/maps/${mapId}`);
	await page.getByTestId('palette-trigger').click();
	await page.getByPlaceholder('System, alias, occupier or notes…').fill('Jita');

	// Both rows are offered, but Jita itself comes first; the note match sits below it.
	const rows = page
		.getByTestId('palette-list')
		.locator('[data-testid="palette-add"], [data-testid="palette-hit"]');
	await expect(rows.first()).toContainText('Jita');
	await expect(rows.first()).toHaveAttribute('data-testid', 'palette-add');
	await expect(page.getByTestId('palette-hit').first()).toContainText('Haul the loot');
});

// Searching an organisation finds the wormholes it operates in, from the killmail threat
// analysis rather than from anything anyone typed onto the map.

const ORG_ID = 99000777;
const J155207 = 31002402;

test.afterEach(async () => {
	await clearThreats([ORG_ID]);
});

test('an organisation finds the wormholes it is a threat in', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E PaletteThreat');
	await seedThreat({
		solarSystemId: J122515,
		entityId: ORG_ID,
		entityType: 'alliance',
		name: 'E2E Threat Syndicate',
		kills: 240,
	});
	await seedThreat({
		solarSystemId: J155207,
		entityId: ORG_ID,
		entityType: 'alliance',
		name: 'E2E Threat Syndicate',
		kills: 31,
	});
	await gotoApp(page, `/maps/${mapId}`);

	await page.getByTestId('palette-trigger').click();
	await page.getByPlaceholder('System, alias, occupier or notes…').fill('threat syndicate');

	// One section for the organisation, summarising its reach.
	const group = page.getByTestId('palette-threat-group');
	await expect(group).toHaveText(/E2E Threat Syndicate/);
	await expect(group).toContainText('2 × 271 kills');

	// Its systems, busiest first, each with the kills it has there.
	const rows = page.getByTestId('palette-threat');
	await expect(rows).toHaveCount(2);
	await expect(rows.first()).toContainText('J122515');
	await expect(rows.first().getByTestId('palette-threat-kills')).toHaveText('240');
	await expect(rows.last().getByTestId('palette-threat-kills')).toHaveText('31');

	// Picking one puts it on the map, since it is somewhere you have not been yet.
	await rows.first().click();
	await expect(page.getByTestId('system-node').filter({ hasText: 'J122515' })).toBeVisible();
});

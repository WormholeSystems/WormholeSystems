import { expect, gotoApp, reloadApp, test } from './fixtures';
import { addSystem, createMap } from './arrange';
import { clearKillmails, seedAggressor, seedKillmail } from './db';

// The killmails card: what died in the mapped systems, newest first, with the expensive
// losses standing out.
//
// Rows are asserted by their own seeded ids rather than by count: the ingest is live
// against zKillboard, so a real kill can land in any system at any moment.

const J122515 = 31001882; // C5
const JITA = 30000142;
const AMARR = 30002187; // never placed on the test map

const SLASHER = 585;
const LOKI = 29990;

// Ids well outside anything zKillboard will hand us.
const RECENT = 900000001;
const OLD = 900000002;
const ELSEWHERE = 900000003;
const KSPACE = 900000004;
const SEEDED = [RECENT, OLD, ELSEWHERE, KSPACE];

test.afterEach(async () => {
	await clearKillmails(SEEDED);
});

test('recent kills in mapped systems are listed, and nothing else is', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Kills');
	await addSystem(api, mapId, J122515, { x: 200 });
	await addSystem(api, mapId, JITA, { x: 500 });

	await seedKillmail({
		id: RECENT,
		solarSystemId: J122515,
		minutesAgo: 20,
		victimShipTypeId: LOKI,
		totalValue: 2_400_000_000,
		attackerCount: 4,
		finalBlowShipTypeId: SLASHER,
	});
	// Older than the card's week-long window.
	await seedKillmail({
		id: OLD,
		solarSystemId: J122515,
		minutesAgo: 60 * 24 * 9,
		victimShipTypeId: SLASHER,
		totalValue: 5_000_000,
		attackerCount: 1,
	});
	// A system nobody put on this map.
	await seedKillmail({
		id: ELSEWHERE,
		solarSystemId: AMARR,
		minutesAgo: 10,
		victimShipTypeId: SLASHER,
		totalValue: 5_000_000,
		attackerCount: 1,
	});

	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);
	const card = page.getByTestId('killmails-card');
	await expect(card).toBeVisible();

	const recent = card.locator(`[data-kill="${RECENT}"]`);
	await expect(recent).toBeVisible({ timeout: 10_000 });
	await expect(recent).toContainText('Loki');
	await expect(recent).toContainText('J122515');
	await expect(recent.getByTestId('killmail-attackers')).toHaveText('4');
	// Over a billion, so it is called out rather than sitting in the muted column.
	await expect(recent.getByTestId('killmail-value')).toHaveText('2.4B');
	await expect(recent.getByTestId('killmail-value')).toHaveClass(/amber/);

	await expect(card.locator(`[data-kill="${OLD}"]`)).toHaveCount(0);
	await expect(card.locator(`[data-kill="${ELSEWHERE}"]`)).toHaveCount(0);
});

test('a solo kill is marked, and the filter narrows to one half of the chain', async ({
	page,
	api,
}) => {
	const mapId = await createMap(api, 'E2E KillsFilter');
	await addSystem(api, mapId, J122515, { x: 200 });
	await addSystem(api, mapId, JITA, { x: 500 });

	await seedKillmail({
		id: RECENT,
		solarSystemId: J122515,
		minutesAgo: 5,
		victimShipTypeId: SLASHER,
		totalValue: 12_000_000,
		attackerCount: 1,
		isSolo: true,
	});
	await seedKillmail({
		id: KSPACE,
		solarSystemId: JITA,
		minutesAgo: 6,
		victimShipTypeId: SLASHER,
		totalValue: 8_000_000,
		attackerCount: 3,
	});

	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);
	const card = page.getByTestId('killmails-card');
	await expect(card.locator(`[data-kill="${RECENT}"]`)).toBeVisible({ timeout: 10_000 });
	await expect(card.locator(`[data-kill="${KSPACE}"]`)).toBeVisible();

	// One attacker: a hunter, not a fleet, and coloured to say so.
	await expect(
		card.locator(`[data-kill="${RECENT}"]`).getByTestId('killmail-attackers'),
	).toHaveClass(/amber/);

	// Wormhole space only drops the Jita kill.
	await card.getByTestId('killmail-filter').click();
	await page.getByTestId('killmail-filter-jspace').click();
	await expect(card.locator(`[data-kill="${KSPACE}"]`)).toHaveCount(0, { timeout: 10_000 });
	await expect(card.locator(`[data-kill="${RECENT}"]`)).toBeVisible();

	// And it survives a reload, because it is stored against the user.
	await reloadApp(page);
	await expect(card.locator(`[data-kill="${RECENT}"]`)).toBeVisible({ timeout: 10_000 });
	await expect(card.locator(`[data-kill="${KSPACE}"]`)).toHaveCount(0);

	// Put it back, so the next test starts from the default.
	await card.getByTestId('killmail-filter').click();
	await page.getByTestId('killmail-filter-all').click();
	await expect(card.locator(`[data-kill="${KSPACE}"]`)).toBeVisible({ timeout: 10_000 });
});

test('the selected system filter follows the selection', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E KillsSelected');
	await addSystem(api, mapId, J122515, { x: 200 });
	await addSystem(api, mapId, JITA, { x: 500 });

	await seedKillmail({
		id: RECENT,
		solarSystemId: J122515,
		minutesAgo: 5,
		victimShipTypeId: SLASHER,
		totalValue: 12_000_000,
		attackerCount: 2,
	});
	await seedKillmail({
		id: KSPACE,
		solarSystemId: JITA,
		minutesAgo: 6,
		victimShipTypeId: SLASHER,
		totalValue: 8_000_000,
		attackerCount: 3,
	});

	await gotoApp(page, `/maps/${mapId}`);
	const card = page.getByTestId('killmails-card');
	await expect(card.locator(`[data-kill="${KSPACE}"]`)).toBeVisible({ timeout: 10_000 });

	// Nothing is selected yet, so the card asks for a system rather than showing them all.
	await card.getByTestId('killmail-filter').click();
	await page.getByTestId('killmail-filter-selected_system').click();
	await expect(card.getByTestId('killmails-select-hint')).toBeVisible({ timeout: 10_000 });
	await expect(card.getByTestId('killmail-row')).toHaveCount(0);

	await page.getByTestId('system-node').filter({ hasText: 'J122515' }).click();
	await expect(card.locator(`[data-kill="${RECENT}"]`)).toBeVisible({ timeout: 10_000 });
	await expect(card.locator(`[data-kill="${KSPACE}"]`)).toHaveCount(0);

	await page.getByTestId('system-node').filter({ hasText: 'Jita' }).click();
	await expect(card.locator(`[data-kill="${KSPACE}"]`)).toBeVisible({ timeout: 10_000 });
	await expect(card.locator(`[data-kill="${RECENT}"]`)).toHaveCount(0);

	// Put it back, so the next test starts from the default.
	await card.getByTestId('killmail-filter').click();
	await page.getByTestId('killmail-filter-all').click();
	await expect(card.locator(`[data-kill="${RECENT}"]`)).toBeVisible({ timeout: 10_000 });
});

test('adding a system pulls in its kills straight away', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E KillsAdd');
	await addSystem(api, mapId, J122515, { x: 200 });

	// A kill in a system that is not on the map yet.
	await seedKillmail({
		id: KSPACE,
		solarSystemId: JITA,
		minutesAgo: 4,
		victimShipTypeId: SLASHER,
		totalValue: 9_000_000,
		attackerCount: 2,
	});

	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);
	const card = page.getByTestId('killmails-card');
	await expect(card).toBeVisible();
	await expect(card.locator(`[data-kill="${KSPACE}"]`)).toHaveCount(0);

	// Putting the system on the map brings its history with it. The list is scoped to the
	// map's systems, so changing that set has to refetch, not wait for the next kill.
	await addSystem(api, mapId, JITA, { x: 500 });
	await expect(card.locator(`[data-kill="${KSPACE}"]`)).toBeVisible({ timeout: 15_000 });
});

test('a killmail row right-clicks to the system menu', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E KillsMenu');
	await addSystem(api, mapId, J122515, { x: 200 });
	await seedKillmail({
		id: RECENT,
		solarSystemId: J122515,
		minutesAgo: 3,
		victimShipTypeId: SLASHER,
		totalValue: 1_000_000,
		attackerCount: 2,
	});

	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);
	const row = page.getByTestId('killmails-card').locator(`[data-kill="${RECENT}"]`);
	await expect(row).toBeVisible({ timeout: 10_000 });

	await row.click({ button: 'right' });
	const menu = page.getByTestId('system-menu');
	await expect(menu).toBeVisible();
	await expect(menu.getByTestId('menu-route')).toBeVisible();
	// J122515 is wormhole space, where the autopilot cannot take anyone, so the menu offers
	// no destination there, the same as the system's own node.
	await expect(menu.getByTestId('menu-set-destination')).toHaveCount(0);
});

// Ids that will never collide with anything zKillboard hands us.
const HUNTER = 900000101;
const HUNTER_CORP = 900000102;
const HUNTER_ALLIANCE = 900000103;

test('a wide card names the aggressor, a narrow one keeps the count', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Aggressor');
	await addSystem(api, mapId, J122515, { x: 200 });
	await seedAggressor({
		characterId: HUNTER,
		name: 'Zvi Sarok',
		corporationId: HUNTER_CORP,
		corporationName: 'Hard Knocks Inc.',
		corporationTicker: 'HKRAB',
		allianceId: HUNTER_ALLIANCE,
		allianceName: 'Hard Knocks Citizens',
		allianceTicker: 'HKC',
	});
	await seedKillmail({
		id: RECENT,
		solarSystemId: J122515,
		minutesAgo: 5,
		victimShipTypeId: LOKI,
		totalValue: 2_400_000_000,
		attackerCount: 4,
		finalBlowShipTypeId: SLASHER,
		finalBlowCharacterId: HUNTER,
		finalBlowCorporationId: HUNTER_CORP,
		finalBlowAllianceId: HUNTER_ALLIANCE,
	});

	await gotoApp(page, `/maps/${mapId}`);
	const row = page.locator(`[data-kill="${RECENT}"]`);
	await expect(row).toBeVisible();

	// The count is always there. The name needs room, and the room is the card's, so a
	// narrower window takes it away and a wider one gives it back.
	await expect(row.getByTestId('killmail-attackers')).toHaveText('4');

	await page.setViewportSize({ width: 900, height: 720 });
	await expect(row.getByTestId('killmail-attackers')).toHaveText('4');
	await expect(row.getByTestId('killmail-aggressor')).toBeHidden();

	await page.setViewportSize({ width: 1600, height: 720 });
	await expect(row.getByTestId('killmail-aggressor')).toHaveText('Zvi Sarok');
	await expect(row.getByTestId('killmail-aggressor-org')).toHaveText('HKC');

	await api.delete(`/api/maps/${mapId}`);
});

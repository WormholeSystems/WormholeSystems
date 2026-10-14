import { expect, gotoApp, test } from './fixtures';
import { addSystem, createMap } from './arrange';

// The EVE Scout card: public wormholes out of Thera and Turnur, one hub at a time, sorted
// by how far away they are through your own chain.
//
// The upstream is the stub's fixed list (see EVE_SCOUT in esi-stub.mjs): two holes out of
// Thera (J122515 and Jita) and one out of Turnur (Amarr).

const J122515 = 31001882;
const JITA = 30000142;

/**
 * These assert exact rows, so they need the stubbed upstream (EVE_SCOUT_URL in
 * playwright.config.ts). A dev API server started outside the suite, say one that
 * restarts on every Rust change, is reused as-is and talks to the real eve-scout.com,
 * whose contents change by the hour. Skip rather than fail on someone else's
 * data, and say which it was.
 */
test.beforeEach(async ({ api }) => {
	const rows = (await (await api.get('/api/evescout')).json()) as { hub_signature: string }[];
	test.skip(
		!rows.some((r) => r.hub_signature === 'THE-001'),
		'API server is not using the EVE Scout stub (started outside the e2e suite)',
	);
});

test('the card lists a hub at a time, resolved and sorted', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Scout');
	await addSystem(api, mapId, JITA, { x: 200 });
	await gotoApp(page, `/maps/${mapId}?system=${JITA}`);

	const card = page.getByTestId('evescout-card');
	await expect(card).toBeVisible();

	// Thera first, with its two holes; the Turnur one is not in this list.
	const rows = card.getByTestId('evescout-row');
	await expect(rows).toHaveCount(2, { timeout: 10_000 });
	await expect(rows.filter({ hasText: 'J122515' })).toBeVisible();
	await expect(rows.filter({ hasText: 'Jita' })).toBeVisible();
	await expect(rows.filter({ hasText: 'Amarr' })).toHaveCount(0);

	// Resolved server-side data the row could not know on its own.
	const jita = rows.filter({ hasText: 'Jita' });
	await expect(jita).toContainText('The Forge');
	await expect(jita.getByTestId('evescout-signature')).toHaveText('THE-002');
	// Half an hour left reads in minutes, and is called out.
	await expect(jita.getByTestId('evescout-ttl')).toHaveText('30m');
	await expect(jita.getByTestId('evescout-ttl')).toHaveClass(/red/);

	// The other hub is one tab away.
	await card.getByTestId('evescout-hub-turnur').click();
	await expect(rows).toHaveCount(1);
	await expect(rows.filter({ hasText: 'Amarr' })).toBeVisible();
	await expect(card.getByTestId('evescout-ttl')).toHaveText('14h');
});

test('jumps are measured through the map, and the row opens the system menu', async ({
	page,
	api,
}) => {
	const mapId = await createMap(api, 'E2E ScoutJumps');
	await addSystem(api, mapId, JITA, { x: 200 });
	await gotoApp(page, `/maps/${mapId}?system=${JITA}`);

	const card = page.getByTestId('evescout-card');
	const jita = card.getByTestId('evescout-row').filter({ hasText: 'Jita' });
	await expect(jita).toBeVisible({ timeout: 10_000 });

	// The active system is Jita, and one of Thera's holes comes out in Jita: nothing to fly.
	await expect(jita.getByTestId('evescout-jumps')).toHaveText('0j');

	// J122515 is a wormhole with no gate route, and the hub is deliberately not a stepping
	// stone, so it has no jump count at all rather than a route through Thera.
	const hole = card.getByTestId('evescout-row').filter({ hasText: 'J122515' });
	await expect(hole).toContainText('--');

	// Right-click reaches the system menu, same as anywhere else a system is named.
	await jita.click({ button: 'right' });
	const menu = page.getByRole('menu');
	await expect(menu).toBeVisible();
	await expect(menu.getByText('Set destination')).toBeVisible();
});

test('columns sort, and clicking the same one again reverses it', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E ScoutSort');
	await addSystem(api, mapId, J122515, { x: 200 });
	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);

	const card = page.getByTestId('evescout-card');
	const rows = card.getByTestId('evescout-row');
	await expect(rows).toHaveCount(2, { timeout: 10_000 });

	const names = () => rows.evaluateAll((els) => els.map((e) => e.getAttribute('data-system')));

	// Legacy's ordering: known space before wormholes, not alphabetical.
	await card.getByRole('button', { name: 'System' }).click();
	expect(await names()).toEqual(['Jita', 'J122515']);
	await card.getByRole('button', { name: 'System' }).click();
	expect(await names()).toEqual(['J122515', 'Jita']);

	// Soonest to collapse first.
	await card.getByRole('button', { name: 'TTL' }).click();
	expect(await names()).toEqual(['Jita', 'J122515']);
});

import { createdId, expect, gotoApp, test } from './fixtures';
import { createMap } from './arrange';
import { ageStaleConnections, createIdentity, grantAccess } from './db';

// The map status bar: identity, the live-feed dot, the view toggles, and the undo/redo
// pair driven by the command journal.

const J122515 = 31001882; // C5, Wolf-Rayet Star
const JITA = 30000142;
const AMARR = 30002187;
const DODIXIE = 30002659;

test('shows the map name and a live socket once connected', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Status');
	await gotoApp(page, `/maps/${mapId}`);

	await expect(page.getByTestId('status-bar-name')).toHaveText('E2E Status');
	// The socket reports itself open, which is the whole point of the dot.
	await expect(page.getByTestId('socket-dot')).toHaveAttribute('data-state', 'open');
});

test('the view toggles persist through a reload', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Toggles');
	await gotoApp(page, `/maps/${mapId}`);

	const threat = page.getByTestId('threat-toggle');
	await expect(threat).toHaveAttribute('aria-pressed', 'true');
	await threat.click();
	await expect(threat).toHaveAttribute('aria-pressed', 'false');

	await gotoApp(page, `/maps/${mapId}`);
	await expect(page.getByTestId('threat-toggle')).toHaveAttribute('aria-pressed', 'false');
});

test('undo and redo settle instead of toggling for ever', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Undo');
	await gotoApp(page, `/maps/${mapId}`);

	const undo = page.getByTestId('undo-button');
	const redo = page.getByTestId('redo-button');
	// Nothing has happened yet, so neither direction is available.
	await expect(undo).toBeDisabled();
	await expect(redo).toBeDisabled();

	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: J122515, x: 200, y: 200, alias: null },
	});
	const node = page.getByTestId('system-node').filter({ hasText: 'J122515' });
	await expect(node).toBeVisible();
	await expect(undo).toBeEnabled();

	// Walking back and forth repeatedly must come to rest each time, not keep offering a
	// redo that toggles the same change on and off.
	for (let i = 0; i < 3; i++) {
		await undo.click();
		await expect(node).toHaveCount(0);
		await expect(undo).toBeDisabled();
		await expect(redo).toBeEnabled();

		await redo.click();
		await expect(node).toBeVisible();
		await expect(undo).toBeEnabled();
		await expect(redo).toBeDisabled();
	}

	// And the walking never grew the history.
	await page.getByTestId('history-button').click();
	await expect(page.getByTestId('history-row')).toHaveCount(1);
});

test('a change after an undo branches, and the branch can be re-entered', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Branch');
	const add = (sys: number, x: number) =>
		api.post(`/api/maps/${mapId}/systems/add`, {
			data: { map_id: mapId, solar_system_id: sys, x, y: 200, alias: null },
		});
	await add(J122515, 200);
	await add(JITA, 500);
	await gotoApp(page, `/maps/${mapId}`);

	const wh = page.getByTestId('system-node').filter({ hasText: 'J122515' });
	const jita = page.getByTestId('system-node').filter({ hasText: 'Jita' });
	await expect(jita).toBeVisible();

	// Undo the Jita placement, then do something else: Jita's step is not destroyed.
	await page.getByTestId('undo-button').click();
	await expect(jita).toHaveCount(0);
	await add(AMARR, 800);
	const amarr = page.getByTestId('system-node').filter({ hasText: 'Amarr' });
	await expect(amarr).toBeVisible();
	await expect(wh).toBeVisible();

	// The history draws the fork: the trunk stays at depth 0 and the abandoned step is
	// indented under the point it forked from.
	await page.getByTestId('history-button').click();
	const row = (name: string) => page.getByTestId('history-row').filter({ hasText: name });
	await expect(row('J122515')).toHaveAttribute('data-depth', '0');
	await expect(row('Amarr')).toHaveAttribute('data-depth', '0');
	await expect(row('Amarr')).toHaveAttribute('data-head', 'true');
	await expect(row('Jita')).toHaveAttribute('data-depth', '1');
	await expect(row('Jita')).toHaveAttribute('data-applied', 'false');

	// Newest first, so the most recent work is at the top without scrolling.
	const order = await page.getByTestId('history-row').allTextContents();
	expect(order.findIndex((t) => t.includes('Amarr'))).toBeLessThan(
		order.findIndex((t) => t.includes('J122515')),
	);

	// Jumping to the abandoned step swaps branches.
	await row('Jita').click();
	await expect(jita).toBeVisible();
	await expect(amarr).toHaveCount(0);
});

test('rewinding a straight history keeps it straight', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Straight');
	const add = (sys: number, x: number) =>
		api.post(`/api/maps/${mapId}/systems/add`, {
			data: { map_id: mapId, solar_system_id: sys, x, y: 200, alias: null },
		});
	await add(J122515, 200);
	await add(JITA, 500);
	await add(AMARR, 800);
	await gotoApp(page, `/maps/${mapId}`);

	// Step back to the very first change. Nothing has diverged, so nothing should indent:
	// the later steps are simply ahead of where the map is sitting.
	await page.getByTestId('undo-button').click();
	await expect(page.getByTestId('system-node')).toHaveCount(2);
	await page.getByTestId('undo-button').click();
	await expect(page.getByTestId('system-node')).toHaveCount(1);

	await page.getByTestId('history-button').click();
	const rows = page.getByTestId('history-row');
	await expect(rows).toHaveCount(3);
	for (const depth of await rows.evaluateAll((els) =>
		els.map((e) => e.getAttribute('data-depth')),
	)) {
		expect(depth).toBe('0');
	}
	// The ones ahead of the cursor are still marked as not in effect.
	await expect(rows.filter({ hasText: 'Amarr' })).toHaveAttribute('data-applied', 'false');
	await expect(rows.filter({ hasText: 'J122515' })).toHaveAttribute('data-applied', 'true');
});

test('only the first row of a branch connects back to the line it left', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Connectors');
	const add = (sys: number, x: number) =>
		api.post(`/api/maps/${mapId}/systems/add`, {
			data: { map_id: mapId, solar_system_id: sys, x, y: 200, alias: null },
		});
	await add(J122515, 200);
	await add(JITA, 500);
	await add(AMARR, 800);
	await gotoApp(page, `/maps/${mapId}`);

	// Rewind behind both, then diverge: Jita and Amarr become a two-step abandoned branch.
	await page.getByTestId('undo-button').click();
	await expect(page.getByTestId('system-node')).toHaveCount(2);
	await page.getByTestId('undo-button').click();
	await expect(page.getByTestId('system-node')).toHaveCount(1);
	await add(DODIXIE, 1100);
	await expect(page.getByTestId('system-node').filter({ hasText: 'Dodixie' })).toBeVisible();

	await page.getByTestId('history-button').click();
	const row = (name: string) => page.getByTestId('history-row').filter({ hasText: name });
	// The whole branch sits at one indent; it does not staircase.
	await expect(row('Jita')).toHaveAttribute('data-depth', '1');
	await expect(row('Amarr')).toHaveAttribute('data-depth', '1');
	// Only the step that left the line draws a connector back to it.
	await expect(row('Jita')).toHaveAttribute('data-forks', 'true');
	await expect(row('Amarr')).toHaveAttribute('data-forks', 'false');
	await expect(row('Dodixie')).toHaveAttribute('data-forks', 'false');
});

test('history lists what happened and who did it', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E History');
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: JITA, x: 100, y: 100, alias: null },
	});
	await gotoApp(page, `/maps/${mapId}`);

	await page.getByTestId('history-button').click();
	const list = page.getByTestId('history-list');
	await expect(list).toBeVisible();
	await expect(list.getByText('E2E Pilot')).toBeVisible();
	await expect(list.getByText(/Jita/)).toBeVisible();
});

test('a viewer cannot move the history but can read it', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E ViewerBar');
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: JITA, x: 100, y: 100, alias: null },
	});

	const viewer = await createIdentity(1);
	await grantAccess(mapId, viewer.characterId, 'viewer');
	const viewerCtx = await page.context().browser()!.newContext();
	await viewerCtx.addCookies([
		{ name: 'ws_session', value: viewer.session, domain: 'localhost', path: '/' },
	]);
	const viewerPage = await viewerCtx.newPage();
	await viewerPage.goto(`http://localhost:5173/maps/${mapId}`);
	await viewerPage.waitForSelector('html[data-hydrated="true"]');

	// Undo/redo are write actions, so a viewer does not get them at all.
	await expect(viewerPage.getByTestId('undo-button')).toHaveCount(0);
	// History is readable, though: seeing who changed the chain is part of using it.
	await viewerPage.getByTestId('history-button').click();
	await expect(viewerPage.getByTestId('history-list').getByText('E2E Pilot')).toBeVisible();
	await viewerCtx.close();
});

test('a character outside every grant is warned about limited access', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Limited');
	await gotoApp(page, `/maps/${mapId}`);
	// The e2e identity owns this map through its own character, so no warning.
	await expect(page.getByText('Limited access')).toHaveCount(0);
});

test('stale connections are offered for a one-click sweep', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Stale');
	const add = async (sys: number, x: number) => {
		const res = await api.post(`/api/maps/${mapId}/systems/add`, {
			data: { map_id: mapId, solar_system_id: sys, x, y: 200, alias: null },
		});
		return await createdId(res);
	};
	const a = await add(J122515, 200);
	const b = await add(JITA, 500);
	const conn = await api.post(`/api/maps/${mapId}/connections/add`, {
		data: { map_id: mapId, from_system: a, to_system: b, kind: 'wormhole' },
	});
	const connId = await createdId(conn);
	await api.post(`/api/maps/${mapId}/connections/set-status`, {
		data: { map_id: mapId, connection_id: connId, time_status: 'critical' },
	});
	await ageStaleConnections(mapId);

	await gotoApp(page, `/maps/${mapId}`);
	const badge = page.getByTestId('stale-badge');
	await expect(badge).toHaveText(/1 stale/);
	await badge.click();
	await expect(page.getByTestId('stale-list')).toContainText('J122515');

	await page.getByTestId('clean-stale').click();
	// The edge and both bare endpoints go, as one undoable change.
	await expect(page.getByTestId('system-node')).toHaveCount(0);
	await expect(page.getByTestId('stale-badge')).toHaveCount(0);

	await page.getByTestId('undo-button').click();
	await expect(page.getByTestId('system-node')).toHaveCount(2);
});

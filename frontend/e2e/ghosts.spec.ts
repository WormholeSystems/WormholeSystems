import { createdId, expect, gotoApp, test } from './fixtures';

// Ghost nodes: a scanned wormhole nobody has flown, drawn so the chain can be laid out
// and named before anyone does, and merged away once its system is known.

const J122515 = 31001882; // C5 wormhole, static H296
const JITA = 30000142;

async function createMap(api: import('@playwright/test').APIRequestContext, name: string) {
	const res = await api.post('/api/maps', { data: { name } });
	expect(res.ok()).toBe(true);
	return await createdId(res);
}

async function addSystem(
	api: import('@playwright/test').APIRequestContext,
	mapId: number,
	solarSystemId: number,
) {
	const res = await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: solarSystemId, x: 200, y: 200, alias: null },
	});
	expect(res.ok()).toBe(true);
	return await createdId(res);
}

/** Turn on the map-wide setting that makes a pasted wormhole a node. */
async function enableGhosting(api: import('@playwright/test').APIRequestContext, mapId: number) {
	const res = await api.post(`/api/maps/${mapId}/update`, {
		data: { map_id: mapId, ghost_unlinked_wormholes: true },
	});
	expect(res.ok()).toBe(true);
}

async function pasteViaEvent(page: import('@playwright/test').Page, text: string) {
	await expect(page.getByTestId('signatures-card')).toBeVisible();
	await page.evaluate((t) => {
		const dt = new DataTransfer();
		dt.setData('text/plain', t);
		window.dispatchEvent(new ClipboardEvent('paste', { clipboardData: dt }));
	}, text);
}

test('a pasted wormhole becomes a node, and assigning a system names it', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Ghosts');
	await addSystem(api, mapId, J122515);
	await enableGhosting(api, mapId);

	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);
	await pasteViaEvent(page, 'WHX-301\tCosmic Signature\tWormhole\tUnstable Wormhole\t100%\t1 AU');

	// The scan alone puts the far side on the map, named after the signature it came from,
	// which is the only thing that identifies it until someone flies it.
	const ghost = page.locator('[data-testid="system-node"][data-ghost="true"]');
	await expect(ghost).toHaveCount(1);
	await expect(ghost.getByTestId('ghost-signature')).toHaveText('WHX-301');

	// Nothing to draw a connection from: what it leads to is the open question.
	await ghost.hover();
	await expect(ghost.getByTestId('connection-handle')).toHaveCount(0);

	// The menu does not offer it either, from the node or from the API.
	await ghost.click({ button: 'right' });
	await expect(page.getByRole('menuitem', { name: 'Assign a system' })).toBeVisible();
	await expect(page.getByRole('menuitem', { name: 'Add connection' })).toHaveCount(0);
	await expect(page.getByRole('menuitem', { name: 'Pin', exact: true })).toHaveCount(0);
	await page.keyboard.press('Escape');

	// And the API says the same, whichever end the connection is asked for.
	const view = await (await api.get(`/api/maps/${mapId}`)).json();
	type Node =
		{ kind: 'system'; id: number; solar_system_id: number } | { kind: 'ghost'; id: number };
	const ghostId = view.systems.find((s: Node) => s.kind === 'ghost').id;
	const scanned = view.systems.find(
		(s: Node) => s.kind === 'system' && s.solar_system_id === J122515,
	).id;
	const refused = await api.post(`/api/maps/${mapId}/connections/add`, {
		data: { map_id: mapId, from_system: scanned, to_system: ghostId, kind: 'wormhole' },
	});
	expect(refused.status()).toBe(400);

	// It is a node like any other: it can be named before anyone flies it.
	await ghost.dblclick();
	await page.getByPlaceholder('Alias', { exact: true }).fill('1a');
	await page.getByRole('button', { name: 'Save' }).click();
	await expect(ghost).toContainText('1a');
	await page.keyboard.press('Escape');

	// Saying what it leads to turns it into that system, where it already sits.
	await ghost.click({ button: 'right' });
	await page.getByRole('menuitem', { name: 'Assign a system' }).click();
	await page.getByPlaceholder('This hole leads to…').fill('Jita');
	// Off the map, so it comes from the add section; assigning takes either.
	await page.getByTestId('palette-add').first().click();

	await expect(page.locator('[data-testid="system-node"][data-ghost="true"]')).toHaveCount(0);
	const named = page.getByTestId('system-node').filter({ hasText: 'Jita' });
	await expect(named).toContainText('1a');
});

test('a ghost that turns out to be on the map already is merged into it', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E GhostMerge');
	await addSystem(api, mapId, J122515);
	await addSystem(api, mapId, JITA);
	await enableGhosting(api, mapId);

	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);
	await pasteViaEvent(page, 'WHX-302\tCosmic Signature\tWormhole\tUnstable Wormhole\t100%\t1 AU');

	const ghost = page.locator('[data-testid="system-node"][data-ghost="true"]');
	await expect(ghost).toHaveCount(1);

	await ghost.click({ button: 'right' });
	await page.getByRole('menuitem', { name: 'Assign a system' }).click();
	await page.getByPlaceholder('This hole leads to…').fill('Jita');
	await page.getByTestId('palette-hit').first().click();

	// One Jita, not two: the edge moved to the placement that was already there.
	await expect(page.locator('[data-testid="system-node"][data-ghost="true"]')).toHaveCount(0);
	await expect(page.getByTestId('system-node').filter({ hasText: 'Jita' })).toHaveCount(1);
	await expect(page.getByTestId('system-node')).toHaveCount(2);
});

test('a signature typed in and called a wormhole gets a node too', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E GhostManual');
	await addSystem(api, mapId, J122515);
	await enableGhosting(api, mapId);

	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);

	// Typed in by hand, uncategorised: nothing is known about it yet, so nothing is drawn.
	await page.getByTestId('new-signature').click();
	await page.getByTestId('new-signature-id').fill('WHX-303');
	await page.getByTestId('new-signature-id').press('Enter');
	await expect(page.getByTestId('sig-row')).toHaveCount(1);
	await expect(page.locator('[data-testid="system-node"][data-ghost="true"]')).toHaveCount(0);

	// Calling it a wormhole is saying the hole is there, which is what puts it on the map.
	await page.getByTestId('sig-row').getByTestId('sig-category').click();
	await page.getByRole('option', { name: 'Wormhole' }).click();

	const ghost = page.locator('[data-testid="system-node"][data-ghost="true"]');
	await expect(ghost).toHaveCount(1);
	await expect(ghost.getByTestId('ghost-signature')).toHaveText('WHX-303');
});

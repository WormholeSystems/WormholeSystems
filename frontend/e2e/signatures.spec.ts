import { createdId, expect, gotoApp, reloadApp, test } from './fixtures';
import { addSystem, createMap } from './arrange';
import { createIdentity, grantAccess, setCharacterPresence } from './db';

// The full signatures panel (legacy parity): catalog-backed selects, paste diff with
// lazy delete, sorting/filters, row actions, and role gating.

const J122515 = 31001882; // C5 wormhole, static H296
const JITA = 30000142;

/**
 * Open the map as a dedicated member identity (slot 6). The main e2e character's
 * presence is toggled by other spec files (waypoints), which would trip the paste
 * mismatch dialog on the shared session.
 */
async function openAsMember(
	browser: import('@playwright/test').Browser,
	mapId: number,
	query: string,
) {
	const member = await createIdentity(6);
	await grantAccess(mapId, member.characterId, 'member');
	const ctx = await browser.newContext();
	await ctx.addCookies([
		{ name: 'ws_session', value: member.session, domain: 'localhost', path: '/' },
	]);
	const page = await ctx.newPage();
	await page.goto(`http://localhost:5173/maps/${mapId}${query}`);
	await page.waitForSelector('html[data-hydrated="true"]');
	return { page, ctx };
}

async function pasteViaEvent(page: import('@playwright/test').Page, text: string) {
	// The window-paste handler only runs once the map (and the caller's role) is loaded.
	await expect(page.getByTestId('signatures-card')).toBeVisible();
	await page.evaluate((t) => {
		const dt = new DataTransfer();
		dt.setData('text/plain', t);
		window.dispatchEvent(new ClipboardEvent('paste', { clipboardData: dt }));
	}, text);
}

test('wormhole type select: statics first, K162 group, chosen type survives repaste', async ({
	browser,
	api,
}) => {
	const mapId = await createMap(api, 'E2E SigTypes');
	await addSystem(api, mapId, J122515, { x: 200, y: 200 });

	const { page, ctx } = await openAsMember(browser, mapId, `?system=${J122515}`);
	await page.getByTestId('statics-first-toggle').click();
	await expect(page.getByTestId('signatures-pasted-at')).toHaveCount(0);
	await pasteViaEvent(page, 'WHX-101\tCosmic Signature\tWormhole\tUnstable Wormhole\t100%\t1 AU');

	const row = page.getByTestId('sig-row');
	await expect(row).toHaveCount(1);
	await expect(page.getByTestId('signatures-pasted-at')).toHaveText(/^\s*\d+s$/);
	await row.getByTestId('sig-type').click();
	// Statics section leads with the system's static; K162 has its own group.
	const headings = page.locator('[data-command-group-heading]');
	await expect(headings.filter({ hasText: 'Statics' })).toBeVisible();
	await expect(headings.filter({ hasText: 'K162' })).toBeVisible();
	const h296 = page.getByRole('option', { name: /H296/ });
	await expect(h296.first()).toBeVisible();

	// Searching looks at where a hole leads, not only at how its name starts: typing the
	// destination class finds the holes that go there.
	await page.getByTestId('sig-type-search').fill('c5');
	await expect(page.getByRole('option', { name: /H296/ }).first()).toBeVisible();
	await page.getByTestId('sig-type-search').fill('zzzz');
	await expect(page.getByText('No type matches')).toBeVisible();
	await page.getByTestId('sig-type-search').fill('H296');
	await h296.first().click();
	await expect(row.getByTestId('sig-type')).toContainText('H296');

	// A repaste without a type keeps the manually chosen wormhole type.
	await pasteViaEvent(page, 'WHX-101\tCosmic Signature\tWormhole\tUnstable Wormhole\t100%\t1 AU');
	await expect(row.getByTestId('sig-type')).toContainText('H296');
	await ctx.close();
});

test('paste diff tints and lazy delete cascade to connection and orphan endpoint', async ({
	browser,
	api,
}) => {
	const mapId = await createMap(api, 'E2E SigDiff');
	const near = await addSystem(api, mapId, J122515, { x: 200, y: 200 });
	const far = await addSystem(api, mapId, JITA, { x: 500, y: 200 });
	// Keep the scanned system: only the far endpoint is orphan-eligible.
	await api.post(`/api/maps/${mapId}/systems/set-pinned`, {
		data: { map_id: mapId, map_solar_system_id: near, value: true },
	});
	const connRes = await api.post(`/api/maps/${mapId}/connections/add`, {
		data: { map_id: mapId, from_system: near, to_system: far, kind: 'wormhole' },
	});
	const connId = await createdId(connRes);

	const { page, ctx } = await openAsMember(browser, mapId, `?system=${J122515}`);
	await pasteViaEvent(
		page,
		'WHX-201\tCosmic Signature\tWormhole\tUnstable Wormhole\t100%\t1 AU\n' +
			'DAT-201\tCosmic Signature\tData Site\tUnsecured Frontier Receiver\t100%\t1 AU',
	);
	const whRow = page.getByTestId('sig-row').filter({ hasText: 'WHX-201' });
	await expect(whRow).toHaveAttribute('data-status', 'new');

	// Link the wormhole sig to the connection from its row.
	await whRow.getByTestId('sig-connection').click();
	await page.getByRole('option', { name: /Jita/ }).click();
	await expect(whRow.getByTestId('sig-connection')).toContainText('Jita');

	// Second paste without the wormhole row: updated vs missing tints.
	await pasteViaEvent(
		page,
		'DAT-201\tCosmic Signature\tData Site\tUnsecured Frontier Receiver\t100%\t1 AU',
	);
	await expect(page.getByTestId('sig-row').filter({ hasText: 'DAT-201' })).toHaveAttribute(
		'data-status',
		'updated',
	);
	await expect(whRow).toHaveAttribute('data-status', 'deleted');

	// Lazy delete: the missing sig goes, taking the connection and the orphaned
	// unpinned endpoint (Jita) with it.
	await page.getByTestId('delete-missing').click();
	await expect(page.getByTestId('sig-row')).toHaveCount(1);
	await expect(page.getByTestId('system-node').filter({ hasText: 'Jita' })).toHaveCount(0);
	const conns = await api.get(`/api/maps/${mapId}`);
	const view = await conns.json();
	expect(view.connections.filter((c: { id: number }) => c.id === connId)).toHaveLength(0);
	await ctx.close();
});

test('a teammate scanning after your paste is never offered for deletion', async ({
	browser,
	api,
}) => {
	const mapId = await createMap(api, 'E2E SigTeammate');
	await addSystem(api, mapId, J122515, { x: 200, y: 200 });

	const { page, ctx } = await openAsMember(browser, mapId, `?system=${J122515}`);
	await pasteViaEvent(
		page,
		'AAA-101\tCosmic Signature\tWormhole\t\t100%\t1 AU\n' +
			'BBB-101\tCosmic Signature\tWormhole\t\t100%\t1 AU',
	);
	await expect(page.getByTestId('sig-row')).toHaveCount(2);
	// A rescan that no longer sees BBB: the diff stays up for review.
	await pasteViaEvent(page, 'AAA-101\tCosmic Signature\tWormhole\t\t100%\t1 AU');
	const missing = page.getByTestId('sig-row').filter({ hasText: 'BBB-101' });
	await expect(missing).toHaveAttribute('data-status', 'deleted');

	// Meanwhile somebody else adds one. It arrives over the socket, after the scan.
	await api.post(`/api/maps/${mapId}/signatures/add`, {
		data: { map_id: mapId, solar_system_id: J122515, signature_id: 'CCC-101', group: 'wormhole' },
	});
	const theirs = page.getByTestId('sig-row').filter({ hasText: 'CCC-101' });
	await expect(theirs).toBeVisible();
	await expect(theirs).not.toHaveAttribute('data-status');
	await expect(missing).toHaveAttribute('data-status', 'deleted');

	await page.getByTestId('delete-missing').click();
	await expect(page.getByTestId('sig-row')).toHaveCount(2);
	await expect(missing).toHaveCount(0);
	await expect(theirs).toBeVisible();
	await ctx.close();
});

test('category change clears the connection link', async ({ browser, api }) => {
	const mapId = await createMap(api, 'E2E SigRecat');
	const near = await addSystem(api, mapId, J122515, { x: 200, y: 200 });
	const far = await addSystem(api, mapId, JITA, { x: 500, y: 200 });
	await api.post(`/api/maps/${mapId}/connections/add`, {
		data: { map_id: mapId, from_system: near, to_system: far, kind: 'wormhole' },
	});

	const { page, ctx } = await openAsMember(browser, mapId, `?system=${J122515}`);
	await pasteViaEvent(page, 'WHX-301\tCosmic Signature\tWormhole\t\t100%\t1 AU');
	const row = page.getByTestId('sig-row');
	await row.getByTestId('sig-connection').click();
	await page.getByRole('option', { name: /Jita/ }).click();
	await expect(row.getByTestId('sig-connection')).toContainText('Jita');

	await row.getByTestId('sig-category').click();
	await page.getByRole('option', { name: 'Data Site' }).click();
	// A site row has no connection cell; the link is gone server-side too.
	await expect(row.getByTestId('sig-category')).toContainText('Data');
	await expect(row.getByTestId('sig-connection')).toHaveCount(0);
	const sigs = await (await api.get(`/api/maps/${mapId}/signatures`)).json();
	expect(sigs[0].connection_id).toBeNull();
	await ctx.close();
});

test('wormhole type and connection can be unset back to Unknown', async ({ browser, api }) => {
	const mapId = await createMap(api, 'E2E SigUnset');
	const near = await addSystem(api, mapId, J122515, { x: 200, y: 200 });
	const far = await addSystem(api, mapId, JITA, { x: 500, y: 200 });
	await api.post(`/api/maps/${mapId}/connections/add`, {
		data: { map_id: mapId, from_system: near, to_system: far, kind: 'wormhole' },
	});

	const { page, ctx } = await openAsMember(browser, mapId, `?system=${J122515}`);
	await pasteViaEvent(page, 'WHX-501\tCosmic Signature\tWormhole\t\t100%\t1 AU');
	const row = page.getByTestId('sig-row');

	// Link first; the type list then narrows to the target's class (highsec → K162 side).
	await row.getByTestId('sig-connection').click();
	await page.getByRole('option', { name: /Jita/ }).click();
	await expect(row.getByTestId('sig-connection')).toContainText('Jita');
	await row.getByTestId('sig-type').click();
	await page.getByRole('option', { name: /K162/ }).first().click();
	await expect(row.getByTestId('sig-type')).toContainText('K162');

	// Unset the type: back to the placeholder, cleared server-side.
	await row.getByTestId('sig-type').click();
	await page.getByRole('option', { name: 'Unknown' }).click();
	await expect(row.getByTestId('sig-type')).toContainText('Type');
	let sigs = await (await api.get(`/api/maps/${mapId}/signatures`)).json();
	expect(sigs[0].signature_type_id).toBeNull();

	// Unset the connection: unlinked, but the connection itself stays on the map.
	await row.getByTestId('sig-connection').click();
	await page.getByRole('option', { name: 'Unknown' }).click();
	await expect(row.getByTestId('sig-connection')).toContainText('Connection');
	sigs = await (await api.get(`/api/maps/${mapId}/signatures`)).json();
	expect(sigs[0].connection_id).toBeNull();
	const view = await (await api.get(`/api/maps/${mapId}`)).json();
	expect(view.connections).toHaveLength(1);
	await ctx.close();
});

test('sorting and category filters with hidden count', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E SigSort');
	await addSystem(api, mapId, J122515, { x: 200, y: 200 });
	await api.post(`/api/maps/${mapId}/signatures/paste`, {
		data: {
			map_id: mapId,
			solar_system_id: J122515,
			signatures: [
				{ signature_id: 'AAA-111', group: 'wormhole' },
				{ signature_id: 'BBB-222', group: 'data' },
				{ signature_id: 'CCC-333', group: 'combat' },
			],
		},
	});

	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);
	// Scoped to the card: the page has other panels, and an accessible name match is a
	// substring, so a bare "ID" catches anything that merely contains it.
	const card = page.getByTestId('signatures-card');
	const rows = page.getByTestId('sig-row');
	await expect(rows).toHaveCount(3);
	// Default sort: id desc.
	await expect(rows.first()).toContainText('CCC-333');
	await card.getByRole('button', { name: 'ID' }).click();
	await expect(rows.first()).toContainText('AAA-111');

	// Hiding a category shows the hidden count in the header.
	await page.getByTestId('filter-data').click();
	await expect(rows).toHaveCount(2);
	await expect(page.getByText('1 hidden')).toBeVisible();
	await page.getByTestId('filter-data').click();
	await expect(rows).toHaveCount(3);
});

test('compact toggle persists via map user settings', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E SigCompact');
	await addSystem(api, mapId, J122515, { x: 200, y: 200 });

	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);
	await page.getByTestId('compact-toggle').click();
	await expect(page.getByTestId('compact-toggle')).toHaveAttribute('aria-pressed', 'true');
	await reloadApp(page);
	await expect(page.getByTestId('compact-toggle')).toHaveAttribute('aria-pressed', 'true');
});

test('row actions: EOL color, preserve mass, copy bookmark', async ({ browser, api }) => {
	const mapId = await createMap(api, 'E2E SigActions');
	const near = await addSystem(api, mapId, J122515, { x: 200, y: 200 });
	const far = await addSystem(api, mapId, JITA, { x: 500, y: 200 });
	await api.post(`/api/maps/${mapId}/connections/add`, {
		data: { map_id: mapId, from_system: near, to_system: far, kind: 'wormhole' },
	});

	const { page, ctx } = await openAsMember(browser, mapId, `?system=${J122515}`);
	await pasteViaEvent(page, 'WHX-401\tCosmic Signature\tWormhole\t\t100%\t1 AU');
	const row = page.getByTestId('sig-row');
	await row.getByTestId('sig-connection').click();
	await page.getByRole('option', { name: /Jita/ }).click();

	// Lifetime → End of Life colors the age cell purple.
	await row.getByLabel('Signature menu').click();
	await page.getByRole('menuitemradio', { name: 'End of Life' }).click();
	const age = row.locator('.sig-time');
	await expect(age).toHaveAttribute('data-lifetime', 'eol');
	const { expected, actual } = await age.evaluate((el) => {
		const probe = document.createElement('div');
		probe.style.color = 'oklch(62.7% 0.265 303.9)';
		document.body.appendChild(probe);
		const expected = getComputedStyle(probe).color;
		probe.remove();
		return { expected, actual: getComputedStyle(el).color };
	});
	expect(actual).toBe(expected);

	// Preserve mass toggles on the linked connection.
	await row.getByLabel('Signature menu').click();
	await page.getByRole('menuitemcheckbox', { name: 'Preserve mass' }).click();
	await row.getByLabel('Signature menu').click();
	await expect(page.getByRole('menuitemcheckbox', { name: 'Preserve mass' })).toHaveAttribute(
		'aria-checked',
		'true',
	);
	await page.keyboard.press('Escape');

	// The bookmark names the far side of the hole, since that is what it has to tell you
	// in-game: Jita is k-space, so it takes the k-space format. Nothing has named the exit
	// yet, so the copy takes the chain's next alias and writes it on the map.
	await page.context().grantPermissions(['clipboard-read', 'clipboard-write']);
	await row.getByLabel('Copy bookmark').click();
	const clip = await page.evaluate(() => navigator.clipboard.readText());
	expect(clip).toBe('1 HS WHX Jita The Forge');
	await ctx.close();
});

test('viewer sees a read-only panel', async ({ page, api, browser }) => {
	const mapId = await createMap(api, 'E2E SigViewer');
	await addSystem(api, mapId, J122515, { x: 200, y: 200 });
	await api.post(`/api/maps/${mapId}/signatures/paste`, {
		data: {
			map_id: mapId,
			solar_system_id: J122515,
			signatures: [{ signature_id: 'VWR-101', group: 'wormhole' }],
		},
	});
	const viewer = await createIdentity(4);
	await grantAccess(mapId, viewer.characterId, 'viewer');

	const viewerCtx = await browser.newContext();
	await viewerCtx.addCookies([
		{ name: 'ws_session', value: viewer.session, domain: 'localhost', path: '/' },
	]);
	const viewerPage = await viewerCtx.newPage();
	await viewerPage.goto(`http://localhost:5173/maps/${mapId}?system=${J122515}`);
	await viewerPage.waitForSelector('html[data-hydrated="true"]');

	const row = viewerPage.getByTestId('sig-row');
	await expect(row).toHaveCount(1);
	await expect(viewerPage.getByTestId('paste-clipboard')).toHaveCount(0);
	await expect(viewerPage.getByTestId('new-signature')).toHaveCount(0);
	await expect(row.getByLabel('Signature menu')).toHaveCount(0);
	await expect(row.getByTestId('sig-category')).toBeDisabled();
	// Copy bookmark stays available to viewers.
	await expect(row.getByLabel('Copy bookmark')).toBeVisible();
	await viewerCtx.close();
	void page;
});

test('pasting into a system the character is not in warns first', async ({ api, browser }) => {
	const mapId = await createMap(api, 'E2E SigMismatch');
	await addSystem(api, mapId, J122515, { x: 200, y: 200 });
	// A separate identity so the presence doesn't disturb parallel tests.
	const member = await createIdentity(5);
	await grantAccess(mapId, member.characterId, 'member');
	await setCharacterPresence(member.characterId, JITA);

	const ctx = await browser.newContext();
	await ctx.addCookies([
		{ name: 'ws_session', value: member.session, domain: 'localhost', path: '/' },
	]);
	const memberPage = await ctx.newPage();
	await memberPage.goto(`http://localhost:5173/maps/${mapId}?system=${J122515}`);
	await memberPage.waitForSelector('html[data-hydrated="true"]');
	await expect(memberPage.getByTestId('signatures-card')).toBeVisible();

	await pasteViaEvent(memberPage, 'MSM-101\tCosmic Signature\tWormhole\t\t100%\t1 AU');
	const dialog = memberPage.getByTestId('paste-mismatch');
	await expect(dialog).toBeVisible();
	await expect(dialog).toContainText('Jita');
	await dialog.getByRole('button', { name: 'Paste Anyway' }).click();
	await expect(memberPage.getByTestId('sig-row')).toHaveCount(1);

	await setCharacterPresence(member.characterId, JITA, false);
	await ctx.close();
});

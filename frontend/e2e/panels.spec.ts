import { expect, gotoApp, reloadApp, test } from './fixtures';
import type { SetNotes } from '../src/lib/api/types/SetNotes';
import { addSystem, createMap, type Api } from './arrange';
import { createIdentity, grantAccess } from './db';

// The active-system model and the side panels (System Info, Signatures, Notes).

const J122515 = 31001882; // C5, Wolf-Rayet Star, static H296

test('clicking a node activates it: ring, URL, and System Info', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Active');
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: J122515, x: 200, y: 200, alias: 'Staging' },
	});

	await gotoApp(page, `/maps/${mapId}`);
	await expect(page.getByTestId('system-info-empty')).toBeVisible();

	const node = page.getByTestId('system-node').filter({ hasText: 'J122515' });
	await node.click();

	// Amber ring + deep link.
	await expect(node).toHaveClass(/ring-amber-500/);
	await expect(page).toHaveURL(new RegExp(`system=${J122515}`));

	// System Info content.
	const info = page.getByTestId('system-info');
	await expect(info.getByText('Staging')).toBeVisible();
	await expect(info.getByText('(J122515)')).toBeVisible();
	await expect(info.getByText('Wolf-Rayet Star')).toBeVisible();
	await expect(info.getByRole('link', { name: 'zKill' })).toHaveAttribute(
		'href',
		`https://zkillboard.com/system/${J122515}/`,
	);
	await expect(info.getByRole('link', { name: 'Anoik' })).toHaveAttribute(
		'href',
		'https://anoik.is/systems/J122515',
	);
	await expect(info.getByText('H296')).toBeVisible();

	// The signatures card follows the active system.
	await expect(page.getByTestId('signatures-card')).toBeVisible();
});

test('deep link ?system= restores the active system on load', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E DeepLink');
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: J122515, x: 200, y: 200, alias: null },
	});

	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);
	const node = page.getByTestId('system-node').filter({ hasText: 'J122515' });
	await expect(node).toHaveClass(/ring-amber-500/);
	await expect(page.getByTestId('system-info')).toBeVisible();
});

test('notes round-trip with markdown rendering', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Notes');
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: J122515, x: 200, y: 200, alias: null },
	});

	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);
	const card = page.getByTestId('notes-card');
	await expect(card.getByText('No notes')).toBeVisible();

	await card.getByLabel('Edit notes').click();
	await card.getByPlaceholder('Add notes...').fill('**Danger**: hostile Astrahus');
	// The card renders the new notes optimistically, so wait for the write itself before
	// reloading; otherwise the navigation can abort the request in flight.
	const saved = page.waitForResponse(
		(r) => r.url().includes('/systems/set-notes') && r.request().method() === 'POST',
	);
	await card.getByRole('button', { name: 'Save' }).click();
	await expect(card.locator('strong', { hasText: 'Danger' })).toBeVisible();
	await saved;

	// Survives a reload (persisted server-side).
	await reloadApp(page);
	await expect(card.locator('strong', { hasText: 'Danger' })).toBeVisible();
});

test('notes cannot run script', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Notes XSS');
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: J122515, x: 200, y: 200, alias: null },
	});
	const dialogs: string[] = [];
	page.on('dialog', (dialog) => {
		dialogs.push(dialog.message());
		void dialog.dismiss();
	});

	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);
	const card = page.getByTestId('notes-card');
	await card.getByLabel('Edit notes').click();
	await card
		.getByPlaceholder('Add notes...')
		.fill('<img src=x onerror="alert(1)"> [bait](javascript:alert(2)) **kept**');
	const saved = page.waitForResponse(
		(r) => r.url().includes('/systems/set-notes') && r.request().method() === 'POST',
	);
	await card.getByRole('button', { name: 'Save' }).click();
	await saved;

	// Rendered from what the server sends back too, not just the optimistic copy.
	await reloadApp(page);
	await expect(card.locator('strong', { hasText: 'kept' })).toBeVisible();
	const img = card.locator('img');
	await expect(img).toHaveCount(1);
	await expect(img).not.toHaveAttribute('onerror');
	await expect(card.getByText('bait')).toBeVisible();
	await expect(card.locator('a[href^="javascript:"]')).toHaveCount(0);
	expect(dialogs).toEqual([]);
});

/** Save a placement's notes over the ones the caller last read. */
function saveNotes(api: Api, mapId: number, id: number, notes: string, expected: string | null) {
	return api.post(`/api/maps/${mapId}/systems/set-notes`, {
		data: { map_id: mapId, map_solar_system_id: id, notes, expected } satisfies SetNotes,
	});
}

test('a note saved over a newer one is refused', async ({ api, playwright }) => {
	const mapId = await createMap(api, 'E2E NotesRace');
	const id = await addSystem(api, mapId, J122515);
	const member = await createIdentity(45);
	await grantAccess(mapId, member.characterId, 'member');
	const other = await playwright.request.newContext({
		baseURL: 'http://127.0.0.1:3000',
		extraHTTPHeaders: { cookie: `ws_session=${member.session}` },
	});

	// Both read an empty note; the owner saves first.
	expect((await saveNotes(api, mapId, id, 'static rolled', null)).status()).toBe(200);
	expect((await saveNotes(other, mapId, id, 'frig hole', null)).status()).toBe(409);
	const kept = await api.get(`/api/maps/${mapId}/systems/${id}/details`);
	expect((await kept.json()).notes).toBe('static rolled');

	// Having read theirs, the member writes over it on purpose.
	expect((await saveNotes(other, mapId, id, 'frig hole', 'static rolled')).status()).toBe(200);
	await other.dispose();
});

test('the editor offers to load or overwrite notes someone else saved', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E NotesConflict');
	const id = await addSystem(api, mapId, J122515);
	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);
	const card = page.getByTestId('notes-card');
	const editor = card.getByPlaceholder('Add notes...');
	const conflict = card.getByTestId('notes-conflict');
	const saved = () =>
		page.waitForResponse(
			(r) => r.url().includes('/systems/set-notes') && r.request().method() === 'POST',
		);

	// Someone saves while the editor is open on the empty note.
	await card.getByLabel('Edit notes').click();
	await editor.fill('mine');
	await saveNotes(api, mapId, id, 'theirs', null);
	let refused = saved();
	await card.getByRole('button', { name: 'Save' }).click();
	expect((await refused).status()).toBe(409);
	await expect(conflict).toBeVisible();
	await expect(editor).toHaveValue('mine');

	await conflict.getByRole('button', { name: 'Load theirs' }).click();
	await expect(conflict).toHaveCount(0);
	await expect(editor).toHaveValue('theirs');

	// Again, and this time ours wins.
	await editor.fill('mine after all');
	await saveNotes(api, mapId, id, 'theirs again', 'theirs');
	refused = saved();
	await card.getByRole('button', { name: 'Save' }).click();
	expect((await refused).status()).toBe(409);
	await expect(conflict).toBeVisible();
	// The buttons wait for the card to have read theirs back, which is what Overwrite
	// replaces.
	const overwritten = saved();
	await conflict.getByRole('button', { name: 'Overwrite' }).click();
	expect((await overwritten).status()).toBe(200);
	await expect(card.getByText('mine after all')).toBeVisible();
	await expect
		.poll(async () => (await api.get(`/api/maps/${mapId}/systems/${id}/details`)).json())
		.toEqual({ notes: 'mine after all' });
});

test('viewers cannot see notes', async ({ page, api, browser }) => {
	const mapId = await createMap(api, 'E2E ViewerNotes');
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: J122515, x: 200, y: 200, alias: null },
	});
	const viewer = await createIdentity(1);
	await grantAccess(mapId, viewer.characterId, 'viewer');

	// The details endpoint refuses viewers outright.
	const viewerCtx = await browser.newContext();
	await viewerCtx.addCookies([
		{ name: 'ws_session', value: viewer.session, domain: 'localhost', path: '/' },
	]);
	const viewerPage = await viewerCtx.newPage();
	await viewerPage.goto(`http://localhost:5173/maps/${mapId}?system=${J122515}`);
	await viewerPage.waitForSelector('html[data-hydrated="true"]');
	await expect(viewerPage.getByTestId('system-info')).toBeVisible();
	await expect(viewerPage.getByTestId('notes-card')).toHaveCount(0);
	await viewerCtx.close();
	void page;
});

import { expect, gotoApp, reloadApp, test } from './fixtures';
import { createMap } from './arrange';
import { createIdentity, deleteMapRow, grantAccess } from './db';

// Map settings: renaming on the General section, the names the map hands out under
// Naming, and the Access section that decides who sees the chain.

test('renaming the map shows up on the map itself', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Before');
	await gotoApp(page, `/maps/${mapId}/settings`);

	const input = page.getByTestId('map-name-input');
	await expect(input).toHaveValue('E2E Before');
	await input.fill('E2E After');
	await page.getByTestId('rename-button').click();
	// Save goes disabled once the reloaded map carries the new name, which is the signal
	// that the write landed. Re-checking the input would pass instantly, since `fill` had
	// already set it, and navigating then aborts the request in flight.
	await expect(page.getByTestId('rename-button')).toBeDisabled();

	await gotoApp(page, `/maps/${mapId}`);
	await expect(page.getByTestId('status-bar-name')).toHaveText('E2E After');
});

test('the owner is listed, and granting adds a second entry', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Access');
	// A second identity, so the grant below has somebody to be granted to.
	await createIdentity(2);
	await gotoApp(page, `/maps/${mapId}/settings/access`);

	const list = page.getByTestId('access-list');
	await expect(list.getByText('E2E Pilot')).toBeVisible();
	// Every role explains itself, so picking one is not a guess.
	const help = page.getByTestId('role-help');
	await expect(help).toContainText('Viewer');
	await expect(help).toContainText('Everything a manager does, and can delete the map.');

	// The grant search only knows entities WormholeSystems has cached, so a known character resolves.
	// Driven by keyboard alone, which is what a combobox is for.
	await page.getByTestId('grant-search').click();
	await page.getByPlaceholder('Name, ticker, or an EVE id…').fill('E2E Extra 2');
	await expect(page.getByTestId('grant-match').first()).toBeVisible();
	await page.keyboard.press('ArrowDown');
	await page.keyboard.press('Enter');
	await expect(page.getByTestId('grant-search')).toContainText('E2E Extra 2');
	await page.getByTestId('grant-button').click();
	await expect(page.getByTestId('access-row')).toHaveCount(2);
});

test('a grant can be given an end date, and taken back to permanent', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Expiry');
	const mate = await createIdentity(4);
	await gotoApp(page, `/maps/${mapId}/settings/access`);

	await page.getByTestId('grant-search').click();
	await page.getByPlaceholder('Name, ticker, or an EVE id…').fill(String(mate.characterId));
	await page.getByTestId('grant-duration').click();
	await page.getByTestId('duration-24').click();
	await page.getByTestId('grant-button').click();

	await expect(page.getByTestId('access-row')).toHaveCount(2);
	// The row says when it runs out, rather than looking like any other grant.
	await expect(page.getByTestId('access-expiry')).toHaveCount(1);
	const stored = await (await api.get(`/api/maps/${mapId}/access`)).json();
	expect(
		stored.find((e: { subject_id: number }) => e.subject_id === mate.characterId).expires_at,
	).not.toBeNull();

	// Dropping the end date is confirmed first: it is a permission that stops expiring.
	await page.getByTestId('access-expiry').click();
	await expect(page.getByTestId('confirm-dialog')).toContainText('until somebody takes it away');
	await page.getByTestId('confirm-accept').click();
	await expect(page.getByTestId('access-expiry')).toHaveCount(0);
});

test('ownership is handed on from the danger zone, not granted from the list', async ({
	page,
	api,
}) => {
	const mapId = await createMap(api, 'E2E Ownership');
	const mate = await createIdentity(7);
	await grantAccess(mapId, mate.characterId, 'manager');

	await gotoApp(page, `/maps/${mapId}/settings/access`);
	// Owner is described but not offered: it is not a permission to hand out.
	await expect(page.getByTestId('role-help')).toContainText('Owner');
	await page.getByTestId('grant-role').click();
	await expect(page.getByRole('option', { name: 'Owner' })).toHaveCount(0);
	await page.keyboard.press('Escape');

	await gotoApp(page, `/maps/${mapId}/settings`);
	const zone = page.getByTestId('danger-zone');
	await expect(zone).toContainText('Hand the map to someone else');
	await zone.getByTestId('transfer-target').click();
	await page.getByRole('option', { name: 'E2E Extra 7' }).click();
	await zone.getByTestId('transfer-button').click();
	await page.getByTestId('confirm-accept').click();

	// One owner, and the old one keeps running the map.
	await expect
		.poll(async () => {
			const entries = await (await api.get(`/api/maps/${mapId}/access`)).json();
			return entries
				.filter((e: { role: string }) => e.role === 'owner')
				.map((e: { subject_id: number }) => e.subject_id);
		})
		.toEqual([mate.characterId]);
	// And the danger zone is theirs now, not ours.
	await expect(page.getByTestId('danger-zone')).toHaveCount(0);

	// The map belongs to the other pilot now, so only the database can take it back.
	await deleteMapRow(mapId);
});

test('the access list filters and sorts, and takes a date of its own', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E AccessTable');
	for (const slot of [8, 9]) {
		const mate = await createIdentity(slot);
		await grantAccess(mapId, mate.characterId, slot === 8 ? 'viewer' : 'manager');
	}
	await gotoApp(page, `/maps/${mapId}/settings/access`);
	await expect(page.getByTestId('access-row')).toHaveCount(3);

	// Filtering narrows to what was typed.
	await page.getByTestId('access-filter').fill('Extra 8');
	await expect(page.getByTestId('access-row')).toHaveCount(1);
	await page.getByTestId('access-filter').fill('');

	// Sorting by name, then again for the other direction.
	await page.getByTestId('sort-name').click();
	const first = () => page.getByTestId('access-row').first().textContent();
	const ascending = await first();
	await page.getByTestId('sort-name').click();
	expect(await first()).not.toBe(ascending);

	// A grant can end on a date picked from the calendar rather than a fixed span.
	const mate = await createIdentity(6);
	await page.getByTestId('grant-search').click();
	await page.getByPlaceholder('Name, ticker, or an EVE id…').fill('E2E Extra 6');
	await page.getByTestId('grant-match').first().click();
	await page.getByTestId('grant-duration').click();
	// The last day of the shown month that is still selectable, so the pick never lands on
	// a greyed-out day from the month either side.
	await page
		.locator('[data-calendar-day]:not([data-outside-month]):not([aria-disabled="true"])')
		.last()
		.click();
	await expect(page.getByTestId('grant-duration')).not.toContainText('No end date');
	await page.getByTestId('grant-button').click();

	await expect
		.poll(async () => {
			const rows = await (await api.get(`/api/maps/${mapId}/access`)).json();
			return rows.find((e: { subject_id: number }) => e.subject_id === mate.characterId)
				?.expires_at;
		})
		.toBeTruthy();
});

test('a viewer sees the roles but cannot change them', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E ViewerSettings');
	const viewer = await createIdentity(3);
	await grantAccess(mapId, viewer.characterId, 'viewer');

	const ctx = await page.context().browser()!.newContext();
	await ctx.addCookies([
		{ name: 'ws_session', value: viewer.session, domain: 'localhost', path: '/' },
	]);
	const viewerPage = await ctx.newPage();
	await viewerPage.goto(`http://localhost:5173/maps/${mapId}/settings/access`);
	await viewerPage.waitForSelector('html[data-hydrated="true"]');

	await expect(viewerPage.getByTestId('access-list').getByText('E2E Pilot')).toBeVisible();
	// No grant form: granting is manager/owner work.
	await expect(viewerPage.getByTestId('grant-search')).toHaveCount(0);
	// And the sections that change the map for everyone are not offered at all, rather
	// than offered and refused.
	const nav = viewerPage.getByTestId('settings-nav');
	await expect(nav).toContainText('Routing');
	await expect(nav).not.toContainText('General');
	await expect(nav).not.toContainText('Discord alerts');
	await ctx.close();
});

test('a manager runs the general settings, and only the owner gets the danger zone', async ({
	page,
	api,
}) => {
	const mapId = await createMap(api, 'E2E ManagerSettings');
	const manager = await createIdentity(40);
	await grantAccess(mapId, manager.characterId, 'manager');

	const ctx = await page.context().browser()!.newContext();
	await ctx.addCookies([
		{ name: 'ws_session', value: manager.session, domain: 'localhost', path: '/' },
	]);
	const managerPage = await ctx.newPage();
	await managerPage.goto(`http://localhost:5173/maps/${mapId}/settings`);
	await managerPage.waitForSelector('html[data-hydrated="true"]');

	await managerPage.getByTestId('map-name-input').fill('E2E Managed');
	await managerPage.getByTestId('rename-button').click();
	await expect(managerPage.getByTestId('rename-button')).toBeDisabled();
	await expect
		.poll(async () => (await (await api.get(`/api/maps/${mapId}`)).json()).map.name)
		.toBe('E2E Managed');

	// Deleting the map and handing it on stay the owner's.
	await expect(managerPage.getByTestId('danger-zone')).toHaveCount(0);
	await ctx.close();
});

test('chain naming previews as you type and survives a reload', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Naming');
	await gotoApp(page, `/maps/${mapId}/settings/naming`);

	// The defaults are legacy's, so an untouched map already names things sensibly.
	await expect(page.getByTestId('alias-preview')).toHaveText('1, 2, 3, 11');
	await expect(page.getByTestId('bookmark_wormhole-preview')).toHaveText('1a ABC C5');
	await expect(page.getByTestId('bookmark_kspace-preview')).toHaveText('1b HS ABC Jita The Forge');

	// Alphabetical skips H, L, N and P, which belong to the k-space exits.
	await page.getByTestId('alias-scheme').getByText('Alphabetical').click();
	await expect(page.getByTestId('alias-preview')).toHaveText('A, B, C, AA');

	await page.getByTestId('bookmark_wormhole').fill('{alias} {sig} {class} {wh} {life}');
	await expect(page.getByTestId('bookmark_wormhole-preview')).toHaveText('1a ABC C5 H296 EOL');

	await page.getByTestId('save-naming').click();
	await expect(page.getByTestId('save-naming')).toBeDisabled();

	await reloadApp(page);
	await expect(page.getByTestId('bookmark_wormhole')).toHaveValue(
		'{alias} {sig} {class} {wh} {life}',
	);
	await expect(page.getByTestId('alias-preview')).toHaveText('A, B, C, AA');

	// Copying the bookmark is per user, and gated on location sharing a fresh map lacks.
	const copy = page.locator('[data-setting="copy-bookmark"]');
	await expect(copy.getByRole('switch')).toBeDisabled();
	await expect(copy).toContainText('Needs location sharing');
});

test('settings are split into sections, and the per-user ones save themselves', async ({
	page,
	api,
}) => {
	const mapId = await createMap(api, 'E2E Sections');
	await gotoApp(page, `/maps/${mapId}/settings`);

	// Every section is reachable from any other.
	const nav = page.getByTestId('settings-nav');
	await expect(nav.getByTestId('settings-section')).toHaveCount(8);
	await expect(nav.locator('[data-active="true"]')).toContainText('General');

	// A per-user setting saves on the spot: there is no form to submit.
	await nav.getByText('Routing').click();
	await expect(page.getByTestId('route-preference')).toBeVisible();
	await page.getByTestId('route-preference').click();
	await page.getByRole('option', { name: 'Safer' }).click();
	await expect
		.poll(
			async () =>
				(await (await api.get(`/api/maps/${mapId}/settings/user`)).json()).route_preference,
		)
		.toBe('safer');

	// And it survives leaving and coming back, because it is stored per map, not per tab.
	await nav.getByText('Display').click();
	await nav.getByText('Routing').click();
	await expect(page.getByTestId('route-preference')).toContainText('Safer');
});

test('mapping settings hold back what location sharing gates', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Gated');
	await gotoApp(page, `/maps/${mapId}/settings/mapping`);

	// Sharing is off on a new map, so the ones that depend on it say why rather than
	// silently doing nothing.
	const prompt = page.locator('[data-setting="prompt-for-signature"]');
	await expect(prompt.getByRole('switch')).toBeDisabled();
	await expect(prompt).toContainText('Needs location sharing');
	const preselect = page.locator('[data-setting="preselect-signature"]');
	await expect(preselect.getByRole('switch')).toBeDisabled();
	await expect(preselect).toContainText('Needs location sharing');
});

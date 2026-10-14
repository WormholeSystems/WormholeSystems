import { expect, gotoApp, test } from './fixtures';
import { createMap } from './arrange';
import { createIdentity, grantAccess } from './db';

// Discord alerts: the settings page that creates them, and the gating that decides who
// may. Whether a message actually reaches Discord is the delivery layer's problem and is
// covered by the Rust tests; this is about the rules being editable and safe.

const JITA = 30000142;
const WEBHOOK = 'https://discord.com/api/webhooks/123456789/abcdefghijklmnop';

/** Destinations are registered once per map and pointed at by name. */
async function addDestination(page: import('@playwright/test').Page, name: string) {
	await page.getByTestId('destination-name').fill(name);
	await page.getByTestId('destination-url').fill(WEBHOOK);
	await page.getByTestId('destination-add').click();
	await expect(page.getByTestId('destination-row').filter({ hasText: name })).toBeVisible();
}

test('a killmail alert can be created, edited and turned off', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Alerts');
	await gotoApp(page, `/maps/${mapId}/settings/alerts`);

	await expect(page.getByTestId('alerts-empty')).toBeVisible();
	await addDestination(page, 'Intel channel');

	await page.getByTestId('alert-new').click();
	await page.getByTestId('alert-name').fill('Kills at home');
	await page.getByTestId('alert-jumps').fill('3');
	await page.getByTestId('alert-destination').click();
	await page.getByRole('option', { name: 'Intel channel' }).click();
	await page.getByTestId('alert-save').click();

	const row = page.getByTestId('alert-row');
	await expect(row).toHaveCount(1);
	await expect(row).toContainText('Kills at home');
	await expect(row).toContainText('Anything that dies within 3 jumps');
	// The alert names its destination; the URL itself is never handed back out.
	await expect(row).toContainText('Intel channel');
	await expect(row).not.toContainText('abcdefghijklmnop');
	const listed = await (await api.get(`/api/maps/${mapId}/alerts`)).text();
	expect(listed).not.toContain('abcdefghijklmnop');

	// Editing keeps the destination it already points at.
	await page.getByTestId('alert-edit').click();
	await page.getByTestId('alert-name').fill('Kills near home');
	await page.getByTestId('alert-save').click();
	await expect(row).toContainText('Kills near home');
	await expect(row).toContainText('Intel channel');

	// Turning it off says so, and says why.
	await row.getByRole('switch').click();
	await expect(page.getByTestId('alert-reason')).toContainText('Turned off by hand');

	// And every one of those is on the record.
	const events = page.getByTestId('alert-events');
	await expect(events).toContainText('created');
	await expect(events).toContainText('updated');
	await expect(events).toContainText('disabled');
});

test('filters narrow a killmail alert to who is involved', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E AlertFilters');
	await gotoApp(page, `/maps/${mapId}/settings/alerts`);

	await addDestination(page, 'Intel');
	await page.getByTestId('alert-new').click();
	await page.getByTestId('alert-name').fill('Their losses');
	await page.getByTestId('alert-destination').click();
	await page.getByRole('option', { name: 'Intel' }).click();
	await page.getByTestId('alert-add-filter').click();
	await page.getByTestId('alert-filter-ids').fill('99000001, 99000002');
	await page.getByTestId('alert-save').click();

	await expect(page.getByTestId('alert-row')).toContainText('1 filter (any)');

	// The rule survives a round trip through the server.
	const alerts = await (await api.get(`/api/maps/${mapId}/alerts`)).json();
	expect(alerts[0].filters).toEqual([
		{ subject: 'alliance', side: 'either', mode: 'include', ids: [99000001, 99000002] },
	]);
});

test('a proximity alert needs a system, and reports it', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E AlertProximity');
	await gotoApp(page, `/maps/${mapId}/settings/alerts`);

	await addDestination(page, 'Intel');
	await page.getByTestId('alert-new').click();
	await page.getByTestId('alert-name').fill('Jita gets close');
	await page.getByTestId('alert-destination').click();
	await page.getByRole('option', { name: 'Intel' }).click();
	await page.getByTestId('alert-kind').click();
	await page.getByRole('option', { name: 'System near the chain' }).click();

	// Without a target there is nothing to measure to, so saving is refused.
	await expect(page.getByTestId('alert-save')).toBeDisabled();

	await page.getByTestId('system-picker-pick a system').click();
	await page.getByPlaceholder('Search…').fill('jita');
	await page.getByTestId('picker-result').first().click();
	await page.getByTestId('alert-save').click();

	await expect(page.getByTestId('alert-row')).toContainText('Jita within 5 jumps');

	const alerts = await (await api.get(`/api/maps/${mapId}/alerts`)).json();
	expect(alerts[0].target_solar_system_id).toBe(JITA);
});

test('members cannot see or change a map alerts', async ({ page, api, browser }) => {
	const mapId = await createMap(api, 'E2E AlertGating');
	const member = await createIdentity(37);
	await grantAccess(mapId, member.characterId, 'member');

	const ctx = await browser.newContext();
	await ctx.addCookies([
		{ name: 'ws_session', value: member.session, domain: 'localhost', path: '/' },
	]);
	const memberPage = await ctx.newPage();
	await memberPage.goto(`http://localhost:5173/maps/${mapId}/settings/alerts`);
	await memberPage.waitForSelector('html[data-hydrated="true"]');
	await expect(memberPage.getByTestId('alerts-error')).toBeVisible();
	await expect(memberPage.getByTestId('alert-new')).toHaveCount(0);
	await ctx.close();
	void page;
});

test('a jump range alert measures light years for the hull you pick', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E AlertJump');
	await gotoApp(page, `/maps/${mapId}/settings/alerts`);
	await addDestination(page, 'Capitals');

	await page.getByTestId('alert-new').click();
	await page.getByTestId('alert-name').fill('Rens in range');
	await page.getByTestId('alert-destination').click();
	await page.getByRole('option', { name: 'Capitals' }).click();
	await page.getByTestId('alert-kind').click();
	await page.getByRole('option', { name: 'Capital jump range' }).click();

	// Gate jumps are not the question here, so that field is gone.
	await expect(page.getByTestId('alert-jumps')).toHaveCount(0);
	// A dreadnought at JDC 5 reaches 7 ly, and the form says so while you choose.
	await expect(page.getByTestId('alert-range')).toContainText('7.0 ly');
	await page.getByTestId('alert-jdc').fill('0');
	await expect(page.getByTestId('alert-range')).toContainText('3.5 ly');
	await page.getByTestId('alert-jdc').fill('5');

	await page.getByTestId('alert-ship').click();
	await page.getByRole('option', { name: 'Titan' }).click();
	await expect(page.getByTestId('alert-range')).toContainText('6.0 ly');

	await page.getByTestId('system-picker-pick a system').click();
	await page.getByPlaceholder('Search…').fill('jita');
	await page.getByTestId('picker-result').first().click();
	await page.getByTestId('alert-save').click();

	await expect(page.getByTestId('alert-row')).toContainText('Titan range (JDC 5)');
	const alerts = await (await api.get(`/api/maps/${mapId}/alerts`)).json();
	expect(alerts[0].ship_type).toBe('titan');
	expect(alerts[0].jdc_level).toBe(5);
});

test('a destination is registered once and shared by its alerts', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E AlertShare');
	await gotoApp(page, `/maps/${mapId}/settings/alerts`);
	await addDestination(page, 'Shared');

	for (const name of ['First', 'Second']) {
		await page.getByTestId('alert-new').click();
		await page.getByTestId('alert-name').fill(name);
		await page.getByTestId('alert-destination').click();
		await page.getByRole('option', { name: 'Shared' }).click();
		await page.getByTestId('alert-save').click();
	}

	await expect(page.getByTestId('alert-row')).toHaveCount(2);
	// The destination knows what depends on it, so deleting it can warn.
	await expect(page.getByTestId('destination-row')).toContainText('2 alerts');
	void api;
});

import { expect, gotoApp, test } from './fixtures';
import { addSystem, connect, createMap } from './arrange';
import { createIdentity, grantAccess } from './db';

// The connection details popover: left-click an edge → signatures, status, wormhole
// properties, and mass tracking with the jump log.

const J122515 = 31001882; // C5 wormhole, static H296
const JITA = 30000142;

async function pasteSig(
	api: import('@playwright/test').APIRequestContext,
	mapId: number,
	solarSystemId: number,
	sig: Record<string, unknown>,
) {
	const res = await api.post(`/api/maps/${mapId}/signatures/paste`, {
		data: { map_id: mapId, solar_system_id: solarSystemId, signatures: [sig] },
	});
	expect(res.ok()).toBe(true);
}

async function linkSig(
	api: import('@playwright/test').APIRequestContext,
	mapId: number,
	signatureId: string,
	connectionId: number,
) {
	const sigs = await (await api.get(`/api/maps/${mapId}/signatures`)).json();
	const sig = sigs.find((s: { signature_id: string }) => s.signature_id === signatureId);
	const res = await api.post(`/api/maps/${mapId}/signatures/link`, {
		data: { map_id: mapId, signature_pk: sig.id, connection_id: connectionId },
	});
	expect(res.ok()).toBe(true);
}

async function openPopover(page: import('@playwright/test').Page) {
	await page.locator('[data-testid="edge-hit"]').first().click({ force: true });
	const popover = page.getByTestId('connection-popover');
	await expect(popover).toBeVisible();
	return popover;
}

test('click opens the popover: status, no signatures, empty mass section', async ({
	page,
	api,
}) => {
	const mapId = await createMap(api, 'E2E ConnPopover');
	const a = await addSystem(api, mapId, J122515, { x: 200, y: 200 });
	const b = await addSystem(api, mapId, JITA, { x: 560, y: 200 });
	await connect(api, mapId, a, b);

	await gotoApp(page, `/maps/${mapId}`);
	const popover = await openPopover(page);

	await expect(popover.getByText('No signatures assigned')).toBeVisible();
	await expect(popover.getByText('Wormhole', { exact: true })).toBeVisible();
	await expect(popover.getByText('Healthy')).toBeVisible();
	await expect(popover.getByText('Unknown', { exact: true })).toBeVisible();
	// No resolved wormhole type → no bar/Remaining, but the Jumped row shows.
	await expect(popover.getByTestId('mass-tracking')).toBeVisible();
	await expect(popover.getByTestId('mass-bar')).toHaveCount(0);
	await expect(popover.getByTestId('mass-jumped')).toHaveText('0');
	await expect(popover.getByTestId('jump-log-trigger')).toContainText('0 jumps');

	// Opening must not auto-focus a tooltip trigger (a stray tooltip's dismiss layer
	// would eat the first outside click).
	await expect(page.locator('[data-slot="tooltip-content"]')).toHaveCount(0);

	// A single outside click closes it, and the map stays interactive afterwards.
	await page.mouse.click(900, 620);
	await expect(popover).toHaveCount(0);
	await page.getByTestId('system-node').filter({ hasText: 'Jita' }).click();
	await expect(page.getByTestId('system-info')).toBeVisible();

	// Reopen: Escape closes it too.
	await openPopover(page);
	await page.keyboard.press('Escape');
	await expect(popover).toHaveCount(0);
});

test('typed signatures drive Out/In sections, properties, and the mass bar', async ({
	page,
	api,
}) => {
	const mapId = await createMap(api, 'E2E ConnPhysics');
	const a = await addSystem(api, mapId, J122515, { x: 200, y: 200 });
	const b = await addSystem(api, mapId, JITA, { x: 560, y: 200 });
	const conn = await connect(api, mapId, a, b);

	const catalog = await (await api.get('/api/signature-types')).json();
	const h296 = catalog.types.find((t: { signature: string | null }) => t.signature === 'H296');
	expect(h296.total_mass).toBeGreaterThan(0);
	const k162 = catalog.types.find((t: { signature: string | null }) => t.signature === 'K162');

	await pasteSig(api, mapId, J122515, {
		signature_id: 'OUT-001',
		group: 'wormhole',
		signature_type_id: h296.id,
	});
	await linkSig(api, mapId, 'OUT-001', conn);
	await pasteSig(api, mapId, JITA, {
		signature_id: 'INN-001',
		group: 'wormhole',
		signature_type_id: k162.id,
	});
	await linkSig(api, mapId, 'INN-001', conn);

	await gotoApp(page, `/maps/${mapId}`);
	const popover = await openPopover(page);

	// The H296 side is outbound, the K162 side inbound.
	const outSection = popover.locator('div', { hasText: /^Out Sig/ }).first();
	await expect(popover.getByText('Out Sig')).toBeVisible();
	await expect(popover.getByText('In Sig')).toBeVisible();
	await expect(popover.getByText('OUT-001')).toBeVisible();
	await expect(popover.getByText('INN-001')).toBeVisible();
	void outSection;

	// Physics from the H296 catalog row.
	const props = popover.getByTestId('popover-properties');
	await expect(props).toBeVisible();
	await expect(props.getByText('Total Mass')).toBeVisible();
	await expect(
		props.getByText(`${(h296.total_mass / 1_000_000).toLocaleString('en-US')} kt`),
	).toBeVisible();

	// Fresh hole → full bar and ≈100% remaining.
	await expect(popover.getByTestId('mass-bar')).toBeVisible();
	await expect(popover.getByTestId('mass-remaining')).toContainText('(100%)');
});

test('stargate connections hide mass tracking', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E ConnGate');
	const a = await addSystem(api, mapId, JITA, { x: 200, y: 200 });
	const b = await addSystem(api, mapId, 30000144, { x: 560, y: 200 }); // Perimeter
	await connect(api, mapId, a, b, { kind: 'stargate' });

	await gotoApp(page, `/maps/${mapId}`);
	const popover = await openPopover(page);
	await expect(popover.getByText('Stargate')).toBeVisible();
	await expect(popover.getByTestId('mass-tracking')).toHaveCount(0);
});

test('EOL and preserve-mass surface in the status section', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E ConnStatus');
	const a = await addSystem(api, mapId, J122515, { x: 200, y: 200 });
	const b = await addSystem(api, mapId, JITA, { x: 560, y: 200 });
	const conn = await connect(api, mapId, a, b);
	const res = await api.post(`/api/maps/${mapId}/connections/set-status`, {
		data: { map_id: mapId, connection_id: conn, time_status: 'eol', preserve_mass: true },
	});
	expect(res.ok()).toBe(true);

	await gotoApp(page, `/maps/${mapId}`);
	const popover = await openPopover(page);
	await expect(popover.getByTestId('popover-lifetime')).toContainText('End of Life');
	await expect(popover.getByTestId('popover-preserve-mass')).toContainText('Yes');
});

test('manual jump flow: log, edit direction, delete', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E ConnJumps');
	const a = await addSystem(api, mapId, J122515, { x: 200, y: 200 });
	const b = await addSystem(api, mapId, JITA, { x: 560, y: 200 });
	const conn = await connect(api, mapId, a, b);

	await gotoApp(page, `/maps/${mapId}`);
	const popover = await openPopover(page);

	await popover.getByTestId('jump-log-trigger').click();
	const log = page.getByTestId('jump-log');
	await expect(log).toBeVisible();
	await expect(log.getByText('No jumps logged yet')).toBeVisible();

	// Log a Rifter: the hull mass autofills from the ship search.
	await log.getByTestId('log-jump').click();
	const form = page.getByTestId('jump-form');
	await expect(form).toBeVisible();
	// Keyboard-driven pick: type, highlighted first result, Enter.
	await form.getByTestId('ship-search').fill('Rifter');
	await expect(page.getByRole('option', { name: /Rifter/ }).first()).toBeVisible();
	await form.getByTestId('ship-search').press('Enter');
	await expect(form.getByText('Rifter', { exact: true })).toBeVisible();
	await expect(form.getByTestId('jump-mass')).toHaveValue(/1\.?\d*/);
	await form.getByRole('button', { name: 'Save' }).click();

	const row = log.getByTestId('jump-row');
	await expect(row).toHaveCount(1);
	await expect(row.getByText('Rifter')).toBeVisible();
	await expect(row.getByText('manual')).toBeVisible();
	await expect(page.getByTestId('mass-jumped')).not.toHaveText('0');
	const jumps = await (await api.get(`/api/maps/${mapId}/connections/${conn}/jumps`)).json();
	expect(jumps).toHaveLength(1);
	expect(jumps[0].is_manual).toBe(true);

	// Edit: flip the direction to inbound.
	await row.getByLabel('Jump actions').click();
	await page.getByRole('menuitem', { name: 'Edit' }).click();
	await expect(form).toBeVisible();
	await form.getByTestId('jump-direction').click();
	await form.getByRole('button', { name: 'Save' }).click();
	await expect
		.poll(async () => {
			const j = await (await api.get(`/api/maps/${mapId}/connections/${conn}/jumps`)).json();
			return j[0].from_solar_system_id;
		})
		.toBe(JITA);

	// Delete: the log empties.
	await row.getByLabel('Jump actions').click();
	await page.getByRole('menuitem', { name: 'Delete' }).click();
	await expect(log.getByTestId('jump-row')).toHaveCount(0);
	await expect(log.getByText('No jumps logged yet')).toBeVisible();
});

test('viewers get a read-only popover', async ({ page, api, browser }) => {
	const mapId = await createMap(api, 'E2E ConnViewer');
	const a = await addSystem(api, mapId, J122515, { x: 200, y: 200 });
	const b = await addSystem(api, mapId, JITA, { x: 560, y: 200 });
	await connect(api, mapId, a, b);
	const viewer = await createIdentity(7);
	await grantAccess(mapId, viewer.characterId, 'viewer');

	const ctx = await browser.newContext();
	await ctx.addCookies([
		{ name: 'ws_session', value: viewer.session, domain: 'localhost', path: '/' },
	]);
	const viewerPage = await ctx.newPage();
	await viewerPage.goto(`http://localhost:5173/maps/${mapId}`);
	await viewerPage.waitForSelector('html[data-hydrated="true"]');

	const popover = await openPopover(viewerPage);
	await popover.getByTestId('jump-log-trigger').click();
	const log = viewerPage.getByTestId('jump-log');
	await expect(log).toBeVisible();
	await expect(log.getByTestId('log-jump')).toHaveCount(0);
	await ctx.close();
	void page;
});

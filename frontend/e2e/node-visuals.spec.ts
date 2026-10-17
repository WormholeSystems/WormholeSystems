import { expect, gotoApp, test } from './fixtures';
import { addSystem, createMap } from './arrange';

// Legacy node parity: displayed information, colors, tooltips, and the alias editor.

const JITA = 30000142; // highsec k-space
const J122515 = 31001882; // C5, Wolf-Rayet Star, static H296 (to C5)
const J005482 = 31002515; // C2, shattered
const THERA = 31000005; // class 12

test('node shows class, alias, occupier, statics, effect, and region per space kind', async ({
	page,
	api,
}) => {
	const mapId = await createMap(api, 'E2E Visuals');
	const wh = await addSystem(api, mapId, J122515, { x: 200, y: 200, alias: 'Home' });
	await addSystem(api, mapId, JITA, { x: 200, y: 400 });
	await addSystem(api, mapId, THERA, { x: 200, y: 600 });
	await api.post(`/api/maps/${mapId}/systems/set-occupier`, {
		data: { map_id: mapId, map_solar_system_id: wh, occupier: 'LZHX' },
	});

	await gotoApp(page, `/maps/${mapId}`);
	const whNode = page.getByTestId('system-node').filter({ hasText: 'J122515' });

	// Class label with the class color token.
	const classLabel = whNode.getByText('C5', { exact: true }).first();
	await expect(classLabel).toBeVisible();
	await expect(classLabel).toHaveAttribute('style', /--color-c5/);

	// Alias bright, real name dimmed, occupier in parentheses.
	await expect(whNode.getByText('Home', { exact: true })).toBeVisible();
	await expect(whNode.getByText('(LZHX)')).toBeVisible();

	// W-space second row: colored static label with a physics tooltip.
	const staticLabel = whNode.getByTestId('static-badge');
	await expect(staticLabel).toHaveText('C5');
	await expect(staticLabel).toHaveAttribute('style', /--color-c5/);
	await staticLabel.hover();
	const tooltip = page.locator('[data-slot="tooltip-content"]');
	await expect(tooltip.getByText('H296')).toBeVisible();
	await expect(tooltip.getByText('Total Mass')).toBeVisible();
	await expect(tooltip.getByText('3,300 kt')).toBeVisible();
	await expect(tooltip.getByText('24h')).toBeVisible();

	// The effect's own HUD icon (no sovereignty in J-space), popover lists modifiers.
	const effect = whNode.getByLabel('Wolf-Rayet Star');
	await expect(effect).toBeVisible();
	await expect
		.poll(() =>
			effect.locator('img').evaluate((el: HTMLImageElement) => el.complete && el.naturalWidth > 0),
		)
		.toBe(true);
	// It opens something when clicked, so it gets the ordinary pointer, not the help one.
	await expect(effect).toHaveCSS('cursor', 'pointer');
	await effect.click();
	await expect(page.getByText('Armor HP')).toBeVisible();
	// Buffs and debuffs apart, each stat with the client's icon for it, loaded.
	const card = page.locator('[data-slot="popover-content"]').getByTestId('effect-modifiers');
	await expect(card.getByText('Buffs', { exact: true })).toBeVisible();
	await expect(card.getByText('Debuffs', { exact: true })).toBeVisible();
	const rows = card.getByTestId('effect-modifier');
	expect(await rows.count()).toBeGreaterThan(3);
	for (const img of await rows.locator('img').all()) {
		await expect
			.poll(() => img.evaluate((el: HTMLImageElement) => el.complete && el.naturalWidth > 0))
			.toBe(true);
	}
	expect(await rows.locator('img').count()).toBe(await rows.count());
	await page.keyboard.press('Escape');

	// K-space node shows the region, not statics.
	const jita = page.getByTestId('system-node').filter({ hasText: 'Jita' });
	await expect(jita.getByText('The Forge')).toBeVisible();
	await expect(jita.getByText('H', { exact: true })).toHaveAttribute('style', /--color-hs/);

	// Thera resolves to class 12.
	const thera = page.getByTestId('system-node').filter({ hasText: 'Thera' });
	await expect(thera.getByText('C12', { exact: true })).toBeVisible();
});

test('a long occupier is cut short inside the node, in full on hover', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Long Occupier');
	const wh = await addSystem(api, mapId, J122515, { x: 200, y: 200, alias: 'Home' });
	await api.post(`/api/maps/${mapId}/systems/set-home`, {
		data: { map_id: mapId, map_solar_system_id: wh, value: true },
	});
	const occupier = 'The Very Long Named Wormhole Corporation Holding This Hole';
	await api.post(`/api/maps/${mapId}/systems/set-occupier`, {
		data: { map_id: mapId, map_solar_system_id: wh, occupier },
	});

	await gotoApp(page, `/maps/${mapId}`);
	const node = page.getByTestId('system-node').filter({ hasText: 'J122515' });
	const label = node.getByTestId('node-occupier');
	await expect(label).toBeVisible();

	// Cut short rather than spilling out, and the icons after it stay inside the node.
	const box = (await node.boundingBox())!;
	const labelBox = (await label.boundingBox())!;
	expect(labelBox.x + labelBox.width).toBeLessThanOrEqual(box.x + box.width);
	expect(await label.evaluate((el) => el.scrollWidth > el.clientWidth)).toBe(true);
	const home = node.locator('.text-amber-400');
	await expect(home).toBeVisible();
	const homeBox = (await home.boundingBox())!;
	expect(homeBox.x + homeBox.width).toBeLessThanOrEqual(box.x + box.width);

	// The whole name is still there for whoever points at it.
	await label.hover();
	await expect(page.locator('[data-slot="tooltip-content"]')).toHaveText(occupier);

	await api.delete(`/api/maps/${mapId}`);
});

test('icon cluster: shattered, signatures, unmapped wormholes, home, pin', async ({
	page,
	api,
}) => {
	const mapId = await createMap(api, 'E2E Icons');
	const shattered = await addSystem(api, mapId, J005482, { x: 200, y: 200 });
	await api.post(`/api/maps/${mapId}/systems/set-home`, {
		data: { map_id: mapId, map_solar_system_id: shattered, value: true },
	});
	// One categorized wormhole sig (drives the fan icon: 1 wormhole sig, 0 connections)
	// and one uncategorized sig (turns the satellite rose).
	await api.post(`/api/maps/${mapId}/signatures/paste`, {
		data: {
			map_id: mapId,
			solar_system_id: J005482,
			signatures: [
				{ signature_id: 'ABC-123', group: 'wormhole', name: 'K162' },
				{ signature_id: 'DEF-456', group: 'unknown', name: null },
			],
		},
	});

	await gotoApp(page, `/maps/${mapId}`);
	const node = page.getByTestId('system-node').filter({ hasText: 'J005482' });

	await expect(node.getByTestId('shattered-icon')).toBeVisible();
	await expect(node.getByTestId('sig-icon')).toBeVisible();
	await expect(node.getByTestId('unmapped-icon')).toBeVisible();

	await node.getByTestId('sig-icon').hover();
	await expect(page.getByText('2 signatures, 1 uncategorized')).toBeVisible();

	await node.getByTestId('unmapped-icon').hover();
	await expect(page.getByText('Has 1 unmapped wormhole')).toBeVisible();

	// Home icon present; pin the system and the drag handle disappears.
	await expect(node.locator('.text-amber-400')).toBeVisible();
	await api.post(`/api/maps/${mapId}/systems/set-pinned`, {
		data: { map_id: mapId, map_solar_system_id: shattered, value: true },
	});
	await node.hover();
	await expect(node.getByTestId('drag-handle')).toHaveCount(0);
});

test('status vocabulary drives border and icon', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Status');
	await addSystem(api, mapId, JITA, { x: 200, y: 200 });

	await gotoApp(page, `/maps/${mapId}`);
	const node = page.getByTestId('system-node').filter({ hasText: 'Jita' });

	// A fresh placement defaults to unknown: neutral border, no status icon.
	await expect(node).toHaveAttribute('data-status', 'unknown');
	await expect(node.locator('[aria-label]').filter({ hasText: '' })).toHaveCount(0);

	await node.click({ button: 'right' });
	await page.getByTestId('status-subtrigger').hover();
	await page.getByTestId('status-submenu').getByRole('menuitem', { name: 'Empty' }).click();
	await expect(node).toHaveAttribute('data-status', 'empty');
	await expect(node.getByLabel('empty')).toBeVisible();
	// The status token must actually resolve to the emerald empty color at runtime
	// (poll: border-color transitions for 200ms).
	await expect
		.poll(async () => node.evaluate((el) => getComputedStyle(el).borderTopColor))
		.toBe('oklch(0.765 0.177 163.223)');

	await node.click({ button: 'right' });
	await page.getByTestId('status-subtrigger').hover();
	await page.getByTestId('status-submenu').getByRole('menuitem', { name: 'Active' }).click();
	await expect(node).toHaveAttribute('data-status', 'active');
	await expect(node.getByLabel('active')).toBeVisible();
});

test('double click opens the alias editor and saves', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Alias');
	await addSystem(api, mapId, JITA, { x: 200, y: 200 });

	await gotoApp(page, `/maps/${mapId}`);
	const node = page.getByTestId('system-node').filter({ hasText: 'Jita' });
	await node.dblclick();

	await page.getByPlaceholder('Alias', { exact: true }).fill('Market');
	await page.getByPlaceholder('Occupier alias').fill('Caldari');
	await page.getByRole('button', { name: 'Save' }).click();

	await expect(node.getByText('Market')).toBeVisible();
	await expect(node.getByText('(Caldari)')).toBeVisible();
});

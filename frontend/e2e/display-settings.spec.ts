import { expect, gotoApp, test } from './fixtures';
import { addSystem, createMap } from './arrange';

// Account-wide display preferences: kept on the account, so a change follows the pilot to
// every map.

const J122515 = 31001882; // C5, Wolf-Rayet Star

test('effect icons can be swapped for letters, and back', async ({ page, api }) => {
	// The suite shares one account, so whatever this test sets is put back afterwards.
	await api.post('/api/me/preferences', { data: { effect_icons: true } });
	const mapId = await createMap(api, 'E2E Effect Letters');
	await addSystem(api, mapId, J122515, { x: 200, y: 200 });

	try {
		await gotoApp(page, '/settings/display');
		const toggle = page.getByTestId('effect-icons-switch');
		await expect(toggle).toBeChecked();
		await toggle.click();
		await expect(toggle).not.toBeChecked();

		await gotoApp(page, `/maps/${mapId}`);
		const badge = page
			.getByTestId('system-node')
			.filter({ hasText: 'J122515' })
			.getByLabel('Wolf-Rayet Star');
		await expect(badge).toHaveText('W');
		await expect(badge.locator('img')).toHaveCount(0);

		// The setting is the account's, so the API says the same as the page did.
		const saved = await api.get('/api/me/preferences');
		expect(await saved.json()).toEqual({ effect_icons: false });

		await gotoApp(page, '/settings/display');
		await page.getByTestId('effect-icons-switch').click();
		await expect(page.getByTestId('effect-icons-switch')).toBeChecked();
		await gotoApp(page, `/maps/${mapId}`);
		await expect(badge.locator('img')).toBeVisible();
	} finally {
		await api.post('/api/me/preferences', { data: { effect_icons: true } });
		await api.delete(`/api/maps/${mapId}`);
	}
});

import { expect, gotoApp, test } from './fixtures';
import { createMap } from './arrange';

// Tooltips in place of the browser's `title`: they wait a second, then follow the
// pointer from control to control without waiting again, and a click puts them away.

test('a hint waits a second, then moves along without waiting', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Hints');
	await gotoApp(page, `/maps/${mapId}`);
	const hint = page.locator('[data-slot="tooltip-content"]');
	const settings = page.getByTestId('route-settings');

	await settings.hover();
	await page.waitForTimeout(500);
	await expect(hint).toHaveCount(0);
	await expect(hint).toHaveText('Route settings');

	// Straight to the neighbour: already warm, so it answers at once.
	await page.getByTestId('watchlist-add').hover();
	// The last one may still be fading out alongside it.
	await expect(hint.filter({ hasText: 'Add to the map watchlist' })).toBeVisible({
		timeout: 300,
	});

	// A click is not a request for a hint.
	await page.mouse.down();
	await expect(hint).toHaveCount(0);
	await page.mouse.up();
});

test('a control that carries a hint still does its own job', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Hints Controls');
	await gotoApp(page, `/maps/${mapId}`);

	// A popover trigger under a hint still opens its popover.
	await page.getByTestId('background-button').click();
	await expect(page.getByTestId('background-popover')).toBeVisible();
	await page.keyboard.press('Escape');

	// Its own `disabled` wins over the tooltip's: on Find the + cannot add anything.
	const card = page.getByTestId('navigation-card');
	await card.getByTestId('navigation-tab-find').click();
	await expect(card.getByTestId('watchlist-add')).toBeDisabled();
	await card.getByTestId('navigation-tab-watchlist').click();
	await expect(card.getByTestId('watchlist-add')).toBeEnabled();
});

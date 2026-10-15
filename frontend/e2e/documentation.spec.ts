import { expect, gotoApp, test } from './fixtures';

// The in-app documentation: Markdown under frontend/src/docs, one folder per category.

test('the documentation renders, navigates and links onward', async ({ page }) => {
	// The section root goes to the first page rather than an index nobody reads.
	await gotoApp(page, '/documentation');
	await expect(page).toHaveURL(/\/documentation\/getting-started\/overview$/);

	const article = page.getByTestId('docs-page');
	await expect(article.locator('h1')).toHaveText('Overview');
	// The beta note is on the first page somebody lands on.
	await expect(article).toContainText('Beta');

	// The sidebar reaches a page in another category, and tables survive rendering.
	await page.getByRole('link', { name: 'Mass', exact: true }).click();
	await expect(page).toHaveURL(/\/documentation\/connections\/mass$/);
	await expect(article.locator('table')).toBeVisible();
	await expect(article).toContainText('Critical');

	// Prev/next walk the same order the sidebar shows.
	await page.getByTestId('docs-next').click();
	await expect(article.locator('h1')).toHaveText('Ship size');

	// A page written for this app, not ported from the old one.
	await page.getByRole('link', { name: 'Unmapped holes' }).click();
	await expect(article).toContainText('scanned wormhole that leads nowhere yet');

	// And an unknown page is a 404, not a blank shell.
	const missing = await page.goto('/documentation/getting-started/nope');
	expect(missing?.status()).toBe(404);
});

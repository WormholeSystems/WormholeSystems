import { expect, test } from '@playwright/test';

// Anonymous tests: no session cookie (uses the plain Playwright test, not the fixture).

test('home page renders', async ({ page }) => {
	await page.goto('/');
	await expect(page.getByRole('heading', { level: 1 })).toContainText('Map the chain');
	await expect(page.getByRole('link', { name: 'Open your maps' }).first()).toBeVisible();
});

test('maps requires login', async ({ page }) => {
	await page.goto('/maps');
	await expect(page).toHaveURL('/login?return_to=%2Fmaps');
	// And signing in from there comes back to the list.
	await expect(page.getByRole('link', { name: 'Log in with EVE Online' })).toHaveAttribute(
		'href',
		'/auth/login?return_to=%2Fmaps',
	);
});

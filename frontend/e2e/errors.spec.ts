import { expect, gotoApp, test } from './fixtures';
import { createMap } from './arrange';
import { createIdentity, deleteMapRow } from './db';

// The error pages: each failure ends on the page that says what happened, with the status
// that goes with it, inside the app's own frame.

test('an unknown address is a 404 inside the app', async ({ page }) => {
	const response = await page.goto('/no-such-page');
	expect(response?.status()).toBe(404);

	const error = page.getByTestId('error-page');
	await expect(error).toHaveAttribute('data-status', '404');
	await expect(page.getByTestId('error-title')).toHaveText('There is nothing at this address.');
	await expect(page.getByTestId('error-status')).toHaveText(/Error 404/i);
	// The top bar is still there to go somewhere else from.
	await expect(page.getByRole('navigation')).toBeVisible();
	await expect(page).toHaveTitle(/Error 404/);
});

// The API answers "not found" for a map the caller may not see, deliberately, so a private
// map never confirms it exists. The page says both possibilities and what to do.
test("somebody else's map says it is not open to you, and who to ask", async ({ page }) => {
	const stranger = await createIdentity(41);
	const res = await page.request.post('http://127.0.0.1:3000/api/maps', {
		headers: { cookie: `ws_session=${stranger.session}` },
		data: { name: 'E2E Not Yours' },
	});
	const { id } = (await res.json()) as { id: number };

	const response = await page.goto(`/maps/${id}`);
	expect(response?.status()).toBe(404);
	await expect(page.getByTestId('error-page')).toHaveAttribute('data-status', '404');
	await expect(page.getByTestId('error-title')).toHaveText('This map is not open to you.');
	await expect(page.getByText(/Ask the map's owner/)).toBeVisible();
	await expect(page.getByRole('link', { name: 'Your maps' })).toHaveAttribute('href', '/maps');

	// Its settings say the same rather than crashing.
	const settings = await page.goto(`/maps/${id}/settings`);
	expect(settings?.status()).toBe(404);
	await expect(page.getByTestId('error-title')).toHaveText('This map is not open to you.');

	await deleteMapRow(id);
});

test('a deleted map is a 404, not a 500', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Deleted');
	await api.delete(`/api/maps/${mapId}`);

	const response = await page.goto(`/maps/${mapId}`);
	expect(response?.status()).toBe(404);
	await expect(page.getByTestId('error-title')).toHaveText('This map is not open to you.');
});

test('signed out, a private map asks you to sign in and come back', async ({ browser, api }) => {
	const mapId = await createMap(api, 'E2E Signed Out');
	const guest = await browser.newContext();
	const page = await guest.newPage();

	const response = await page.goto(`http://localhost:5173/maps/${mapId}`);
	expect(response?.status()).toBe(401);
	await expect(page.getByTestId('error-title')).toHaveText('Sign in to open this map.');
	await expect(page.getByRole('link', { name: 'Log in with EVE Online' })).toHaveAttribute(
		'href',
		`/auth/login?return_to=${encodeURIComponent(`/maps/${mapId}`)}`,
	);

	await guest.close();
	await api.delete(`/api/maps/${mapId}`);
});

test('a share link that leads nowhere says so', async ({ page }) => {
	await gotoApp(page, '/');
	const response = await page.goto('/share/not-a-real-token');
	expect(response?.status()).toBe(404);
	await expect(page.getByTestId('error-title')).toHaveText('This share link no longer works.');
});

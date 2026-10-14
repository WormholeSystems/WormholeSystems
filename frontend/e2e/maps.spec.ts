import { expect, gotoApp, test } from './fixtures';

test('nav shows the signed-in character', async ({ page }) => {
	await gotoApp(page, '/');
	await page.getByLabel('Account').click();
	await expect(page.getByText('E2E Pilot')).toBeVisible();
	await expect(page.getByText('Log out')).toBeVisible();
});

test('create a map, then delete it from the list', async ({ page }) => {
	await gotoApp(page, '/maps');

	await page.getByTestId('new-map').click();
	await page.getByTestId('new-map-name').fill('E2E List Map');
	await page.getByTestId('new-map-description').fill('Created by the suite');
	await page.getByTestId('new-map-create').click();

	// Creating a map is how you say you want to use it, so it opens.
	await page.waitForURL(/\/maps\/\d+/);
	await page.waitForSelector('[data-testid="panel-grid"]');

	await gotoApp(page, '/maps');
	const card = page.getByTestId('map-card').filter({ hasText: 'E2E List Map' });
	await expect(card).toBeVisible();
	await expect(card).toContainText('Created by the suite');
	await expect(card.getByText('owner')).toBeVisible();

	// Deleting asks first: the old list did it on one unguarded click.
	await card.getByTestId('map-menu').click();
	await page.getByTestId('map-delete').click();
	await expect(page.getByTestId('confirm-dialog')).toContainText('E2E List Map');
	await page.getByTestId('confirm-accept').click();
	await expect(card).toHaveCount(0);
});

test('archiving hides a map without losing it', async ({ page, api }) => {
	// Named per run: the list is filtered by name, and a leftover from an interrupted run
	// would otherwise match twice and fail this one.
	const name = `E2E Archive ${Date.now()}`;
	const res = await api.post('/api/maps', { data: { name } });
	expect(res.ok()).toBe(true);

	await gotoApp(page, '/maps');
	const card = page.getByTestId('map-card').filter({ hasText: name });
	await expect(card).toBeVisible();

	await card.getByTestId('map-menu').click();
	await page.getByTestId('map-archive').click();
	await expect(page.getByTestId('map-card').filter({ hasText: name })).toHaveCount(0);

	// Still there, just out of the way, and it survives a reload because it is stored.
	await page.reload();
	// The toggle is in the server-rendered page before it answers clicks.
	await page.waitForSelector('html[data-hydrated="true"]');
	await page.getByTestId('toggle-archived').click();
	const archived = page.getByTestId('map-card').filter({ hasText: name });
	await expect(archived).toBeVisible();

	await archived.getByTestId('map-menu').click();
	await page.getByTestId('map-archive').click();
	await expect(page.getByTestId('map-card').filter({ hasText: name })).toBeVisible();

	// Clean up: this map is not scoped to one test's map id.
	await api.delete(`/api/maps/${(await res.json()).id}`);
});

test('search narrows the list by name and description', async ({ page, api }) => {
	const a = await (await api.post('/api/maps', { data: { name: 'E2E Alpha Chain' } })).json();
	const b = await (
		await api.post('/api/maps', { data: { name: 'E2E Beta', description: 'staging only' } })
	).json();

	await gotoApp(page, '/maps');
	await expect(page.getByTestId('map-card').filter({ hasText: 'E2E Alpha Chain' })).toBeVisible();

	await page.getByTestId('map-search').fill('alpha');
	await expect(page.getByTestId('map-card').filter({ hasText: 'E2E Alpha Chain' })).toBeVisible();
	await expect(page.getByTestId('map-card').filter({ hasText: 'E2E Beta' })).toHaveCount(0);

	// The description is searched too, which is most of why it is worth having.
	await page.getByTestId('map-search').fill('staging only');
	await expect(page.getByTestId('map-card').filter({ hasText: 'E2E Beta' })).toBeVisible();

	await api.delete(`/api/maps/${a.id}`);
	await api.delete(`/api/maps/${b.id}`);
});

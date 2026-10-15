import { expect, gotoApp, test } from './fixtures';

// The landing page: what someone sees before they have any reason to trust this.

test('the landing page states what it is and how to run it', async ({ page }) => {
	await gotoApp(page, '/');

	await expect(page.getByRole('heading', { level: 1 })).toContainText('Map the chain');

	// The numbers are this install's own reference tables, not decoration, so they have to
	// be real ones rather than the zeroes an unreachable API falls back to.
	const stats = page.getByTestId('landing-stat');
	await expect(stats).toHaveCount(4);
	for (const stat of await stats.all()) {
		await expect(stat).toHaveText(/^[1-9][\d,]*\s/);
	}

	// The install command is the point of the self-host section, so it is real text on the
	// page rather than an image someone has to retype from.
	await expect(page.getByTestId('copy-command')).toBeVisible();
	await expect(page.getByText('install.wormhole.systems').first()).toBeVisible();
	await expect(page.getByText('wsctl setup').first()).toBeVisible();

	await page.getByRole('link', { name: 'Open your maps' }).first().click();
	await expect(page).toHaveURL(/\/maps$/);
});

// Sections fade in on scroll. Visible has to be the resting state: hidden-until-observed
// means a section is missing outright for anyone whose observer never runs.
test('every section is present without scrolling', async ({ page }) => {
	await gotoApp(page, '/');

	for (const heading of ['Run it yourself. It is one command.', 'Ready to map the void?']) {
		await expect(page.getByRole('heading', { name: heading })).toBeAttached();
	}
	const cta = page.getByRole('heading', { name: 'Ready to map the void?' });
	await cta.scrollIntoViewIfNeeded();
	await expect(cta).toBeVisible();
});

// The access roles are the app's own strings, so the page cannot drift from the settings
// screen that grants them.
test('the access section lists every role', async ({ page }) => {
	await gotoApp(page, '/');
	for (const role of ['Viewer', 'Member', 'Manager', 'Owner']) {
		await expect(page.getByText(role, { exact: true }).first()).toBeAttached();
	}
});

test('the page does not scroll sideways on a phone', async ({ page }) => {
	await page.setViewportSize({ width: 390, height: 844 });
	await gotoApp(page, '/');
	await page.getByTestId('landing-stat').first().waitFor();

	const overflows = await page.evaluate(
		() => document.documentElement.scrollWidth > document.documentElement.clientWidth,
	);
	expect(overflows).toBe(false);
});

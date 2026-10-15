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

// The screenshots come in both themes and follow the toggle, not the OS, with only the
// one on screen ever downloaded.
test('the screenshots follow the theme toggle', async ({ page }) => {
	await page.addInitScript(() => localStorage.setItem('darkmode', 'true'));
	const fetched: string[] = [];
	page.on('request', (req) => {
		const shot = /\/landing\/(.+\.webp)$/.exec(req.url());
		if (shot) fetched.push(shot[1]);
	});
	await gotoApp(page, '/');

	const hero = page.locator('img[src^="/landing/app-"]');
	await expect(hero.locator('visible=true')).toHaveAttribute('src', '/landing/app-dark.webp');
	await expect.poll(() => fetched).toContain('app-dark.webp');
	expect(fetched).not.toContain('app-light.webp');

	await page.getByRole('button', { name: 'Toggle theme' }).click();
	await expect(hero.locator('visible=true')).toHaveAttribute('src', '/landing/app-light.webp');
	await expect.poll(() => fetched).toContain('app-light.webp');
	// Every shot has its size in the markup, so nothing jumps as they arrive.
	for (const img of await page.locator('img[src^="/landing/"]').all()) {
		await expect(img).toHaveAttribute('width', /^\d+$/);
		await expect(img).toHaveAttribute('height', /^\d+$/);
	}
});

// The one live part: the signature panel's own rows, diffing a scan on the spot.
test('the signature demo diffs a pasted scan', async ({ page }) => {
	await gotoApp(page, '/');
	const demo = page.getByTestId('signature-demo');
	await demo.scrollIntoViewIfNeeded();
	await expect(demo.getByTestId('sig-row')).toHaveCount(5);

	await page.getByTestId('signature-demo-paste').click();
	await expect(demo.locator('[data-testid="sig-row"][data-status="new"]')).toHaveCount(2);
	await expect(demo.locator('[data-testid="sig-row"][data-status="deleted"]')).toHaveCount(1);
	await expect(demo).toContainText('2 new');

	// A scan of the visitor's own goes through the same parser.
	await page.getByRole('button', { name: 'Reset' }).click();
	await demo.evaluate((el) => {
		const dt = new DataTransfer();
		dt.setData('text/plain', 'XYZ-111\tCosmic Signature\tCombat Site\tCore Garrison\t100.0%\t3 AU');
		el.dispatchEvent(new ClipboardEvent('paste', { clipboardData: dt, bubbles: true }));
	});
	await expect(demo.locator('[data-testid="sig-row"][data-status="new"]')).toHaveCount(1);
	await expect(demo.locator('[data-testid="sig-row"][data-status="deleted"]')).toHaveCount(5);
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

import { expect, reloadApp, test } from './fixtures';
import { createMap } from './arrange';
import { createIdentity, grantAccess, setScopes, showIntroduction } from './db';

// The one-time walkthrough a map opens with: ESI permissions, the preferences those
// permissions unlock, and a summary.
//
// Driven as its own identity throughout. Every other spec has the walkthrough skipped for
// it, and this one needs to control exactly which scopes the character holds — which is
// not something to do to the identity the rest of the suite shares.

const LOCATION = 'esi-location.read_location.v1';
const WAYPOINT = 'esi-ui.write_waypoint.v1';
const ALL_SCOPES = [
	LOCATION,
	'esi-location.read_online.v1',
	'esi-location.read_ship_type.v1',
	WAYPOINT,
];

type Api = import('@playwright/test').APIRequestContext;
type Browser = import('@playwright/test').Browser;

/** The settings write the dialog makes on its way out; a reload would cancel it in flight. */
function saved(page: import('@playwright/test').Page) {
	return page.waitForResponse(
		(r) => r.url().includes('/settings/user') && r.request().method() === 'POST',
	);
}

/** A map the given identity is a member of, with the walkthrough still to come. */
async function openAsNewcomer(
	browser: Browser,
	api: Api,
	name: string,
	slot: number,
	scopes: string[],
) {
	const mapId = await createMap(api, name);
	// A slot per test: `setScopes` replaces the character's tokens outright, and the specs
	// run in parallel.
	const identity = await createIdentity(slot);
	await setScopes(identity.characterId, scopes);
	await grantAccess(mapId, identity.characterId, 'member');
	await showIntroduction(mapId, identity.characterId);

	const ctx = await browser.newContext();
	await ctx.addCookies([
		{ name: 'ws_session', value: identity.session, domain: 'localhost', path: '/' },
	]);
	const page = await ctx.newPage();
	await page.goto(`http://localhost:5173/maps/${mapId}`);
	await page.waitForSelector('html[data-hydrated="true"]');
	return { page, ctx, mapId, identity };
}

test('a new map walks you through permissions and preferences, once', async ({ browser, api }) => {
	const { page, ctx } = await openAsNewcomer(browser, api, 'E2E Intro', 31, ALL_SCOPES);

	const dialog = page.getByTestId('introduction');
	await expect(dialog).toBeVisible();

	// Step 1 lists every permission the app can use. This character holds all of them, so
	// each row reports itself granted rather than linking out to the SSO.
	await expect(dialog).toContainText('Grant permissions');
	await expect(dialog.locator('[data-scope]')).toHaveCount(4);
	await expect(dialog.getByTestId('scope-granted')).toHaveCount(4);

	// Step 2 is the settings those permissions unlock. Location sharing is off on a new
	// map, and the two that depend on it stay disabled until it is on.
	await dialog.getByTestId('introduction-next').click();
	await expect(dialog).toContainText('Choose what it does');
	const prompt = dialog.locator('[data-setting="prompt_for_signature"]');
	await expect(prompt.getByRole('switch')).toBeDisabled();

	await dialog.locator('[data-setting="tracking_allowed"]').getByRole('switch').click();
	await expect(prompt.getByRole('switch')).toBeEnabled();

	// The finale reports where it all ended up.
	await dialog.getByTestId('introduction-next').click();
	await expect(dialog).toContainText('Ready to fly');
	await expect(dialog).toContainText('All granted');

	await Promise.all([saved(page), dialog.getByTestId('introduction-done').click()]);
	await expect(dialog).toHaveCount(0);

	// And it stays gone: a walkthrough, not a greeting.
	await reloadApp(page);
	await expect(page.getByTestId('introduction')).toHaveCount(0);
	await ctx.close();
});

test('missing permissions are offered, and the settings that need them are held back', async ({
	browser,
	api,
}) => {
	// Everything except the waypoint scope, which nothing on the settings step depends on.
	const scopes = ALL_SCOPES.filter((s) => s !== WAYPOINT);
	const { page, ctx } = await openAsNewcomer(browser, api, 'E2E IntroPartial', 32, scopes);

	const dialog = page.getByTestId('introduction');
	await expect(dialog.getByTestId('scope-granted')).toHaveCount(3);

	// The missing one links to consent for itself *plus* everything already granted: SSO
	// reissues the token wholesale, so asking for one alone would revoke the other three.
	const grant = dialog.locator(`[data-scope="${WAYPOINT}"]`).getByRole('link', { name: 'Grant' });
	const href = await grant.getAttribute('href');
	expect(href).toContain(encodeURIComponent(WAYPOINT));
	expect(href).toContain('return_to=');

	await dialog.getByTestId('introduction-next').click();
	const tracking = dialog.locator('[data-setting="tracking_allowed"]');
	await expect(tracking.getByRole('switch')).toBeEnabled();
	await ctx.close();
});

test('without the location scope, sharing cannot be turned on at all', async ({ browser, api }) => {
	const { page, ctx } = await openAsNewcomer(browser, api, 'E2E IntroNoLoc', 33, [WAYPOINT]);

	const dialog = page.getByTestId('introduction');
	await expect(dialog.getByTestId('scope-granted')).toHaveCount(1);

	await dialog.getByTestId('introduction-next').click();
	const tracking = dialog.locator('[data-setting="tracking_allowed"]');
	await expect(tracking.getByRole('switch')).toBeDisabled();
	await expect(tracking).toContainText('Needs the character location permission');

	// Reaching the end still counts as done, even with nothing granted and nothing enabled.
	await dialog.getByTestId('introduction-next').click();
	await Promise.all([saved(page), dialog.getByTestId('introduction-done').click()]);
	await reloadApp(page);
	await expect(page.getByTestId('introduction')).toHaveCount(0);
	await ctx.close();
});

test('back steps return through the walkthrough', async ({ browser, api }) => {
	const { page, ctx } = await openAsNewcomer(browser, api, 'E2E IntroBack', 34, ALL_SCOPES);

	const dialog = page.getByTestId('introduction');
	await expect(dialog).toContainText('Grant permissions');
	await expect(dialog.getByTestId('introduction-back')).toBeDisabled();
	await dialog.getByTestId('introduction-next').click();
	await expect(dialog).toContainText('Choose what it does');
	await dialog.getByTestId('introduction-back').click();
	await expect(dialog).toContainText('Grant permissions');
	await expect(dialog.getByTestId('introduction-back')).toBeDisabled();
	await ctx.close();
});

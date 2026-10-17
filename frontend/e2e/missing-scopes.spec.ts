import { expect, test } from './fixtures';
import { createIdentity, setScopes } from './db';

// A character short of an ESI permission does less without saying so, so the portrait in
// the navigation bar carries a warning until it is granted.
//
// As its own identity: `setScopes` replaces the character's tokens, which is not something
// to do to the account the rest of the suite shares.

const ALL_SCOPES = [
	'esi-location.read_location.v1',
	'esi-location.read_online.v1',
	'esi-location.read_ship_type.v1',
	'esi-ui.write_waypoint.v1',
];

async function signedInAs(browser: import('@playwright/test').Browser, session: string) {
	const ctx = await browser.newContext();
	await ctx.addCookies([{ name: 'ws_session', value: session, domain: 'localhost', path: '/' }]);
	const page = await ctx.newPage();
	return { ctx, page };
}

test('a character missing permissions is flagged on the portrait and in the menu', async ({
	browser,
}) => {
	const identity = await createIdentity(46);
	// Location and ship type unticked at EVE's consent screen.
	await setScopes(identity.characterId, [
		'esi-location.read_online.v1',
		'esi-ui.write_waypoint.v1',
	]);
	const { ctx, page } = await signedInAs(browser, identity.session);

	await page.goto('http://localhost:5173/maps');
	await page.waitForSelector('html[data-hydrated="true"]');
	const warning = page.getByTestId('missing-scopes-warning');
	await expect(warning).toBeVisible();

	await page.getByRole('button', { name: /missing permissions/ }).click();
	const flag = page.getByTestId('character-missing-scopes');
	await flag.hover();
	await expect(page.locator('[data-slot="tooltip-content"]')).toContainText(
		/Character location, Ship type/,
	);
	const grant = page.getByTestId('grant-missing-scopes');
	await expect(grant).toHaveAttribute('href', '/settings/characters');

	// Everything granted: nothing to warn about.
	await setScopes(identity.characterId, ALL_SCOPES);
	await page.goto('http://localhost:5173/maps');
	await page.waitForSelector('html[data-hydrated="true"]');
	await expect(page.getByRole('button', { name: 'Account' })).toBeVisible();
	await expect(warning).toHaveCount(0);

	await ctx.close();
});

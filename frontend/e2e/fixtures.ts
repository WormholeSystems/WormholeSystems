import { test as base, type APIRequestContext, type Page } from '@playwright/test';

import { E2E_SESSION, OWN_ACCOUNT_SLOT, createIdentity, type Identity } from './db';

const API_BASE = 'http://127.0.0.1:3000';

// All tests run as the e2e identity created in global-setup: the session cookie is set on
// the browser context, and `api` is a request context carrying the same cookie for
// arranging state directly against the backend.
export const test = base.extend<{ api: APIRequestContext }>({
	context: async ({ context }, use) => {
		await context.addCookies([
			{ name: 'ws_session', value: E2E_SESSION, domain: 'localhost', path: '/' },
		]);
		await use(context);
	},
	api: async ({ playwright }, use) => {
		const api = await playwright.request.newContext({
			baseURL: API_BASE,
			extraHTTPHeaders: { cookie: `ws_session=${E2E_SESSION}` },
		});
		await use(api);
		await api.dispose();
	},
});

/**
 * For specs that change the account itself (its characters, which one is preferred, or
 * whether it is online): every worker signs in as an identity of its own, so a spec in
 * another worker never sees the change halfway through. `account` says which one.
 */
export const ownAccountTest = base.extend<{ api: APIRequestContext }, { account: Identity }>({
	account: [
		// oxlint-disable-next-line no-empty-pattern -- Playwright reads the fixtures from it.
		async ({}, use, workerInfo) => {
			await use(await createIdentity(OWN_ACCOUNT_SLOT + workerInfo.parallelIndex));
		},
		{ scope: 'worker' },
	],
	context: async ({ context, account }, use) => {
		await context.addCookies([
			{ name: 'ws_session', value: account.session, domain: 'localhost', path: '/' },
		]);
		await use(context);
	},
	api: async ({ playwright, account }, use) => {
		const api = await playwright.request.newContext({
			baseURL: API_BASE,
			extraHTTPHeaders: { cookie: `ws_session=${account.session}` },
		});
		await use(api);
		await api.dispose();
	},
});

export { expect } from '@playwright/test';

/** The session the page's browser context signs in with. */
async function pageSession(page: Page): Promise<string> {
	const cookies = await page.context().cookies('http://localhost:5173');
	return cookies.find((c) => c.name === 'ws_session')?.value ?? E2E_SESSION;
}

/**
 * Hydrated, and on a map page past the loader: it holds until the graph and the panel
 * arrangement have both arrived, or says why the map cannot be shown.
 */
async function settled(page: Page) {
	await page.waitForSelector('html[data-hydrated="true"]');
	if (/\/maps\/\d+$/.test(new URL(page.url()).pathname)) {
		await page.waitForSelector(
			'[data-testid="panel-grid"], [data-testid="map-error"], [data-testid="error-page"]',
		);
	}
}

/**
 * Navigate and wait until the page is hydrated (controls are interactive).
 *
 * A map you have not been through the introduction on opens it as a modal over everything,
 * so this marks it done first, for whoever the page is signed in as. Pass
 * `{ introduction: true }` to leave it alone, which is what the introduction's own spec does.
 */
export async function gotoApp(page: Page, path: string, options: { introduction?: boolean } = {}) {
	const map = /\/maps\/(\d+)(\?|$)/.exec(path);
	if (map && !options.introduction) {
		// The API is a different host from the app, so the context cookie is not sent with it.
		await page.request.post(`${API_BASE}/api/maps/${map[1]}/settings/user`, {
			headers: { cookie: `ws_session=${await pageSession(page)}` },
			data: { introduction_confirmed: true },
		});
	}
	await page.goto(path);
	await settled(page);
}

/**
 * Reload and wait the same way `gotoApp` does: the server-rendered page is on screen before
 * it answers clicks, and a map before its panels are arranged.
 */
export async function reloadApp(page: Page) {
	await page.reload();
	await settled(page);
}

/**
 * The id a create endpoint answers with.
 *
 * Every arrangement step in these tests posts something and needs the id back, and a
 * response body is untyped by nature. Reading it in one place means one checked claim
 * rather than sixty unchecked ones, and a body that is not what we expect fails here
 * naming the response instead of somewhere later as a mysterious undefined.
 */
export async function createdId(response: import('@playwright/test').APIResponse): Promise<number> {
	const body: unknown = await response.json();
	if (typeof body !== 'object' || body === null || !('id' in body)) {
		throw new Error(`expected a body with an id, got ${JSON.stringify(body)}`);
	}
	// SAFETY: `id` is present, and checked to be a number, immediately above.
	const { id } = body as { id: unknown };
	if (typeof id !== 'number') {
		throw new Error(`expected a numeric id, got ${JSON.stringify(id)}`);
	}
	return id;
}

import { createdId, expect, gotoApp, reloadApp, test } from './fixtures';
import { addSystem, createMap, holds } from './arrange';
import {
	createIdentity,
	grantAccess,
	grantLocationScopes,
	setCharacterOnline,
	markUserActive,
	setCharacterPresence,
	withDb,
} from './db';

// Everything that needs the API's own ESI poller to run: Tranquility's status, the gate it
// puts on ESI-backed work, and a jump detected end to end.
//
// These drive process-global state, so they live in their own Playwright project that runs
// after the rest of the suite. Sharing a worker with other specs would mean taking the
// server down underneath them.
//
// They also need the API pointed at the stub (ESI_BASE_URL). Against a dev stack still on
// the real ESI there is nothing to drive, so they skip rather than fail.

const STUB = 'http://127.0.0.1:3999';
const J122515 = 31001882;
const J005482 = 31002515;
const JITA = 30000142;

type Playwright = typeof import('@playwright/test');
type Api = import('@playwright/test').APIRequestContext;

async function setServer(
	playwright: Playwright,
	state: { players?: number; vip?: boolean; unreachable?: boolean } | null,
) {
	const ctx = await playwright.request.newContext();
	const res = state
		? await ctx.put(`${STUB}/_stub/server`, { data: state })
		: await ctx.delete(`${STUB}/_stub/server`);
	expect(res.ok()).toBe(true);
	await ctx.dispose();
}

/** What the API believes, which only its own poller can set. */
async function apiStatus(api: Api) {
	const res = await api.get('/api/server-status');
	return res.ok()
		? ((await res.json()) as { state: string; players: number })
		: { state: 'unreachable', players: 0 };
}

async function apiState(api: Api) {
	return (await apiStatus(api)).state;
}

/**
 * Whether the API is wired to the stub, so these tests can bow out politely against a dev
 * stack still on the real ESI.
 *
 * It waits for an implausible headcount rather than for `online`: Tranquility is usually
 * up, so "the API says online" would be satisfied by reality and the guard would wave
 * through tests that cannot possibly pass.
 */
const WIRED_MARKER = 12_345;

async function stubIsWired(api: Api, playwright: Playwright) {
	await setServer(playwright, { players: WIRED_MARKER });
	const deadline = Date.now() + 15_000;
	while (Date.now() < deadline) {
		if ((await apiStatus(api)).players === WIRED_MARKER) return true;
		await new Promise((resolve) => setTimeout(resolve, 500));
	}
	return false;
}

const SKIP_REASON =
	'the API is not pointed at the ESI stub — restart it with ESI_BASE_URL=http://127.0.0.1:3999';

test.afterEach(async ({ playwright }) => {
	// Hand the real ESI back, so a later spec is not left in a fake downtime.
	await setServer(playwright, null);
});

test('the header reports the server, and says so when it is down', async ({
	page,
	api,
	playwright,
}) => {
	test.skip(!(await stubIsWired(api, playwright)), SKIP_REASON);

	await setServer(playwright, { players: 24_512 });
	// Wait for the count, not just the state: the guard above already left it `online`, so
	// polling on the state alone would pass before the new figure had landed.
	await expect.poll(async () => (await apiStatus(api)).players, { timeout: 15_000 }).toBe(24_512);

	await gotoApp(page, '/maps');
	const indicator = page.getByTestId('server-status');
	await expect(indicator).toHaveAttribute('data-state', 'online');
	// Compact, because the exact figure is tooltip material and the header is not.
	await expect(indicator).toContainText('24.5K');

	// Downtime: ESI still answers, but nobody is playing.
	await setServer(playwright, { players: 0 });
	await expect(indicator).toHaveAttribute('data-state', 'offline', { timeout: 30_000 });
	await expect(indicator).toContainText('Down');

	// VIP is its own state: the server is up and still unusable.
	await setServer(playwright, { players: 40, vip: true });
	await expect(indicator).toHaveAttribute('data-state', 'vip', { timeout: 30_000 });
	await expect(indicator).toContainText('VIP');

	// ESI itself failing is not the same as knowing the server is down.
	await setServer(playwright, { unreachable: true });
	await expect(indicator).toHaveAttribute('data-state', 'unreachable', { timeout: 30_000 });
	await expect(indicator).toContainText('ESI down');
});

test('nothing is polled while the server is down, and it resumes when it returns', async ({
	api,
	browser,
	playwright,
}) => {
	test.skip(!(await stubIsWired(api, playwright)), SKIP_REASON);

	const mapId = await (async () => {
		const res = await api.post('/api/maps', { data: { name: 'E2E ServerGate' } });
		return await createdId(res);
	})();
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: J122515, x: 200, y: 200, alias: null },
	});

	const identity = await createIdentity(13);
	await grantAccess(mapId, identity.characterId, 'member');
	await grantLocationScopes(identity.characterId);
	await setCharacterOnline(identity.characterId);

	const pilotApi = await playwright.request.newContext({
		baseURL: 'http://127.0.0.1:3000',
		extraHTTPHeaders: { cookie: `ws_session=${identity.session}` },
	});
	await pilotApi.post(`/api/maps/${mapId}/settings/user`, {
		data: { tracking_allowed: true, prompt_for_signature: false },
	});

	const position = async () => {
		const mine = (await (await pilotApi.get('/api/me/characters')).json()) as {
			character_id: number;
			solar_system_id: number | null;
		}[];
		return mine.find((c) => c.character_id === identity.characterId)?.solar_system_id ?? null;
	};

	const ctx = await browser.newContext();
	await ctx.addCookies([
		{ name: 'ws_session', value: identity.session, domain: 'localhost', path: '/' },
	]);
	const page = await ctx.newPage();

	try {
		const stubCtx = await playwright.request.newContext();
		await stubCtx.put(`${STUB}/_stub/characters/${identity.characterId}`, {
			data: { online: true, solar_system_id: J122515 },
		});

		// The page holds the socket open, which is what marks the user active enough to poll.
		await page.goto(`http://localhost:5173/maps/${mapId}?system=${J122515}`);
		await page.waitForSelector('html[data-hydrated="true"]');
		await page.waitForSelector('[data-testid="panel-grid"]');
		await expect.poll(position, { timeout: 20_000 }).toBe(J122515);

		// Take the server down, then move the pilot. The location poller is gated, so the
		// move must not be picked up however long we wait.
		await setServer(playwright, { players: 0 });
		await expect.poll(() => apiState(api), { timeout: 20_000 }).toBe('offline');
		await stubCtx.put(`${STUB}/_stub/characters/${identity.characterId}`, {
			data: { online: true, solar_system_id: J005482 },
		});
		await holds(position, J122515, 8_000, 1_000);

		// Back up, and the same move is picked up without anything else changing.
		await setServer(playwright, { players: 18_000 });
		await expect.poll(position, { timeout: 30_000 }).toBe(J005482);

		await stubCtx.delete(`${STUB}/_stub/characters/${identity.characterId}`);
		await stubCtx.dispose();
	} finally {
		await setCharacterPresence(identity.characterId, J005482, false);
		await pilotApi.dispose();
		await ctx.close();
	}
});

// The same jump, but nothing about it is faked below the client: the API's own poller asks
// the ESI stub where the pilot is, writes `character_status`, publishes to the user channel
// and pushes it down the socket, which is what makes the map notice.

async function stubPilot(
	playwright: Playwright,
	characterId: number,
	patch: { online?: boolean; solar_system_id?: number },
) {
	const ctx = await playwright.request.newContext();
	const res = await ctx.put(`${STUB}/_stub/characters/${characterId}`, { data: patch });
	expect(res.ok()).toBe(true);
	await ctx.dispose();
}

async function stubHits(playwright: Playwright, characterId: number) {
	const ctx = await playwright.request.newContext();
	const res = await ctx.get(`${STUB}/_stub/hits/${characterId}`);
	const body = res.ok() ? await res.json() : { hits: 0 };
	await ctx.dispose();
	return body.hits as number;
}

/** Poll `read` until `done`, returning whether it got there inside the budget. */
async function pollUntil<T>(
	read: () => Promise<T>,
	done: (value: T) => boolean,
	timeoutMs: number,
): Promise<boolean> {
	const deadline = Date.now() + timeoutMs;
	while (Date.now() < deadline) {
		if (done(await read())) return true;
		await new Promise((resolve) => setTimeout(resolve, 500));
	}
	return false;
}

/** Where the API currently believes the pilot is, which only its own poller can set. */
async function polledPosition(api: Api, characterId: number) {
	const mine = (await (await api.get('/api/me/characters')).json()) as {
		character_id: number;
		solar_system_id: number | null;
	}[];
	return mine.find((c) => c.character_id === characterId)?.solar_system_id ?? null;
}

async function paste(page: import('@playwright/test').Page, text: string) {
	await expect(page.getByTestId('signatures-card')).toBeVisible();
	await page.evaluate((t) => {
		const dt = new DataTransfer();
		dt.setData('text/plain', t);
		window.dispatchEvent(new ClipboardEvent('paste', { clipboardData: dt }));
	}, text);
}

async function graph(api: Api, mapId: number) {
	const view = await (await api.get(`/api/maps/${mapId}`)).json();
	return view as {
		systems: { id: number; solar_system_id: number }[];
		connections: { id: number }[];
	};
}

test('a pilot moving shows up on everyone else\u2019s map, not just their own', async ({
	api,
	browser,
	playwright,
}) => {
	test.skip(!(await stubIsWired(api, playwright)), SKIP_REASON);

	const mapId = await createMap(api, 'E2E PilotsLive');
	await addSystem(api, mapId, J122515);
	await addSystem(api, mapId, JITA, { x: 500 });

	// The pilot who flies, and a second member who is only watching.
	const flyer = await createIdentity(15);
	await grantAccess(mapId, flyer.characterId, 'member');
	await grantLocationScopes(flyer.characterId);
	await setCharacterOnline(flyer.characterId);
	await withDb((db) =>
		db.query(
			`insert into map_user_settings (map_id, user_id, tracking_allowed) values ($1, $2, true)
			 on conflict (map_id, user_id) do update set tracking_allowed = true`,
			[mapId, flyer.userId],
		),
	);
	await stubPilot(playwright, flyer.characterId, { online: true, solar_system_id: J122515 });
	// The flyer has their own tab open somewhere; without recent activity the poller would
	// rightly ignore them and there would be nothing to broadcast.
	await markUserActive(flyer.userId);

	const watcher = await createIdentity(16);
	await grantAccess(mapId, watcher.characterId, 'member');
	const ctx = await browser.newContext();
	await ctx.addCookies([
		{ name: 'ws_session', value: watcher.session, domain: 'localhost', path: '/' },
	]);
	const page = await ctx.newPage();

	try {
		await page.goto(`http://localhost:5173/maps/${mapId}?system=${J122515}`);
		await page.waitForSelector('html[data-hydrated="true"]');
		await page.waitForSelector('[data-testid="panel-grid"]');

		const row = page.getByTestId('characters-card').getByTestId('pilot-row');
		// The pilot appears as soon as they are known to be online; where they are follows
		// on the next location poll, so the location needs its own wait.
		await expect(row).toHaveCount(1, { timeout: 25_000 });
		await expect(row).toContainText('J122515', { timeout: 25_000 });

		// The watcher never touches their own tab. The pilot's move reaches them over the
		// map socket, which is the whole point of the map-wide presence event.
		await stubPilot(playwright, flyer.characterId, { online: true, solar_system_id: JITA });
		await expect(row).toContainText('Jita', { timeout: 25_000 });
	} finally {
		await stubPilot(playwright, flyer.characterId, { online: false });
		await setCharacterPresence(flyer.characterId, JITA, false);
		await ctx.close();
	}
});

const JITA_LAVA = 40009116; // Jita's lava planet, so a skyhook can sit next door.
const KAZNA_ICE = 40001246;
const KAZNA = 30000018;

async function setSkyhooks(
	playwright: Playwright,
	rows: { planet_id: number; solar_system_id: number; opens_in_min: number; hours?: number }[],
) {
	const ctx = await playwright.request.newContext();
	const res = await ctx.put(`${STUB}/_stub/skyhooks`, { data: { skyhooks: rows } });
	expect(res.ok()).toBe(true);
	await ctx.dispose();
}

test('skyhook windows are read the way a raider reads them', async ({ page, api, playwright }) => {
	test.skip(!(await stubIsWired(api, playwright)), SKIP_REASON);

	// One open with hours to run, one about to close, one not open yet, and one ice so the
	// tabs have something to separate.
	await setSkyhooks(playwright, [
		{ planet_id: JITA_LAVA, solar_system_id: JITA, opens_in_min: -30, hours: 2 },
		{ planet_id: KAZNA_ICE, solar_system_id: KAZNA, opens_in_min: -115, hours: 2 },
	]);

	const mapId = await createMap(api, 'E2E Skyhooks');
	await addSystem(api, mapId, JITA);

	await expect
		.poll(async () => ((await (await api.get('/api/skyhooks')).json()) as unknown[]).length, {
			timeout: 20_000,
		})
		.toBe(2);

	await gotoApp(page, `/maps/${mapId}?system=${JITA}`);
	const card = page.getByTestId('skyhooks-card');
	await expect(card).toBeVisible();

	// Lava first: the Jita hook is open, and zero jumps from the system in focus.
	const row = card.getByTestId('skyhook-row');
	await expect(row).toHaveCount(1, { timeout: 15_000 });
	await expect(row).toHaveAttribute('data-planet', 'Jita VI');
	await expect(row).toHaveAttribute('data-status', 'open');
	await expect(row.getByTestId('skyhook-jumps')).toHaveText('0j');
	// Opened 30 minutes into a two-hour window, so an hour and a half is left. A minute of
	// slack: the window is stamped when the stub is told about it, and the page renders a
	// few seconds later.
	await expect(row.getByTestId('skyhook-timer')).toHaveText(/^1h (29|30)m$/);

	// A skyhook is a system like any other: right-click reaches the same menu.
	await row.click({ button: 'right' });
	const menu = page.getByRole('menu');
	await expect(menu).toBeVisible();
	await expect(menu.getByText('Set destination')).toBeVisible();
	await page.keyboard.press('Escape');
	await expect(menu).toBeHidden();

	// The ice one has five minutes left, and lives behind the other half of the toggle.
	await card.getByTestId('skyhook-kind-ice').click();
	const ice = card.getByTestId('skyhook-row');
	await expect(ice).toHaveCount(1);
	await expect(ice).toHaveAttribute('data-status', 'closing');
	await expect(ice.getByTestId('skyhook-timer')).toHaveText(/^[45]m$/);

	// A window that has already run out stops being listed at all.
	await setSkyhooks(playwright, [
		{ planet_id: JITA_LAVA, solar_system_id: JITA, opens_in_min: -180, hours: 2 },
	]);
	// Wait for the server to have mirrored it before reloading: the card refetches on mount
	// and then only every five minutes, so reloading too early would just re-read the old
	// list and sit there.
	await expect
		.poll(async () => ((await (await api.get('/api/skyhooks')).json()) as unknown[]).length, {
			timeout: 20_000,
		})
		.toBe(0);
	await reloadApp(page);
	await expect(card.getByTestId('skyhooks-empty')).toBeVisible();
});

test('the poller drives the whole jump, from ESI to the prompt', async ({
	api,
	browser,
	playwright,
}) => {
	const mapId = await createMap(api, 'E2E TrackingLive');
	await addSystem(api, mapId, J122515);

	const identity = await createIdentity(12);
	await grantAccess(mapId, identity.characterId, 'member');
	// A token the poller will actually use, and the online flag the 60s tier-1 poll would
	// have set — tier 2 (every 5s) only looks at characters already known to be online.
	await grantLocationScopes(identity.characterId);
	await setCharacterOnline(identity.characterId);
	await stubPilot(playwright, identity.characterId, {
		online: true,
		solar_system_id: J122515,
	});

	const pilotApi = await playwright.request.newContext({
		baseURL: 'http://127.0.0.1:3000',
		extraHTTPHeaders: { cookie: `ws_session=${identity.session}` },
	});
	// Preselected, so confirming the prompt links the one scanned hole: without it the prompt
	// opens on Unknown (the default since preselection became a setting) and maps it unlinked.
	await pilotApi.post(`/api/maps/${mapId}/settings/user`, {
		data: { tracking_allowed: true, prompt_for_signature: true, preselect_signature: true },
	});

	const ctx = await browser.newContext();
	await ctx.addCookies([
		{ name: 'ws_session', value: identity.session, domain: 'localhost', path: '/' },
	]);
	const page = await ctx.newPage();
	// Opening the map is also what marks the user active: the poller ignores anyone whose
	// last activity is over five minutes old, so without a page open nothing is polled.
	await page.goto(`http://localhost:5173/maps/${mapId}?system=${J122515}`);
	await page.waitForSelector('html[data-hydrated="true"]');
	await page.waitForSelector('[data-testid="panel-grid"]');

	try {
		// An API left over from a dev stack may still be pointed at the real ESI, in which
		// case there is nothing to drive and this test has nothing to say.
		const polled = await pollUntil(
			() => stubHits(playwright, identity.characterId),
			(hits) => hits > 0,
			15_000,
		);
		test.skip(
			!polled,
			'the API is not pointed at the ESI stub — restart it with ESI_BASE_URL=http://127.0.0.1:3999',
		);

		// Wait for the poller's answer rather than assuming a tick has landed: the position
		// starts null, and only the poll can fill it in.
		await expect
			.poll(() => polledPosition(pilotApi, identity.characterId), { timeout: 20_000 })
			.toBe(J122515);

		await paste(page, 'WHX-401\tCosmic Signature\tWormhole\t\t100%\t1 AU');
		await expect(page.getByTestId('sig-row')).toHaveCount(1);

		// Fly. Nothing else is touched: the poller picks the new system up within 5s, the
		// socket pushes it, and the map works out that a jump happened.
		await stubPilot(playwright, identity.characterId, {
			online: true,
			solar_system_id: J005482,
		});

		const dialog = page.getByTestId('tracking-dialog');
		await expect(dialog).toBeVisible({ timeout: 20_000 });
		await expect(dialog.getByTestId('tracking-target')).toHaveText('J005482');
		await dialog.getByTestId('tracking-confirm').click();

		await expect
			.poll(async () => (await graph(api, mapId)).systems.length, { timeout: 10_000 })
			.toBe(2);
		const view = await graph(api, mapId);
		expect(view.connections).toHaveLength(1);
		const sigs = await (await api.get(`/api/maps/${mapId}/signatures`)).json();
		expect(sigs[0].connection_id).toBe(view.connections[0].id);
	} finally {
		await stubPilot(playwright, identity.characterId, { online: false });
		await setCharacterPresence(identity.characterId, J005482, false);
		await pilotApi.dispose();
		await ctx.close();
	}
});

// The landing page's product shots. `npm run screenshots` seeds the demo map (demo-map.ts)
// and writes static/landing/<shot>-<theme>.webp, at twice the CSS size.
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';

import { test, type Browser, type Locator, type Page } from '@playwright/test';

import { LOGOS, PORTRAITS, SYS, clearDemo, seedDemoMap, type DemoMap } from './demo-map';

const OUT = path.resolve(import.meta.dirname, '../../static/landing');
const MANIFEST = path.resolve(import.meta.dirname, '../../src/routes/_landing/shots.json');

type Theme = 'dark' | 'light';
type Item = { i: string; x: number; y: number; w: number; h: number };

const item = (i: string, x: number, y: number, w: number, h: number): Item => ({ i, x, y, w, h });
const desktop = (items: Item[]) => ({ lg: { cols: 10, row_height: 89, items } });

// The whole window: the chain, and the panels somebody looks at all evening around it.
// Ten rows of 89px fill a 1000px window under the two bars exactly.
const HERO_LAYOUT = desktop([
	item('map', 0, 0, 6, 7),
	item('characters', 0, 7, 6, 3),
	item('signatures', 6, 0, 4, 6),
	item('system-info', 6, 6, 4, 4),
]);
/** The hero keeps to the chain, its scans and its people; the rest is for the tour. */
const HERO_HIDDEN = ['navigation', 'threat', 'notes', 'killmails', 'skyhooks', 'evescout'];

// For the single-panel shots: each one sized to what it holds, in a taller window so none
// of them has to scroll into view.
const PANEL_LAYOUT = desktop([
	item('map', 0, 0, 6, 6),
	item('signatures', 6, 0, 4, 6),
	item('navigation', 6, 6, 4, 4),
	item('threat', 0, 6, 2, 2),
	item('notes', 0, 8, 2, 3),
	item('killmails', 2, 6, 3, 5),
	item('characters', 0, 12, 3, 4),
	item('system-info', 3, 12, 3, 4),
]);

const MAP_ONLY_LAYOUT = desktop([
	item('map', 0, 0, 10, 15),
	item('signatures', 0, 15, 5, 4),
	item('navigation', 5, 15, 5, 4),
	item('threat', 0, 19, 3, 3),
	item('notes', 3, 19, 3, 3),
	item('killmails', 6, 19, 4, 3),
	item('characters', 0, 22, 5, 3),
	item('system-info', 5, 22, 5, 3),
]);

const HIDDEN = ['skyhooks', 'evescout'];

const WINDOW = { width: 1600, height: 1000 };
const TALL_WINDOW = { width: 1600, height: 1500 };

/** A plain emblem for a made-up corporation or alliance. */
function emblem(color: string) {
	return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">
		<rect width="64" height="64" fill="#18181b"/>
		<path d="M32 10 L52 22 L52 42 L32 54 L12 42 L12 22 Z" fill="none" stroke="${color}" stroke-width="4"/>
		<circle cx="32" cy="32" r="7" fill="${color}"/>
	</svg>`;
}

async function arrange(demo: DemoMap, layout: object, hidden = HIDDEN, mapId = demo.mapId) {
	await demo.viewer.api.post(`/api/maps/${mapId}/settings/user`, {
		data: { layout_breakpoints: layout, hidden_panels: hidden },
	});
}

/**
 * A signed-in browser on the demo map. Real image-server pictures are swapped for the demo's
 * made-up people, the clock stands still, and toasts and the caret stay out of frame.
 */
async function open(
	browser: Browser,
	demo: DemoMap,
	theme: Theme,
	url: string,
	viewport = WINDOW,
): Promise<Page> {
	const ctx = await browser.newContext({ viewport, deviceScaleFactor: 2, reducedMotion: 'reduce' });
	await ctx.addCookies([
		{ name: 'ws_session', value: demo.viewer.session, domain: 'localhost', path: '/' },
	]);
	await ctx.addInitScript(
		(dark) => localStorage.setItem('darkmode', String(dark)),
		theme === 'dark',
	);
	await ctx.route('https://images.evetech.net/**', async (route) => {
		const u = new URL(route.request().url());
		const [, kind, id] = u.pathname.split('/');
		if (kind === 'characters') {
			const npc = PORTRAITS.get(Number(id)) ?? 1;
			return route.continue({ url: `${u.origin}/characters/${npc}/portrait${u.search}` });
		}
		const color = LOGOS.get(Number(id));
		if ((kind === 'corporations' || kind === 'alliances') && color) {
			return route.fulfill({ contentType: 'image/svg+xml', body: emblem(color) });
		}
		return route.continue();
	});
	const page = await ctx.newPage();
	await page.clock.setFixedTime(new Date());
	await page.goto(url);
	await page.waitForSelector('html[data-hydrated="true"]');
	await page.addStyleTag({
		content: `
			[data-sonner-toaster] { display: none !important; }
			* { caret-color: transparent !important; }
		`,
	});
	return page;
}

async function openMap(
	browser: Browser,
	demo: DemoMap,
	theme: Theme,
	{ system = SYS.home as number, viewport = WINDOW, mapId = demo.mapId } = {},
) {
	const page = await open(browser, demo, theme, `/maps/${mapId}?system=${system}`, viewport);
	await page.waitForSelector('[data-testid="panel-grid"]');
	await page.getByTestId('system-node').first().waitFor();
	return page;
}

/** Let images, fonts and the panels' queries arrive, and leave nothing hovered or focused. */
async function settle(page: Page) {
	await page.waitForLoadState('networkidle').catch(() => {});
	await page.evaluate(async () => {
		await document.fonts.ready;
		(document.activeElement as HTMLElement | null)?.blur();
		window.scrollTo(0, 0);
	});
	await page.mouse.move(0, 0);
	await page.waitForTimeout(500);
}

/** PNG in, WebP out, encoded by the browser itself so no image library is needed. */
async function toWebp(page: Page, png: Buffer, quality: number): Promise<Buffer> {
	const b64 = await page.evaluate(
		async ({ data, quality }) => {
			const bytes = Uint8Array.from(atob(data), (c) => c.charCodeAt(0));
			const bitmap = await createImageBitmap(new Blob([bytes], { type: 'image/png' }));
			const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
			canvas.getContext('2d')!.drawImage(bitmap, 0, 0);
			const blob = await canvas.convertToBlob({ type: 'image/webp', quality });
			const buf = new Uint8Array(await blob.arrayBuffer());
			let s = '';
			for (let i = 0; i < buf.length; i += 0x8000) {
				s += String.fromCharCode(...buf.subarray(i, i + 0x8000));
			}
			return btoa(s);
		},
		{ data: png.toString('base64'), quality },
	);
	return Buffer.from(b64, 'base64');
}

/** The smallest rectangle around all of `parts`, grown by `pad` on every side. */
async function around(parts: Locator[], pad = 0) {
	const boxes = await Promise.all(parts.map((p) => p.boundingBox()));
	if (boxes.some((b) => !b)) throw new Error('a part of the shot is not on screen');
	const real = boxes as { x: number; y: number; width: number; height: number }[];
	const x = Math.max(0, Math.min(...real.map((b) => b.x)) - pad);
	const y = Math.max(0, Math.min(...real.map((b) => b.y)) - pad);
	const right = Math.max(...real.map((b) => b.x + b.width)) + pad;
	const bottom = Math.max(...real.map((b) => b.y + b.height)) + pad;
	return { x, y, width: right - x, height: bottom - y };
}

interface Framing {
	/** Room around the parts, for popovers that should not touch the edge. */
	pad?: number;
	/** Cut the shot off at this element's bottom edge, for panels taller than their rows. */
	until?: Locator;
	/** How far past `until` to keep. */
	slack?: number;
	/** Hide the page behind the parts, so a popover is shot on its own. */
	alone?: boolean;
}

async function save(
	page: Page,
	name: string,
	theme: Theme,
	parts: Locator[],
	{ pad = 0, until, slack = 1, alone = false }: Framing = {},
) {
	await settle(page);
	const clip = await around(parts, pad);
	if (until) {
		const box = await until.boundingBox();
		if (box) clip.height = Math.min(clip.height, box.y + box.height + slack - clip.y);
	}
	const hide = alone
		? await page.addStyleTag({
				content: `body * { visibility: hidden; }
					[data-slot="popover-content"], [data-slot="popover-content"] * { visibility: visible; }`,
			})
		: null;
	const png = await page.screenshot({ animations: 'disabled', clip });
	await hide?.evaluate((el) => el.remove());
	const webp = await toWebp(page, png, 0.82);
	await writeFile(path.join(OUT, `${name}-${theme}.webp`), webp);
	console.log(`${name}-${theme}.webp  ${(webp.length / 1024).toFixed(0)} KB`);

	// The page sets width and height from this, so a reshot never shifts its layout. CSS
	// pixels, read from the PNG header: the shots are taken at twice the size.
	const sizes = JSON.parse(await readFile(MANIFEST, 'utf8').catch(() => '{}'));
	sizes[name] = { width: png.readUInt32BE(16) / 2, height: png.readUInt32BE(20) / 2 };
	const sorted = Object.fromEntries(Object.entries(sizes).sort(([a], [b]) => a.localeCompare(b)));
	await writeFile(MANIFEST, `${JSON.stringify(sorted, null, '\t')}\n`);
}

const tile = (page: Page, panel: string) =>
	page.locator(`[data-testid="panel-tile"][data-panel="${panel}"]`);

/** Home to Jita: out through B's high-sec exit, then the gates. */
async function planRoute(page: Page) {
	const home = page.getByTestId('system-node').filter({ hasText: 'J115405' });
	await home.click({ button: 'right' });
	await page.getByTestId('menu-route').hover();
	await page.getByTestId('route-submenu').getByRole('menuitem', { name: 'Set as origin' }).click();
	await page.getByTestId('system-picker-destination').click();
	await page.getByPlaceholder('Search…').locator('visible=true').fill('Jita');
	await page.getByRole('option', { name: /Jita/ }).first().click();
	await page.getByTestId('route-jumps').waitFor();
}

// The demo shares the e2e database, and its kills and threat sit on systems the suite uses.
test.afterAll(clearDemo);

for (const theme of ['dark', 'light'] as const) {
	test.describe(theme, () => {
		test.describe.configure({ mode: 'serial' });
		let demo: DemoMap;

		test.beforeAll(async () => {
			demo = await seedDemoMap();
			await mkdir(OUT, { recursive: true });
		});

		test('app', async ({ browser }) => {
			await arrange(demo, HERO_LAYOUT, HERO_HIDDEN, demo.heroMapId);
			const page = await openMap(browser, demo, theme, { mapId: demo.heroMapId });
			await save(page, 'app', theme, [page.locator('body')], {
				until: tile(page, 'characters'),
			});
			await page.context().close();
		});

		// A connection's details with its jump log open, as they sit over the map: the map
		// takes the whole window here, so nothing but the chain shows behind them.
		test('connection', async ({ browser }) => {
			await arrange(demo, MAP_ONLY_LAYOUT);
			const page = await openMap(browser, demo, theme, { viewport: TALL_WINDOW });
			await page
				.locator(`[data-testid="edge-hit"][data-connection-id="${demo.connections.homeS1}"]`)
				.click({ force: true });
			const popover = page.getByTestId('connection-popover');
			await popover.waitFor();
			await popover.getByTestId('jump-log-trigger').click();
			const log = page.getByTestId('jump-log');
			await log.waitFor();
			await save(page, 'connection', theme, [popover, log], { pad: 24 });
			await page.context().close();
		});

		test('panels', async ({ browser }) => {
			await arrange(demo, PANEL_LAYOUT);
			let page = await openMap(browser, demo, theme, { viewport: TALL_WINDOW });

			await planRoute(page);
			await save(page, 'route', theme, [tile(page, 'navigation')], {
				until: page.getByTestId('route-list'),
				slack: 12,
			});

			await page.getByTestId('history-button').click();
			const history = page.locator('[data-slot="popover-content"]').filter({
				has: page.getByTestId('history-list'),
			});
			await history.waitFor();
			await save(page, 'history', theme, [history], { alone: true });
			await page.context().close();

			// Next door: who lives there, what they have been killing, and what we know.
			page = await openMap(browser, demo, theme, { system: SYS.s1, viewport: TALL_WINDOW });
			await page.getByTestId('threat-badge').waitFor();
			await save(page, 'intel', theme, [
				tile(page, 'threat'),
				tile(page, 'notes'),
				tile(page, 'killmails'),
			]);
			await page.context().close();
		});

		test('settings', async ({ browser }) => {
			let page = await open(
				browser,
				demo,
				theme,
				`/maps/${demo.mapId}/settings/access`,
				TALL_WINDOW,
			);
			// The grants themselves; the page around them explains the roles in words.
			const grants = page.getByTestId('access-list');
			await grants.getByText('Outer Drift Compact').waitFor();
			await save(page, 'access', theme, [page.getByTestId('access-filter'), grants], { pad: 16 });
			await page.context().close();

			page = await open(browser, demo, theme, `/maps/${demo.mapId}/settings/alerts`, TALL_WINDOW);
			const alerts = page.getByTestId('alert-row');
			await alerts.first().waitFor();
			await save(page, 'alerts', theme, [alerts.first(), alerts.last()], { pad: 16 });
			await page.context().close();
		});
	});
}

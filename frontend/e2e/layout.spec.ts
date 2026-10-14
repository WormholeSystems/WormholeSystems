import { expect, gotoApp, reloadApp, test } from './fixtures';
import { createMap } from './arrange';

// Arranging the map page. The layout is per user per map, so everything here has to
// survive a reload, and the map canvas is a tile like any other.

const J122515 = 31001882;

/** Position and size of a tile, read off the DOM. */
async function box(page: import('@playwright/test').Page, panel: string) {
	const el = page.locator(`[data-testid="panel-tile"][data-panel="${panel}"]`);
	return {
		x: Number(await el.getAttribute('data-x')),
		y: Number(await el.getAttribute('data-y')),
		w: Number(await el.getAttribute('data-w')),
		h: Number(await el.getAttribute('data-h')),
	};
}

/** Drag a tile by whole grid cells using its edit-mode shield. */
async function dragTile(
	page: import('@playwright/test').Page,
	panel: string,
	dx: number,
	dy: number,
) {
	const shield = page.locator(`[data-testid="tile-shield"][data-panel="${panel}"]`);
	// The grid is taller than the window, so a tile can start below the fold; mouse
	// coordinates are viewport-relative.
	await shield.scrollIntoViewIfNeeded();
	const from = await shield.boundingBox();
	if (!from) throw new Error(`no shield for ${panel}`);
	const startX = from.x + Math.min(from.width / 2, 60);
	const startY = from.y + 20;
	await page.mouse.move(startX, startY);
	await page.mouse.down();
	// Two steps so the gesture clears its 4px hysteresis before the real move.
	await page.mouse.move(startX + 8, startY + 8);
	await page.mouse.move(startX + dx, startY + dy, { steps: 8 });
	await page.mouse.up();
	// Tiles preview the gesture while it runs; wait for it to commit before reading
	// positions, or an intermediate step gets mistaken for the result.
	await expect(page.getByTestId('panel-grid')).toHaveAttribute('data-dragging', 'false');
}

test('the map is a tile, and the grid follows the window width', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Grid');
	await page.setViewportSize({ width: 1700, height: 1000 });
	await gotoApp(page, `/maps/${mapId}`);

	await expect(page.getByTestId('panel-grid')).toHaveAttribute('data-breakpoint', 'lg');
	// The canvas is one of the tiles, not a fixed sidebar neighbour.
	await expect(page.locator('[data-testid="panel-tile"][data-panel="map"]')).toBeVisible();
	await expect(page.getByTestId('map-canvas')).toBeVisible();

	// A narrow window switches to a different arrangement.
	await page.setViewportSize({ width: 800, height: 1000 });
	await expect(page.getByTestId('panel-grid')).toHaveAttribute('data-breakpoint', 'sm');
});

test('dragging a tile moves it and the move survives a reload', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Drag');
	await page.setViewportSize({ width: 1700, height: 1000 });
	await gotoApp(page, `/maps/${mapId}`);

	const before = await box(page, 'notes');
	await page.getByTestId('layout-toggle').click();
	await expect(page.getByTestId('layout-toolbar')).toBeVisible();

	// Drag notes a long way left; it should end in a different column.
	await dragTile(page, 'notes', -700, 0);
	const after = await box(page, 'notes');
	expect(after.x).toBeLessThan(before.x);

	await expect(page.getByTestId('layout-save')).toBeEnabled();
	await page.getByTestId('layout-save').click();
	await expect(page.getByTestId('layout-toolbar')).toHaveCount(0);

	await gotoApp(page, `/maps/${mapId}`);
	// Settings arrive after hydration, so this has to be a retrying assertion: reading
	// once can catch the built-in arrangement before the stored one lands.
	await expect(page.locator('[data-testid="panel-tile"][data-panel="notes"]')).toHaveAttribute(
		'data-x',
		String(after.x),
	);
});

test('a held tile follows the cursor while a ghost shows where it lands', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Float');
	await page.setViewportSize({ width: 1700, height: 1300 });
	await gotoApp(page, `/maps/${mapId}`);
	await page.getByTestId('layout-toggle').click();

	const tile = page.locator('[data-testid="panel-tile"][data-panel="navigation"]');
	const shield = page.locator('[data-testid="tile-shield"][data-panel="navigation"]');
	const start = (await shield.boundingBox())!;

	await page.mouse.move(start.x + 100, start.y + 40);
	await page.mouse.down();
	await page.mouse.move(start.x + 110, start.y + 50);
	// Land deliberately off-grid, so a snapped tile and a floating one cannot agree.
	await page.mouse.move(start.x + 100 - 430, start.y + 40 + 170, { steps: 10 });

	const ghost = page.getByTestId('tile-placeholder');
	await expect(ghost).toBeVisible();
	const held = (await tile.boundingBox())!;
	const target = (await ghost.boundingBox())!;
	// The tile tracks the pointer exactly while the ghost is snapped to the grid, so they
	// sit apart by whatever is left over inside the cell.
	expect(Math.round(held.x - target.x)).not.toBe(0);
	expect(held.width).toBeCloseTo(target.width, 0);
	// The cell the ghost is claiming, in grid units, which does not depend on any animation.
	const cell = {
		x: await ghost.getAttribute('data-x'),
		y: await ghost.getAttribute('data-y'),
	};

	// Releasing drops the tile into exactly that cell.
	await page.mouse.up();
	await expect(page.getByTestId('panel-grid')).toHaveAttribute('data-dragging', 'false');
	await expect(ghost).toHaveCount(0);
	await expect(tile).toHaveAttribute('data-x', cell.x!);
	await expect(tile).toHaveAttribute('data-y', cell.y!);
});

test('a resize grows freely while the ghost snaps', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E FloatResize');
	await page.setViewportSize({ width: 1700, height: 1300 });
	await gotoApp(page, `/maps/${mapId}`);
	await page.getByTestId('layout-toggle').click();

	const tile = page.locator('[data-testid="panel-tile"][data-panel="notes"]');
	const handle = page.locator('[data-testid="tile-resize"][data-panel="notes"]');
	await handle.scrollIntoViewIfNeeded();
	const grip = (await handle.boundingBox())!;
	const before = (await tile.boundingBox())!;

	await page.mouse.move(grip.x + 2, grip.y + 2);
	await page.mouse.down();
	await page.mouse.move(grip.x + 12, grip.y + 12);
	await page.mouse.move(grip.x + 2, grip.y + 2 + 130, { steps: 8 });

	const held = (await tile.boundingBox())!;
	const target = (await page.getByTestId('tile-placeholder').boundingBox())!;
	expect(held.height).toBeGreaterThan(before.height);
	// Free height versus a whole number of rows.
	expect(Math.round(held.height - target.height)).not.toBe(0);
	await page.mouse.up();
});

test('arranging never gives the window a horizontal scrollbar', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Overflow');
	await page.setViewportSize({ width: 1700, height: 1000 });
	await gotoApp(page, `/maps/${mapId}`);
	await page.getByTestId('layout-toggle').click();

	const overflows = () =>
		page.evaluate(
			() => document.documentElement.scrollWidth > document.documentElement.clientWidth,
		);
	expect(await overflows()).toBe(false);

	// Drag a tile hard right, well past the edge of the grid.
	const shield = page.locator('[data-testid="tile-shield"][data-panel="notes"]');
	await shield.scrollIntoViewIfNeeded();
	const grip = (await shield.boundingBox())!;
	await page.mouse.move(grip.x + 60, grip.y + 20);
	await page.mouse.down();
	await page.mouse.move(grip.x + 70, grip.y + 30);
	await page.mouse.move(grip.x + 60 + 600, grip.y + 20, { steps: 6 });
	expect(await overflows()).toBe(false);
	await page.mouse.up();
	await expect(page.getByTestId('panel-grid')).toHaveAttribute('data-dragging', 'false');
	expect(await overflows()).toBe(false);

	// And the same while resizing past the right-hand edge.
	const handle = page.locator('[data-testid="tile-resize"][data-panel="notes"]');
	await handle.scrollIntoViewIfNeeded();
	const corner = (await handle.boundingBox())!;
	await page.mouse.move(corner.x + 2, corner.y + 2);
	await page.mouse.down();
	await page.mouse.move(corner.x + 12, corner.y + 12);
	await page.mouse.move(corner.x + 2 + 800, corner.y + 2, { steps: 6 });
	expect(await overflows()).toBe(false);
	await page.mouse.up();
	await expect(page.getByTestId('panel-grid')).toHaveAttribute('data-dragging', 'false');
	expect(await overflows()).toBe(false);
});

test('a tile is not a containing block for the map context menu', async ({ page, api }) => {
	// Tiles are placed with left/top rather than a transform on purpose: a transform makes
	// the tile the containing block for `position: fixed` descendants, which silently threw
	// the map's context menus off by the tile's own offset. Whatever the menu is built from,
	// it has to open exactly where the canvas was clicked.
	const mapId = await createMap(api, 'E2E Fixed');
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: J122515, x: 300, y: 200, alias: null },
	});
	await page.setViewportSize({ width: 1700, height: 1000 });
	await gotoApp(page, `/maps/${mapId}`);

	const canvas = (await page.getByTestId('map-canvas').boundingBox())!;
	const at = { x: Math.round(canvas.x + 400), y: Math.round(canvas.y + 300) };
	await page.mouse.click(at.x, at.y, { button: 'right' });

	const menu = page.getByTestId('context-menu');
	await expect(menu).toBeVisible();
	// Positioned on the next frame, so wait for it to settle on the click.
	await expect
		.poll(async () => {
			const box = (await menu.boundingBox())!;
			return { x: Math.round(box.x), y: Math.round(box.y) };
		})
		.toEqual(at);
});

test('arranging leaves room to drag a tile past the bottom', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Slack');
	await page.setViewportSize({ width: 1700, height: 1000 });
	await gotoApp(page, `/maps/${mapId}`);

	const height = () => page.evaluate(() => document.documentElement.scrollHeight);
	const resting = await height();
	await page.getByTestId('layout-toggle').click();
	// Edit mode adds empty rows below the layout, so a tile already at the bottom still has
	// somewhere to be dragged to.
	expect(await height()).toBeGreaterThan(resting);

	// Leaving puts the page back to the size the content actually needs.
	await page.getByTestId('layout-exit').click();
	await expect(page.getByTestId('layout-toolbar')).toHaveCount(0);
	expect(await height()).toBe(resting);
});

test('arrow keys move a tile without a pointer', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Keyboard');
	await page.setViewportSize({ width: 1700, height: 1000 });
	await gotoApp(page, `/maps/${mapId}`);
	await page.getByTestId('layout-toggle').click();

	const before = await box(page, 'notes');
	await page.locator('[data-testid="tile-shield"][data-panel="notes"]').focus();
	await page.keyboard.press('ArrowLeft');
	expect((await box(page, 'notes')).x).toBe(before.x - 1);

	// Shift resizes instead of moving.
	const width = (await box(page, 'notes')).w;
	await page.keyboard.press('Shift+ArrowRight');
	expect((await box(page, 'notes')).w).toBe(width + 1);
});

test('a resize will not go below the panel minimum', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Minimum');
	await page.setViewportSize({ width: 1700, height: 1000 });
	await gotoApp(page, `/maps/${mapId}`);
	await page.getByTestId('layout-toggle').click();

	const shield = page.locator('[data-testid="tile-shield"][data-panel="notes"]');
	await shield.focus();
	// Shrink far past the minimum; it clamps rather than collapsing.
	for (let i = 0; i < 8; i++) await page.keyboard.press('Shift+ArrowLeft');
	expect((await box(page, 'notes')).w).toBe(2);
});

test('hiding a panel and adding it back from the library', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Hide');
	await page.setViewportSize({ width: 1700, height: 1000 });
	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);
	await page.getByTestId('layout-toggle').click();

	await page.locator('[data-testid="tile-hide"][data-panel="notes"]').click();
	await expect(page.locator('[data-testid="panel-tile"][data-panel="notes"]')).toHaveCount(0);
	await page.getByTestId('layout-save').click();
	// The toolbar closes once the save lands; navigating before that would race it.
	await expect(page.getByTestId('layout-toolbar')).toHaveCount(0);

	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);
	await expect(page.locator('[data-testid="panel-tile"][data-panel="notes"]')).toHaveCount(0);

	// The card library offers it back.
	await page.getByTestId('layout-toggle').click();
	await page.getByTestId('card-library').click();
	await page.getByTestId('add-notes').click();
	await expect(page.locator('[data-testid="panel-tile"][data-panel="notes"]')).toBeVisible();

	// The map can never be hidden; there would be nothing left to look at.
	await expect(page.locator('[data-testid="tile-hide"][data-panel="map"]')).toHaveCount(0);
});

test('hiding a panel in the middle closes the hole it leaves', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Hole');
	await page.setViewportSize({ width: 1700, height: 1300 });
	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);

	// At lg, signatures sits directly under navigation in the right-hand column.
	const before = await box(page, 'signatures');
	expect(before.y).toBeGreaterThan(0);

	await page.getByTestId('layout-toggle').click();
	await page.locator('[data-testid="tile-hide"][data-panel="navigation"]').click();

	// Signatures rises into the space navigation was occupying rather than leaving a gap.
	await expect(page.locator('[data-testid="panel-tile"][data-panel="signatures"]')).toHaveAttribute(
		'data-y',
		'0',
	);

	// Putting it back restores the original arrangement, so the position was remembered.
	await page.getByTestId('card-library').click();
	await page.getByTestId('add-navigation').click();
	await expect(page.locator('[data-testid="panel-tile"][data-panel="navigation"]')).toBeVisible();
});

test('leaving with unsaved changes offers to save, from either control', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E ExitPrompt');
	await page.setViewportSize({ width: 1700, height: 1000 });
	await gotoApp(page, `/maps/${mapId}`);

	const before = await box(page, 'notes');
	await page.getByTestId('layout-toggle').click();
	await dragTile(page, 'notes', -700, 0);
	const moved = await box(page, 'notes');
	expect(moved.x).not.toBe(before.x);

	// The status-bar toggle is the other way out of edit mode, and it must not drop the
	// changes on the floor either.
	await page.getByTestId('layout-toggle').click();
	await expect(page.getByTestId('layout-exit-prompt')).toBeVisible();
	await page.getByRole('button', { name: 'Keep editing' }).click();
	await expect(page.getByTestId('layout-toolbar')).toBeVisible();
	expect(await box(page, 'notes')).toEqual(moved);

	// Saving keeps the arrangement and leaves edit mode.
	await page.getByTestId('layout-exit').click();
	await page.getByTestId('layout-save-exit').click();
	await expect(page.getByTestId('layout-toolbar')).toHaveCount(0);

	await reloadApp(page);
	expect((await box(page, 'notes')).x).toBe(moved.x);
});

test('leaving an untouched layout does not ask', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E NoPrompt');
	await page.setViewportSize({ width: 1700, height: 1000 });
	await gotoApp(page, `/maps/${mapId}`);

	await page.getByTestId('layout-toggle').click();
	await page.getByTestId('layout-toggle').click();
	await expect(page.getByTestId('layout-exit-prompt')).toHaveCount(0);
	await expect(page.getByTestId('layout-toolbar')).toHaveCount(0);
});

test('discarding puts the arrangement back', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Discard');
	await page.setViewportSize({ width: 1700, height: 1000 });
	await gotoApp(page, `/maps/${mapId}`);

	const before = await box(page, 'notes');
	await page.getByTestId('layout-toggle').click();
	await dragTile(page, 'notes', -700, 0);
	expect((await box(page, 'notes')).x).not.toBe(before.x);

	await page.getByTestId('layout-exit').click();
	await page.getByTestId('layout-discard').click();
	await expect(page.getByTestId('layout-toolbar')).toHaveCount(0);
	expect(await box(page, 'notes')).toEqual(before);
});

test('reset returns one breakpoint to the built-in arrangement', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Reset');
	await page.setViewportSize({ width: 1700, height: 1000 });
	await gotoApp(page, `/maps/${mapId}`);

	const before = await box(page, 'notes');
	await page.getByTestId('layout-toggle').click();
	await dragTile(page, 'notes', -700, 0);
	expect((await box(page, 'notes')).x).not.toBe(before.x);

	await page.getByTestId('layout-more').click();
	await page.getByTestId('layout-reset').click();
	expect(await box(page, 'notes')).toEqual(before);
});

test('each breakpoint keeps its own arrangement', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Breakpoints');
	await page.setViewportSize({ width: 1700, height: 1000 });
	await gotoApp(page, `/maps/${mapId}`);
	await page.getByTestId('layout-toggle').click();

	// The desktop arrangement puts the map beside the panels; the phone one stacks.
	await expect(page.getByTestId('panel-grid')).toHaveAttribute('data-breakpoint', 'lg');
	const wide = await box(page, 'map');
	await page.getByTestId('breakpoint-xs').click();
	await expect(page.getByTestId('panel-grid')).toHaveAttribute('data-breakpoint', 'xs');
	const narrow = await box(page, 'map');
	expect(narrow.w).toBeLessThan(wide.w);
});

test('the canvas still pans after the map tile is resized', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Pan');
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: J122515, x: 300, y: 200, alias: null },
	});
	await page.setViewportSize({ width: 1700, height: 1000 });
	await gotoApp(page, `/maps/${mapId}`);

	await page.getByTestId('layout-toggle').click();
	await page.locator('[data-testid="tile-shield"][data-panel="map"]').focus();
	await page.keyboard.press('Shift+ArrowLeft');
	await page.getByTestId('layout-save').click();
	await expect(page.getByTestId('layout-toolbar')).toHaveCount(0);

	// Middle-drag the canvas; the node must move with the pan.
	const node = page.getByTestId('system-node').first();
	const start = await node.boundingBox();
	const canvas = await page.getByTestId('map-canvas').boundingBox();
	if (!start || !canvas) throw new Error('missing geometry');
	await page.mouse.move(canvas.x + canvas.width / 2, canvas.y + canvas.height / 2);
	await page.mouse.down({ button: 'middle' });
	await page.mouse.move(canvas.x + canvas.width / 2 - 120, canvas.y + canvas.height / 2, {
		steps: 8,
	});
	await page.mouse.up({ button: 'middle' });

	const moved = await node.boundingBox();
	expect(moved!.x).toBeLessThan(start.x);
});

// Tiles are placed in percentages of the grid rather than in pixels worked out after the
// container has been measured, so the first painted frame is the arrangement. Sampling every
// animation frame is the only way to see this: by the time an element can be queried, a
// correction has already finished.
test('tiles are painted in place rather than gliding there', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E FirstPaint');
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: J122515, x: 300, y: 200, alias: null },
	});

	await page.addInitScript(() => {
		const seen: number[] = [];
		const w = window as unknown as { __tileX: number[]; __frames: number };
		w.__tileX = seen;
		w.__frames = 0;
		const sample = () => {
			w.__frames += 1;
			const el = document.querySelector('[data-testid="panel-tile"][data-panel="signatures"]');
			if (el) {
				const x = Math.round(el.getBoundingClientRect().x);
				if (seen[seen.length - 1] !== x) seen.push(x);
			}
			requestAnimationFrame(sample);
		};
		requestAnimationFrame(sample);
	});

	await gotoApp(page, `/maps/${mapId}`);
	// Watch a second's worth of frames after load, however long that takes to render.
	await expect
		.poll(() => page.evaluate(() => (window as unknown as { __frames: number }).__frames))
		.toBeGreaterThan(60);
	const positions = await page.evaluate(() => (window as unknown as { __tileX: number[] }).__tileX);
	expect(positions).toHaveLength(1);
});

// The title bar is the first place anyone grabs a card, and in edit mode it sits above the
// drag shield, so it has to start a drag itself.
test('a card can be dragged by its header while arranging', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E HeaderDrag');
	await gotoApp(page, `/maps/${mapId}`);
	await page.getByTestId('layout-toggle').click();
	await expect(page.getByTestId('layout-toolbar')).toBeVisible();

	const before = await box(page, 'notes');
	const header = page.locator('[data-testid="tile-header"][data-panel="notes"]');
	await header.scrollIntoViewIfNeeded();
	const from = await header.boundingBox();
	if (!from) throw new Error('no header');
	await page.mouse.move(from.x + from.width / 2, from.y + from.height / 2);
	await page.mouse.down();
	await page.mouse.move(from.x + from.width / 2 + 8, from.y + from.height / 2 + 8);
	await page.mouse.move(from.x + from.width / 2 + 8, from.y + from.height / 2 + 260, { steps: 8 });
	await page.mouse.up();
	await expect(page.getByTestId('panel-grid')).toHaveAttribute('data-dragging', 'false');

	expect(await box(page, 'notes')).not.toEqual(before);
});

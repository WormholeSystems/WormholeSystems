import { expect, gotoApp, test } from './fixtures';
import { createMap } from './arrange';
import { setThreat } from './db';

// Threat rings on nodes (setting-gated, suppressed when active) and the Threat card.

const J122515 = 31001882; // C5 wormhole

test.afterEach(async () => {
	// Threat lives on the shared wormhole_systems row; reset it for other tests.
	await setThreat(J122515, 'unknown', []);
});

test('threat ring, setting toggle, and threat card', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E Threat');
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: J122515, x: 200, y: 200, alias: null },
	});
	await setThreat(J122515, 'critical', [
		{ id: 99000001, type: 'alliance', name: 'Threat Alliance', kills: 61 },
		{ id: 98000010, type: 'corporation', name: 'Threat Corp', kills: 12 },
	]);

	await gotoApp(page, `/maps/${mapId}`);
	const node = page.getByTestId('system-node').filter({ hasText: 'J122515' });
	await expect(node).toHaveAttribute('data-threat', 'critical');

	// The ring must actually render in the threat-critical color, not a fallback.
	const { expected, shadow } = await node.evaluate((el) => {
		const probe = document.createElement('div');
		probe.style.color = 'var(--color-threat-critical)';
		document.body.appendChild(probe);
		const color = getComputedStyle(probe).color;
		probe.remove();
		return { expected: color, shadow: getComputedStyle(el).boxShadow };
	});
	expect(shadow).toContain(expected);

	// Toggling the setting off removes the ring.
	await page.getByTestId('threat-toggle').click();
	await expect(node).not.toHaveAttribute('data-threat', 'critical');
	await page.getByTestId('threat-toggle').click();
	await expect(node).toHaveAttribute('data-threat', 'critical');

	// Activating the node suppresses the threat ring (amber active ring wins) and shows
	// the threat card.
	await node.click();
	await expect(node).not.toHaveAttribute('data-threat', 'critical');
	const card = page.getByTestId('threat-card');
	await expect(card.getByTestId('threat-badge')).toHaveText('Critical');
	await expect(card.getByText('Threat Alliance')).toBeVisible();
	await expect(card.getByText('61 kills')).toBeVisible();
	await expect(
		card.getByRole('link', { name: 'What they have killed in this system, on zKillboard' }).first(),
	).toHaveAttribute('href', `https://zkillboard.com/alliance/99000001/system/${J122515}/`);
	// A fresh analysis reads "just now" rather than "0 min ago".
	await expect(card.getByText(/Analyzed (just now|.* ago)/)).toBeVisible();
});

test('unknown threat shows no ring and an empty card', async ({ page, api }) => {
	const mapId = await createMap(api, 'E2E ThreatNone');
	await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: J122515, x: 200, y: 200, alias: null },
	});

	await gotoApp(page, `/maps/${mapId}?system=${J122515}`);
	const node = page.getByTestId('system-node').filter({ hasText: 'J122515' });
	await expect(node).not.toHaveAttribute('data-threat', /critical|high/);
	await expect(
		page.getByTestId('threat-card').getByText('No significant activity detected.'),
	).toBeVisible();
});

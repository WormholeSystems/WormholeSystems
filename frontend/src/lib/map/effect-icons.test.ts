import { describe, expect, it } from 'vitest';

import effects from '../../../../data/static/wormhole_effects.json';
import { effectIcon, statIcon } from './effect-icons';

describe('statIcon', () => {
	// The effect data the seed loads, so a stat added there without an icon fails here.
	const stats = new Set(
		Object.values(effects as Record<string, Record<string, Record<string, string[]>>>).flatMap(
			(effect) => Object.values(effect).flatMap((kind) => Object.keys(kind)),
		),
	);

	it('has an icon for every stat a wormhole effect changes', () => {
		expect(stats.size).toBeGreaterThan(20);
		const missing = [...stats].filter((stat) => statIcon(stat) === null);
		expect(missing).toEqual([]);
	});

	it('knows nothing about a stat it was never told', () => {
		expect(statIcon('Cloak Stability')).toBeNull();
	});
});

describe('effectIcon', () => {
	it('has an icon for every wormhole effect', () => {
		const missing = Object.keys(effects).filter((name) => effectIcon(name) === null);
		expect(missing).toEqual([]);
	});
});

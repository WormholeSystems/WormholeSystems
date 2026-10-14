import { describe, expect, it } from 'vitest';

import { decodeLayout, encodeLayout } from './layout-codec';
import { DEFAULT_LAYOUTS } from './registry';

describe('layout codec', () => {
	it('round-trips a copied layout, hidden panels included', () => {
		const text = encodeLayout({ breakpoints: DEFAULT_LAYOUTS, hidden: ['skyhooks'] });
		expect(decodeLayout(text)).toEqual({ breakpoints: DEFAULT_LAYOUTS, hidden: ['skyhooks'] });
		expect(decodeLayout(` ${text} `)).not.toBeNull();
	});

	it('refuses anything that is not a pasted layout', () => {
		expect(decodeLayout('not base64 at all')).toBeNull();
		expect(decodeLayout(btoa('"just a string"'))).toBeNull();
		expect(decodeLayout(btoa('{"breakpoints":{"lg":{"cols":12}}}'))).toBeNull();
	});
});

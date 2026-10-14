import { describe, expect, it } from 'vitest';

import type { CharacterRef } from '$lib/api/types/CharacterRef';
import type { MapSystemView } from '$lib/api/types/MapSystemView';
import { followStep } from './follow.svelte';

const JITA = 30000142;
const AMARR = 30002187;

const systems = [
	{ kind: 'system', id: 1, solar_system_id: JITA },
	{ kind: 'system', id: 2, solar_system_id: AMARR },
	{ kind: 'ghost', id: 3 },
] as MapSystemView[];

function pilot(id: number, at: number | null, online = true): CharacterRef {
	return { character_id: id, solar_system_id: at, online } as CharacterRef;
}

describe('followStep', () => {
	it('selects where a pilot is, once, and then leaves the selection to the viewer', () => {
		const followed = new Map<number, number>();
		expect(followStep(followed, [pilot(7, JITA)], systems)).toBe(1);
		// Still in Jita: a click elsewhere since then is not undone.
		expect(followStep(followed, [pilot(7, JITA)], systems)).toBeNull();
	});

	it('moves with the pilot', () => {
		const followed = new Map([[7, JITA]]);
		expect(followStep(followed, [pilot(7, AMARR)], systems)).toBe(2);
	});

	it('waits for a system that is not on the map yet, and takes it once it is', () => {
		const followed = new Map<number, number>();
		const elsewhere = 30000144;
		expect(followStep(followed, [pilot(7, elsewhere)], systems)).toBeNull();
		const drawn = [...systems, { kind: 'system', id: 4, solar_system_id: elsewhere }];
		expect(followStep(followed, [pilot(7, elsewhere)], drawn as MapSystemView[])).toBe(4);
	});

	it('ignores offline pilots and pilots nowhere', () => {
		const followed = new Map<number, number>();
		expect(followStep(followed, [pilot(7, JITA, false), pilot(8, null)], systems)).toBeNull();
	});
});

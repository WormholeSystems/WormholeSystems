import { beforeEach, describe, expect, it, vi } from 'vitest';

const api = vi.hoisted(() => ({
	currentCharacter: vi.fn(),
	myMaps: vi.fn(async () => []),
	serverStatus: vi.fn(async () => null),
	instance: vi.fn(async () => null),
	myPreferences: vi.fn(async () => null),
}));
vi.mock('$lib/server/api', () => api);

const renew = vi.hoisted(() => vi.fn());
vi.mock('$lib/server/session', () => ({ renewSessionCookie: renew }));

import { load } from './+layout.server';

const event = { cookies: { get: () => undefined } } as never;

describe('root layout', () => {
	beforeEach(() => renew.mockClear());

	it('renews the session cookie for someone signed in', async () => {
		api.currentCharacter.mockResolvedValue({ character_id: 1, name: 'A' });
		await load(event);
		expect(renew).toHaveBeenCalledOnce();
	});

	it('leaves a signed-out visitor alone', async () => {
		api.currentCharacter.mockResolvedValue(null);
		await load(event);
		expect(renew).not.toHaveBeenCalled();
	});
});

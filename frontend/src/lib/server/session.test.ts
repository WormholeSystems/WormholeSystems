import { describe, expect, it } from 'vitest';

import { SESSION_COOKIE, SESSION_DAYS, renewSessionCookie } from './session';

type Options = Record<string, unknown>;

function event(cookie: string | undefined, url: string, proto?: string) {
	const set: [string, string, Options][] = [];
	return {
		set,
		cookies: {
			get: (name: string) => (name === SESSION_COOKIE ? cookie : undefined),
			set: (name: string, value: string, options: Options) => set.push([name, value, options]),
		},
		request: new Request(url, { headers: proto ? { 'x-forwarded-proto': proto } : {} }),
		url: new URL(url),
	};
}

describe('renewSessionCookie', () => {
	it('sets the cookie again for another full lifetime', () => {
		const e = event('abc', 'http://web:3001/maps', 'https');
		renewSessionCookie(e as never);
		expect(e.set).toEqual([
			[
				SESSION_COOKIE,
				'abc',
				{
					path: '/',
					httpOnly: true,
					sameSite: 'lax',
					secure: true,
					maxAge: SESSION_DAYS * 86400,
				},
			],
		]);
	});

	it('leaves Secure off for a plain HTTP install', () => {
		const e = event('abc', 'http://localhost:8090/', 'http');
		renewSessionCookie(e as never);
		expect(e.set[0][2].secure).toBe(false);
	});

	it('does nothing without a session', () => {
		const e = event(undefined, 'https://map.example.com/');
		renewSessionCookie(e as never);
		expect(e.set).toEqual([]);
	});
});

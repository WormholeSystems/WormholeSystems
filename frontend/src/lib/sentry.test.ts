import { describe, expect, it } from 'vitest';
import type { ErrorEvent } from '@sentry/sveltekit';

import { IGNORED_ERRORS, sampleRate, scrubEvent, sentryOptions, stripQuery } from './sentry';

describe('sentryOptions', () => {
	it('is off without a DSN', () => {
		expect(sentryOptions({})).toBeNull();
		expect(sentryOptions({ PUBLIC_SENTRY_DSN: '  ' })).toBeNull();
	});

	it('sends no PII and no traces by default', () => {
		const options = sentryOptions({ PUBLIC_SENTRY_DSN: 'https://key@errors.example.com/2' });
		expect(options?.sendDefaultPii).toBe(false);
		expect(options?.tracesSampleRate).toBe(0);
		expect(options?.environment).toBeUndefined();
	});
});

describe('IGNORED_ERRORS', () => {
	const ignored = (message: string) => IGNORED_ERRORS.some((pattern) => pattern.test(message));

	it('drops chunks and stylesheets that did not load', () => {
		expect(
			ignored(
				'TypeError: Failed to fetch dynamically imported module: https://wormhole.systems/_app/immutable/nodes/0.Bt6oGAkU.js',
			),
		).toBe(true);
		expect(ignored('Unable to preload CSS for https://wormhole.systems/_app/a.css')).toBe(true);
	});

	it('keeps everything else', () => {
		expect(ignored('TypeError: d(...).toSorted is not a function')).toBe(false);
	});
});

describe('sampleRate', () => {
	it('takes a fraction and clamps it', () => {
		expect(sampleRate('0.25')).toBe(0.25);
		expect(sampleRate('4')).toBe(1);
		expect(sampleRate('-1')).toBe(0);
		expect(sampleRate('lots')).toBe(0);
		expect(sampleRate(undefined)).toBe(0);
	});
});

describe('scrubEvent', () => {
	it('drops query strings, cookies and headers', () => {
		expect(stripQuery('https://map.example.com/auth/callback?code=abc&state=xyz')).toBe(
			'https://map.example.com/auth/callback',
		);
		const event = scrubEvent({
			type: undefined,
			request: {
				url: 'https://map.example.com/maps/1?code=abc#frag',
				query_string: 'code=abc',
				cookies: { ws_session: 'secret' },
				headers: { Authorization: 'Bearer wst_secret' },
			},
			user: { id: '1', ip_address: '10.0.0.1' },
			breadcrumbs: [{ category: 'fetch', data: { url: '/api/x?state=s', method: 'GET' } }],
		} as ErrorEvent);
		expect(event.request).toEqual({ url: 'https://map.example.com/maps/1' });
		expect(event.user).toBeUndefined();
		expect(event.breadcrumbs?.[0].data).toEqual({ url: '/api/x', method: 'GET' });
	});
});

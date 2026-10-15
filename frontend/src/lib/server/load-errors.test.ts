import { isHttpError } from '@sveltejs/kit';
import { describe, expect, it } from 'vitest';

import { ApiError } from '$lib/api/client';
import { mapLoadFailed, offline, unreachable } from './load-errors';

/** What a load would have ended with: the status and the body the error page gets. */
function thrown(f: () => never) {
	try {
		f();
	} catch (err) {
		if (isHttpError(err)) return { status: err.status, body: err.body };
		throw err;
	}
	throw new Error('expected the load to end');
}

describe('mapLoadFailed', () => {
	it('is a 404 that explains itself for a map the API will not show', () => {
		const { status, body } = thrown(() => mapLoadFailed(new ApiError(404, 'not found'), true));
		expect(status).toBe(404);
		expect(body).toMatchObject({ kind: 'map', message: 'This map is not open to you.' });
	});

	it('asks a signed-out visitor to sign in instead', () => {
		expect(thrown(() => mapLoadFailed(new ApiError(404, 'not found'), false)).status).toBe(401);
		expect(thrown(() => mapLoadFailed(new ApiError(401, 'nope'), true)).status).toBe(401);
	});

	it('passes a refused role through as a 403', () => {
		expect(thrown(() => mapLoadFailed(new ApiError(403, 'forbidden'), true)).status).toBe(403);
	});

	it('treats a broken or unreachable backend as down, with its request id', () => {
		const broken = thrown(() => mapLoadFailed(new ApiError(500, 'db exploded', 'req-42'), true));
		expect(broken).toEqual({
			status: 503,
			body: {
				message: 'The server is not answering right now.',
				kind: 'offline',
				errorId: 'req-42',
			},
		});

		const refused = thrown(() => mapLoadFailed(new TypeError('fetch failed'), true));
		expect(refused.status).toBe(503);
		// Whatever was thrown stays in the log, never on the page.
		expect(JSON.stringify(refused.body)).not.toContain('fetch failed');
	});
});

describe('unreachable', () => {
	it('is an answer below 500 that is not', () => {
		expect(unreachable(new ApiError(404, 'x'))).toBe(false);
		expect(unreachable(new ApiError(502, 'x'))).toBe(true);
		expect(unreachable(new Error('ECONNREFUSED'))).toBe(true);
	});
});

describe('offline', () => {
	it('carries no id when nothing gave one', () => {
		expect(thrown(() => offline()).body).toEqual({
			message: 'The server is not answering right now.',
			kind: 'offline',
			errorId: undefined,
		});
	});
});

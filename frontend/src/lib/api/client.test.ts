import { describe, expect, it } from 'vitest';

import { ApiError, createApi } from './client';

/** A fetch that records what it was asked and answers with `respond`. */
function recording(respond: () => Response) {
	const calls: { url: string; headers: Headers; method: string }[] = [];
	const fetchFn: typeof fetch = async (input, init) => {
		calls.push({
			url: String(input),
			headers: new Headers(init?.headers),
			method: init?.method ?? 'GET',
		});
		return respond();
	};
	return { calls, fetchFn };
}

describe('createApi', () => {
	it('prefixes the base and sends the default headers alongside its own', async () => {
		const { calls, fetchFn } = recording(() => Response.json(null));
		const api = createApi(fetchFn, 'http://backend:3000', { cookie: 'session=abc' });

		await api.setPinned({ map_id: 4, map_solar_system_id: 9, value: true });

		expect(calls[0].url).toBe('http://backend:3000/api/maps/4/systems/set-pinned');
		expect(calls[0].method).toBe('POST');
		expect(calls[0].headers.get('cookie')).toBe('session=abc');
		expect(calls[0].headers.get('content-type')).toBe('application/json');
	});

	it("keeps the server's status and message on a failure", async () => {
		const { fetchFn } = recording(() => Response.json({ error: 'no such map' }, { status: 404 }));
		const api = createApi(fetchFn);

		const err = await api.fetchMap(4).catch((e: unknown) => e);

		expect(err).toBeInstanceOf(ApiError);
		expect((err as ApiError).status).toBe(404);
		expect((err as ApiError).message).toBe('no such map');
	});

	it('falls back to the status text when the error body is not JSON', async () => {
		const { fetchFn } = recording(
			() => new Response('<html>', { status: 502, statusText: 'Bad Gateway' }),
		);
		const err = await createApi(fetchFn)
			.myMaps()
			.catch((e: unknown) => e);
		expect((err as ApiError).status).toBe(502);
		expect((err as ApiError).message).toBe('Bad Gateway');
	});
});

// Turning what the API said into the error page a load should end on. One place, so a map
// that was deleted, a map that was never yours and a backend that is down each reach the
// page that explains them, with the status that matches, instead of all of them falling
// through to a 500.

import { error } from '@sveltejs/kit';

import { ApiError } from '$lib/api/client';

/** True when the API could not be reached or broke, rather than said no. */
export function unreachable(err: unknown): boolean {
	return !(err instanceof ApiError) || err.status >= 500;
}

/** Ends the load on the backend-is-down page. */
export function offline(err?: unknown): never {
	const errorId = err instanceof ApiError ? (err.requestId ?? undefined) : undefined;
	error(503, {
		message: 'The server is not answering right now.',
		kind: 'offline',
		errorId,
	});
}

/**
 * A map that could not be opened. The API answers 404 both for a map that does not exist
 * and for one the caller may not see, on purpose, so the page cannot tell those apart
 * either; it says both, and what to do about the second. Signed out, the answer is to sign
 * in.
 */
export function mapLoadFailed(err: unknown, signedIn: boolean): never {
	if (unreachable(err)) offline(err);
	// SAFETY: `unreachable` is false only for an `ApiError`.
	const apiError = err as ApiError;
	const errorId = apiError.requestId ?? undefined;
	if (apiError.status === 401 || (apiError.status === 404 && !signedIn)) {
		error(401, { message: 'Sign in to open this map.', kind: 'map', errorId });
	}
	if (apiError.status === 403) {
		error(403, { message: 'Your role on this map does not allow this.', kind: 'map', errorId });
	}
	error(404, { message: 'This map is not open to you.', kind: 'map', errorId });
}

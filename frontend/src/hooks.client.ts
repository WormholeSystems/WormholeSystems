import type { ClientInit, HandleClientError } from '@sveltejs/kit';

import { env } from '$env/dynamic/public';
import { TUNNEL, sentryOptions } from '$lib/sentry';

let report: ((error: unknown) => void) | null = null;

// The SDK is loaded only when reporting is on, so an install without it ships none of it.
// The default integrations, without replay or feedback, which are only there when asked for.
export const init: ClientInit = async () => {
	const options = sentryOptions(env);
	if (!options) return;
	const Sentry = await import('@sentry/sveltekit');
	Sentry.init({ ...options, tunnel: TUNNEL });
	report = (error) =>
		Sentry.captureException(error, { mechanism: { type: 'sveltekit', handled: false } });
};

// What the error page gets for anything nobody threw on purpose: never the thrown text,
// which is for whoever reads the log, but a short id that is in the log next to it.
export const handleError: HandleClientError = ({ error, status, message }) => {
	if (status === 404) return { message };
	report?.(error);
	const errorId = crypto.randomUUID().slice(0, 8);
	console.error(`[error ${errorId}]`, error);
	return { message: 'Something went wrong on our side.', errorId };
};

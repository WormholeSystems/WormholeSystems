import type { HandleServerError, ServerInit } from '@sveltejs/kit';

import { env } from '$env/dynamic/public';
import { sentryOptions } from '$lib/sentry';

let report: ((error: unknown) => void) | null = null;

// Errors only, and the SDK loaded only when reporting is on. No `sentryHandle`: its tracing
// tags and inline fetch script are not wanted, and a strict Content-Security-Policy would
// refuse the script anyway.
export const init: ServerInit = async () => {
	const options = sentryOptions(env);
	if (!options) return;
	const Sentry = await import('@sentry/sveltekit');
	Sentry.init(options);
	report = (error) =>
		Sentry.captureException(error, { mechanism: { type: 'sveltekit', handled: false } });
};

// What the error page gets for anything nobody threw on purpose: never the thrown text,
// which is for whoever reads the log, but a short id that is in the log next to it.
export const handleError: HandleServerError = ({ error, status, message }) => {
	if (status === 404) return { message };
	report?.(error);
	const errorId = crypto.randomUUID().slice(0, 8);
	console.error(`[error ${errorId}]`, error);
	return { message: 'Something went wrong on our side.', errorId };
};

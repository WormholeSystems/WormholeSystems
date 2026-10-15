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

export const handleError: HandleServerError = ({ error, status }) => {
	if (status !== 404) report?.(error);
};

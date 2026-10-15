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

export const handleError: HandleClientError = ({ error, status }) => {
	if (status !== 404) report?.(error);
};

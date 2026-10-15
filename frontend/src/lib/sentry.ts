// Error reporting, off unless the operator sets PUBLIC_SENTRY_DSN. Read at runtime rather
// than build time: the images are built once and shared by every install, and each
// operator decides for themselves whether and where errors go. Works with anything that
// speaks the Sentry protocol (Sentry, GlitchTip, Bugsink).
//
// What goes out is kept to the error itself: no default PII, no session replay, no tracing
// unless SENTRY_TRACES_SAMPLE_RATE says so, and every URL loses its query string, which is
// where an SSO `code` or `state` would be.

import type { Breadcrumb, ErrorEvent } from '@sentry/sveltekit';

/** The runtime variables this reads, as `$env/dynamic/public` hands them over. */
export type SentryEnv = Partial<Record<string, string>>;

/** The browser posts here and the API forwards it, so nothing leaves for a third origin. */
export const TUNNEL = '/api/sentry-tunnel';

/** The shared part of the client and server options, or `null` when reporting is off. */
export function sentryOptions(env: SentryEnv) {
	const dsn = env.PUBLIC_SENTRY_DSN?.trim();
	if (!dsn) return null;
	return {
		dsn,
		environment: env.PUBLIC_SENTRY_ENVIRONMENT?.trim() || undefined,
		release: env.PUBLIC_SENTRY_RELEASE?.trim() || undefined,
		sendDefaultPii: false,
		tracesSampleRate: sampleRate(env.PUBLIC_SENTRY_TRACES_SAMPLE_RATE),
		beforeSend: scrubEvent,
		beforeBreadcrumb: scrubBreadcrumb,
	};
}

/** A fraction between 0 and 1; anything else, or nothing, is 0. */
export function sampleRate(value: string | undefined): number {
	const rate = Number(value);
	return Number.isFinite(rate) && rate > 0 ? Math.min(rate, 1) : 0;
}

/** `url` without its query string and fragment. */
export function stripQuery(url: string): string {
	const end = url.search(/[?#]/);
	return end === -1 ? url : url.slice(0, end);
}

export function scrubEvent(event: ErrorEvent): ErrorEvent {
	if (event.request) {
		if (event.request.url) event.request.url = stripQuery(event.request.url);
		delete event.request.query_string;
		delete event.request.cookies;
		delete event.request.headers;
		delete event.request.data;
	}
	delete event.user;
	delete event.server_name;
	event.breadcrumbs = event.breadcrumbs?.map(scrubBreadcrumb);
	return event;
}

export function scrubBreadcrumb(crumb: Breadcrumb): Breadcrumb {
	if (crumb.data) {
		for (const field of ['url', 'from', 'to']) {
			const value = crumb.data[field];
			if (typeof value === 'string') crumb.data[field] = stripQuery(value);
		}
	}
	return crumb;
}

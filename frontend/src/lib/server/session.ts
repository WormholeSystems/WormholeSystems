// The session cookie's lifetime, renewed on every page load while signed in. The API renews
// the session itself when it is used (`src/session.rs`, `SESSION_DAYS`); this keeps the
// browser's copy from running out first. Browsers cap a cookie at 400 days.

import type { RequestEvent } from '@sveltejs/kit';

export const SESSION_COOKIE = 'ws_session';
export const SESSION_DAYS = 400;

/** Set the session cookie again with a fresh lifetime, attributes as the API sets them. */
export function renewSessionCookie(event: Pick<RequestEvent, 'cookies' | 'request' | 'url'>) {
	const id = event.cookies.get(SESSION_COOKIE);
	if (!id) return;
	// Behind the proxy the request arrives as plain HTTP; the proxy says what the browser used.
	const scheme =
		event.request.headers.get('x-forwarded-proto') ?? event.url.protocol.replace(':', '');
	event.cookies.set(SESSION_COOKIE, id, {
		path: '/',
		httpOnly: true,
		sameSite: 'lax',
		secure: scheme === 'https',
		maxAge: SESSION_DAYS * 24 * 60 * 60,
	});
}

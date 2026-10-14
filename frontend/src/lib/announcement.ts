// The operator's site-wide banner. A dismissal is a cookie rather than local storage so the
// server can leave a dismissed banner out of the page it renders: one that hydration took
// away again would flash on every navigation.

import type { Announcement } from '$lib/api/types/Announcement';

export const DISMISSED_COOKIE = 'announcement_dismissed';

/** A year: long enough that nobody sees the same announcement twice. */
const REMEMBER_FOR = 60 * 60 * 24 * 365;

/**
 * The announcement to show, given the id this visitor dismissed last. A dismissal only
 * counts for the exact announcement it was made on, and never for one that cannot be closed.
 */
export function visibleAnnouncement(
	announcement: Announcement | null,
	dismissedId: string | undefined,
): Announcement | null {
	if (!announcement) return null;
	if (announcement.dismissible && dismissedId === announcement.id) return null;
	return announcement;
}

/** The `document.cookie` assignment that remembers dismissing `id`. */
export function dismissalCookie(id: string, secure: boolean): string {
	const parts = [
		`${DISMISSED_COOKIE}=${encodeURIComponent(id)}`,
		'path=/',
		`max-age=${REMEMBER_FOR}`,
		'samesite=lax',
	];
	if (secure) parts.push('secure');
	return parts.join('; ');
}

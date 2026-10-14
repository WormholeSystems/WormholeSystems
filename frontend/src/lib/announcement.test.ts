import { describe, expect, it } from 'vitest';

import type { Announcement } from '$lib/api/types/Announcement';
import { dismissalCookie, visibleAnnouncement } from './announcement';

const announcement: Announcement = {
	id: 'abc123def456',
	level: 'info',
	title: null,
	message: 'We are migrating.',
	link: null,
	dismissible: true,
};

describe('visibleAnnouncement', () => {
	it('shows nothing when there is no announcement', () => {
		expect(visibleAnnouncement(null, undefined)).toBeNull();
	});

	it('shows an announcement nobody dismissed', () => {
		expect(visibleAnnouncement(announcement, undefined)).toBe(announcement);
	});

	it('hides the announcement that was dismissed', () => {
		expect(visibleAnnouncement(announcement, announcement.id)).toBeNull();
	});

	it('shows a new announcement after an older one was dismissed', () => {
		expect(visibleAnnouncement(announcement, '000000000000')).toBe(announcement);
	});

	it('ignores a dismissal of one that cannot be dismissed', () => {
		const permanent = { ...announcement, dismissible: false };
		expect(visibleAnnouncement(permanent, permanent.id)).toBe(permanent);
	});
});

describe('dismissalCookie', () => {
	it('remembers the id site-wide', () => {
		expect(dismissalCookie('abc', false)).toBe(
			'announcement_dismissed=abc; path=/; max-age=31536000; samesite=lax',
		);
		expect(dismissalCookie('abc', true)).toMatch(/; secure$/);
	});
});

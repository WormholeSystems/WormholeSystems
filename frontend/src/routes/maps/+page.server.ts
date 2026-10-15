import { redirect } from '@sveltejs/kit';

import { offline } from '$lib/server/load-errors';
import type { PageServerLoad } from './$types';

// The map list is yours, so it needs an account. A single map is not gated here: it may
// have been opened up to watchers, and the API decides that per request.
//
// The list itself is the one the root layout already fetched for the top bar; asking the
// API for it a second time would only cost another round trip.
export const load: PageServerLoad = async (event) => {
	const { me, offline: down } = await event.parent();
	// Down is not the same as signed out: sending somebody to sign in would only fail again.
	if (down) offline();
	if (!me) redirect(302, `/login?return_to=${encodeURIComponent(event.url.pathname)}`);
	return { seo: { title: 'Maps' } };
};

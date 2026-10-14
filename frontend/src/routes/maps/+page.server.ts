import { redirect } from '@sveltejs/kit';

import type { PageServerLoad } from './$types';

// The map list is yours, so it needs an account. A single map is not gated here: it may
// have been opened up to watchers, and the API decides that per request.
//
// The list itself is the one the root layout already fetched for the top bar; asking the
// API for it a second time would only cost another round trip.
export const load: PageServerLoad = async (event) => {
	const { me } = await event.parent();
	if (!me) redirect(302, '/login');
	return { seo: { title: 'Maps' } };
};

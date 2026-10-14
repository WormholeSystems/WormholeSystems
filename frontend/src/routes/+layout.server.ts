import { DISMISSED_COOKIE, visibleAnnouncement } from '$lib/announcement';
import { currentCharacter, instance, myMaps, serverStatus } from '$lib/server/api';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async (event) => {
	// Everything the top bar draws, resolved with the page so none of it appears a moment
	// later: the map list its shortcuts come from, the server state beside the clock, and the
	// operator's banner above it all. At once, since none of them needs an answer from the
	// others; a signed-out visitor gets an empty list rather than a wave spent finding out
	// they are signed out.
	const [me, maps, status, deployment] = await Promise.all([
		currentCharacter(event),
		myMaps(event).catch(() => []),
		serverStatus(event),
		instance(event),
	]);
	// Decided here rather than in the browser, so a banner this visitor closed is never
	// rendered only for hydration to take it away again.
	const announcement = visibleAnnouncement(
		deployment?.announcement ?? null,
		event.cookies.get(DISMISSED_COOKIE),
	);
	return { me, maps, status, announcement };
};

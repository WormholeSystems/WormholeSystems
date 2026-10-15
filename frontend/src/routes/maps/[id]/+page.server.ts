import { gridConfig, mapSignatures, mapUserSettings, mapView } from '$lib/server/api';
import { mapLoadFailed } from '$lib/server/load-errors';
import type { PageServerLoad } from './$types';

// The graph and the viewer's arrangement, fetched with the page rather than after it, so
// the first frame the page paints is the map rather than a placeholder. The signatures and
// the grid ride along for the same reason: the signature panel paints with the map instead
// of a beat later, and the canvas is never drawn on the built-in grid first. `MapState`
// starts from all of it and takes over.
//
// The graph is the one part the page cannot do without: a map that is gone, was never
// shared with this visitor, or a backend that is down ends on the error page that says
// which, with the status that goes with it. The rest only fall back to the client.
export const load: PageServerLoad = async (event) => {
	const id = Number(event.params.id);
	const [view, settings, signatures, grid] = await Promise.all([
		mapView(event, id).catch((err: unknown) =>
			err instanceof Error ? err : new Error(String(err)),
		),
		mapUserSettings(event, id).catch(() => null),
		mapSignatures(event, id).catch(() => null),
		gridConfig(event).catch(() => null),
	]);
	if (view instanceof Error || !Number.isSafeInteger(id)) {
		const { me } = await event.parent();
		mapLoadFailed(view, me !== null);
	}
	// A map link is the one people actually paste at each other, so it says which map.
	return { view, settings, signatures, grid, seo: { title: view.map.name } };
};

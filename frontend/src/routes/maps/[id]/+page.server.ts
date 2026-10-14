import { gridConfig, mapSignatures, mapUserSettings, mapView } from '$lib/server/api';
import type { PageServerLoad } from './$types';

// The graph and the viewer's arrangement, fetched with the page rather than after it, so
// the first frame the page paints is the map rather than a placeholder. The signatures and
// the grid ride along for the same reason: the signature panel paints with the map instead
// of a beat later, and the canvas is never drawn on the built-in grid first. `MapState`
// starts from all of it and takes over; a failure is left to the client, which knows how to
// say why.
export const load: PageServerLoad = async (event) => {
	const id = Number(event.params.id);
	const [view, settings, signatures, grid] = await Promise.all([
		mapView(event, id).catch(() => null),
		mapUserSettings(event, id).catch(() => null),
		mapSignatures(event, id).catch(() => null),
		gridConfig(event).catch(() => null),
	]);
	// A map link is the one people actually paste at each other, so it says which map.
	return { view, settings, signatures, grid, seo: { title: view?.map.name } };
};

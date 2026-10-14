// "Follow me": the selection moves to wherever the viewer's tracked pilots fly.

import type { CharacterRef } from '$lib/api/types/CharacterRef';
import type { MapSystemView } from '$lib/api/types/MapSystemView';
import { trackedPilots } from '$lib/map/tracked-pilots';
import type { MapState } from './map-state.svelte';

/**
 * The node to select now, or null to leave the selection alone. `followed` remembers where
 * each pilot last took the selection, so clicking elsewhere holds until one of them moves;
 * it is updated for the pilots that moved onto the map.
 */
export function followStep(
	followed: Map<number, number>,
	pilots: CharacterRef[],
	systems: MapSystemView[],
): number | null {
	let next: number | null = null;
	for (const pilot of pilots) {
		const systemId = pilot.online ? (pilot.solar_system_id ?? null) : null;
		if (systemId === null || followed.get(pilot.character_id) === systemId) continue;
		// A jump into an unmapped hole lands here again once the tracker has drawn it.
		const placed = systems.find((s) => s.kind === 'system' && s.solar_system_id === systemId);
		if (!placed) continue;
		followed.set(pilot.character_id, systemId);
		next = placed.id;
	}
	return next;
}

/** Component init only: follows while the viewer has the setting on. */
export function followCharacters(map: MapState): void {
	const followed = new Map<number, number>();
	$effect(() => {
		const settings = map.userSettings;
		if (!settings?.follow_character) return;
		const pilots = trackedPilots(map.characters.mine, settings.tracked_character_ids);
		const next = followStep(followed, pilots, map.systems.all);
		if (next !== null) map.activeId = next;
	});
}

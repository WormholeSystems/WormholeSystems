// What the navigation pickers offer before anything is typed: the systems already in play.
// Each sits inside its own picker, so choosing one is unambiguous about which field it fills.

import type { SystemSearchResult } from '$lib/api/types/SystemSearchResult';
import type { MapState } from '../../../state/map-state.svelte';

export interface PickerSuggestion {
	system: SystemSearchResult;
	reason: string;
	icon: 'selected' | 'location' | 'pinned';
}

/** Read inside a `$derived`: it follows the selection, the pilot and both watchlists. */
export function pickerSuggestions(map: MapState): PickerSuggestion[] {
	const picks: { id: number; reason: string; icon: PickerSuggestion['icon'] }[] = [];
	const offer = (id: number, reason: string, icon: PickerSuggestion['icon']) => {
		if (!picks.some((p) => p.id === id)) picks.push({ id, reason, icon });
	};
	const active = map.activeSystem;
	if (active?.kind === 'system') offer(active.solar_system_id, 'Selected system', 'selected');
	const character = map.characters.mine.find((c) => c.online && c.solar_system_id !== null);
	if (character?.solar_system_id != null) {
		offer(character.solar_system_id, 'Where you are', 'location');
	}
	for (const entry of map.personalWatchlist.all.filter((w) => w.is_pinned).slice(0, 5)) {
		offer(entry.solar_system_id, 'Pinned on your watchlist', 'pinned');
	}
	for (const entry of map.watchlist.all.filter((w) => w.is_pinned).slice(0, 5)) {
		offer(entry.solar_system_id, 'Pinned on the watchlist', 'pinned');
	}
	// Only those we can render as a proper row; an unresolved one would show as a bare id.
	return picks.flatMap((p) => {
		const system = map.systemInfo(p.id);
		return system ? [{ system, reason: p.reason, icon: p.icon }] : [];
	});
}

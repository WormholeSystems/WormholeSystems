// Undo and redo move the map's cursor through the history tree rather than recording
// anything, so the server is the only thing that decides whether they are available.

import { api } from '$lib/api/client';
import type { MapHistory } from '$lib/api/types/MapHistory';
import { headEntry, redoEntry, stepDetail } from '$lib/map/history-tree';
import type { RunAction } from '$lib/map/actions';

export interface HistoryHost {
	mapId: number;
	data(): MapHistory | null;
	run: RunAction;
}

export class HistoryStore {
	private host: HistoryHost;

	constructor(host: HistoryHost) {
		this.host = host;
	}

	get data() {
		return this.host.data();
	}

	entries = $derived(this.data?.entries ?? []);
	canUndo = $derived(this.data?.can_undo ?? false);
	canRedo = $derived(this.data?.can_redo ?? false);
	/** The step the map is sitting on, for labelling the undo button. */
	headEntry = $derived.by(() => headEntry(this.data));
	redoEntry = $derived.by(() => redoEntry(this.data));

	// Undo and redo are read before they run: the step being walked past is the head now,
	// and after the call it is somewhere else. "Undone" alone leaves you to work out what
	// went, which on a map somebody else is also editing is not obvious.
	undo() {
		this.host.run('undo', () => api.undoMapEvent(this.host.mapId), stepDetail(this.headEntry));
	}

	redo() {
		this.host.run('redo', () => api.redoMapEvent(this.host.mapId), stepDetail(this.redoEntry));
	}

	/** Jump the map to any step, which is how a branch left behind by an undo is re-entered. */
	gotoEvent(eventId: number | null) {
		const target = this.entries.find((e) => e.id === eventId) ?? null;
		this.host.run(
			'goToEvent',
			() => api.gotoMapEvent({ map_id: this.host.mapId, event_id: eventId }),
			eventId === null ? 'Back to the empty map' : stepDetail(target),
		);
	}
}

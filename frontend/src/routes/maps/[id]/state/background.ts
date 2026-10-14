// The picture behind this viewer's map, set from the map itself.

import type { BackgroundImageWriter } from '$lib/map/user-settings';

export interface BackgroundHost {
	mapId: number;
	writer: BackgroundImageWriter;
}

export class BackgroundApi {
	constructor(private host: BackgroundHost) {}

	/** An upload or removal is on the wire. */
	get pending(): boolean {
		return this.host.writer.pending;
	}

	/** Checked here first, so a wrong file is turned away before anything is sent. */
	upload(file: File) {
		this.host.writer.upload(this.host.mapId, file);
	}

	remove() {
		this.host.writer.remove(this.host.mapId);
	}
}

import type { KillmailScope } from '$lib/api/types/KillmailScope';

/** The killmail scope filter, worded to read in both the card and the settings page. */
export const KILLMAIL_FILTERS: { value: KillmailScope; label: string }[] = [
	{ value: 'all', label: 'Everywhere on the map' },
	{ value: 'jspace', label: 'Wormhole space only' },
	{ value: 'kspace', label: 'Known space only' },
	{ value: 'selected_system', label: 'Selected system only' },
];

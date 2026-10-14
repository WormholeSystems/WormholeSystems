// The map transfer file, as the UI needs it: which sections exist, what to call them, and
// a reader that checks a chosen file is one of ours before any of it is sent anywhere.

export const TRANSFER_FORMAT = 'wormholesystems-map-export';
export const TRANSFER_VERSION = 1;

import type { TransferSectionId } from '$lib/api/client';

export type { TransferSectionId };

export const TRANSFER_SECTIONS: {
	id: TransferSectionId;
	label: string;
	description: string;
}[] = [
	{
		id: 'settings',
		label: 'Settings',
		description: 'Name, layout, naming and bookmark formats, home and rally points.',
	},
	{
		id: 'access',
		label: 'Access',
		description: 'Every grant except the owner. Never the share link.',
	},
	{
		id: 'solarsystems',
		label: 'Systems',
		description: 'Placed systems with positions and aliases, plus saved intel.',
	},
	{
		id: 'connections',
		label: 'Connections',
		description: 'The edges, with wormhole type, mass, size and lifetime.',
	},
	{
		id: 'signatures',
		label: 'Signatures',
		description: 'Scanned signatures, still linked to their connections.',
	},
	{
		id: 'routes',
		label: 'Routes',
		description: 'The navigation watchlist.',
	},
];

export type ExportFilePeek = {
	fileName: string;
	mapName: string;
	/** The sections the file actually contains, in display order. */
	sections: TransferSectionId[];
	/** The file verbatim, for the request body. */
	content: string;
};

/**
 * Read a chosen file and check it is a map export the server will take, so a wrong file
 * fails at the picker with a reason instead of after an upload. The server re-validates
 * everything; this is only the early no.
 */
export async function readExportFile(file: File): Promise<ExportFilePeek> {
	const content = await file.text();
	let parsed: unknown;
	try {
		parsed = JSON.parse(content);
	} catch {
		throw new Error('this file is not valid JSON');
	}
	const data = parsed as Record<string, unknown>;
	if (data?.format !== TRANSFER_FORMAT) {
		throw new Error('this file is not a wormholesystems map export');
	}
	if (data.version !== TRANSFER_VERSION) {
		throw new Error('this file was exported by an incompatible version of the application');
	}
	const mapName = typeof data.map_name === 'string' ? data.map_name : '';
	const available =
		typeof data.sections === 'object' && data.sections !== null ? Object.keys(data.sections) : [];
	const sections = TRANSFER_SECTIONS.map((s) => s.id).filter((id) => available.includes(id));
	if (sections.length === 0) {
		throw new Error('the file does not contain any sections');
	}
	return { fileName: file.name, mapName, sections, content };
}

import { describe, expect, it } from 'vitest';

import { TRANSFER_FORMAT, TRANSFER_VERSION, readExportFile } from './transfer';

function file(content: unknown, name = 'chain.json'): File {
	const text = typeof content === 'string' ? content : JSON.stringify(content);
	return new File([text], name, { type: 'application/json' });
}

const valid = {
	format: TRANSFER_FORMAT,
	version: TRANSFER_VERSION,
	map_name: 'Home chain',
	// Out of display order on purpose, plus a section this client does not know.
	sections: { signatures: [], solarsystems: [], settings: {}, extra: [] },
};

describe('readExportFile', () => {
	it('reads the name and the sections it carries, in display order', async () => {
		const peek = await readExportFile(file(valid));
		expect(peek.fileName).toBe('chain.json');
		expect(peek.mapName).toBe('Home chain');
		expect(peek.sections).toEqual(['settings', 'solarsystems', 'signatures']);
		expect(JSON.parse(peek.content)).toEqual(valid);
	});

	it('turns away what is not JSON', async () => {
		await expect(readExportFile(file('{nope'))).rejects.toThrow('not valid JSON');
	});

	it('turns away JSON that is not a map export', async () => {
		await expect(readExportFile(file({ ...valid, format: 'something-else' }))).rejects.toThrow(
			'not a wormholesystems map export',
		);
		await expect(readExportFile(file(null))).rejects.toThrow('not a wormholesystems map export');
	});

	it('turns away another version of the format', async () => {
		await expect(readExportFile(file({ ...valid, version: TRANSFER_VERSION + 1 }))).rejects.toThrow(
			'incompatible version',
		);
	});

	it('turns away an export with nothing in it it could import', async () => {
		await expect(readExportFile(file({ ...valid, sections: { extra: [] } }))).rejects.toThrow(
			'does not contain any sections',
		);
		await expect(readExportFile(file({ ...valid, sections: null }))).rejects.toThrow(
			'does not contain any sections',
		);
	});

	it('leaves the name blank when the file has none', async () => {
		const peek = await readExportFile(file({ ...valid, map_name: 42 }));
		expect(peek.mapName).toBe('');
	});
});

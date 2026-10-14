// The background behind a viewer's map: the grid lines, the picture the server accepts and
// the two ways it can sit. Shared by the canvas, the on-map popover and the display settings.
import type { BackgroundMode } from '$lib/api/types/BackgroundMode';

export const MAX_IMAGE_BYTES = 8 * 1024 * 1024;

export const ACCEPTED_IMAGE_TYPES = 'image/png,image/jpeg,image/gif,image/webp';

export const BACKGROUND_MODES: readonly {
	value: BackgroundMode;
	label: string;
	hint: string;
}[] = [
	{
		value: 'grid',
		label: 'Moves with the map',
		hint: 'Spans the grid, so it pans and zooms with the systems',
	},
	{
		value: 'viewport',
		label: 'Fills the panel',
		hint: 'Fills the visible panel and stays put while you navigate',
	},
];

export const BACKGROUND_MODE_VALUES = BACKGROUND_MODES.map((m) => m.value);

/** Why a picked file cannot be sent, or null when it can. */
export function rejectImage(file: File): string | null {
	if (!file.type.startsWith('image/')) return 'That is not an image.';
	if (file.size > MAX_IMAGE_BYTES) return 'That image is over 8 MiB. Shrink it first.';
	return null;
}

const GRID_LINES = [
	'linear-gradient(to right, var(--color-grid) 1px, transparent 1px)',
	'linear-gradient(to bottom, var(--color-grid) 1px, transparent 1px)',
];

export function gridBackground(): string {
	return GRID_LINES.join(', ');
}

interface BackgroundLayer {
	image: string;
	size: string;
	repeat: string;
	position: string;
}

function declarations(layers: BackgroundLayer[]): string {
	if (layers.length === 0) return '';
	const list = (pick: (l: BackgroundLayer) => string) => layers.map(pick).join(', ');
	return (
		`background-image: ${list((l) => l.image)}; ` +
		`background-size: ${list((l) => l.size)}; ` +
		`background-repeat: ${list((l) => l.repeat)}; ` +
		`background-position: ${list((l) => l.position)};`
	);
}

/**
 * The backgrounds of the canvas's two layers as CSS declarations: the scaled world the
 * nodes sit on, and the fixed viewport around it. A `grid` image is painted onto the world
 * so it pans and zooms with the systems; a `viewport` image fills the panel and stays put.
 */
export function canvasBackground(opts: {
	gridLines: boolean;
	cell: number;
	image: string | null;
	mode: BackgroundMode;
}): { world: string; viewport: string } {
	const cell = `${opts.cell}px ${opts.cell}px`;
	const picture: BackgroundLayer | null = opts.image
		? {
				image: `url("${opts.image.replaceAll('"', '%22')}")`,
				size: 'cover',
				repeat: 'no-repeat',
				position: 'center center',
			}
		: null;
	const world: BackgroundLayer[] = opts.gridLines
		? GRID_LINES.map((image) => ({ image, size: cell, repeat: 'repeat', position: '0 0' }))
		: [];
	if (picture && opts.mode === 'grid') world.push(picture);
	return {
		world: declarations(world),
		viewport: declarations(picture && opts.mode === 'viewport' ? [picture] : []),
	};
}

// Solar-system class metadata, ported from the legacy solarsystem_classes catalogue and
// keyed by our numeric wormhole_class_id (k-space uses 7/8/9, Pochven 25).

import { ccpRoundSecurity } from '$lib/security';

export interface ClassMeta {
	/** Short node label, e.g. `C5`, `H`, `P`. */
	short: string;
	/** Color token (a `--color-<token>` custom property, usable as `text-<token>`). */
	token: string;
	sortWeight: number;
	isWormholeSpace: boolean;
	isKnownSpace: boolean;
	isDrifter: boolean;
}

function meta(
	short: string,
	token: string,
	sortWeight: number,
	kind: 'wormhole' | 'known' | 'other',
	isDrifter = false,
): ClassMeta {
	return {
		short,
		token,
		sortWeight,
		isWormholeSpace: kind === 'wormhole',
		isKnownSpace: kind === 'known',
		isDrifter,
	};
}

const CLASSES = new Map<number, ClassMeta>([
	[1, meta('C1', 'c1', 11, 'wormhole')],
	[2, meta('C2', 'c2', 12, 'wormhole')],
	[3, meta('C3', 'c3', 13, 'wormhole')],
	[4, meta('C4', 'c4', 14, 'wormhole')],
	[5, meta('C5', 'c5', 15, 'wormhole')],
	[6, meta('C6', 'c6', 16, 'wormhole')],
	[7, meta('H', 'hs', 0, 'known')],
	[8, meta('L', 'ls', 1, 'known')],
	[9, meta('N', 'ns', 2, 'known')],
	[12, meta('C12', 'c12', 22, 'wormhole')], // Thera
	[13, meta('C13', 'c13', 23, 'wormhole')], // shattered frigate holes
	[14, meta('C14', 'c14', 24, 'wormhole', true)], // Sentinel
	[15, meta('C15', 'c15', 25, 'wormhole', true)], // Barbican
	[16, meta('C16', 'c16', 26, 'wormhole', true)], // Vidette
	[17, meta('C17', 'c17', 27, 'wormhole', true)], // Conflux
	[18, meta('C18', 'c18', 28, 'wormhole', true)], // Redoubt
	[19, meta('C19', 'unknown', 29, 'other')],
	[20, meta('C20', 'unknown', 30, 'other')],
	[21, meta('C21', 'unknown', 31, 'other')],
	[22, meta('C22', 'unknown', 32, 'other')],
	[23, meta('C23', 'unknown', 33, 'other')],
	[25, meta('P', 'pochven', 3, 'other')],
]);

const UNKNOWN: ClassMeta = meta('?', 'unknown', 99, 'other');

/** Class id from a security status, for the few systems without a class id. */
function classFromSecurity(security: number): number {
	const rounded = ccpRoundSecurity(security);
	if (rounded >= 0.5) return 7;
	if (rounded >= 0.1) return 8;
	return 9;
}

/**
 * Metadata for a system's class, falling back to the security band. A ghost has neither,
 * and reads as `?`.
 */
export function classMeta(wormholeClassId: number | null, security: number | null): ClassMeta {
	if (wormholeClassId === null && security === null) return UNKNOWN;
	return CLASSES.get(wormholeClassId ?? classFromSecurity(security ?? 0)) ?? UNKNOWN;
}

/** Metadata for a static's destination class. */
export function destClassMeta(destClass: number | null): ClassMeta {
	return destClass === null ? UNKNOWN : (CLASSES.get(destClass) ?? UNKNOWN);
}

export function isWormholeClass(wormholeClassId: number | null): boolean {
	return wormholeClassId !== null && (CLASSES.get(wormholeClassId)?.isWormholeSpace ?? false);
}

/** Text color for a wormhole effect, matched on keyword. */
export function effectTextColor(name: string | null | undefined): string {
	const n = name?.toLowerCase() ?? '';
	if (n.includes('pulsar')) return 'text-blue-400';
	if (n.includes('magnetar')) return 'text-pink-400';
	if (n.includes('wolf')) return 'text-amber-600';
	if (n.includes('black')) return 'text-neutral-400';
	if (n.includes('cataclysmic')) return 'text-yellow-400';
	if (n.includes('red giant')) return 'text-red-400';
	return 'text-muted-foreground';
}

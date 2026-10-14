// Connection bookmark names. The name is read from the in-game bookmark list, so it has to
// say where the hole goes without the map open. Formats are per map because every group
// names their chain differently.

import { isIgnoredAlias } from '$lib/naming/alias';
import { classMeta, isWormholeClass } from '$lib/map/classes';
import type { MapNaming } from '$lib/api/types/MapNaming';
import type { MassStatus } from '$lib/api/types/MassStatus';
import type { TimeStatus } from '$lib/api/types/TimeStatus';
import type { WormholeSize } from '$lib/api/types/WormholeSize';

export const BOOKMARK_TOKENS = [
	'alias',
	'sig',
	'class',
	'name',
	'region',
	'occupier',
	'size',
	'wh',
	'mass',
	'life',
] as const;

export type BookmarkToken = (typeof BOOKMARK_TOKENS)[number];

export const DEFAULT_FORMAT_WORMHOLE = '{alias} {sig} {class}';
export const DEFAULT_FORMAT_KSPACE = '{alias} {class} {sig} {name} {region}';
/** The leading `*` sorts the way home to the top of the in-game folder. */
export const DEFAULT_FORMAT_RETURN = '*{alias} {sig} {class}';
export const DEFAULT_IGNORED_ALIAS = 'HOME';

/** Large is the common case and deliberately blank, so the token only shows restrictive holes. */
const SIZE_LABELS: Partial<Record<WormholeSize, string>> = {
	small: 'SM',
	medium: 'MD',
	xl: 'XM',
};
/** Stable resolves to nothing, so mass only appears once the hole has actually degraded. */
const MASS_LABELS: Partial<Record<MassStatus, string>> = { reduced: 'reduced', critical: 'crit' };
/** Kept in the EOL vocabulary so it never reads as mass "crit". */
const LIFE_LABELS: Partial<Record<TimeStatus, string>> = {
	eol: 'EOL',
	critical: 'EOL!',
	expired: 'EOL!!',
};

export interface BookmarkSystem {
	alias: string | null;
	name: string;
	region: string | null;
	wormholeClassId: number | null;
	/** `null` when the far side is unknown, which blanks the class rather than guessing. */
	security: number | null;
	occupier: string | null;
}

export interface BookmarkContext {
	signatureId: string | null;
	size: WormholeSize | null;
	massStatus: MassStatus | null;
	timeStatus: TimeStatus | null;
	/** The wormhole code, e.g. `H296`. */
	wormholeCode: string | null;
}

export interface BookmarkFormats {
	wormhole?: string | null;
	kspace?: string | null;
	return?: string | null;
	ignoredAlias?: string | null;
}

/**
 * The map's naming row as the formats a bookmark renders with. Every copy goes through
 * this, so a new one cannot quietly fall back to the defaults while the map has its own.
 */
export function formatsFromNaming(naming: MapNaming | null | undefined): BookmarkFormats {
	return {
		wormhole: naming?.bookmark_wormhole,
		kspace: naming?.bookmark_kspace,
		return: naming?.bookmark_return,
		ignoredAlias: naming?.ignored_alias,
	};
}

/**
 * Whether naming `destinationAlias` describes the way back up the chain.
 *
 * Chain aliases extend their parent's, so an ancestor is always a prefix of the alias you are
 * standing on, while a sibling branch (`B` seen from `AB`) is not.
 */
export function isReturnBookmark(
	destinationAlias: string | null | undefined,
	oppositeAlias: string | null | undefined,
	ignoredAlias: string | null | undefined,
): boolean {
	const opposite = (oppositeAlias ?? '').trim();
	if (!opposite) return false;
	if (isIgnoredAlias(destinationAlias, ignoredAlias)) return true;

	const destination = (destinationAlias ?? '').trim();
	if (!destination) return false;
	return opposite.toLowerCase().startsWith(destination.toLowerCase());
}

/**
 * `C3` for wormhole space, otherwise `HS` / `LS` / `NS` / `P`. Blank when the destination is
 * unknown, since a security of zero would label an unscanned hole as lowsec.
 */
export function bookmarkClass(wormholeClassId: number | null, security: number | null): string {
	if (wormholeClassId === null && security === null) return '';
	const meta = classMeta(wormholeClassId, security ?? 0);
	if (isWormholeClass(wormholeClassId)) return meta.short;
	return { H: 'HS', L: 'LS', N: 'NS' }[meta.short] ?? meta.short;
}

/** The scanner id's first three characters: `ABC-123` is bookmarked as `ABC`. */
export function shortSignatureId(signatureId: string | null | undefined): string {
	return signatureId ? signatureId.slice(0, 3) : '';
}

export function bookmarkTokens(
	system: BookmarkSystem,
	context: BookmarkContext,
): Record<BookmarkToken, string> {
	return {
		alias: system.alias ?? '',
		sig: shortSignatureId(context.signatureId),
		class: bookmarkClass(system.wormholeClassId, system.security),
		name: system.name,
		region: system.region ?? '',
		occupier: system.occupier ?? '',
		size: context.size ? (SIZE_LABELS[context.size] ?? '') : '',
		wh: context.wormholeCode ?? '',
		mass: context.massStatus ? (MASS_LABELS[context.massStatus] ?? '') : '',
		life: context.timeStatus ? (LIFE_LABELS[context.timeStatus] ?? '') : '',
	};
}

/**
 * Substitute `{token}` placeholders, collapsing the gap empty ones leave. Unknown
 * placeholders are left as written so a typo in the format stays visible.
 */
export function renderBookmark(template: string, values: Record<BookmarkToken, string>): string {
	return template
		.replace(/\{(\w+)\}/g, (match, token: string) =>
			token in values ? values[token as BookmarkToken] : match,
		)
		.replace(/\s+/g, ' ')
		.trim();
}

/**
 * The bookmark name for a system across a connection, using the map's formats.
 *
 * `oppositeAlias` is the alias at the other end: when it marks this bookmark as pointing back
 * up the chain, the return format replaces the wormhole/k-space choice.
 */
export function formatBookmark(
	system: BookmarkSystem,
	context: BookmarkContext,
	formats?: BookmarkFormats | null,
	oppositeAlias?: string | null,
): string {
	const ignored = formats?.ignoredAlias ?? DEFAULT_IGNORED_ALIAS;
	const template = isReturnBookmark(system.alias, oppositeAlias, ignored)
		? formats?.return || DEFAULT_FORMAT_RETURN
		: isWormholeClass(system.wormholeClassId)
			? formats?.wormhole || DEFAULT_FORMAT_WORMHOLE
			: formats?.kspace || DEFAULT_FORMAT_KSPACE;

	return renderBookmark(template, bookmarkTokens(system, context));
}

// Connection bookmark names. The name is read from the in-game bookmark list, so it has to
// say where the hole goes without the map open. Formats are per map because every group
// names their chain differently.

import { isIgnoredAlias } from '$lib/naming/alias';
import { classMeta, isWormholeClass } from '$lib/map/classes';
import type { MapConnection } from '$lib/api/types/MapConnection';
import type { MapNaming } from '$lib/api/types/MapNaming';
import type { MapSystemView } from '$lib/api/types/MapSystemView';
import type { Signature } from '$lib/api/types/Signature';
import type { SignatureTypeInfo } from '$lib/api/types/SignatureTypeInfo';
import type { MappedSystem } from '$lib/map/system';
import type { MassStatus } from '$lib/api/types/MassStatus';
import type { TimeStatus } from '$lib/api/types/TimeStatus';
import type { WormholeSize } from '$lib/api/types/WormholeSize';

export const BOOKMARK_TOKENS = [
	'alias',
	'current',
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
	/** The map's home system: a bookmark pointing there is the way home, alias or not. */
	isHome: boolean;
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

/** `currentAlias` is the system the bookmark is made in, for `{current}`. */
export function bookmarkTokens(
	system: BookmarkSystem,
	context: BookmarkContext,
	currentAlias?: string | null,
): Record<BookmarkToken, string> {
	return {
		alias: system.alias ?? '',
		current: currentAlias?.trim() ?? '',
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
 * `oppositeAlias` is the alias at the other end, where the bookmark is made: it fills
 * `{current}`, and when it marks this bookmark as pointing back up the chain, or the system
 * is home, the return format replaces the wormhole/k-space choice.
 */
export function formatBookmark(
	system: BookmarkSystem,
	context: BookmarkContext,
	formats?: BookmarkFormats | null,
	oppositeAlias?: string | null,
): string {
	const ignored = formats?.ignoredAlias ?? DEFAULT_IGNORED_ALIAS;
	const template =
		system.isHome || isReturnBookmark(system.alias, oppositeAlias, ignored)
			? formats?.return || DEFAULT_FORMAT_RETURN
			: isWormholeClass(system.wormholeClassId)
				? formats?.wormhole || DEFAULT_FORMAT_WORMHOLE
				: formats?.kspace || DEFAULT_FORMAT_KSPACE;

	return renderBookmark(template, bookmarkTokens(system, context, oppositeAlias));
}

/** A placed system as a bookmark describes it. */
export function bookmarkSystemOf(system: MappedSystem): BookmarkSystem {
	return {
		alias: system.alias,
		name: system.name,
		region: system.region,
		wormholeClassId: system.wormhole_class_id,
		security: system.security_status,
		occupier: system.occupying_group,
		isHome: system.is_home,
	};
}

/** The wormhole code (`H296`) of a signature type, when the catalog knows it. */
export type WormholeCodeOf = (signatureTypeId: number | null) => string | null;

/**
 * The bookmark for each end of a connection, as it would be written in game. A bookmark is
 * made while standing at the hole, so the name for one end carries the signature read on
 * the other: the far side is what the name has to describe, the near side is where the
 * scanner was. A ghost end has no system to name, so it gets none.
 */
export function connectionBookmarks(
	connection: MapConnection,
	systems: readonly MapSystemView[],
	signatures: readonly Signature[],
	codeOf: WormholeCodeOf,
	formats: BookmarkFormats | null,
): string[] {
	const placed = (id: number) => systems.find((s) => s.id === id) ?? null;
	const from = placed(connection.from_system);
	const to = placed(connection.to_system);
	if (from === null || to === null) return [];

	const nameFor = (end: MapSystemView, near: MapSystemView): string | null => {
		if (end.kind !== 'system') return null;
		const sig =
			near.kind === 'system'
				? (signatures.find(
						(s) => s.connection_id === connection.id && s.solar_system_id === near.solar_system_id,
					) ?? null)
				: null;
		return formatBookmark(
			bookmarkSystemOf(end),
			{
				signatureId: sig?.signature_id ?? null,
				size: connection.size,
				massStatus: connection.mass_status,
				timeStatus: connection.time_status,
				wormholeCode: sig ? codeOf(sig.signature_type_id) : null,
			},
			formats,
			near.alias,
		);
	};
	return [nameFor(from, to), nameFor(to, from)].filter((name) => name !== null);
}

/**
 * The bookmark a scanned signature copies: where its hole leads, named from the system it
 * was scanned in. Until the hole is mapped, the class its type promises is all there is,
 * and the name and region stay blank rather than borrowing the near side's.
 */
export function signatureBookmark(
	sig: Signature,
	type: Pick<SignatureTypeInfo, 'signature' | 'target_class'> | null,
	far: MappedSystem | null,
	alias: string | null,
	scannedFromAlias: string | null,
	formats: BookmarkFormats | null,
): string {
	return formatBookmark(
		{
			alias,
			name: far?.name ?? '',
			region: far?.region ?? null,
			wormholeClassId: far?.wormhole_class_id ?? type?.target_class ?? null,
			security: far?.security_status ?? null,
			occupier: far?.occupying_group ?? null,
			isHome: far?.is_home ?? false,
		},
		{
			signatureId: sig.signature_id,
			size: sig.size,
			massStatus: sig.mass_status,
			timeStatus: sig.time_status,
			wormholeCode: type?.signature ?? null,
		},
		formats,
		scannedFromAlias,
	);
}

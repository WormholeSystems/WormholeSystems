import type { MapConnection } from '$lib/api/types/MapConnection';
import type { MapNaming } from '$lib/api/types/MapNaming';
import type { MapSystemView } from '$lib/api/types/MapSystemView';
import type { Signature } from '$lib/api/types/Signature';
import type { UpdateSignature } from '$lib/api/types/UpdateSignature';

/**
 * The fields a row may change. Taken from the wire type rather than restated, so a field
 * that changes shape on the server stops compiling here; an open dictionary would have
 * accepted a misspelled key and quietly changed nothing.
 */
export type SignaturePatch = Omit<UpdateSignature, 'map_id' | 'signature_pk'>;

/**
 * What the signature row and its inputs need from the map around them: the placements and
 * edges they resolve names against, and the writes they can make.
 *
 * They used to take the whole `MapState`, which meant they could only ever be rendered on
 * a live map. Naming the surface here keeps every write in one place, lets the row be
 * tested against plain arrays, and lets the landing page render the real component from
 * static data instead of a look-alike that drifts.
 */
export interface SignatureContext {
	systems: MapSystemView[];
	connections: MapConnection[];
	/** The same placements and edges by id, built once, since every row looks its own up. */
	systemById: ReadonlyMap<number, MapSystemView>;
	connectionById: ReadonlyMap<number, MapConnection>;
	sigs: Signature[];
	/** How the chain is named, for the bookmark a row copies. */
	naming: MapNaming | null;
	/** Absent in a read-only rendering; with no actions nothing can be written. */
	actions?: SignatureActions;
}

export interface SignatureActions {
	update(signaturePk: number, patch: SignaturePatch): void;
	remove(signaturePk: number): void;
	link(signaturePk: number, connectionId: number): void;
	unlink(signaturePk: number): void;
	setPreserveMass(connectionId: number, preserve: boolean): void;
	/** Name a placement, so the alias a copied bookmark guessed is what the map says. */
	setAlias(system: MapSystemView, alias: string): void;
}

/** The map's domain namespaces, narrowed to what the signature rows use. */
interface SignatureHost {
	readonly naming: MapNaming | null;
	systems: {
		readonly all: MapSystemView[];
		readonly byId: ReadonlyMap<number, MapSystemView>;
		rename(system: MapSystemView, alias: string | null, occupier: string | null): void;
	};
	connections: {
		readonly all: MapConnection[];
		readonly byId: ReadonlyMap<number, MapConnection>;
		patch(connectionId: number, patch: { preserve_mass: boolean }): void;
	};
	signatures: { readonly all: Signature[] } & Omit<
		SignatureActions,
		'setPreserveMass' | 'setAlias'
	>;
}

/**
 * The live context over a map. The rows and their inputs take this rather than the whole
 * map, so the same components render from static data elsewhere; every write they can
 * make is named here.
 */
export function makeSignatureContext(map: SignatureHost): SignatureContext {
	return {
		get systems() {
			return map.systems.all;
		},
		get connections() {
			return map.connections.all;
		},
		get systemById() {
			return map.systems.byId;
		},
		get connectionById() {
			return map.connections.byId;
		},
		get sigs() {
			return map.signatures.all;
		},
		get naming() {
			return map.naming;
		},
		actions: {
			update: (signaturePk, patch) => map.signatures.update(signaturePk, patch),
			remove: (signaturePk) => map.signatures.remove(signaturePk),
			link: (signaturePk, connectionId) => map.signatures.link(signaturePk, connectionId),
			unlink: (signaturePk) => map.signatures.unlink(signaturePk),
			setPreserveMass: (connectionId, preserve) =>
				map.connections.patch(connectionId, { preserve_mass: preserve }),
			// A ghost owns only its alias; a real system keeps its occupier as it is.
			setAlias: (system, alias) =>
				map.systems.rename(system, alias, system.kind === 'system' ? system.occupying_group : null),
		},
	};
}

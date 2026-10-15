// Building the chain as you fly it. Watching the active character's system is the whole
// trick, and every guard here exists because something other than a jump also changes that
// number: switching character, logging in, taking a gate. Anything ambiguous is dropped
// rather than guessed at.

import { ghostAliases, ghostsFrom, existingConnection } from '$lib/map/ghosts';
import { detectJump } from '$lib/map/jump-detection';
import { trackedPilots } from '$lib/map/tracked-pilots';
import { solarSystemId, type MappedSystem } from '$lib/map/system';
import type { CharacterRef } from '$lib/api/types/CharacterRef';
import type { GridConfig } from '$lib/api/types/GridConfig';
import type { MapConnection } from '$lib/api/types/MapConnection';
import type { MapNaming } from '$lib/api/types/MapNaming';
import type { MapSystemView } from '$lib/api/types/MapSystemView';
import type { MapUserSettings } from '$lib/api/types/MapUserSettings';
import type { ResolveGhostSystem } from '$lib/api/types/ResolveGhostSystem';
import type { Signature } from '$lib/api/types/Signature';
import type { SignatureCatalog } from '$lib/api/types/SignatureCatalog';
import type { SystemSearchResult } from '$lib/api/types/SystemSearchResult';
import type { TrackJump } from '$lib/api/types/TrackJump';
import { aliasTargetKind, suggestAlias } from '$lib/naming/alias';
import { freePosition } from '$lib/map/geometry';
import { sizeForJumpMass } from '$lib/map/mass';
import { classMeta, isWormholeClass } from '$lib/map/classes';
import { typeById } from '$lib/map/signatures';
import { groupSignatures, type SignatureGroups } from '$lib/signatures/compatibility';

/** A jump waiting on the user to say which signature it was. */
export interface JumpPrompt {
	/** Who flew it, for the pilot who is mapping with several. */
	pilot: string;
	origin: MappedSystem;
	targetSolarSystemId: number;
	targetName: string;
	targetClassId: number | null;
	targetSecurity: number;
	/** The placement the target already has on the map, if it was reached another way. */
	existing: MappedSystem | null;
	groups: SignatureGroups;
	catalog: SignatureCatalog;
	suggestedAlias: string | null;
	/**
	 * The names already given to the nodes these signatures are drawn as, by signature. A
	 * hole scanned and named before anyone flew it keeps that name when the jump finally
	 * says what it is, rather than asking for it a second time.
	 */
	ghostAliases: Map<number, string>;
	/** Where the new system would go, decided when the jump happened. */
	at: { x: number; y: number };
}

interface Jump {
	from: number;
	to: number;
	pilot: string;
}

/** The mass and lifetime a hole starts at, taken from the signature if it was scanned. */
function statusFromSignature(signature: Signature | null) {
	return {
		size: signature?.size ?? null,
		mass_status: signature?.mass_status ?? null,
		time_status: signature?.time_status ?? null,
	};
}

/**
 * What the tracker reads off the map, and the commands a decided jump issues. Narrow on
 * purpose, like [`LayoutHost`]: a test hands in a plain object, and the commands carry no
 * eagerly-built request promises.
 */
export interface TrackerHost {
	myCharacters(): CharacterRef[];
	systems(): MapSystemView[];
	connections(): MapConnection[];
	sigs(): Signature[];
	grid(): GridConfig;
	settings(): Pick<
		MapUserSettings,
		'tracking_allowed' | 'prompt_for_signature' | 'suggest_alias' | 'tracked_character_ids'
	> | null;
	naming(): MapNaming | null;
	/** The map's ignore list: a jump into one of these is flown, not mapped. */
	ignored(): Set<number>;
	stargates(): Map<number, number[]> | null;
	whenRoutingLoaded(): Promise<void>;
	loadCatalog(): Promise<SignatureCatalog>;
	resolveSystem(id: number): Promise<SystemSearchResult | undefined>;
	/** Places the system, connects it, and links the signature in one undoable step. */
	trackJump(cmd: Omit<TrackJump, 'map_id'>): void;
	/** The ghost turned out to be a real system: name it where it already sits. */
	resolveGhost(cmd: Omit<ResolveGhostSystem, 'map_id'>): void;
}

export class JumpTracker {
	private map: TrackerHost;

	prompt = $state<JumpPrompt | null>(null);

	// What the last poll saw of each tracked pilot, so a change can be told apart from the
	// first reading.
	private seen = new Map<number, number | null>();
	// Jumps that landed while a prompt was open, asked about in turn.
	private queued: Jump[] = [];

	constructor(map: TrackerHost) {
		this.map = map;
	}

	private get enabled(): boolean {
		return this.map.settings()?.tracking_allowed === true;
	}

	/** Diff each tracked pilot's fresh reading against the last; [`detectJump`] holds the rules. */
	observe() {
		const pilots = trackedPilots(
			this.map.myCharacters(),
			this.map.settings()?.tracked_character_ids ?? [],
		);
		const jumps: Jump[] = [];
		const seen = new Map<number, number | null>();
		for (const pilot of pilots) {
			const characterId = pilot.character_id;
			const next = {
				characterId,
				systemId: pilot.online ? (pilot.solar_system_id ?? null) : null,
			};
			const prev = { characterId, systemId: this.seen.get(characterId) ?? null };
			seen.set(characterId, next.systemId);
			const jump = detectJump(prev, next);
			if (jump) jumps.push({ ...jump, pilot: pilot.name });
		}
		// A pilot dropped from the set starts over when brought back, rather than arriving
		// from wherever they were last seen.
		this.seen = seen;

		if (!this.enabled) return;
		for (const jump of jumps) void this.handleJump(jump);
	}

	private async handleJump({ from: fromSystemId, to: toSystemId, pilot }: Jump) {
		const map = this.map;
		// Only a jump *out of* the mapped chain extends it. Flying around known space with
		// the map open should not start drawing it.
		const origin = map.systems().find((s) => solarSystemId(s) === fromSystemId) ?? null;
		// Only a system can be jumped out of; a hole nobody has been through is nowhere yet.
		if (origin?.kind !== 'system') return;

		// A gate jump is not a new hole, so the gate table has to be in before judging one.
		await map.whenRoutingLoaded();
		if (map.stargates()?.get(fromSystemId)?.includes(toSystemId)) return;

		const arrival = map.systems().find((s) => solarSystemId(s) === toSystemId);
		const existing = arrival?.kind === 'system' ? arrival : null;
		// A trade hub the map keeps off stays off. Placed by hand it is somebody's choice,
		// and the jump into it is worth recording.
		if (!existing && map.ignored().has(toSystemId)) return;
		const linked = existing
			? existingConnection(origin, existing, map.connections(), map.sigs())
			: null;
		// Already mapped and already explained: there is nothing left to record.
		if (linked?.signature) return;

		const catalog = await map.loadCatalog();
		const originSignatures = map.sigs().filter((s) => s.solar_system_id === fromSystemId);
		const target = await this.describeTarget(toSystemId, existing);
		if (!target) return;

		const ghosts = ghostsFrom(origin, map.systems(), map.connections());
		const groups = groupSignatures(
			originSignatures,
			new Map(catalog.types.map((t) => [t.id, t])),
			target.classId,
			new Set(ghosts.keys()),
		);

		// Nothing to ask about: the question is off, or no signature on the origin could be
		// this hole.
		if (!map.settings()?.prompt_for_signature || groups.likely.length === 0) {
			if (linked) return;
			// Nobody to ask, but the chain already says there is exactly one unflown hole
			// here: that is the one just flown.
			if (ghosts.size === 1) {
				const [ghost] = ghosts.values();
				this.resolve(ghost, toSystemId);
				return;
			}
			this.submit({
				origin,
				targetSolarSystemId: toSystemId,
				signaturePk: null,
				alias: this.suggestAliasFor(origin, target, existing),
				at: this.placeNear(origin),
			});
			return;
		}

		// One question at a time: a second pilot's jump waits its turn.
		if (this.prompt !== null) {
			this.queued.push({ from: fromSystemId, to: toSystemId, pilot });
			return;
		}
		this.prompt = {
			pilot,
			origin,
			targetSolarSystemId: toSystemId,
			targetName: target.name,
			targetClassId: target.classId,
			targetSecurity: target.security,
			existing,
			groups,
			catalog,
			suggestedAlias: this.suggestAliasFor(origin, target, existing),
			ghostAliases: ghostAliases(ghosts, originSignatures, map.systems()),
			at: this.placeNear(origin),
		};
	}

	/** A system already on the map carries its own class; anywhere else has to be resolved. */
	private async describeTarget(id: number, existing: MappedSystem | null) {
		if (existing) {
			return {
				name: existing.name,
				classId: existing.wormhole_class_id,
				security: existing.security_status,
			};
		}
		const hit = await this.map.resolveSystem(id);
		if (!hit) return null;
		return { name: hit.name, classId: hit.wormhole_class_id, security: hit.security };
	}

	private suggestAliasFor(
		origin: MappedSystem,
		target: { classId: number | null; security: number },
		existing: MappedSystem | null,
	): string | null {
		// A system already on the map keeps the name the chain knows it by.
		if (existing?.alias) return existing.alias;
		if (!this.map.settings()?.suggest_alias) return null;

		const naming = this.map.naming();
		const targetIsWormhole = isWormholeClass(target.classId);
		return suggestAlias({
			parentAlias: origin.alias,
			targetIsWormhole,
			originIsWormhole: isWormholeClass(origin.wormhole_class_id),
			aliases: this.map
				.systems()
				.map((s) => s.alias)
				.filter((alias): alias is string => Boolean(alias)),
			scheme: naming?.alias_scheme,
			targetKind: aliasTargetKind(
				targetIsWormhole,
				classMeta(target.classId, target.security).short,
			),
			ignoredAlias: naming?.ignored_alias,
		});
	}

	/** The first free slot beside the system jumped from. */
	private placeNear(origin: MappedSystem): { x: number; y: number } {
		return freePosition(
			this.map.systems(),
			{ x: origin.position_x, y: origin.position_y },
			this.map.grid(),
		);
	}

	/** One command, so a mis-picked signature is one undo. */
	submit(choice: {
		origin: MappedSystem;
		targetSolarSystemId: number;
		signaturePk: number | null;
		alias: string | null;
		at: { x: number; y: number };
		size?: Signature['size'];
		massStatus?: Signature['mass_status'];
		timeStatus?: Signature['time_status'];
	}) {
		const signature =
			choice.signaturePk === null
				? null
				: (this.map.sigs().find((s) => s.id === choice.signaturePk) ?? null);

		const fromSignature = statusFromSignature(signature);
		const catalog = this.prompt?.catalog ?? null;
		const type = signature && catalog ? typeById(catalog, signature.signature_type_id) : null;
		const size =
			choice.size ?? sizeForJumpMass(type?.max_jump_mass) ?? fromSignature.size ?? undefined;

		// That signature is already drawn as an unflown hole: this jump says where it goes,
		// and carries everything the dialog was told about it.
		if (signature?.connection_id != null) {
			const ghost = ghostsFrom(choice.origin, this.map.systems(), this.map.connections()).get(
				signature.connection_id,
			);
			if (ghost !== undefined) {
				this.resolve(ghost, choice.targetSolarSystemId, {
					alias: choice.alias?.trim() || undefined,
					size,
					mass_status: choice.massStatus ?? fromSignature.mass_status ?? undefined,
					time_status: choice.timeStatus ?? fromSignature.time_status ?? undefined,
				});
				return;
			}
		}

		// Nothing picked, and the hole is already drawn: there is nothing left to record, and
		// asking the server to add it again is only refused.
		if (signature === null) {
			const arrival = this.map
				.systems()
				.find((s) => solarSystemId(s) === choice.targetSolarSystemId);
			if (
				arrival?.kind === 'system' &&
				existingConnection(choice.origin, arrival, this.map.connections(), this.map.sigs())
			) {
				return;
			}
		}

		this.map.trackJump({
			from_map_solar_system_id: choice.origin.id,
			to_solar_system_id: choice.targetSolarSystemId,
			x: choice.at.x,
			y: choice.at.y,
			// The optional fields are omitted rather than nulled: absent means "nothing
			// known", which is what an unscanned hole actually is.
			signature_pk: choice.signaturePk ?? undefined,
			alias: choice.alias?.trim() || undefined,
			size,
			mass_status: choice.massStatus ?? fromSignature.mass_status ?? undefined,
			time_status: choice.timeStatus ?? fromSignature.time_status ?? undefined,
		});
	}

	private resolve(
		ghostPlacementId: number,
		solarSystemId: number,
		details: Partial<ResolveGhostSystem> = {},
	) {
		this.map.resolveGhost({
			...details,
			map_solar_system_id: ghostPlacementId,
			solar_system_id: solarSystemId,
		});
	}

	dismiss() {
		this.prompt = null;
		// Asked again from the map as it is now, not as it was when the jump landed.
		const next = this.queued.shift();
		if (next) void this.handleJump(next);
	}
}

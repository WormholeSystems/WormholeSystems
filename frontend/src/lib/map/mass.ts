// Wormhole mass: how it is written, what is left of it, and which ship size it admits.

import type { MapSystemView } from '$lib/api/types/MapSystemView';
import type { WormholeSize } from '$lib/api/types/WormholeSize';
import { isWormholeClass } from '$lib/map/classes';
import { SIZE_OPTIONS } from '$lib/map/connection-status';
import type { MappedSystem } from '$lib/map/system';
import { ccpRoundSecurity } from '$lib/security';

/** kg → kilotons, `en-US` grouping, one decimal, no unit suffix (legacy formatKilotons). */
export function formatKt(kg: number): string {
	return (kg / 1_000_000).toLocaleString('en-US', { maximumFractionDigits: 1 });
}

/**
 * What is left of a hole's estimated total mass after the logged jumps, clamped at empty.
 * Null when the total is unknown, so the caller can leave the bar out entirely.
 */
export function remainingMass(
	totalKg: number | null,
	jumpedKg: number,
): { kg: number; percent: number } | null {
	if (totalKg === null || totalKg <= 0) return null;
	return {
		kg: Math.max(0, totalKg - jumpedKg),
		percent: Math.max(0, 100 - (jumpedKg / totalKg) * 100),
	};
}

/**
 * The wormhole size a hole of this jump mass admits; an identified hole dictates its own.
 * The game's jump limits are 5M (frigates), 62M (cruisers), 300M or 375M (battleships) and
 * 1B and up (capitals and freighters), so each band ends at its largest limit.
 */
export function sizeForJumpMass(kg: number | null | undefined): WormholeSize | null {
	if (!kg || kg <= 0) return null;
	if (kg <= 5_000_000) return 'small';
	if (kg <= 62_000_000) return 'medium';
	if (kg <= 375_000_000) return 'large';
	return 'xl';
}

/** Short ship-size label for a connection's size, as the edge badge shows it. */
export function sizeLetter(size: WormholeSize): string {
	return SIZE_OPTIONS.find((o) => o.value === size)!.letter;
}

/** The size letter for a jump mass, so a hole's details read the same as its edge badge. */
export function jumpMassLetter(kg: number | null | undefined): string {
	const size = sizeForJumpMass(kg);
	return size ? sizeLetter(size) : '—';
}

/**
 * A guess, not a measurement: the pair of classes rules some sizes out, and the rest is
 * left unset for whoever scans it.
 */
export function heuristicSize(
	systems: MapSystemView[],
	fromId: number,
	toId: number,
): WormholeSize | undefined {
	const a = systems.find((s) => s.id === fromId);
	const b = systems.find((s) => s.id === toId);
	// Nobody has been through an unmapped hole, so there is nothing about it to guess from.
	if (a?.kind !== 'system' || b?.kind !== 'system') return undefined;
	const TURNUR = 30002086;
	const classes = [a.wormhole_class_id, b.wormhole_class_id];
	if (classes.includes(13)) return 'small';
	if (classes.includes(1)) return 'medium';
	const highsec = (s: MappedSystem) =>
		s.wormhole_class_id === 7 || ccpRoundSecurity(s.security_status) >= 0.5;
	const thera = (s: MappedSystem) => s.wormhole_class_id === 12;
	const wh = (s: MappedSystem) => isWormholeClass(s.wormhole_class_id);
	if ((thera(a) && highsec(b)) || (thera(b) && highsec(a))) return 'medium';
	if ((a.solar_system_id === TURNUR && wh(b)) || (b.solar_system_id === TURNUR && wh(a))) {
		return 'medium';
	}
	return undefined;
}

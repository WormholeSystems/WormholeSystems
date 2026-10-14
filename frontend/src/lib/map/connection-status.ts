// The one vocabulary for a wormhole connection's degradable statuses: what each state is
// called, the threshold it stands for, and the colors it is drawn in.

import type { MassStatus } from '$lib/api/types/MassStatus';
import type { TimeStatus } from '$lib/api/types/TimeStatus';
import type { WormholeSize } from '$lib/api/types/WormholeSize';

export interface StatusOption<T> {
	value: T;
	label: string;
	hint: string | null;
	/** For inline-styled dots and edge badges. */
	color: string;
	/** For Tailwind-styled dots. */
	dot: string;
}

const HEALTHY_DOT = 'oklch(55.6% 0.007 260)';
const ROUTE_STROKE = '#f59e0b'; // amber-500
const STARGATE_STROKE = '#0ea5e9'; // sky-500

export const LIFETIME_OPTIONS: StatusOption<TimeStatus>[] = [
	{ value: 'stable', label: 'Healthy', hint: null, color: HEALTHY_DOT, dot: 'bg-neutral-500' },
	{ value: 'eol', label: 'End of Life', hint: '< 4h', color: '#a855f7', dot: 'bg-purple-500' },
	{ value: 'critical', label: 'Critical', hint: '< 1h', color: '#ef4444', dot: 'bg-red-500' },
	// In game "Expired, closure imminent": red like critical, and pulsing so the two still
	// read apart.
	{
		value: 'expired',
		label: 'Expired',
		hint: 'imminent',
		color: '#ef4444',
		dot: 'bg-red-500 motion-safe:animate-pulse',
	},
];

export const MASS_OPTIONS: StatusOption<MassStatus>[] = [
	{ value: 'stable', label: 'Fresh', hint: '≥ 50%', color: HEALTHY_DOT, dot: 'bg-neutral-500' },
	// Orange, not amber: amber is the route highlight, and a reduced hole on the route has to
	// read apart from it.
	{ value: 'reduced', label: 'Reduced', hint: '< 50%', color: '#f97316', dot: 'bg-orange-500' },
	{ value: 'critical', label: 'Critical', hint: '≤ 15%', color: '#ef4444', dot: 'bg-red-500' },
];

/**
 * The signature row's mass picker speaks the same states with "X Mass" labels, and its
 * "fresh" choice means no status at all rather than a recorded healthy one, hence the
 * `unknown` value. Derived here so the colors and thresholds live once.
 */
export const SIGNATURE_MASS_OPTIONS = MASS_OPTIONS.map((o) => ({
	value: o.value === 'stable' ? 'unknown' : o.value,
	label: `${o.label} Mass`,
	dot: o.dot,
}));

// `small` is what EVE calls frigate-sized.
export const SIZE_OPTIONS: { value: WormholeSize; label: string; letter: string }[] = [
	{ value: 'small', label: 'Frigate', letter: 'S' },
	{ value: 'medium', label: 'Medium', letter: 'M' },
	{ value: 'large', label: 'Large', letter: 'L' },
	{ value: 'xl', label: 'Extra Large', letter: 'XL' },
];

/** The edge badge color for a degraded lifetime; a healthy one draws no badge. */
export function timeBadgeColor(status: TimeStatus | null): string | null {
	if (status === null || status === 'stable') return null;
	return LIFETIME_OPTIONS.find((o) => o.value === status)?.color ?? null;
}

/** The edge badge color for a degraded mass status; a fresh one draws no badge. */
export function massBadgeColor(status: MassStatus | null): string | null {
	if (status === null || status === 'stable') return null;
	return MASS_OPTIONS.find((o) => o.value === status)?.color ?? null;
}

const optionColor = <T>(options: StatusOption<T>[], value: T) =>
	options.find((o) => o.value === value)!.color;

/**
 * The connection's stroke by precedence: on the route, a stargate, critical (or expired),
 * reduced, end of life, else neutral. The degraded colors are the badges' own.
 */
export function edgeColor(
	kind: 'wormhole' | 'stargate',
	mass: MassStatus | null,
	time: TimeStatus | null,
	onRoute: boolean,
): string {
	if (onRoute) return ROUTE_STROKE;
	if (kind === 'stargate') return STARGATE_STROKE;
	if (mass === 'critical') return optionColor(MASS_OPTIONS, 'critical');
	if (time === 'critical' || time === 'expired') return optionColor(LIFETIME_OPTIONS, time);
	if (mass === 'reduced') return optionColor(MASS_OPTIONS, 'reduced');
	if (time === 'eol') return optionColor(LIFETIME_OPTIONS, 'eol');
	return 'var(--color-edge)';
}

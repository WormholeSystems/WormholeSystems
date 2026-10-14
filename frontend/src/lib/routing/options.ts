// The route-calculation vocabulary, shared between the routing settings page (full copy)
// and the map's compact route-settings popover (short copy). One table per concept, so the
// two surfaces cannot drift apart; both copies are preserved verbatim as separate columns.

import type { MassStatus } from '$lib/api/types/MassStatus';
import type { RoutePreference } from '$lib/api/types/RoutePreference';
import type { TimeStatus } from '$lib/api/types/TimeStatus';

export interface RouteOption<T> {
	value: T;
	label: string;
	hint: string;
	shortLabel: string;
	shortHint: string;
}

export const ROUTE_PREFS: RouteOption<RoutePreference>[] = [
	{
		value: 'shorter',
		label: 'Shortest',
		hint: 'Fewest jumps, whatever the security',
		shortLabel: 'Shortest',
		shortHint: 'Min jumps',
	},
	{
		value: 'safer',
		label: 'Safer',
		hint: 'Prefers high security',
		shortLabel: 'Safer',
		shortHint: 'High-sec',
	},
	{
		value: 'less_secure',
		label: 'Less secure',
		hint: 'Prefers low and null',
		shortLabel: 'Less Secure',
		shortHint: 'Low-sec',
	},
];

export const ROUTE_LIFETIMES: RouteOption<TimeStatus>[] = [
	{
		value: 'expired',
		label: 'Anything',
		hint: 'Including holes closing any moment',
		shortLabel: 'Expired',
		shortHint: 'imminent',
	},
	{
		value: 'critical',
		label: 'Not expired',
		hint: 'Including holes about to collapse',
		shortLabel: 'Critical',
		shortHint: '< 1 hour',
	},
	{
		value: 'eol',
		label: 'Not critical',
		hint: 'Avoids the last hour',
		shortLabel: 'End of Life',
		shortHint: '< 4 hours',
	},
	{
		value: 'stable',
		label: 'Healthy only',
		hint: 'Avoids end-of-life holes',
		shortLabel: 'Healthy Only',
		shortHint: '> 4 hours',
	},
];

// The critical threshold reads "< 10%" here and "≤ 15%" in the connection vocabulary
// (lib/map/connection-status.ts); left as found, flagged rather than silently unified.
export const ROUTE_MASSES: RouteOption<MassStatus>[] = [
	{
		value: 'critical',
		label: 'Anything',
		hint: 'Including nearly-collapsed holes',
		shortLabel: 'Critical Mass',
		shortHint: '< 10%',
	},
	{
		value: 'reduced',
		label: 'Not critical',
		hint: 'Avoids the last 10%',
		shortLabel: 'Reduced Mass',
		shortHint: '< 50%',
	},
	{
		value: 'stable',
		label: 'Fresh only',
		hint: 'Avoids reduced holes',
		shortLabel: 'High Mass',
		shortHint: '> 50%',
	},
];

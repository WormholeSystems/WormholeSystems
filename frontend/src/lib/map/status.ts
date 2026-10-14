// The system-status icon set, shared by the node and the context menu.
import ActivityIcon from '@lucide/svelte/icons/activity';
import CircleDashedIcon from '@lucide/svelte/icons/circle-dashed';
import CircleHelpIcon from '@lucide/svelte/icons/circle-help';
import RadarIcon from '@lucide/svelte/icons/radar';
import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';
import SkullIcon from '@lucide/svelte/icons/skull';

import type { SystemStatus } from '$lib/api/types/SystemStatus';

export const STATUS_ICONS = {
	friendly: ShieldCheckIcon,
	hostile: SkullIcon,
	active: ActivityIcon,
	unscanned: RadarIcon,
	empty: CircleDashedIcon,
	unknown: CircleHelpIcon,
} satisfies Record<SystemStatus, typeof ShieldCheckIcon>;

export const STATUS_OPTIONS: SystemStatus[] = [
	'unknown',
	'friendly',
	'hostile',
	'active',
	'unscanned',
	'empty',
];

export function statusLabel(status: SystemStatus): string {
	return status[0].toUpperCase() + status.slice(1);
}

/** The node border / status icon color for a system's intel status. */
export function statusColor(status: SystemStatus): string {
	return `var(--color-status-${status})`;
}

// The alert form's pure half: what the fields amount to, and whether they are enough.

import type { AlertDelivery } from '$lib/api/types/AlertDelivery';
import type { AlertKind } from '$lib/api/types/AlertKind';
import type { AlertMention } from '$lib/api/types/AlertMention';
import type { Match } from '$lib/api/types/Match';
import type { JumpShip } from '$lib/api/types/JumpShip';
import type { Rule } from '$lib/api/types/Rule';
import type { SaveAlert } from '$lib/api/types/SaveAlert';

export interface AlertDraft {
	name: string;
	kind: AlertKind;
	delivery: AlertDelivery;
	webhookId: number | null;
	mention: AlertMention;
	roleRef: number | null;
	target: number | null;
	/** Proximity only: measure from here through the chain instead of from the nearest mapped system. */
	origin: number | null;
	maxJumps: number;
	shipType: JumpShip;
	jdcLevel: number;
	filters: Rule[];
	filterMatch: Match;
}

/** A comma-separated id list as the ids it names; anything not a positive number drops. */
export function parseIds(value: string): number[] {
	return value
		.split(',')
		.map((part) => Number(part.trim()))
		.filter((id) => Number.isFinite(id) && id > 0);
}

/** Whether the draft can be saved: a name, a target where the kind needs one, and a wired delivery. */
export function isValidAlert(draft: AlertDraft): boolean {
	return (
		draft.name.trim().length > 0 &&
		(draft.kind === 'killmail' || draft.target !== null) &&
		(draft.delivery !== 'webhook' || draft.webhookId !== null) &&
		(draft.mention !== 'role' || draft.roleRef !== null)
	);
}

/** The save body, with every field the kind does not use stripped rather than nulled. */
export function toSaveAlert(draft: AlertDraft): SaveAlert {
	return {
		name: draft.name.trim(),
		kind: draft.kind,
		delivery: draft.delivery,
		map_webhook_id: draft.delivery === 'webhook' ? (draft.webhookId ?? undefined) : undefined,
		map_webhook_role_id: draft.mention === 'role' ? (draft.roleRef ?? undefined) : undefined,
		mention: draft.mention,
		target_solar_system_id: draft.kind === 'killmail' ? undefined : (draft.target ?? undefined),
		origin_solar_system_id: draft.kind === 'proximity' ? (draft.origin ?? undefined) : undefined,
		max_jumps: draft.maxJumps,
		ship_type: draft.kind === 'jump_range' ? draft.shipType : undefined,
		jdc_level: draft.kind === 'jump_range' ? draft.jdcLevel : undefined,
		filters: draft.kind === 'killmail' ? draft.filters.filter((r) => r.ids.length > 0) : [],
		filter_match: draft.filterMatch,
	};
}

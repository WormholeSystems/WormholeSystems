// The alert vocabulary, shared between the alerts page (labelling saved alerts) and the
// alert form (offering the choices). One table per concept, so the two surfaces cannot
// drift apart.

import type { AlertDelivery } from '$lib/api/types/AlertDelivery';
import type { AlertKind } from '$lib/api/types/AlertKind';
import type { AlertMention } from '$lib/api/types/AlertMention';
import type { DisabledReason } from '$lib/api/types/DisabledReason';
import type { JumpShip } from '$lib/api/types/JumpShip';
import type { Side } from '$lib/api/types/Side';
import type { Subject } from '$lib/api/types/Subject';

export const ALERT_KINDS: { value: AlertKind; label: string; blurb: string }[] = [
	{
		value: 'killmail',
		label: 'Kills near the chain',
		blurb: 'Every kill within reach, optionally narrowed to who is involved.',
	},
	{
		value: 'proximity',
		label: 'System near the chain',
		blurb: 'Fires when the chain comes within gate range of a system you name.',
	},
	{
		value: 'jump_range',
		label: 'Capital jump range',
		blurb:
			'Fires when a k-space exit lands within a capital jump of a system you name. ' +
			'Measured in light years for the hull you pick, not in gates.',
	},
];

export function kindLabel(kind: AlertKind): string {
	return ALERT_KINDS.find((k) => k.value === kind)?.label ?? kind;
}

/** `base` is the hull's jump range in light years at JDC 0, mirroring the server. */
export const JUMP_SHIPS: { value: JumpShip; label: string; base: number }[] = [
	{ value: 'dreadnought', label: 'Dreadnought', base: 3.5 },
	{ value: 'carrier', label: 'Carrier', base: 3.5 },
	{ value: 'force_auxiliary', label: 'Force Auxiliary', base: 3.5 },
	{ value: 'supercarrier', label: 'Supercarrier', base: 3.0 },
	{ value: 'titan', label: 'Titan', base: 3.0 },
	{ value: 'jump_freighter', label: 'Jump Freighter', base: 5.0 },
	{ value: 'rorqual', label: 'Rorqual', base: 5.0 },
	{ value: 'black_ops', label: 'Black Ops', base: 4.0 },
];

export function shipLabel(ship: string | null): string | null {
	return JUMP_SHIPS.find((s) => s.value === ship)?.label ?? null;
}

/** The same arithmetic the server does: "JDC 5" means nothing until it is light years. */
export function jumpRangeLy(ship: JumpShip, jdcLevel: number): string {
	const base = JUMP_SHIPS.find((s) => s.value === ship)?.base ?? 3.5;
	return (base * (1 + 0.2 * jdcLevel)).toFixed(1);
}

export const DELIVERY_LABEL = {
	webhook: 'Channel webhook',
	discord_dm: 'Direct message',
	discord_channel: 'Bot channel',
} satisfies Record<AlertDelivery, string>;

/**
 * `label` is the picker's imperative copy, `summary` the saved row's descriptive copy.
 * `creator` is never offered by the form (it filters it out), but a saved alert can
 * carry it, so the row copy knows it.
 */
export const ALERT_MENTIONS: { value: AlertMention; label: string; summary: string }[] = [
	{ value: 'none', label: 'No ping', summary: 'No ping' },
	{ value: 'creator', label: 'Ping the creator', summary: 'Pings the creator' },
	{ value: 'role', label: 'Ping a role', summary: 'Pings a role' },
	{ value: 'everyone', label: 'Ping everyone', summary: 'Pings everyone' },
];

export function mentionSummary(mention: AlertMention): string {
	return ALERT_MENTIONS.find((m) => m.value === mention)?.summary ?? mention;
}

export const RULE_SUBJECTS: { value: Subject; label: string }[] = [
	{ value: 'alliance', label: 'Alliance' },
	{ value: 'corporation', label: 'Corporation' },
	{ value: 'character', label: 'Character' },
	{ value: 'ship_type', label: 'Ship type' },
	{ value: 'ship_group', label: 'Ship group' },
];

export const RULE_SIDES: { value: Side; label: string }[] = [
	{ value: 'either', label: 'either side' },
	{ value: 'victim', label: 'the victim' },
	{ value: 'attacker', label: 'the killer' },
];

/** Why an alert was switched off, when it was not by hand. */
export const DISABLED_REASON = {
	manual: 'Turned off by hand',
	discord_unlinked: 'The creator unlinked their Discord account',
	access_revoked: 'The creator lost access to this map',
	destination_gone: 'Discord rejected the destination: the webhook or channel is gone',
	delivery_failed: 'Too many failed deliveries',
} satisfies Record<DisabledReason, string>;

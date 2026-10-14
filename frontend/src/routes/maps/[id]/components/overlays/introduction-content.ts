// The introduction walkthrough's content and derivations, out of the dialog so the
// component is wiring and markup.

import MousePointerClickIcon from '@lucide/svelte/icons/mouse-pointer-click';
import SignatureIcon from '@lucide/svelte/icons/scan-line';
import type { Component } from 'svelte';

import type { MapUserSettings } from '$lib/api/types/MapUserSettings';
import {
	AUTOMATION,
	settingBlocker,
	settingOn,
	settingValue,
	type AutomationSetting,
} from '$lib/map/user-settings';

export const INTRO_STEPS = [
	{
		title: 'Grant permissions',
		blurb: 'What the map may read from your EVE client. All optional.',
	},
	{ title: 'Choose what it does', blurb: 'How much of the mapping you want done for you.' },
	{ title: 'Ready to fly', blurb: 'Here is where everything ended up.' },
];

export const INTRO_TIPS: { icon: Component; text: string }[] = [
	{
		icon: SignatureIcon,
		text: 'Paste a signature scan anywhere on the map to fill in a system.',
	},
	{
		icon: MousePointerClickIcon,
		text: 'Right-click a system for its menu, and drag between two to connect them.',
	},
];

export function introSummary(
	settings: MapUserSettings | null,
	granted: number,
	total: number,
): { label: string; value: string; good: boolean }[] {
	const sharing = settingValue(settings, 'tracking_allowed');
	const prompting = settingOn(settings, 'prompt_for_signature');
	return [
		{
			label: 'Permissions',
			value: granted === total ? 'All granted' : `${granted} of ${total} granted`,
			good: granted === total,
		},
		{ label: 'Location sharing', value: sharing ? 'On' : 'Off', good: sharing },
		{ label: 'Signature prompt', value: prompting ? 'On' : 'Off', good: prompting },
	];
}

export interface IntroToggle {
	setting: AutomationSetting;
	value: boolean;
	enabled: boolean;
	blocked: string | null;
}

/** The automation the introduction offers, read off the one settings table. */
export function introToggles(
	settings: MapUserSettings | null,
	hasLocation: boolean,
): IntroToggle[] {
	return (['tracking_allowed', 'prompt_for_signature', 'suggest_alias'] as const).map((key) => {
		const blocked = settingBlocker(settings, key, hasLocation);
		return {
			setting: AUTOMATION[key],
			value: settingValue(settings, key),
			enabled: blocked === null,
			blocked,
		};
	});
}

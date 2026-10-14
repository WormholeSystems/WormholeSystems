// The viewer's own settings on a map: what the automation switches are called, what they
// default to and lean on, and the one way any setting is written.

import ClipboardIcon from '@lucide/svelte/icons/clipboard';
import CrosshairIcon from '@lucide/svelte/icons/crosshair';
import EyeIcon from '@lucide/svelte/icons/eye';
import ListChecksIcon from '@lucide/svelte/icons/list-checks';
import SignatureIcon from '@lucide/svelte/icons/scan-line';
import TagIcon from '@lucide/svelte/icons/tag';
import { createMutation, useQueryClient } from '@tanstack/svelte-query';
import { toast } from 'svelte-sonner';

import { api, errorMessage } from '$lib/api/client';
import { key } from '$lib/api/queries';
import type { MapUserSettings } from '$lib/api/types/MapUserSettings';
import type { UpdateMapUserSettings } from '$lib/api/types/UpdateMapUserSettings';
import { MAP_ACTIONS } from '$lib/map/actions';

/** The switches that make the map work for you while you fly. */
export type AutomationKey =
	| 'tracking_allowed'
	| 'prompt_for_signature'
	| 'preselect_signature'
	| 'suggest_alias'
	| 'copy_bookmark'
	| 'follow_character';

export interface AutomationSetting {
	key: AutomationKey;
	label: string;
	/** One line, for the tracking popover. */
	hint: string;
	/** The whole story, for the settings pages and the introduction. */
	body: string;
	icon: typeof EyeIcon;
	/** What the server stores before the viewer has touched it. */
	default: boolean;
	/** The setting this one only means something under. */
	needs: AutomationKey | null;
	/** What a dependent says when this one is what is missing; only the leaned-on have one. */
	missing?: string;
}

/** Said when location sharing itself cannot be turned on. */
export const LOCATION_PERMISSION_MISSING = 'Needs the character location permission from EVE.';

/**
 * The automation switches, once. The defaults are the server's (a viewer with no settings
 * row yet gets these), so the switches read right while the settings are still loading.
 */
export const AUTOMATION: Record<AutomationKey, AutomationSetting> = {
	tracking_allowed: {
		key: 'tracking_allowed',
		label: 'Share my location on this map',
		hint: 'Shows you on the map and builds the chain as you jump.',
		body:
			'Puts you on your system for everyone here, measures distances from where you actually ' +
			'are, and lets the map build the chain as you jump. Revocable at any time.',
		icon: EyeIcon,
		default: false,
		needs: null,
		missing: 'Needs location sharing.',
	},
	prompt_for_signature: {
		key: 'prompt_for_signature',
		label: 'Ask which signature I jumped',
		hint: 'Otherwise the hole is mapped without a signature linked.',
		body:
			'On arriving somewhere new, the map asks which signature the hole was and links it. ' +
			'Without this the connection is still drawn, just unnamed.',
		icon: SignatureIcon,
		default: true,
		needs: 'tracking_allowed',
		missing: 'Needs the signature prompt.',
	},
	preselect_signature: {
		key: 'preselect_signature',
		label: 'Preselect the likeliest signature',
		hint: 'The prompt opens with the best match chosen, so Enter confirms it.',
		body:
			'The prompt opens with the best match already chosen, so working through freshly ' +
			'scanned holes in order is one Enter each. Off, nothing is chosen until you pick it.',
		icon: ListChecksIcon,
		default: false,
		needs: 'prompt_for_signature',
	},
	suggest_alias: {
		key: 'suggest_alias',
		label: 'Name new systems for me',
		hint: "Prefills the next name in the chain's sequence.",
		body:
			"Fills in the next alias from the chain's naming scheme, so holes are named the same " +
			'way by everyone on the map.',
		icon: TagIcon,
		default: true,
		needs: 'tracking_allowed',
	},
	copy_bookmark: {
		key: 'copy_bookmark',
		label: 'Copy a bookmark when I map a hole',
		hint: 'Puts the new bookmark on your clipboard once the jump is mapped.',
		body:
			"Puts the new connection's bookmark name on the clipboard, ready to paste into the " +
			'in-game bookmark dialog.',
		icon: ClipboardIcon,
		default: false,
		needs: 'tracking_allowed',
	},
	follow_character: {
		key: 'follow_character',
		label: 'Follow me',
		hint: 'Selects the system you are in as you fly, so the cards keep up.',
		body:
			'Selects the system you are in as you fly, so the side panels always show where you ' +
			'are rather than where you last clicked.',
		icon: CrosshairIcon,
		default: false,
		needs: 'tracking_allowed',
	},
};

/** The stored value, or the server's default while there is none to read. */
export function settingValue(settings: MapUserSettings | null, key: AutomationKey): boolean {
	return settings?.[key] ?? AUTOMATION[key].default;
}

/**
 * Why a switch cannot be used right now, or null when it can. The chain it leans on is
 * checked from the root down, so the reason names the first thing to turn on: a
 * preselection with location sharing off asks for sharing, not for the prompt.
 */
export function settingBlocker(
	settings: MapUserSettings | null,
	key: AutomationKey,
	hasLocation = true,
): string | null {
	const chain: AutomationKey[] = [];
	for (let need = AUTOMATION[key].needs; need; need = AUTOMATION[need].needs) {
		chain.unshift(need);
	}
	if (chain.length === 0) return hasLocation ? null : LOCATION_PERMISSION_MISSING;
	const off = chain.find((need) => !settingValue(settings, need));
	return off ? (AUTOMATION[off].missing ?? `Needs ${AUTOMATION[off].label.toLowerCase()}.`) : null;
}

/** Whether the switch is in effect: on, and nothing it leans on is off. */
export function settingOn(
	settings: MapUserSettings | null,
	key: AutomationKey,
	hasLocation = true,
): boolean {
	return settingValue(settings, key) && settingBlocker(settings, key, hasLocation) === null;
}

/** The patch that sets one switch. */
export function automationPatch(key: AutomationKey, value: boolean): UpdateMapUserSettings {
	const patch: UpdateMapUserSettings = {};
	patch[key] = value;
	return patch;
}

interface SettingsWrite {
	mapId: number;
	patch: UpdateMapUserSettings;
	/** The caller says how it went itself, so the shared failure toast stays quiet. */
	quiet: boolean;
}

/**
 * Every write of a viewer's settings, wherever it starts: the map, its settings pages or
 * the map list. Component init only (it creates a mutation).
 *
 * The saved settings go straight into the cache rather than being refetched, and a read
 * already on the wire is cancelled first so the slower, older answer cannot win. What a
 * setting changes beyond itself is refreshed here too, so no caller has to remember it.
 */
export function createUserSettingsWriter() {
	const client = useQueryClient();
	const mutation = createMutation(() => ({
		onMutate: ({ mapId }: SettingsWrite) =>
			client.cancelQueries({ queryKey: key.userSettings(mapId) }),
		mutationFn: ({ mapId, patch }: SettingsWrite) => api.updateMapUserSettings(mapId, patch),
		onSuccess: (saved: MapUserSettings, { mapId, patch }: SettingsWrite) => {
			client.setQueryData(key.userSettings(mapId), saved);
			// Sharing your location puts your pilots on the map, or takes them off it.
			if (patch.tracking_allowed !== undefined) {
				void client.invalidateQueries({ queryKey: key.mapCharacters(mapId) });
			}
			// The map list and the top bar's shortcuts read these two.
			if (patch.is_pinned !== undefined || patch.is_archived !== undefined) {
				void client.invalidateQueries({ queryKey: key.maps, exact: true });
			}
		},
		onError: (err: unknown, { quiet }: SettingsWrite) => {
			if (!quiet) toast.error(MAP_ACTIONS.saveSettings.failed, { description: errorMessage(err) });
		},
	}));

	return {
		/** Save and move on; a failure is toasted. */
		save(mapId: number, patch: UpdateMapUserSettings): void {
			mutation.mutate({ mapId, patch, quiet: false });
		},
		/**
		 * Save and wait for the stored settings, for a caller that sequences on them. A
		 * failure is toasted as well as rejected, unless `quiet` says the caller reports it.
		 */
		saveAsync(
			mapId: number,
			patch: UpdateMapUserSettings,
			{ quiet = false }: { quiet?: boolean } = {},
		): Promise<MapUserSettings> {
			return mutation.mutateAsync({ mapId, patch, quiet });
		},
	};
}

export type UserSettingsWriter = ReturnType<typeof createUserSettingsWriter>;

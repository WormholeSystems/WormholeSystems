// Typed client for the Axum JSON API, for the browser and the server-side loads alike.

import type { AddConnection } from './types/AddConnection';
import type { AddConnectionJump } from './types/AddConnectionJump';
import type { AddSignature } from './types/AddSignature';
import type { AddSystem } from './types/AddSystem';
import type { ResolveGhostSystem } from './types/ResolveGhostSystem';
import type { CharacterIdBody } from './types/CharacterIdBody';
import type { CharacterRef } from './types/CharacterRef';
import type { CharacterSummary } from './types/CharacterSummary';
import type { CreateMapBody } from './types/CreateMapBody';
import type { CreateToken } from './types/CreateToken';
import type { CreatedToken } from './types/CreatedToken';
import type { PersonalAccessToken } from './types/PersonalAccessToken';
import type { DiscordAccount } from './types/DiscordAccount';
import type { UpdateUserPreferences } from './types/UpdateUserPreferences';
import type { UserPreferences } from './types/UserPreferences';
import type { MapAlert } from './types/MapAlert';
import type { MapAlertEvent } from './types/MapAlertEvent';
import type { MapWebhook } from './types/MapWebhook';
import type { MapWebhookRole } from './types/MapWebhookRole';
import type { SaveAlert } from './types/SaveAlert';
import type { SaveRole } from './types/SaveRole';
import type { SaveWebhook } from './types/SaveWebhook';
import type { ScopeStatus } from './types/ScopeStatus';
import type { ConnectionJump } from './types/ConnectionJump';
import type { ClearMap } from './types/ClearMap';
import type { EffectModifier } from './types/EffectModifier';
import type { EveScoutConnection } from './types/EveScoutConnection';
import type { GridConfig } from './types/GridConfig';
import type { Instance } from './types/Instance';
import type { LinkSignature } from './types/LinkSignature';
import type { Map } from './types/Map';
import type { MapCharacter } from './types/MapCharacter';
import type { MapConnection } from './types/MapConnection';
import type { MapEntry } from './types/MapEntry';
import type { MapSolarSystem } from './types/MapSolarSystem';
import type { MapUserSettings } from './types/MapUserSettings';
import type { MapView } from './types/MapView';
import type { MoveSystems } from './types/MoveSystems';
import type { PasteSignatures } from './types/PasteSignatures';
import type { RemoveConnection } from './types/RemoveConnection';
import type { RemoveConnectionJump } from './types/RemoveConnectionJump';
import type { RemoveSignature } from './types/RemoveSignature';
import type { RemoveSignatures } from './types/RemoveSignatures';
import type { RemoveSystems } from './types/RemoveSystems';
import type { Sections } from './types/Sections';
import type { SetAlertActive } from './types/SetAlertActive';
import type { SetAlias } from './types/SetAlias';
import type { SetConnectionStatus } from './types/SetConnectionStatus';
import type { SetHome } from './types/SetHome';
import type { SetNotes } from './types/SetNotes';
import type { SetOccupier } from './types/SetOccupier';
import type { SetPinned } from './types/SetPinned';
import type { SetRally } from './types/SetRally';
import type { SetStatus } from './types/SetStatus';
import type { SetWaypointAllBody } from './types/SetWaypointAllBody';
import type { SetWaypointBody } from './types/SetWaypointBody';
import type { ServerStatus } from './types/ServerStatus';
import type { Signature } from './types/Signature';
import type { MapKillmail } from './types/MapKillmail';
import type { Skyhook } from './types/Skyhook';
import type { TrackJump } from './types/TrackJump';
import type { AddEveScoutToMap } from './types/AddEveScoutToMap';
import type { ShipSearchResult } from './types/ShipSearchResult';
import type { SignatureCatalog } from './types/SignatureCatalog';
import type { AddIgnoredSystem } from './types/AddIgnoredSystem';
import type { AddPersonalWatchlistEntry } from './types/AddPersonalWatchlistEntry';
import type { AddWatchlistEntry } from './types/AddWatchlistEntry';
import type { RemovePersonalWatchlistEntry } from './types/RemovePersonalWatchlistEntry';
import type { SetPersonalWatchlistPinned } from './types/SetPersonalWatchlistPinned';
import type { ClearIgnoredSystems } from './types/ClearIgnoredSystems';
import type { IgnoredSystem } from './types/IgnoredSystem';
import type { RemoveIgnoredSystem } from './types/RemoveIgnoredSystem';
import type { RemoveWatchlistEntry } from './types/RemoveWatchlistEntry';
import type { SetWatchlistPinned } from './types/SetWatchlistPinned';
import type { SystemDetails } from './types/SystemDetails';
import type { AccessEntry } from './types/AccessEntry';
import type { StaleConnection } from './types/StaleConnection';
import type { MapSearchHit } from './types/MapSearchHit';
import type { CleanStaleConnections } from './types/CleanStaleConnections';
import type { AccessSubject } from './types/AccessSubject';
import type { SetAccess } from './types/SetAccess';
import type { RevokeAccess } from './types/RevokeAccess';
import type { TransferOwnership } from './types/TransferOwnership';
import type { UpdateMap } from './types/UpdateMap';
import type { MapHistory } from './types/MapHistory';
import type { ReferenceCounts } from './types/ReferenceCounts';
import type { RoutingGraph } from './types/RoutingGraph';
import type { GotoMapEvent } from './types/GotoMapEvent';
import type { WatchlistEntry } from './types/WatchlistEntry';
import type { ThreatAnalysis } from './types/ThreatAnalysis';
import type { SystemSearchResult } from './types/SystemSearchResult';
import type { UnlinkSignature } from './types/UnlinkSignature';
import type { UpdateConnectionJump } from './types/UpdateConnectionJump';
import type { UpdateSignature } from './types/UpdateSignature';
import type { UpdateMapUserSettings } from './types/UpdateMapUserSettings';
import type { ImportSummary } from './types/ImportSummary';
import type { TransferCounts } from './types/TransferCounts';

/**
 * What went wrong, as something worth showing.
 *
 * A catch binding is `unknown`, because anything can be thrown. Asserting it is an `Error`
 * at each of the twenty-odd places we catch one is twenty unchecked claims; asking here is
 * one check, and a thrown string still says something rather than "undefined".
 */
export function errorMessage(err: unknown): string {
	if (err instanceof Error) return err.message;
	return typeof err === 'string' ? err : 'something went wrong';
}

export class ApiError extends Error {
	status: number;
	/**
	 * The backend's id for the request, when it sends one (`x-request-id`), so an error page
	 * can give somebody something to quote that matches a line in the server's log.
	 */
	requestId: string | null;

	constructor(status: number, message: string, requestId: string | null = null) {
		super(message);
		this.status = status;
		this.requestId = requestId;
	}
}

/** The sections a map export can carry, as the server names them. */
export type TransferSectionId = keyof Sections;

/**
 * The typed API, over a given `fetch`. The browser's instance below uses relative paths
 * (the vite dev proxy / Caddy route them); server-side loads point one at the backend and
 * hand it the visitor's cookie (`$lib/server/api`). Both throw the same [`ApiError`].
 */
export function createApi(
	fetchFn: typeof fetch = (input, init) => fetch(input, init),
	base = '',
	defaults: Record<string, string> = {},
) {
	function withDefaults(headers: HeadersInit | undefined): Headers {
		const merged = new Headers(defaults);
		new Headers(headers).forEach((value, name) => merged.set(name, value));
		return merged;
	}

	async function request<T>(path: string, init?: RequestInit): Promise<T> {
		const res = await fetchFn(base + path, { ...init, headers: withDefaults(init?.headers) });
		if (!res.ok) {
			let message = res.statusText;
			try {
				const body = await res.json();
				if (typeof body?.error === 'string') message = body.error;
			} catch {
				// non-JSON error body; keep the status text
			}
			throw new ApiError(res.status, message, res.headers.get('x-request-id'));
		}
		return res.json();
	}

	// A read takes the query's abort signal, so a refetch TanStack Query cancels (a newer one
	// superseding it, a write cancelling a stale read, nobody watching any more) stops
	// downloading instead of finishing a response nobody will use.
	function get<T>(path: string, signal?: AbortSignal): Promise<T> {
		return request<T>(path, { signal });
	}

	function post<T>(path: string, body: unknown): Promise<T> {
		return request<T>(path, {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(body),
		});
	}

	function put<T>(path: string, body: unknown): Promise<T> {
		return request<T>(path, {
			method: 'PUT',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(body),
		});
	}

	function del<T>(path: string): Promise<T> {
		return request<T>(path, { method: 'DELETE' });
	}

	return {
		// Auth / identity
		me: (signal?: AbortSignal) => get<CharacterSummary | null>('/api/me', signal),
		myCharacters: (signal?: AbortSignal) => get<CharacterRef[]>('/api/me/characters', signal),
		myScopes: (signal?: AbortSignal) => get<ScopeStatus[]>('/api/me/scopes', signal),
		myDiscord: (signal?: AbortSignal) => get<DiscordAccount | null>('/api/me/discord', signal),
		unlinkDiscord: () => post<null>('/api/me/discord/unlink', {}),
		myPreferences: (signal?: AbortSignal) => get<UserPreferences>('/api/me/preferences', signal),
		updatePreferences: (change: UpdateUserPreferences) =>
			post<UserPreferences>('/api/me/preferences', change),
		myTokens: (signal?: AbortSignal) => get<PersonalAccessToken[]>('/api/me/tokens', signal),
		createToken: (cmd: CreateToken) => post<CreatedToken>('/api/me/tokens', cmd),
		revokeToken: (id: number) => del<null>(`/api/me/tokens/${id}`),
		switchCharacter: (characterId: number) =>
			post<null>('/api/me/switch-character', {
				character_id: characterId,
			} satisfies CharacterIdBody),
		removeCharacter: (characterId: number) =>
			post<null>('/api/me/remove-character', {
				character_id: characterId,
			} satisfies CharacterIdBody),
		setPreferredCharacter: (characterId: number) =>
			post<null>('/api/me/preferred-character', {
				character_id: characterId,
			} satisfies CharacterIdBody),

		// ESI waypoints
		setWaypoint: (body: SetWaypointBody) => post<null>('/api/waypoints', body),
		setWaypointAll: (body: SetWaypointAllBody) => post<null>('/api/waypoints/all', body),

		// Config / reference data
		instance: (signal?: AbortSignal) => get<Instance>('/api/instance', signal),
		gridConfig: (signal?: AbortSignal) => get<GridConfig>('/api/grid-config', signal),
		effectModifiers: (name: string, wormholeClassId: number, signal?: AbortSignal) =>
			get<EffectModifier[]>(
				`/api/effects?name=${encodeURIComponent(name)}&class=${wormholeClassId}`,
				signal,
			),
		searchSystems: (query: string, signal?: AbortSignal) =>
			get<SystemSearchResult[]>(`/api/systems/search?q=${encodeURIComponent(query)}`, signal),
		resolveSystems: (ids: number[], signal?: AbortSignal) =>
			get<SystemSearchResult[]>(`/api/systems/resolve?ids=${ids.join(',')}`, signal),
		threatAnalysis: (solarSystemId: number, signal?: AbortSignal) =>
			get<ThreatAnalysis>(`/api/threat/${solarSystemId}`, signal),
		// Cached for a day; bump the version param when the payload shape changes.
		routingGraph: (signal?: AbortSignal) => get<RoutingGraph>('/api/routing-graph?v=5', signal),
		referenceCounts: (signal?: AbortSignal) =>
			get<ReferenceCounts>('/api/reference-counts', signal),

		// Maps
		// A share token stands in for access, so this one answers without an account.
		sharedMap: (token: string, signal?: AbortSignal) =>
			get<MapView>(`/api/share/${encodeURIComponent(token)}`, signal),
		myMaps: (signal?: AbortSignal) => get<MapEntry[]>('/api/maps', signal),
		createMap: (body: CreateMapBody) => post<Map>('/api/maps', body),
		deleteMap: (mapId: number) => del<null>(`/api/maps/${mapId}`),
		fetchMap: (mapId: number, signal?: AbortSignal) => get<MapView>(`/api/maps/${mapId}`, signal),
		listSignatures: (mapId: number, signal?: AbortSignal) =>
			get<Signature[]>(`/api/maps/${mapId}/signatures`, signal),
		mapCharacters: (mapId: number, signal?: AbortSignal) =>
			get<MapCharacter[]>(`/api/maps/${mapId}/characters`, signal),
		mapUserSettings: (mapId: number, signal?: AbortSignal) =>
			get<MapUserSettings>(`/api/maps/${mapId}/settings/user`, signal),
		updateMapUserSettings: (mapId: number, update: UpdateMapUserSettings) =>
			post<MapUserSettings>(`/api/maps/${mapId}/settings/user`, update),
		uploadBackgroundImage: (mapId: number, file: File) => {
			const body = new FormData();
			body.append('image', file);
			// No content-type header: the browser sets multipart with its boundary.
			return request<MapUserSettings>(`/api/maps/${mapId}/background-image`, {
				method: 'PUT',
				body,
			});
		},
		removeBackgroundImage: (mapId: number) =>
			del<MapUserSettings>(`/api/maps/${mapId}/background-image`),

		// Systems
		addSystem: (cmd: AddSystem) => post<MapSolarSystem>(`/api/maps/${cmd.map_id}/systems/add`, cmd),
		resolveGhostSystem: (cmd: ResolveGhostSystem) =>
			post<MapSolarSystem>(`/api/maps/${cmd.map_id}/systems/resolve-ghost`, cmd),
		moveSystems: (cmd: MoveSystems) => post<null>(`/api/maps/${cmd.map_id}/systems/move`, cmd),
		removeSystems: (cmd: RemoveSystems) =>
			post<null>(`/api/maps/${cmd.map_id}/systems/remove`, cmd),
		clearMap: (cmd: ClearMap) => post<null>(`/api/maps/${cmd.map_id}/clear`, cmd),
		setAlias: (cmd: SetAlias) => post<null>(`/api/maps/${cmd.map_id}/systems/set-alias`, cmd),
		setStatus: (cmd: SetStatus) => post<null>(`/api/maps/${cmd.map_id}/systems/set-status`, cmd),
		setOccupier: (cmd: SetOccupier) =>
			post<null>(`/api/maps/${cmd.map_id}/systems/set-occupier`, cmd),
		setHome: (cmd: SetHome) => post<null>(`/api/maps/${cmd.map_id}/systems/set-home`, cmd),
		setNotes: (cmd: SetNotes) => post<null>(`/api/maps/${cmd.map_id}/systems/set-notes`, cmd),
		systemDetails: (mapId: number, mss: number, signal?: AbortSignal) =>
			get<SystemDetails>(`/api/maps/${mapId}/systems/${mss}/details`, signal),
		setRally: (cmd: SetRally) => post<null>(`/api/maps/${cmd.map_id}/systems/set-rally`, cmd),
		setPinned: (cmd: SetPinned) => post<null>(`/api/maps/${cmd.map_id}/systems/set-pinned`, cmd),

		serverStatus: (signal?: AbortSignal) => get<ServerStatus>('/api/server-status', signal),
		skyhooks: (signal?: AbortSignal) => get<Skyhook[]>('/api/skyhooks', signal),
		// The selected system only matters to a card that follows the selection; the server
		// ignores it otherwise.
		mapKillmails: (mapId: number, solarSystemId: number | null = null, signal?: AbortSignal) =>
			get<MapKillmail[]>(
				solarSystemId === null
					? `/api/maps/${mapId}/killmails`
					: `/api/maps/${mapId}/killmails?solar_system_id=${solarSystemId}`,
				signal,
			),

		// Places the system, connects it, and links the signature in one undoable step. The
		// response is empty, the socket drives the refetch.
		trackJump: (cmd: TrackJump) => post<null>(`/api/maps/${cmd.map_id}/track-jump`, cmd),

		// Connections
		addConnection: (cmd: AddConnection) =>
			post<MapConnection>(`/api/maps/${cmd.map_id}/connections/add`, cmd),
		setConnectionStatus: (cmd: SetConnectionStatus) =>
			post<MapConnection>(`/api/maps/${cmd.map_id}/connections/set-status`, cmd),
		removeConnection: (cmd: RemoveConnection) =>
			post<null>(`/api/maps/${cmd.map_id}/connections/remove`, cmd),
		listConnectionJumps: (mapId: number, connectionId: number, signal?: AbortSignal) =>
			get<ConnectionJump[]>(`/api/maps/${mapId}/connections/${connectionId}/jumps`, signal),
		addConnectionJump: (cmd: AddConnectionJump) =>
			post<ConnectionJump>(`/api/maps/${cmd.map_id}/connections/jumps/add`, cmd),
		updateConnectionJump: (cmd: UpdateConnectionJump) =>
			post<ConnectionJump>(`/api/maps/${cmd.map_id}/connections/jumps/update`, cmd),
		removeConnectionJump: (cmd: RemoveConnectionJump) =>
			post<null>(`/api/maps/${cmd.map_id}/connections/jumps/remove`, cmd),
		searchShips: (query: string, signal?: AbortSignal) =>
			get<ShipSearchResult[]>(`/api/ships/search?q=${encodeURIComponent(query)}`, signal),

		// Navigation
		eveScout: (signal?: AbortSignal) => get<EveScoutConnection[]>('/api/evescout', signal),
		// Puts a hub's public holes on the map in one undoable step; answers with how many
		// connections it added, the socket drives the refetch.
		addEveScoutToMap: (cmd: AddEveScoutToMap) =>
			post<number>(`/api/maps/${cmd.map_id}/evescout/add`, cmd),
		listWatchlist: (mapId: number, signal?: AbortSignal) =>
			get<WatchlistEntry[]>(`/api/maps/${mapId}/watchlist`, signal),
		addWatchlistEntry: (cmd: AddWatchlistEntry) =>
			post<WatchlistEntry>(`/api/maps/${cmd.map_id}/watchlist/add`, cmd),
		setWatchlistPinned: (cmd: SetWatchlistPinned) =>
			post<WatchlistEntry>(`/api/maps/${cmd.map_id}/watchlist/set-pinned`, cmd),
		removeWatchlistEntry: (cmd: RemoveWatchlistEntry) =>
			post<null>(`/api/maps/${cmd.map_id}/watchlist/remove`, cmd),
		// The caller's own watchlist on a map, which nobody else sees.
		listPersonalWatchlist: (mapId: number, signal?: AbortSignal) =>
			get<WatchlistEntry[]>(`/api/maps/${mapId}/watchlist/personal`, signal),
		addPersonalWatchlistEntry: (mapId: number, body: AddPersonalWatchlistEntry) =>
			post<WatchlistEntry>(`/api/maps/${mapId}/watchlist/personal/add`, body),
		setPersonalWatchlistPinned: (mapId: number, body: SetPersonalWatchlistPinned) =>
			post<WatchlistEntry>(`/api/maps/${mapId}/watchlist/personal/set-pinned`, body),
		removePersonalWatchlistEntry: (mapId: number, body: RemovePersonalWatchlistEntry) =>
			post<null>(`/api/maps/${mapId}/watchlist/personal/remove`, body),
		listIgnored: (mapId: number, signal?: AbortSignal) =>
			get<IgnoredSystem[]>(`/api/maps/${mapId}/ignored`, signal),
		addIgnored: (cmd: AddIgnoredSystem) =>
			post<IgnoredSystem>(`/api/maps/${cmd.map_id}/ignored/add`, cmd),
		removeIgnored: (cmd: RemoveIgnoredSystem) =>
			post<null>(`/api/maps/${cmd.map_id}/ignored/remove`, cmd),
		clearIgnored: (cmd: ClearIgnoredSystems) =>
			post<number>(`/api/maps/${cmd.map_id}/ignored/clear`, cmd),

		// Import / export
		transferCounts: (mapId: number, signal?: AbortSignal) =>
			get<TransferCounts>(`/api/maps/${mapId}/transfer/counts`, signal),
		// A plain URL rather than a call: the browser follows it and saves the download itself.
		exportMapUrl: (mapId: number, sections: TransferSectionId[]) =>
			`/api/maps/${mapId}/transfer/export?sections=${sections.join(',')}`,
		importMap: (mapId: number, sections: TransferSectionId[], content: string) =>
			post<ImportSummary>(`/api/maps/${mapId}/transfer/import`, { sections, content }),
		importMapAsNew: (body: { name?: string; sections: TransferSectionId[]; content: string }) =>
			post<Map>('/api/maps/transfer/import-new', body),

		// Access / settings
		updateMap: (cmd: UpdateMap) => post<Map>(`/api/maps/${cmd.map_id}/update`, cmd),
		listAccess: (mapId: number, signal?: AbortSignal) =>
			get<AccessEntry[]>(`/api/maps/${mapId}/access`, signal),

		// --- Discord alerts ---
		listAlerts: (mapId: number, signal?: AbortSignal) =>
			get<MapAlert[]>(`/api/maps/${mapId}/alerts`, signal),
		alertEvents: (mapId: number, signal?: AbortSignal) =>
			get<MapAlertEvent[]>(`/api/maps/${mapId}/alerts/events`, signal),
		listWebhooks: (mapId: number, signal?: AbortSignal) =>
			get<MapWebhook[]>(`/api/maps/${mapId}/webhooks`, signal),
		createWebhook: (mapId: number, body: SaveWebhook) =>
			post<MapWebhook>(`/api/maps/${mapId}/webhooks`, body),
		deleteWebhook: (mapId: number, webhookId: number) =>
			del<null>(`/api/maps/${mapId}/webhooks/${webhookId}`),
		listAlertRoles: (mapId: number, signal?: AbortSignal) =>
			get<MapWebhookRole[]>(`/api/maps/${mapId}/roles`, signal),
		createAlertRole: (mapId: number, body: SaveRole) =>
			post<MapWebhookRole>(`/api/maps/${mapId}/roles`, body),
		deleteAlertRole: (mapId: number, roleId: number) =>
			del<null>(`/api/maps/${mapId}/roles/${roleId}`),
		createAlert: (mapId: number, body: SaveAlert) =>
			post<MapAlert>(`/api/maps/${mapId}/alerts`, body),
		updateAlert: (mapId: number, alertId: number, body: SaveAlert) =>
			put<MapAlert>(`/api/maps/${mapId}/alerts/${alertId}`, body),
		setAlertActive: (mapId: number, alertId: number, isActive: boolean) =>
			post<MapAlert>(`/api/maps/${mapId}/alerts/${alertId}/active`, {
				is_active: isActive,
			} satisfies SetAlertActive),
		deleteAlert: (mapId: number, alertId: number) =>
			del<null>(`/api/maps/${mapId}/alerts/${alertId}`),
		searchAccessSubjects: (query: string, signal?: AbortSignal) =>
			get<AccessSubject[]>(`/api/access-subjects/search?q=${encodeURIComponent(query)}`, signal),
		setAccess: (cmd: SetAccess) => post<null>(`/api/maps/${cmd.map_id}/access/set`, cmd),
		shareMap: (mapId: number) => post<string>(`/api/maps/${mapId}/share`, {}),
		unshareMap: (mapId: number) => del<null>(`/api/maps/${mapId}/share`),
		transferOwnership: (cmd: TransferOwnership) =>
			post<null>(`/api/maps/${cmd.map_id}/access/transfer`, cmd),
		revokeAccess: (cmd: RevokeAccess) => post<null>(`/api/maps/${cmd.map_id}/access/revoke`, cmd),

		searchMap: (mapId: number, query: string, signal?: AbortSignal) =>
			get<MapSearchHit[]>(`/api/maps/${mapId}/search?q=${encodeURIComponent(query)}`, signal),
		listStaleConnections: (mapId: number, signal?: AbortSignal) =>
			get<StaleConnection[]>(`/api/maps/${mapId}/connections/stale`, signal),
		cleanStaleConnections: (cmd: CleanStaleConnections) =>
			post<number>(`/api/maps/${cmd.map_id}/connections/clean-stale`, cmd),

		// History
		mapHistory: (mapId: number, signal?: AbortSignal) =>
			get<MapHistory>(`/api/maps/${mapId}/events`, signal),
		undoMapEvent: (mapId: number) =>
			post<null>(`/api/maps/${mapId}/events/undo`, { map_id: mapId }),
		redoMapEvent: (mapId: number) =>
			post<null>(`/api/maps/${mapId}/events/redo`, { map_id: mapId }),
		gotoMapEvent: (cmd: GotoMapEvent) => post<null>(`/api/maps/${cmd.map_id}/events/goto`, cmd),

		// Signatures
		signatureCatalog: (signal?: AbortSignal) =>
			get<SignatureCatalog>('/api/signature-types', signal),
		addSignature: (cmd: AddSignature) =>
			post<Signature>(`/api/maps/${cmd.map_id}/signatures/add`, cmd),
		updateSignature: (cmd: UpdateSignature) =>
			post<Signature>(`/api/maps/${cmd.map_id}/signatures/update`, cmd),
		removeSignaturesBulk: (cmd: RemoveSignatures) =>
			post<null>(`/api/maps/${cmd.map_id}/signatures/remove-bulk`, cmd),
		pasteSignatures: (cmd: PasteSignatures) =>
			post<null>(`/api/maps/${cmd.map_id}/signatures/paste`, cmd),
		linkSignature: (cmd: LinkSignature) =>
			post<Signature>(`/api/maps/${cmd.map_id}/signatures/link`, cmd),
		unlinkSignature: (cmd: UnlinkSignature) =>
			post<Signature>(`/api/maps/${cmd.map_id}/signatures/unlink`, cmd),
		removeSignature: (cmd: RemoveSignature) =>
			post<null>(`/api/maps/${cmd.map_id}/signatures/remove`, cmd),
	};
}

export type Api = ReturnType<typeof createApi>;

export const api = createApi();

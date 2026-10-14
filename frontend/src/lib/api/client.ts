// Typed client for the Axum JSON API. Browser-side calls use relative paths (the vite
// dev proxy / Caddy route them); server-side loads go through `$lib/server/api` instead.

import type { AddConnection } from './types/AddConnection';
import type { AddConnectionJump } from './types/AddConnectionJump';
import type { AddSignature } from './types/AddSignature';
import type { AddSystem } from './types/AddSystem';
import type { ResolveGhostSystem } from './types/ResolveGhostSystem';
import type { CharacterRef } from './types/CharacterRef';
import type { CreateToken } from './types/CreateToken';
import type { CreatedToken } from './types/CreatedToken';
import type { PersonalAccessToken } from './types/PersonalAccessToken';
import type { DiscordAccount } from './types/DiscordAccount';
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
import type { AddWatchlistEntry } from './types/AddWatchlistEntry';
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

	constructor(status: number, message: string) {
		super(message);
		this.status = status;
	}
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
	const res = await fetch(path, init);
	if (!res.ok) {
		let message = res.statusText;
		try {
			const body = await res.json();
			if (typeof body?.error === 'string') message = body.error;
		} catch {
			// non-JSON error body; keep the status text
		}
		throw new ApiError(res.status, message);
	}
	return res.json();
}

function get<T>(path: string): Promise<T> {
	return request<T>(path);
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

export const api = {
	// Auth / identity
	myCharacters: () => get<CharacterRef[]>('/api/me/characters'),
	myScopes: () => get<ScopeStatus[]>('/api/me/scopes'),
	myDiscord: () => get<DiscordAccount | null>('/api/me/discord'),
	unlinkDiscord: () => post<null>('/api/me/discord/unlink', {}),
	myTokens: () => get<PersonalAccessToken[]>('/api/me/tokens'),
	createToken: (cmd: CreateToken) => post<CreatedToken>('/api/me/tokens', cmd),
	revokeToken: (id: number) => del<null>(`/api/me/tokens/${id}`),
	switchCharacter: (characterId: number) =>
		post<null>('/api/me/switch-character', { character_id: characterId }),
	removeCharacter: (characterId: number) =>
		post<null>('/api/me/remove-character', { character_id: characterId }),
	setPreferredCharacter: (characterId: number) =>
		post<null>('/api/me/preferred-character', { character_id: characterId }),

	// ESI waypoints
	setWaypoint: (body: SetWaypointBody) => post<null>('/api/waypoints', body),
	setWaypointAll: (body: SetWaypointAllBody) => post<null>('/api/waypoints/all', body),

	// Config / reference data
	instance: () => get<Instance>('/api/instance'),
	gridConfig: () => get<GridConfig>('/api/grid-config'),
	effectModifiers: (name: string, wormholeClassId: number) =>
		get<EffectModifier[]>(`/api/effects?name=${encodeURIComponent(name)}&class=${wormholeClassId}`),
	searchSystems: (query: string) =>
		get<SystemSearchResult[]>(`/api/systems/search?q=${encodeURIComponent(query)}`),
	resolveSystems: (ids: number[]) =>
		get<SystemSearchResult[]>(`/api/systems/resolve?ids=${ids.join(',')}`),
	threatAnalysis: (solarSystemId: number) => get<ThreatAnalysis>(`/api/threat/${solarSystemId}`),
	// Cached for a day; bump the version param when the payload shape changes.
	routingGraph: () => get<RoutingGraph>('/api/routing-graph?v=5'),

	// Maps
	myMaps: () => get<MapEntry[]>('/api/maps'),
	createMap: (name: string, description?: string) => post<Map>('/api/maps', { name, description }),
	deleteMap: (mapId: number) => del<null>(`/api/maps/${mapId}`),
	fetchMap: (mapId: number) => get<MapView>(`/api/maps/${mapId}`),
	listSignatures: (mapId: number) => get<Signature[]>(`/api/maps/${mapId}/signatures`),
	mapCharacters: (mapId: number) => get<MapCharacter[]>(`/api/maps/${mapId}/characters`),
	mapUserSettings: (mapId: number) => get<MapUserSettings>(`/api/maps/${mapId}/settings/user`),
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
	removeSystems: (cmd: RemoveSystems) => post<null>(`/api/maps/${cmd.map_id}/systems/remove`, cmd),
	clearMap: (cmd: ClearMap) => post<null>(`/api/maps/${cmd.map_id}/clear`, cmd),
	setAlias: (cmd: SetAlias) => post<null>(`/api/maps/${cmd.map_id}/systems/set-alias`, cmd),
	setStatus: (cmd: SetStatus) => post<null>(`/api/maps/${cmd.map_id}/systems/set-status`, cmd),
	setOccupier: (cmd: SetOccupier) =>
		post<null>(`/api/maps/${cmd.map_id}/systems/set-occupier`, cmd),
	setHome: (cmd: SetHome) => post<null>(`/api/maps/${cmd.map_id}/systems/set-home`, cmd),
	setNotes: (cmd: SetNotes) => post<null>(`/api/maps/${cmd.map_id}/systems/set-notes`, cmd),
	systemDetails: (mapId: number, mss: number) =>
		get<SystemDetails>(`/api/maps/${mapId}/systems/${mss}/details`),
	setRally: (cmd: SetRally) => post<null>(`/api/maps/${cmd.map_id}/systems/set-rally`, cmd),
	setPinned: (cmd: SetPinned) => post<null>(`/api/maps/${cmd.map_id}/systems/set-pinned`, cmd),

	serverStatus: () => get<ServerStatus>('/api/server-status'),
	skyhooks: () => get<Skyhook[]>('/api/skyhooks'),
	// The selected system only matters to a card that follows the selection; the server
	// ignores it otherwise.
	mapKillmails: (mapId: number, solarSystemId: number | null = null) =>
		get<MapKillmail[]>(
			solarSystemId === null
				? `/api/maps/${mapId}/killmails`
				: `/api/maps/${mapId}/killmails?solar_system_id=${solarSystemId}`,
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
	listConnectionJumps: (mapId: number, connectionId: number) =>
		get<ConnectionJump[]>(`/api/maps/${mapId}/connections/${connectionId}/jumps`),
	addConnectionJump: (cmd: AddConnectionJump) =>
		post<ConnectionJump>(`/api/maps/${cmd.map_id}/connections/jumps/add`, cmd),
	updateConnectionJump: (cmd: UpdateConnectionJump) =>
		post<ConnectionJump>(`/api/maps/${cmd.map_id}/connections/jumps/update`, cmd),
	removeConnectionJump: (cmd: RemoveConnectionJump) =>
		post<null>(`/api/maps/${cmd.map_id}/connections/jumps/remove`, cmd),
	searchShips: (query: string) =>
		get<ShipSearchResult[]>(`/api/ships/search?q=${encodeURIComponent(query)}`),

	// Navigation
	eveScout: () => get<EveScoutConnection[]>('/api/evescout'),
	// Puts a hub's public holes on the map in one undoable step; answers with how many
	// connections it added, the socket drives the refetch.
	addEveScoutToMap: (cmd: AddEveScoutToMap) =>
		post<number>(`/api/maps/${cmd.map_id}/evescout/add`, cmd),
	listWatchlist: (mapId: number) => get<WatchlistEntry[]>(`/api/maps/${mapId}/watchlist`),
	addWatchlistEntry: (cmd: AddWatchlistEntry) =>
		post<WatchlistEntry>(`/api/maps/${cmd.map_id}/watchlist/add`, cmd),
	setWatchlistPinned: (cmd: SetWatchlistPinned) =>
		post<WatchlistEntry>(`/api/maps/${cmd.map_id}/watchlist/set-pinned`, cmd),
	removeWatchlistEntry: (cmd: RemoveWatchlistEntry) =>
		post<null>(`/api/maps/${cmd.map_id}/watchlist/remove`, cmd),
	listIgnored: (mapId: number) => get<IgnoredSystem[]>(`/api/maps/${mapId}/ignored`),
	addIgnored: (cmd: AddIgnoredSystem) =>
		post<IgnoredSystem>(`/api/maps/${cmd.map_id}/ignored/add`, cmd),
	removeIgnored: (cmd: RemoveIgnoredSystem) =>
		post<null>(`/api/maps/${cmd.map_id}/ignored/remove`, cmd),
	clearIgnored: (cmd: ClearIgnoredSystems) =>
		post<number>(`/api/maps/${cmd.map_id}/ignored/clear`, cmd),

	// Import / export
	transferCounts: (mapId: number) => get<TransferCounts>(`/api/maps/${mapId}/transfer/counts`),
	// A plain URL rather than a call: the browser follows it and saves the download itself.
	exportMapUrl: (mapId: number, sections: string[]) =>
		`/api/maps/${mapId}/transfer/export?sections=${sections.join(',')}`,
	importMap: (mapId: number, sections: string[], content: string) =>
		post<ImportSummary>(`/api/maps/${mapId}/transfer/import`, { sections, content }),
	importMapAsNew: (body: { name?: string; sections: string[]; content: string }) =>
		post<Map>('/api/maps/transfer/import-new', body),

	// Access / settings
	updateMap: (cmd: UpdateMap) => post<Map>(`/api/maps/${cmd.map_id}/update`, cmd),
	listAccess: (mapId: number) => get<AccessEntry[]>(`/api/maps/${mapId}/access`),

	// --- Discord alerts ---
	listAlerts: (mapId: number) => get<MapAlert[]>(`/api/maps/${mapId}/alerts`),
	alertEvents: (mapId: number) => get<MapAlertEvent[]>(`/api/maps/${mapId}/alerts/events`),
	listWebhooks: (mapId: number) => get<MapWebhook[]>(`/api/maps/${mapId}/webhooks`),
	createWebhook: (mapId: number, body: SaveWebhook) =>
		post<MapWebhook>(`/api/maps/${mapId}/webhooks`, body),
	deleteWebhook: (mapId: number, webhookId: number) =>
		del<null>(`/api/maps/${mapId}/webhooks/${webhookId}`),
	listAlertRoles: (mapId: number) => get<MapWebhookRole[]>(`/api/maps/${mapId}/roles`),
	createAlertRole: (mapId: number, body: SaveRole) =>
		post<MapWebhookRole>(`/api/maps/${mapId}/roles`, body),
	deleteAlertRole: (mapId: number, roleId: number) =>
		del<null>(`/api/maps/${mapId}/roles/${roleId}`),
	createAlert: (mapId: number, body: SaveAlert) =>
		post<MapAlert>(`/api/maps/${mapId}/alerts`, body),
	updateAlert: (mapId: number, alertId: number, body: SaveAlert) =>
		put<MapAlert>(`/api/maps/${mapId}/alerts/${alertId}`, body),
	setAlertActive: (mapId: number, alertId: number, isActive: boolean) =>
		post<MapAlert>(`/api/maps/${mapId}/alerts/${alertId}/active`, { is_active: isActive }),
	deleteAlert: (mapId: number, alertId: number) =>
		del<null>(`/api/maps/${mapId}/alerts/${alertId}`),
	searchAccessSubjects: (query: string) =>
		get<AccessSubject[]>(`/api/access-subjects/search?q=${encodeURIComponent(query)}`),
	setAccess: (cmd: SetAccess) => post<null>(`/api/maps/${cmd.map_id}/access/set`, cmd),
	shareMap: (mapId: number) => post<string>(`/api/maps/${mapId}/share`, {}),
	unshareMap: (mapId: number) => del<null>(`/api/maps/${mapId}/share`),
	transferOwnership: (cmd: TransferOwnership) =>
		post<null>(`/api/maps/${cmd.map_id}/access/transfer`, cmd),
	revokeAccess: (cmd: RevokeAccess) => post<null>(`/api/maps/${cmd.map_id}/access/revoke`, cmd),

	searchMap: (mapId: number, query: string) =>
		get<MapSearchHit[]>(`/api/maps/${mapId}/search?q=${encodeURIComponent(query)}`),
	listStaleConnections: (mapId: number) =>
		get<StaleConnection[]>(`/api/maps/${mapId}/connections/stale`),
	cleanStaleConnections: (cmd: CleanStaleConnections) =>
		post<number>(`/api/maps/${cmd.map_id}/connections/clean-stale`, cmd),

	// History
	mapHistory: (mapId: number) => get<MapHistory>(`/api/maps/${mapId}/events`),
	undoMapEvent: (mapId: number) => post<null>(`/api/maps/${mapId}/events/undo`, { map_id: mapId }),
	redoMapEvent: (mapId: number) => post<null>(`/api/maps/${mapId}/events/redo`, { map_id: mapId }),
	gotoMapEvent: (cmd: GotoMapEvent) => post<null>(`/api/maps/${cmd.map_id}/events/goto`, cmd),

	// Signatures
	signatureCatalog: () => get<SignatureCatalog>('/api/signature-types'),
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

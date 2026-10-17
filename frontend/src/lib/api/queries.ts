import { queryOptions } from '@tanstack/svelte-query';
import { api } from './client';
import type { KillmailScope } from './types/KillmailScope';

// Every read goes through one of these, so the key hierarchy stays coherent in one place.
// Convention: the map socket invalidates ['maps', mapId, ...] prefixes; no page invalidates
// another page's keys except via these prefixes. `key` holds the prefixes writes invalidate;
// `q` holds one queryOptions per api read, named after the api method.

export const key = {
	me: ['me'] as const,
	myCharacters: ['me', 'characters'] as const,
	myTokens: ['me', 'tokens'] as const,
	serverStatus: ['server-status'] as const,
	// The myMaps list; also the prefix over every per-map subtree, so invalidating it
	// refreshes the list and all open map data at once.
	maps: ['maps'] as const,
	map: (id: number) => ['maps', id] as const,
	mapView: (id: number) => ['maps', id, 'view'] as const,
	signatures: (id: number) => ['maps', id, 'signatures'] as const,
	mapCharacters: (id: number) => ['maps', id, 'characters'] as const,
	userSettings: (id: number) => ['maps', id, 'user-settings'] as const,
	access: (id: number) => ['maps', id, 'access'] as const,
	killmails: (id: number) => ['maps', id, 'killmails'] as const,
	history: (id: number) => ['maps', id, 'history'] as const,
	watchlist: (id: number) => ['maps', id, 'watchlist'] as const,
	personalWatchlist: (id: number) => ['maps', id, 'personal-watchlist'] as const,
	ignored: (id: number) => ['maps', id, 'ignored'] as const,
	transferCounts: (id: number) => ['maps', id, 'transfer-counts'] as const,
	stale: (id: number) => ['maps', id, 'stale-connections'] as const,
	// A placement's member-gated details (the notes); the prefix covers every placement.
	details: (id: number) => ['maps', id, 'details'] as const,
	systemDetails: (id: number, mss: number) => ['maps', id, 'details', mss] as const,
	// Alerts, their events, webhooks and roles share this prefix: the alerts page edits
	// them together, so one invalidation covers all four.
	alerting: (id: number) => ['maps', id, 'alerting'] as const,
	connectionJumps: (id: number, connectionId: number) =>
		['maps', id, 'connection-jumps', connectionId] as const,
};

const FIVE_MINUTES = 5 * 60_000;

export const q = {
	myCharacters: () =>
		queryOptions({ queryKey: key.myCharacters, queryFn: ({ signal }) => api.myCharacters(signal) }),
	myScopes: () =>
		queryOptions({ queryKey: ['me', 'scopes'], queryFn: ({ signal }) => api.myScopes(signal) }),
	myDiscord: () =>
		queryOptions({ queryKey: ['me', 'discord'], queryFn: ({ signal }) => api.myDiscord(signal) }),
	myTokens: () =>
		queryOptions({ queryKey: key.myTokens, queryFn: ({ signal }) => api.myTokens(signal) }),

	instance: () =>
		queryOptions({
			queryKey: ['instance'],
			queryFn: ({ signal }) => api.instance(signal),
			staleTime: Infinity,
		}),
	gridConfig: () =>
		queryOptions({
			queryKey: ['grid-config'],
			queryFn: ({ signal }) => api.gridConfig(signal),
			staleTime: Infinity,
		}),
	serverStatus: () =>
		queryOptions({
			queryKey: key.serverStatus,
			queryFn: ({ signal }) => api.serverStatus(signal),
			refetchInterval: 60_000,
		}),
	// Background refetch stays on for both feeds: a hidden tab must keep the data the
	// jump tracker and the cards read, like the old setInterval did.
	skyhooks: () =>
		queryOptions({
			queryKey: ['skyhooks'],
			queryFn: ({ signal }) => api.skyhooks(signal),
			refetchInterval: FIVE_MINUTES,
			refetchIntervalInBackground: true,
			staleTime: FIVE_MINUTES,
		}),
	eveScout: () =>
		queryOptions({
			queryKey: ['evescout'],
			queryFn: ({ signal }) => api.eveScout(signal),
			refetchInterval: FIVE_MINUTES,
			refetchIntervalInBackground: true,
			staleTime: FIVE_MINUTES,
		}),

	myMaps: () => queryOptions({ queryKey: key.maps, queryFn: ({ signal }) => api.myMaps(signal) }),
	mapView: (id: number) =>
		queryOptions({ queryKey: key.mapView(id), queryFn: ({ signal }) => api.fetchMap(id, signal) }),
	listSignatures: (id: number) =>
		queryOptions({
			queryKey: key.signatures(id),
			queryFn: ({ signal }) => api.listSignatures(id, signal),
		}),
	mapCharacters: (id: number) =>
		queryOptions({
			queryKey: key.mapCharacters(id),
			queryFn: ({ signal }) => api.mapCharacters(id, signal),
		}),
	mapUserSettings: (id: number) =>
		queryOptions({
			queryKey: key.userSettings(id),
			queryFn: ({ signal }) => api.mapUserSettings(id, signal),
		}),
	listAccess: (id: number) =>
		queryOptions({ queryKey: key.access(id), queryFn: ({ signal }) => api.listAccess(id, signal) }),
	// The list is scoped by the viewer's filter and the map's systems, so both are part of
	// the key; a kill frame invalidates the killmails prefix every variant sits under.
	// `selected` matters only to the filter that follows the selection.
	mapKillmails: (id: number, filter: KillmailScope, selected: number | null, systems: string) =>
		queryOptions({
			queryKey: [...key.killmails(id), filter, selected, systems],
			queryFn: ({ signal }) => api.mapKillmails(id, selected, signal),
		}),
	mapHistory: (id: number) =>
		queryOptions({
			queryKey: key.history(id),
			queryFn: ({ signal }) => api.mapHistory(id, signal),
		}),
	listWatchlist: (id: number) =>
		queryOptions({
			queryKey: key.watchlist(id),
			queryFn: ({ signal }) => api.listWatchlist(id, signal),
		}),
	listPersonalWatchlist: (id: number) =>
		queryOptions({
			queryKey: key.personalWatchlist(id),
			queryFn: ({ signal }) => api.listPersonalWatchlist(id, signal),
		}),
	listIgnored: (id: number) =>
		queryOptions({
			queryKey: key.ignored(id),
			queryFn: ({ signal }) => api.listIgnored(id, signal),
		}),
	listStaleConnections: (id: number) =>
		queryOptions({
			queryKey: key.stale(id),
			queryFn: ({ signal }) => api.listStaleConnections(id, signal),
		}),
	transferCounts: (id: number) =>
		queryOptions({
			queryKey: key.transferCounts(id),
			queryFn: ({ signal }) => api.transferCounts(id, signal),
		}),

	listAlerts: (id: number) =>
		queryOptions({
			queryKey: [...key.alerting(id), 'alerts'],
			queryFn: ({ signal }) => api.listAlerts(id, signal),
		}),
	alertEvents: (id: number) =>
		queryOptions({
			queryKey: [...key.alerting(id), 'events'],
			queryFn: ({ signal }) => api.alertEvents(id, signal),
		}),
	listWebhooks: (id: number) =>
		queryOptions({
			queryKey: [...key.alerting(id), 'webhooks'],
			queryFn: ({ signal }) => api.listWebhooks(id, signal),
		}),
	listAlertRoles: (id: number) =>
		queryOptions({
			queryKey: [...key.alerting(id), 'roles'],
			queryFn: ({ signal }) => api.listAlertRoles(id, signal),
		}),

	systemDetails: (id: number, mss: number) =>
		queryOptions({
			queryKey: key.systemDetails(id, mss),
			queryFn: ({ signal }) => api.systemDetails(id, mss, signal),
		}),
	listConnectionJumps: (id: number, connectionId: number) =>
		queryOptions({
			queryKey: key.connectionJumps(id, connectionId),
			queryFn: ({ signal }) => api.listConnectionJumps(id, connectionId, signal),
		}),
	searchMap: (id: number, term: string) =>
		queryOptions({
			queryKey: ['maps', id, 'search', term],
			queryFn: ({ signal }) => api.searchMap(id, term, signal),
		}),

	searchSystems: (term: string) =>
		queryOptions({
			queryKey: ['search', 'systems', term],
			queryFn: ({ signal }) => api.searchSystems(term, signal),
		}),
	searchShips: (term: string) =>
		queryOptions({
			queryKey: ['search', 'ships', term],
			queryFn: ({ signal }) => api.searchShips(term, signal),
		}),
	searchAccessSubjects: (term: string) =>
		queryOptions({
			queryKey: ['search', 'access-subjects', term],
			queryFn: ({ signal }) => api.searchAccessSubjects(term, signal),
		}),

	effectModifiers: (name: string, wormholeClassId: number) =>
		queryOptions({
			queryKey: ['effects', name, wormholeClassId],
			queryFn: ({ signal }) => api.effectModifiers(name, wormholeClassId, signal),
			staleTime: Infinity,
		}),
	threatAnalysis: (solarSystemId: number) =>
		queryOptions({
			queryKey: ['threat', solarSystemId],
			queryFn: ({ signal }) => api.threatAnalysis(solarSystemId, signal),
		}),
	signatureCatalog: () =>
		queryOptions({
			queryKey: ['reference', 'signature-catalog'],
			queryFn: ({ signal }) => api.signatureCatalog(signal),
			staleTime: Infinity,
			gcTime: Infinity,
		}),
	routingGraph: () =>
		queryOptions({
			queryKey: ['reference', 'routing-graph'],
			queryFn: ({ signal }) => api.routingGraph(signal),
			staleTime: Infinity,
			gcTime: Infinity,
		}),
};

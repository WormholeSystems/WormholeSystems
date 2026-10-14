// The Station condition's optional filters: an owner (a corporation, or a whole faction)
// and a service, intersected at the station level so "Caldari Navy stations with repair"
// is answerable from the groups the routing graph already carries.

import type { StationGroup } from '../../../state/route-planner.svelte';

export type OwnerPick = { kind: 'faction' | 'corp'; id: number } | null;

export interface FactionOption {
	id: number;
	name: string;
}

/** The factions that own stations, each once, alphabetically. */
export function factionOptions(corporations: StationGroup[]): FactionOption[] {
	const seen = new Map<number, FactionOption>();
	for (const corp of corporations) {
		if (corp.faction) seen.set(corp.faction.id, corp.faction);
	}
	return [...seen.values()].sort((a, b) => a.name.localeCompare(b.name));
}

/** Corporations sorted by faction first, then their own name; factionless ones last. */
export function byFaction(corporations: StationGroup[]): StationGroup[] {
	return [...corporations].sort(
		(a, b) =>
			(a.faction === null ? 1 : 0) - (b.faction === null ? 1 : 0) ||
			(a.faction?.name ?? '').localeCompare(b.faction?.name ?? '') ||
			a.name.localeCompare(b.name),
	);
}

/** Case-insensitive match on the corporation's own name or its faction's. */
export function matchesOwner(corp: StationGroup, query: string): boolean {
	return (
		corp.name.toLowerCase().includes(query) ||
		(corp.faction?.name.toLowerCase().includes(query) ?? false)
	);
}

function merged(groups: StationGroup[], id: number, name: string): StationGroup {
	const stationsBySystem = new Map<number, { id: number; name: string }[]>();
	for (const group of groups) {
		for (const [system, stations] of group.stationsBySystem) {
			stationsBySystem.set(system, [...(stationsBySystem.get(system) ?? []), ...stations]);
		}
	}
	return { id, name, faction: null, systems: new Set(stationsBySystem.keys()), stationsBySystem };
}

function restrictedTo(group: StationGroup, stationIds: ReadonlySet<number>): StationGroup {
	const stationsBySystem = new Map<number, { id: number; name: string }[]>();
	for (const [system, stations] of group.stationsBySystem) {
		const kept = stations.filter((s) => stationIds.has(s.id));
		if (kept.length > 0) stationsBySystem.set(system, kept);
	}
	return { ...group, systems: new Set(stationsBySystem.keys()), stationsBySystem };
}

/**
 * The stations the picked owner and service agree on, grouped per system; null when
 * neither is picked (every NPC station matches, and there is no list to expand).
 */
export function stationFilter(
	owner: OwnerPick,
	serviceId: number | null,
	corporations: StationGroup[],
	services: StationGroup[],
): StationGroup | null {
	const service = serviceId === null ? null : (services.find((s) => s.id === serviceId) ?? null);
	let picked: StationGroup | null = null;
	if (owner?.kind === 'corp') {
		// A pick naming a corporation the list does not carry matches nothing, the same
		// as a dangling faction: an owner constraint never silently widens.
		const group = corporations.filter((c) => c.id === owner.id);
		picked = merged(group, owner.id, group[0]?.name ?? '…');
	} else if (owner?.kind === 'faction') {
		const members = corporations.filter((c) => c.faction?.id === owner.id);
		picked = merged(members, owner.id, members[0]?.faction?.name ?? '…');
	}
	if (picked === null) return service;
	if (service === null) return picked;
	const allowed = new Set([...service.stationsBySystem.values()].flat().map((s) => s.id));
	return restrictedTo(picked, allowed);
}

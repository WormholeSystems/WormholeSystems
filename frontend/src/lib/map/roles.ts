import type { Role } from '$lib/api/types/Role';

/** Matches the backend's `Role: Ord`, most privileged first. */
const RANK = { owner: 0, manager: 1, member: 2, viewer: 3 } satisfies Record<Role, number>;

/** No role at all is a guest watching a shared map, which is a viewer and nothing more. */
export function atLeast(role: Role | null | undefined, min: Role): boolean {
	return RANK[role ?? 'viewer'] <= RANK[min];
}

export const ROLE_LABEL = {
	viewer: 'Viewer',
	member: 'Member',
	manager: 'Manager',
	owner: 'Owner',
} satisfies Record<Role, string>;

/** What each role may do, in the order they gain it. Shown wherever a role is explained. */
export const ROLE_HELP = {
	viewer: 'Reads the chain: systems, connections, signatures and notes. Changes nothing.',
	member: 'Everything a viewer does, and maps: systems, connections, signatures, intel.',
	manager: 'Everything a member does, and runs the map: access, naming, alerts, settings.',
	owner: 'Everything a manager does, and can delete the map.',
} satisfies Record<Role, string>;

/** The role's text colour in the map list. */
export const ROLE_TONE = {
	owner: 'text-amber-500',
	manager: 'text-purple-400',
	member: 'text-emerald-500',
	viewer: 'text-muted-foreground',
} satisfies Record<Role, string>;

/** Least privileged first, which is the order they are explained in. */
export const ROLES_ASCENDING: Role[] = ['viewer', 'member', 'manager', 'owner'];

/** Owners-first ordering, for the access list. */
export function byRole(a: Role, b: Role): number {
	return RANK[a] - RANK[b];
}

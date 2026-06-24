import { classSortWeight } from '@/const/solarsystemClasses';
import { TMapSolarsystem } from '@/pages/maps';

export function sortByAlias(a: TMapSolarsystem, b: TMapSolarsystem): number {
    if (a.alias && !b.alias) return 1;
    if (!a.alias && b.alias) return -1;

    return a.alias?.localeCompare(b.alias ?? '') || 0;
}

export function sortByClass(a: TMapSolarsystem, b: TMapSolarsystem): number {
    return classSortWeight(a.solarsystem.class) - classSortWeight(b.solarsystem.class);
}

export function sortByRegion(a: TMapSolarsystem, b: TMapSolarsystem): number {
    return a.solarsystem?.region?.name.localeCompare(b.solarsystem?.region?.name ?? '') || 0;
}

export function sortByName(a: TMapSolarsystem, b: TMapSolarsystem): number {
    return a.solarsystem?.name.localeCompare(b.solarsystem?.name ?? '') || 0;
}

/**
 * The canonical system ordering: alias first, then class, then region, then name.
 * Shared by the organize-selection action and the tree layout's sibling order so the
 * two stay consistent.
 */
export function compareSystems(a: TMapSolarsystem, b: TMapSolarsystem): number {
    return sortByAlias(a, b) || sortByClass(a, b) || sortByRegion(a, b) || sortByName(a, b);
}

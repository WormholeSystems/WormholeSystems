import { TMapSolarSystem } from '@/types/models';
import { getSecurityClass } from './security';

export function sortByAlias(a: TMapSolarSystem, b: TMapSolarSystem): number {
    if (a.alias && !b.alias) return 1;
    if (!a.alias && b.alias) return -1;

    return a.alias?.localeCompare(b.alias ?? '') || 0;
}

export function sortByClass(a: TMapSolarSystem, b: TMapSolarSystem): number {
    if (a.class && !b.class) return 1;
    if (!a.class && b.class) return -1;

    const a_security = getSecurityClass(a.solarsystem?.security ?? 0);
    const b_security = getSecurityClass(b.solarsystem?.security ?? 0);

    return a_security.localeCompare(b_security);
}

export function sortByRegion(a: TMapSolarSystem, b: TMapSolarSystem): number {
    return a.solarsystem?.region?.name.localeCompare(b.solarsystem?.region?.name ?? '') || 0;
}

export function sortByName(a: TMapSolarSystem, b: TMapSolarSystem): number {
    return a.solarsystem?.name.localeCompare(b.solarsystem?.name ?? '') || 0;
}

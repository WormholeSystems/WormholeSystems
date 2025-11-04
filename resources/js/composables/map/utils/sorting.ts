import { TMapSolarsystem } from '@/pages/maps';
import { getSecurityClass } from './security';

export function sortByAlias(a: TMapSolarsystem, b: TMapSolarsystem): number {
    if (a.alias && !b.alias) return 1;
    if (!a.alias && b.alias) return -1;

    return a.alias?.localeCompare(b.alias ?? '') || 0;
}

export function sortByClass(a: TMapSolarsystem, b: TMapSolarsystem): number {
    if (a.solarsystem.class && !b.solarsystem.class) return 1;
    if (!a.solarsystem.class && b.solarsystem.class) return -1;

    const a_security = getSecurityClass(a.solarsystem?.security ?? 0);
    const b_security = getSecurityClass(b.solarsystem?.security ?? 0);

    return a_security.localeCompare(b_security);
}

export function sortByRegion(a: TMapSolarsystem, b: TMapSolarsystem): number {
    return a.solarsystem?.region?.name.localeCompare(b.solarsystem?.region?.name ?? '') || 0;
}

export function sortByName(a: TMapSolarsystem, b: TMapSolarsystem): number {
    return a.solarsystem?.name.localeCompare(b.solarsystem?.name ?? '') || 0;
}

export function getSolarsystemClass(system: TMapSolarsystem): number | string {
    if (system.solarsystem.class) return system.solarsystem.class;

    return getSecurityClass(system.solarsystem!.security);
}

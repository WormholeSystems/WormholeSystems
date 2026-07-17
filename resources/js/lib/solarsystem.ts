import { TSolarsystemType } from '@/types/models';

/**
 * Whether a solar system is a wormhole (j-space) system. The static data stores
 * wormhole systems with the type "wh"; k-space systems use "eve".
 */
export function isWormholeSystem(solarsystem?: { type: TSolarsystemType } | null): boolean {
    return solarsystem?.type === 'wh';
}

/** "ALIAS (Name)" when the system carries an alias, otherwise just the name. */
export function aliasedSolarsystemLabel(alias: string | null | undefined, name: string): string {
    return alias ? `${alias} (${name})` : name;
}

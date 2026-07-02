import { TSolarsystemType } from '@/types/models';

/**
 * Whether a solar system is a wormhole (j-space) system. The static data stores
 * wormhole systems with the type "wh"; k-space systems use "eve".
 */
export function isWormholeSystem(solarsystem?: { type: TSolarsystemType } | null): boolean {
    return solarsystem?.type === 'wh';
}

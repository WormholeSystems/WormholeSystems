import { TSolarsystemType } from '@/types/models';
import { mapState } from '../state';
import { TDataMapSolarSystem } from '../types';

export function applyScale(system: TDataMapSolarSystem): TDataMapSolarSystem {
    if (!system.position) return system;
    const scale = mapState.scale;
    return {
        ...system,
        position: {
            x: scale * system.position.x,
            y: scale * system.position.y,
        },
    };
}

/**
 * Whether a solar system is a wormhole (j-space) system. The static data stores
 * wormhole systems with the type "wh"; k-space systems use "eve".
 */
export function isWormholeSystem(solarsystem?: { type: TSolarsystemType } | null): boolean {
    return solarsystem?.type === 'wh';
}

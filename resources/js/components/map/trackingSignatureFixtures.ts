import type { TMapConnection, TMapSolarsystem, TWormhole } from '@/pages/maps';
import type { TLifetimeStatus, TMassStatus, TShipSize, TSignature, TSignatureCategory, TStringedSolarsystemClass } from '@/types/models';

/**
 * Shared TSignature fixtures for the tracking signature dialog: consumed by the
 * component spec and the local debug preview page so both feed the dialog the
 * same shapes.
 */

export const ORIGIN_ID = 1;

export const ORIGIN = {
    id: ORIGIN_ID,
    alias: 'HOME',
    solarsystem: { name: 'J152820' },
} as TMapSolarsystem;

/** Map systems the connected fixtures can lead to. */
export const MAP_SOLARSYSTEMS = [
    ORIGIN,
    { id: 2, alias: 'HOME-A', solarsystem: { name: 'J145510' } } as TMapSolarsystem,
    { id: 3, alias: null, solarsystem: { name: 'J104859' } } as TMapSolarsystem,
];

let nextId = 1;

export function resetSignatureIds(): void {
    nextId = 1;
}

export function sig(input: {
    signatureId: string;
    category?: string;
    /** Wormhole code shown in the type column; defaults to X702 when only a target class is given. */
    code?: string;
    targetClass?: TStringedSolarsystemClass | 'unknown';
    /** Maximum jump mass of the identified wormhole type; locks the ship size select. */
    jumpMass?: number;
    extra?: string | null;
    rawTypeName?: string;
    /** Marks the signature as already tied to a connection leading to this map solarsystem id. */
    connectedToId?: number;
    lifetime?: TLifetimeStatus;
    massStatus?: TMassStatus;
    shipSize?: TShipSize;
}): TSignature {
    const id = nextId++;
    const code = input.code ?? (input.targetClass ? 'X702' : undefined);

    return {
        id,
        signature_id: input.signatureId,
        signature_type_id: code ? id : null,
        signature_category_id: input.category ? id : null,
        raw_type_name: input.rawTypeName ?? null,
        map_connection_id: input.connectedToId ? id : null,
        map_connection: input.connectedToId
            ? ({ id, from_map_solarsystem_id: ORIGIN_ID, to_map_solarsystem_id: input.connectedToId } as TMapConnection)
            : null,
        signature_type: code
            ? {
                  id,
                  name: `${code} - ${input.targetClass ?? '?'}`,
                  signature: code,
                  signature_category_id: id,
                  category_name: input.category === 'wormhole' ? 'Wormhole' : (input.category ?? null),
                  target_class: input.targetClass ?? null,
                  extra: input.extra ?? null,
                  spawn_areas: null,
              }
            : null,
        signature_category: input.category ? ({ id, name: input.category, code: input.category } as TSignatureCategory) : null,
        wormhole: input.jumpMass ? ({ id, name: code ?? 'K162', maximum_jump_mass: input.jumpMass } as TWormhole) : null,
        lifetime: input.lifetime ?? 'healthy',
        mass_status: input.massStatus ?? null,
        ship_size: input.shipSize ?? null,
    } as TSignature;
}

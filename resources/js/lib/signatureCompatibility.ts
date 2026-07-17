import type { TSignature, TStringedSolarsystemClass } from '@/types/models';

/**
 * Whether a scanned signature can be a wormhole connection at all: only
 * signatures categorised as wormholes, or not yet categorised, qualify.
 * Gas, data, relic, combat and ore sites are never connections.
 */
export function signatureCanBeConnection(signature: TSignature): boolean {
    const categoryCode = signature.signature_category?.code;

    return !categoryCode || categoryCode === 'wormhole';
}

/**
 * Whether a signature's assigned wormhole type could lead into a system of the
 * given class. A type with a concrete destination class only fits that exact
 * class — jumping a "leads to Nullsec" hole cannot land you in a C4.
 * Unresolved types and types with an unknown destination (e.g. a bare K162)
 * can always fit.
 */
export function signatureCanLeadToClass(signature: TSignature, targetClass: TStringedSolarsystemClass | null | undefined): boolean {
    const destinationClass = signature.signature_type?.target_class;
    if (!destinationClass || destinationClass === 'unknown') {
        return true;
    }

    if (!targetClass) {
        return true;
    }

    return destinationClass === targetClass;
}

export type TSignatureOptionGroups = {
    /** Unmapped signatures whose type (if any) can lead to the target class. */
    likely: TSignature[];
    /** Signatures already tied to a mapped connection. */
    connected: TSignature[];
    /** Unmapped signatures typed with a destination class that cannot match. */
    unlikely: TSignature[];
};

/**
 * Group a system's signatures into the jump prompt's three sections. Site
 * signatures are dropped entirely; a signature that is already part of a
 * mapped connection cannot be the newly jumped hole regardless of its type.
 */
export function groupSignatureOptions(signatures: TSignature[], targetClass: TStringedSolarsystemClass | null | undefined): TSignatureOptionGroups {
    const candidates = signatures.filter(signatureCanBeConnection);

    return {
        likely: candidates.filter((signature) => !signature.map_connection_id && signatureCanLeadToClass(signature, targetClass)),
        connected: candidates.filter((signature) => Boolean(signature.map_connection_id)),
        unlikely: candidates.filter((signature) => !signature.map_connection_id && !signatureCanLeadToClass(signature, targetClass)),
    };
}

import type { TSignature, TStringedSolarsystemClass } from '@/types/models';

/**
 * Whether a scanned signature could be the wormhole that led into a system of
 * the given class.
 *
 * A signature categorised as anything other than a wormhole cannot be a
 * connection at all. A wormhole type with a concrete destination class only
 * fits that exact class — jumping a "leads to Nullsec" hole cannot land you in
 * a C4. Unscanned signatures, unresolved types, and types with an unknown
 * destination (e.g. a bare K162) can always fit.
 */
export function signatureCanLeadToClass(signature: TSignature, targetClass: TStringedSolarsystemClass | null | undefined): boolean {
    const categoryCode = signature.signature_category?.code;
    if (categoryCode && categoryCode !== 'wormhole') {
        return false;
    }

    const destinationClass = signature.signature_type?.target_class;
    if (!destinationClass || destinationClass === 'unknown') {
        return true;
    }

    if (!targetClass) {
        return true;
    }

    return destinationClass === targetClass;
}

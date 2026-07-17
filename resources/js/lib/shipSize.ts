import type { TShipSize } from '@/types/models';

/** The compact letters the map edges use for ship sizes. */
export const SHIP_SIZE_LETTERS = { frigate: 'S', medium: 'M', large: 'L', xlarge: 'XL' } as const satisfies Record<TShipSize, string>;

/** The selectable ship sizes with their display label and letter, in size order. */
export const SHIP_SIZE_OPTIONS = [
    { value: 'frigate', label: 'Frigate', letter: SHIP_SIZE_LETTERS.frigate },
    { value: 'medium', label: 'Medium', letter: SHIP_SIZE_LETTERS.medium },
    { value: 'large', label: 'Large', letter: SHIP_SIZE_LETTERS.large },
    { value: 'xlarge', label: 'Extra Large', letter: SHIP_SIZE_LETTERS.xlarge },
] as const satisfies ReadonlyArray<{ value: TShipSize; label: string; letter: string }>;

/**
 * The largest ship class that fits through a wormhole with the given maximum
 * jump mass, using the standard 5M / 62M / <2B / 2B+ kg tiers. Unknown or
 * zero masses resolve to null.
 *
 * Mirrored on the backend in App\Enums\ShipSize::fromJumpMass().
 */
export function shipSizeFromJumpMass(maximumJumpMass: number | null | undefined): TShipSize | null {
    if (!maximumJumpMass || maximumJumpMass <= 0) {
        return null;
    }

    if (maximumJumpMass <= 5_000_000) {
        return 'frigate';
    }

    if (maximumJumpMass <= 62_000_000) {
        return 'medium';
    }

    if (maximumJumpMass < 2_000_000_000) {
        return 'large';
    }

    return 'xlarge';
}

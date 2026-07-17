import { shipSizeFromJumpMass } from '@/lib/shipSize';
import { describe, expect, it } from 'vitest';

describe('shipSizeFromJumpMass', () => {
    it('maps the standard jump mass tiers to ship sizes', function () {
        expect(shipSizeFromJumpMass(5_000_000)).toBe('frigate');
        expect(shipSizeFromJumpMass(62_000_000)).toBe('medium');
        expect(shipSizeFromJumpMass(375_000_000)).toBe('large');
        expect(shipSizeFromJumpMass(1_000_000_000)).toBe('large');
        expect(shipSizeFromJumpMass(2_000_000_000)).toBe('xlarge');
    });

    it('resolves unknown and zero masses to null', function () {
        expect(shipSizeFromJumpMass(0)).toBeNull();
        expect(shipSizeFromJumpMass(null)).toBeNull();
        expect(shipSizeFromJumpMass(undefined)).toBeNull();
    });
});

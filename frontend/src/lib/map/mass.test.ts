import { describe, expect, it } from 'vitest';

import { jumpMassLetter, remainingMass, sizeForJumpMass, sizeLetter } from './mass';

describe('sizeForJumpMass', () => {
	// The bands are the game's, and both call sites fall through to "unset" on null rather
	// than guessing, so the boundaries are the whole behaviour.
	it('places each band by its upper bound', () => {
		expect(sizeForJumpMass(5_000_000)).toBe('small');
		expect(sizeForJumpMass(5_000_001)).toBe('medium');
		expect(sizeForJumpMass(62_000_000)).toBe('medium');
		expect(sizeForJumpMass(62_000_001)).toBe('large');
		expect(sizeForJumpMass(300_000_000)).toBe('large');
		expect(sizeForJumpMass(375_000_000)).toBe('large');
		expect(sizeForJumpMass(1_000_000_000)).toBe('xl');
		expect(sizeForJumpMass(2_000_000_000)).toBe('xl');
	});

	it('has no answer without a mass', () => {
		expect(sizeForJumpMass(null)).toBeNull();
		expect(sizeForJumpMass(undefined)).toBeNull();
		expect(sizeForJumpMass(0)).toBeNull();
	});
});

describe('remainingMass', () => {
	it('splits what is left between kilograms and percent', () => {
		expect(remainingMass(1_000_000_000, 250_000_000)).toEqual({ kg: 750_000_000, percent: 75 });
	});

	it('clamps an overshot hole at empty instead of going negative', () => {
		expect(remainingMass(1_000_000_000, 1_200_000_000)).toEqual({ kg: 0, percent: 0 });
	});

	it('says nothing when the total is unknown or nonsense', () => {
		expect(remainingMass(null, 100)).toBeNull();
		expect(remainingMass(0, 100)).toBeNull();
		expect(remainingMass(-5, 100)).toBeNull();
	});
});

describe('jumpMassLetter', () => {
	// A static's details and its edge badge used to disagree on a C1 hole (62M) being large.
	it('reads the same letter as the edge badge for the same mass', () => {
		for (const kg of [5_000_000, 62_000_000, 300_000_000, 375_000_000, 1_350_000_000]) {
			expect(jumpMassLetter(kg)).toBe(sizeLetter(sizeForJumpMass(kg)!));
		}
		expect(jumpMassLetter(62_000_000)).toBe('M');
		expect(jumpMassLetter(1_000_000_000)).toBe('XL');
	});

	it('shows a dash without a mass', () => {
		expect(jumpMassLetter(null)).toBe('—');
	});
});

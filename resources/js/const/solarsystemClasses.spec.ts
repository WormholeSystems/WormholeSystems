import { compareSolarsystemsByClass } from '@/const/solarsystemClasses';
import type { TStringedSolarsystemClass } from '@/types/models';
import { describe, expect, it } from 'vitest';

function system(name: string, klass: TStringedSolarsystemClass): { name: string; class: TStringedSolarsystemClass } {
    return { name, class: klass };
}

describe('compareSolarsystemsByClass', () => {
    it('orders known space before wormhole space', () => {
        expect(compareSolarsystemsByClass(system('Jita', 'h'), system('J152820', '5'))).toBeLessThan(0);
        expect(compareSolarsystemsByClass(system('J152820', '5'), system('Tama', 'l'))).toBeGreaterThan(0);
    });

    it('orders known space high to null', () => {
        expect(compareSolarsystemsByClass(system('Jita', 'h'), system('Tama', 'l'))).toBeLessThan(0);
        expect(compareSolarsystemsByClass(system('Tama', 'l'), system('VFK-IV', 'n'))).toBeLessThan(0);
    });

    it('orders wormhole classes numerically', () => {
        expect(compareSolarsystemsByClass(system('J100001', '1'), system('J100002', '5'))).toBeLessThan(0);
    });

    it('sorts alphabetically within the same class', () => {
        const sorted = [system('Jita', 'h'), system('Amarr', 'h'), system('Dodixie', 'h')].toSorted(compareSolarsystemsByClass);
        expect(sorted.map((s) => s.name)).toEqual(['Amarr', 'Dodixie', 'Jita']);
    });
});

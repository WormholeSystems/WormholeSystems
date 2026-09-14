// @vitest-environment happy-dom
import Edge from '@/map/components/edges/Edge.vue';
import type { EdgeGeometry } from '@/map/core/types';
import type { TMapConnection } from '@/pages/maps';
import type { TShipSize } from '@/types/models';
import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';

const geometry: EdgeGeometry = {
    id: 1,
    kind: 'curve',
    from: { x: 0, y: 0 },
    to: { x: 100, y: 100 },
};

function connection(overrides: Partial<TMapConnection> = {}): TMapConnection {
    return {
        id: 1,
        from_map_solarsystem_id: 1,
        to_map_solarsystem_id: 2,
        type: 'wormhole',
        preserve_mass: false,
        mass_status: 'fresh',
        lifetime_status: 'healthy',
        lifetime_status_updated_at: null,
        signatures: [],
        ship_size: 'large',
        created_at: '2026-01-01T00:00:00Z',
        updated_at: '2026-01-01T00:00:00Z',
        ...overrides,
    } as TMapConnection;
}

function badgeText(overrides: Partial<TMapConnection> = {}): string {
    return mount(Edge, { props: { geometry, connection: connection(overrides), scale: 1 } })
        .text()
        .trim();
}

describe('Edge ship size badge', () => {
    it.each<[TShipSize, string]>([
        ['frigate', 'S'],
        ['medium', 'M'],
        ['large', 'L'],
        ['xlarge', 'XL'],
    ])('labels a %s wormhole with %s', (ship_size, letter) => {
        expect(badgeText({ ship_size })).toBe(letter);
    });

    it('omits the badge for stargates, whose size is only a column default', () => {
        expect(badgeText({ type: 'stargate', ship_size: 'large' })).toBe('');
    });

    it('omits the badge when no size is known', () => {
        expect(badgeText({ ship_size: null })).toBe('');
    });
});

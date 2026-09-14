// @vitest-environment happy-dom
import ConnectionStatus from '@/map/components/overlays/connection/ConnectionStatus.vue';
import type { TMapConnection } from '@/pages/maps';
import { mount } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const MARKED = '2026-09-14T12:00:00Z';

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
        created_at: MARKED,
        updated_at: MARKED,
        ...overrides,
    } as TMapConnection;
}

function render(overrides: Partial<TMapConnection> = {}): string {
    return mount(ConnectionStatus, {
        props: { connection: connection(overrides) },
        global: {
            // The date tooltips need a TooltipProvider; stub them away.
            stubs: {
                Tooltip: { template: '<div><slot /></div>' },
                TooltipTrigger: { template: '<div><slot /></div>' },
                TooltipContent: { template: '<div><slot /></div>' },
            },
        },
    }).text();
}

beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date(Date.parse(MARKED) + 90 * 60 * 1000));
});

afterEach(() => {
    vi.useRealTimers();
});

describe('ConnectionStatus lifetime countdown', () => {
    it('counts down what is left of the four hours after an end of life marking', () => {
        const text = render({ lifetime_status: 'eol', lifetime_status_updated_at: MARKED });

        expect(text).toContain('Collapses in');
        expect(text).toContain('2h 30m 0s');
    });

    it('shows no countdown for a healthy connection', () => {
        expect(render()).not.toContain('Collapses in');
    });
});

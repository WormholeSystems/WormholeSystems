// @vitest-environment happy-dom
import ConnectionStatus from '@/map/components/overlays/connection/ConnectionStatus.vue';
import type { TMapConnection, TMapSolarsystem } from '@/pages/maps';
import { mount } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const MARKED = '2026-09-14T12:00:00Z';

function system(solarsystem_class: string): TMapSolarsystem {
    return { id: 1, solarsystem: { class: solarsystem_class } } as unknown as TMapSolarsystem;
}

function connection(overrides: Partial<TMapConnection> = {}): TMapConnection & { source: TMapSolarsystem; target: TMapSolarsystem } {
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
        source: system('5'),
        target: system('3'),
        ...overrides,
    } as unknown as TMapConnection & { source: TMapSolarsystem; target: TMapSolarsystem };
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

        expect(text).toContain('Time remaining');
        expect(text).toContain('2h 30m');
    });

    it('counts a healthy connection down from the lifetime its shape implies', () => {
        const text = render();

        expect(text).toContain('Time remaining');
        expect(text).toContain('22h 30m');
    });

    it('shows no countdown for a stargate, which is permanent', () => {
        expect(render({ type: 'stargate' })).not.toContain('Time remaining');
    });
});

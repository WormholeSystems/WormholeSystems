// @vitest-environment happy-dom
import { Popover, PopoverTrigger } from '@/components/ui/popover';
import MapConnectionDetails from '@/map/components/overlays/MapConnectionDetails.vue';
import type { TMapConnection, TMapSolarsystem } from '@/pages/maps';
import { flushPromises, mount } from '@vue/test-utils';
import { afterEach, describe, expect, it } from 'vitest';
import { h } from 'vue';

const system = { id: 1, solarsystem: { id: 30000001, name: 'J123456', class: 'c3' } } as unknown as TMapSolarsystem;

type DetailsConnection = TMapConnection & { source: TMapSolarsystem; target: TMapSolarsystem };

function connection(overrides: Partial<TMapConnection> = {}): DetailsConnection {
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
        jumps: [],
        jumps_mass_sum: 0,
        ship_size: 'large',
        created_at: '2026-01-01T00:00:00Z',
        updated_at: '2026-01-01T00:00:00Z',
        source: system,
        target: system,
        ...overrides,
    } as unknown as DetailsConnection;
}

/** The popover content portals to the body, so the panel is read from there. */
async function renderPanel(connection: DetailsConnection): Promise<HTMLElement> {
    mount(Popover, {
        props: { open: true },
        slots: { default: () => [h(PopoverTrigger, () => 'open'), h(MapConnectionDetails, { connection })] },
        attachTo: document.body,
        global: {
            stubs: {
                SignatureSection: true,
                ConnectionStatus: true,
                WormholeProperties: true,
                MassTracking: true,
            },
        },
    });

    await flushPromises();

    const panel = document.querySelector<HTMLElement>('[data-slot="popover-content"]');
    expect(panel).not.toBeNull();

    return panel!;
}

afterEach(() => {
    document.body.innerHTML = '';
});

describe('MapConnectionDetails', () => {
    it('caps the panel to the space the popover has and scrolls past it', async () => {
        const panel = await renderPanel(connection());

        expect(panel.className).toContain('max-h-(--reka-popover-content-available-height)');
        expect(panel.className).toContain('overflow-y-auto');
    });

    it('widens into a second column for the mass tracking list', async () => {
        const panel = await renderPanel(connection());

        expect(panel.className).toContain('sm:w-[30rem]');
        expect(panel.querySelector('.grid')?.className).toContain('sm:grid-cols-2');
        expect(panel.querySelector('mass-tracking-stub')).not.toBeNull();
    });

    it('stays one narrow column for stargates, which track no mass', async () => {
        const panel = await renderPanel(connection({ type: 'stargate' }));

        expect(panel.className).not.toContain('sm:w-[30rem]');
        expect(panel.querySelector('.grid')?.className).not.toContain('sm:grid-cols-2');
        expect(panel.querySelector('mass-tracking-stub')).toBeNull();
    });
});

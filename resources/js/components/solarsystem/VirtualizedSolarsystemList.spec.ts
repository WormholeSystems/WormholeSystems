// @vitest-environment happy-dom
import VirtualizedSolarsystemList from '@/components/solarsystem/VirtualizedSolarsystemList.vue';
import { Combobox } from '@/components/ui/combobox';
import { flushVirtualizer, stubElementMeasurements } from '@/components/ui/combobox/comboboxTestUtils';
import type { TComboboxSection } from '@/lib/comboboxSections';
import type { TStaticSolarsystem } from '@/types/static-data';
import { mount } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { defineComponent } from 'vue';

function solarsystem(id: number, name: string): TStaticSolarsystem {
    return {
        id,
        name,
        region_id: 1,
        constellation_id: 1,
        class: 'h',
        security: 0.9,
        type: 'eve',
        region: { id: 1, name: 'The Forge' },
        sovereignty: null,
        statics: null,
        effect: null,
        has_jove_observatory: false,
        has_stations: true,
        services: [],
        connection_type: null,
    } as unknown as TStaticSolarsystem;
}

const selections: Array<{ name: string; section: string }> = [];

function mountList(sections: TComboboxSection<TStaticSolarsystem>[]) {
    const harness = defineComponent({
        components: { Combobox, VirtualizedSolarsystemList },
        setup() {
            return {
                sections,
                handleSelect: (system: TStaticSolarsystem, section: string) => selections.push({ name: system.name, section }),
            };
        },
        template: `
            <Combobox :open="true" :ignore-filter="true">
                <VirtualizedSolarsystemList :sections="sections" @select="handleSelect" />
            </Combobox>
        `,
    });

    return mount(harness, { attachTo: document.body });
}

beforeEach(() => {
    stubElementMeasurements();
    selections.length = 0;
    // The row's sovereignty badge lazily fetches /api/sovereignties.
    vi.stubGlobal(
        'fetch',
        vi.fn(async () => new Response('{}', { status: 200, headers: { 'Content-Type': 'application/json' } })),
    );
});

afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
    document.body.innerHTML = '';
});

describe('VirtualizedSolarsystemList', () => {
    it('renders section headings and system rows without render errors', async () => {
        mountList([
            { key: 'new', heading: 'Search Results', items: [solarsystem(1, 'Jita'), solarsystem(2, 'Perimeter')] },
            { key: 'existing', heading: 'Already in Map', items: [solarsystem(3, 'Amarr')], selectable: false },
        ]);
        await flushVirtualizer();

        const text = document.body.textContent ?? '';
        expect(text).toContain('Search Results');
        expect(text).toContain('Jita');
        expect(text).toContain('Already in Map');
        expect(text).toContain('Amarr');
    });

    it('skips headings of empty sections and shows the empty state when all are empty', async () => {
        mountList([{ key: 'new', heading: 'Search Results', items: [] }]);
        await flushVirtualizer();

        const text = document.body.textContent ?? '';
        expect(text).not.toContain('Search Results');
        expect(text).toContain('No systems found');
    });

    it('emits selection with the section key, but not for non-selectable sections', async () => {
        mountList([
            { key: 'new', heading: 'Search Results', items: [solarsystem(1, 'Jita')] },
            { key: 'existing', heading: 'Already in Map', items: [solarsystem(3, 'Amarr')], selectable: false },
        ]);
        await flushVirtualizer();

        const rows = [...document.body.querySelectorAll('[role="option"]')] as HTMLElement[];
        for (const row of rows) {
            row.dispatchEvent(new MouseEvent('pointerup', { bubbles: true }));
            row.dispatchEvent(new MouseEvent('click', { bubbles: true }));
        }
        await flushVirtualizer();

        expect(selections).toEqual([{ name: 'Jita', section: 'new' }]);
    });
});

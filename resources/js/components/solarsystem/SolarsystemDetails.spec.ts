// @vitest-environment happy-dom
import SolarsystemDetails from '@/components/solarsystem/SolarsystemDetails.vue';
import type { TResolvedSelectedMapSolarsystem } from '@/pages/maps';
import { mount } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import { computed, reactive } from 'vue';

vi.mock('@/composables/usePermission', () => ({
    default: () => ({ canEdit: computed(() => false) }),
}));

vi.mock('@inertiajs/vue3', () => ({
    useForm: (data: Record<string, unknown>) => reactive({ ...data, submit: vi.fn() }),
}));

function renderNotes(notes: string): HTMLElement {
    const wrapper = mount(SolarsystemDetails, {
        props: {
            map_solarsystem: { id: 1, notes } as unknown as TResolvedSelectedMapSolarsystem,
        },
    });

    return wrapper.element as HTMLElement;
}

describe('SolarsystemDetails', () => {
    it('renders notes as markdown', () => {
        const element = renderNotes('**Static** to C5');

        expect(element.querySelector('strong')?.textContent).toBe('Static');
    });

    it('links bare domains and opens links in a new tab', () => {
        const links = renderNotes('Kills on zkillboard.com/system/31000005 and https://evewho.com').querySelectorAll('a');

        expect(Array.from(links, (link) => link.getAttribute('href'))).toEqual(['http://zkillboard.com/system/31000005', 'https://evewho.com']);
        expect(Array.from(links, (link) => link.getAttribute('target'))).toEqual(['_blank', '_blank']);
        expect(Array.from(links, (link) => link.getAttribute('rel'))).toEqual(['noopener', 'noopener']);
    });
});

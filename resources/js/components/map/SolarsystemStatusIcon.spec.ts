// @vitest-environment happy-dom
import SolarsystemStatusIcon from '@/components/map/SolarsystemStatusIcon.vue';
import type { TMapSolarsystemStatus } from '@/types/models';
import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';

const statuses: TMapSolarsystemStatus[] = ['active', 'unscanned', 'unknown', 'friendly', 'hostile', 'empty'];

describe('SolarsystemStatusIcon', () => {
    it('renders a distinct glyph for every status', () => {
        const svgs = statuses.map((status) => {
            const wrapper = mount(SolarsystemStatusIcon, { props: { status } });
            expect(wrapper.attributes('aria-label')).toBe(status);
            return wrapper.html();
        });

        expect(new Set(svgs).size).toBe(statuses.length);
    });

    it('colors the glyph with the matching status token', () => {
        const wrapper = mount(SolarsystemStatusIcon, { props: { status: 'hostile' } });

        expect(wrapper.classes()).toContain('text-hostile');
    });
});

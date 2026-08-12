// @vitest-environment happy-dom
import SolarsystemEffectBadge from '@/components/solarsystem/SolarsystemEffectBadge.vue';
import type { TWormholeEffectName } from '@/types/models';
import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';

const effects: TWormholeEffectName[] = ['Pulsar', 'Cataclysmic Variable', 'Magnetar', 'Red Giant', 'Wolf-Rayet Star', 'Black Hole'];

describe('SolarsystemEffectBadge', () => {
    it('renders a unique letter for every effect', () => {
        const letters = effects.map((name) => mount(SolarsystemEffectBadge, { props: { name } }).text());

        expect(letters).toEqual(['P', 'C', 'M', 'R', 'W', 'B']);
        expect(new Set(letters).size).toBe(effects.length);
    });

    it('labels the badge with the effect name for assistive tech', () => {
        const wrapper = mount(SolarsystemEffectBadge, { props: { name: 'Wolf-Rayet Star' } });

        expect(wrapper.attributes('aria-label')).toBe('Wolf-Rayet Star');
    });

    it('keeps the effect color and switches size via the size prop', () => {
        const md = mount(SolarsystemEffectBadge, { props: { name: 'Pulsar' } });
        expect(md.classes()).toContain('bg-pulsar');
        expect(md.attributes('data-size')).toBe('md');

        const sm = mount(SolarsystemEffectBadge, { props: { name: 'Pulsar', size: 'sm' } });
        expect(sm.attributes('data-size')).toBe('sm');
    });
});

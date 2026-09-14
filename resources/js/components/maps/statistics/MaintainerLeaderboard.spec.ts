// @vitest-environment happy-dom
import MaintainerLeaderboard from '@/components/maps/statistics/MaintainerLeaderboard.vue';
import type { TMaintainerEntry } from '@/types/models';
import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';

function character(overrides: Partial<TMaintainerEntry['characters'][number]> = {}): TMaintainerEntry['characters'][number] {
    return {
        character_id: 1,
        character_name: 'Alt One',
        user_id: 1,
        nb_added: 1,
        nb_edited: 0,
        nb_deleted: 0,
        points: 1,
        ...overrides,
    };
}

function entry(position: number, overrides: Partial<TMaintainerEntry> = {}): TMaintainerEntry {
    return {
        position,
        points: 10,
        user_id: position,
        display_name: `Pilot ${position}`,
        characters: [character({ character_id: position, character_name: `Pilot ${position}` })],
        ...overrides,
    };
}

describe('MaintainerLeaderboard', () => {
    it('shows an empty state when nobody qualifies', () => {
        const wrapper = mount(MaintainerLeaderboard, { props: { entries: [] } });

        expect(wrapper.text()).toContain('Nobody has scored any points this period yet');
        expect(wrapper.findAll('li').length).toBe(0);
    });

    it('gives the top three medals and plain numbers below that', () => {
        const entries = [entry(1), entry(2), entry(3), entry(4)];
        const wrapper = mount(MaintainerLeaderboard, { props: { entries } });

        const rows = wrapper.findAll('li');
        expect(rows).toHaveLength(4);
        expect(rows[0]!.text()).toContain('🥇');
        expect(rows[1]!.text()).toContain('🥈');
        expect(rows[2]!.text()).toContain('🥉');
        expect(rows[3]!.text()).toContain('#4');
    });

    it('renders the per-alt point breakdown for an entry with multiple characters', () => {
        const entries = [
            entry(1, {
                characters: [
                    character({ character_id: 10, character_name: 'Main Toon', points: 7 }),
                    character({ character_id: 11, character_name: 'Scanning Alt', points: 3 }),
                ],
            }),
        ];
        const wrapper = mount(MaintainerLeaderboard, { props: { entries } });

        const breakdown = wrapper.find('details');
        expect(breakdown.exists()).toBe(true);
        expect(breakdown.text()).toContain('Main Toon');
        expect(breakdown.text()).toContain('Scanning Alt');
        expect(breakdown.text()).toContain('7 pts');
        expect(breakdown.text()).toContain('3 pts');
    });

    it('does not render an alt breakdown for a single-character entry', () => {
        const wrapper = mount(MaintainerLeaderboard, { props: { entries: [entry(1)] } });

        expect(wrapper.find('details').exists()).toBe(false);
    });
});

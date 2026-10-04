// @vitest-environment happy-dom
import SignaturesPastedAt from '@/components/signatures/SignaturesPastedAt.vue';
import { mount } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const NOW = '2026-10-04T12:00:00Z';

function render(pasted_at: string | null): string {
    return mount(SignaturesPastedAt, {
        props: { pasted_at },
        global: {
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
    vi.setSystemTime(new Date(NOW));
});

afterEach(() => {
    vi.useRealTimers();
});

describe('SignaturesPastedAt', () => {
    it('renders nothing when signatures were never pasted', () => {
        expect(render(null)).toBe('');
    });

    it('shows just now for a paste within the last minute', () => {
        expect(render('2026-10-04T11:59:30Z')).toContain('just now');
    });

    it.each([
        ['2026-10-04T11:48:00Z', '12m ago'],
        ['2026-10-04T09:30:00Z', '2h ago'],
        ['2026-10-01T12:00:00Z', '3d ago'],
    ])('shows how long ago %s was pasted', (pasted_at, expected) => {
        expect(render(pasted_at)).toContain(expected);
    });

    it('shows the exact paste time in the tooltip', () => {
        expect(render('2026-10-04T11:48:00Z')).toContain('Signatures last pasted Oct 04, 11:48');
    });
});

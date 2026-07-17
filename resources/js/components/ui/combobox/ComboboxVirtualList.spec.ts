// @vitest-environment happy-dom
import { Combobox, ComboboxItem, ComboboxVirtualList } from '@/components/ui/combobox';
import { flushVirtualizer, stubElementMeasurements } from '@/components/ui/combobox/comboboxTestUtils';
import { mount } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { defineComponent } from 'vue';

/**
 * Mount-level coverage for the virtualized combobox scaffold. The virtualizer
 * clones its slot's first vnode per row, which only works when the row root is
 * a concrete element — a regression there crashes at render time, so mounting
 * with an open popup exercises the failure mode directly.
 */

type TOption = { id: number; name: string };

const options: TOption[] = Array.from({ length: 50 }, (_, index) => ({ id: index + 1, name: `Option ${index + 1}` }));

const selected: TOption[] = [];

function mountList(listOptions: TOption[] = options) {
    const harness = defineComponent({
        components: { Combobox, ComboboxItem, ComboboxVirtualList },
        setup() {
            return {
                options: listOptions,
                handleSelect: (option: TOption) => selected.push(option),
                optionName: (option: TOption) => option.name,
            };
        },
        template: `
            <Combobox :open="true" :ignore-filter="true">
                <ComboboxVirtualList :options="options" :text-content="optionName" empty-text="Nothing here">
                    <template #header><input data-testid="search" /></template>
                    <template #default="{ option }">
                        <ComboboxItem :value="option.name" @select.prevent="() => handleSelect(option)">{{ option.name }}</ComboboxItem>
                    </template>
                </ComboboxVirtualList>
            </Combobox>
        `,
    });

    return mount(harness, { attachTo: document.body });
}

beforeEach(() => {
    stubElementMeasurements();
    selected.length = 0;
});

afterEach(() => {
    vi.restoreAllMocks();
    document.body.innerHTML = '';
});

describe('ComboboxVirtualList', () => {
    it('renders virtual option rows without render errors', async () => {
        const warn = vi.spyOn(console, 'warn').mockImplementation(() => {});

        mountList();
        await flushVirtualizer();

        // The popup teleports to the body; the virtualizer renders only a slice.
        const rendered = document.body.querySelectorAll('[role="option"]');
        expect(rendered.length).toBeGreaterThan(0);
        expect(rendered.length).toBeLessThan(options.length);
        expect(document.body.textContent).toContain('Option 1');

        const renderErrors = warn.mock.calls.filter(([message]) => String(message).includes('Unhandled error'));
        expect(renderErrors).toEqual([]);
    });

    it('renders the header slot outside the scroll viewport', async () => {
        mountList();
        await flushVirtualizer();

        expect(document.body.querySelector('[data-testid="search"]')).not.toBeNull();
    });

    it('shows the empty state when there are no options', async () => {
        mountList([]);
        await flushVirtualizer();

        expect(document.body.querySelectorAll('[role="option"]').length).toBe(0);
        expect(document.body.textContent).toContain('Nothing here');
    });

    it('selects an option through its item', async () => {
        mountList();
        await flushVirtualizer();

        const first = document.body.querySelector('[role="option"]') as HTMLElement;
        first.dispatchEvent(new MouseEvent('pointerup', { bubbles: true }));
        first.dispatchEvent(new MouseEvent('click', { bubbles: true }));
        await flushVirtualizer();

        expect(selected.map((option) => option.name)).toContain('Option 1');
    });
});

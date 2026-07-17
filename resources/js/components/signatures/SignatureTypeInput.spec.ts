// @vitest-environment happy-dom
import SignatureTypeInput from '@/components/signatures/SignatureTypeInput.vue';
import { flushVirtualizer, stubElementMeasurements } from '@/components/ui/combobox/comboboxTestUtils';
import type { TSignatureType } from '@/types/models';
import { mount, type VueWrapper } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const types = Array.from({ length: 60 }, (_, index) => ({
    id: index + 1,
    name: `Forgotten Site ${index + 1}`,
    signature: null,
    target_class: null,
    extra: null,
})) as unknown as TSignatureType[];

function mountInput(props: Partial<InstanceType<typeof SignatureTypeInput>['$props']> = {}): VueWrapper {
    return mount(SignatureTypeInput, {
        props: {
            can_write: true,
            options: types,
            category: 'Relic Site',
            modelValue: null,
            ...props,
        },
        attachTo: document.body,
    });
}

async function openPopup(wrapper: VueWrapper): Promise<void> {
    await wrapper.find('button').trigger('click');
    await flushVirtualizer();
}

beforeEach(() => {
    stubElementMeasurements();
});

afterEach(() => {
    vi.restoreAllMocks();
    document.body.innerHTML = '';
});

describe('SignatureTypeInput', () => {
    it('opens and renders only a virtual slice of a large type list', async () => {
        const wrapper = mountInput();
        await openPopup(wrapper);

        const rendered = document.body.querySelectorAll('[role="option"]');
        expect(rendered.length).toBeGreaterThan(0);
        expect(rendered.length).toBeLessThan(types.length);
    });

    it('filters rows through the popup search input', async () => {
        const wrapper = mountInput();
        await openPopup(wrapper);

        const search = document.body.querySelector('input[placeholder="Search types"]') as HTMLInputElement;
        search.value = 'Site 42';
        search.dispatchEvent(new Event('input', { bubbles: true }));
        await flushVirtualizer();

        const text = document.body.textContent ?? '';
        expect(text).toContain('Forgotten Site 42');
        expect(text).not.toContain('Forgotten Site 41');
    });

    it('emits the selected type id', async () => {
        const wrapper = mountInput();
        await openPopup(wrapper);

        const option = document.body.querySelector('[role="option"]') as HTMLElement;
        option.dispatchEvent(new MouseEvent('pointerup', { bubbles: true }));
        option.dispatchEvent(new MouseEvent('click', { bubbles: true }));
        await flushVirtualizer();

        expect(wrapper.emitted('update:modelValue')).toBeTruthy();
    });

    it('is disabled without a category', async () => {
        const wrapper = mountInput({ category: undefined });
        await openPopup(wrapper);

        expect(document.body.querySelectorAll('[role="option"]').length).toBe(0);
    });
});

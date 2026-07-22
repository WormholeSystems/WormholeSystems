// @vitest-environment happy-dom
import WormholeTypeInput from '@/components/signatures/WormholeTypeInput.vue';
import { flushVirtualizer, stubElementMeasurements } from '@/components/ui/combobox/comboboxTestUtils';
import type { TMapUserSetting, TSignatureType } from '@/types/models';
import { mount, type VueWrapper } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { computed } from 'vue';
vi.mock('@/composables/useMapUserSettings', () => ({
    useMapUserSettings: () => computed(() => ({ compact_signature_list: false }) as unknown as TMapUserSetting),
}));

const types = [
    { id: 1, name: 'K162 - C1', signature: 'K162', target_class: '1', extra: null },
    { id: 2, name: 'K162 - C2', signature: 'K162', target_class: '2', extra: null },
    { id: 3, name: 'H296 - C5', signature: 'H296', target_class: '5', extra: null },
    { id: 4, name: 'U210 - L', signature: 'U210', target_class: 'l', extra: null },
] as unknown as TSignatureType[];

function mountInput(props: Partial<InstanceType<typeof WormholeTypeInput>['$props']> = {}): VueWrapper {
    return mount(WormholeTypeInput, {
        props: {
            can_write: true,
            wormhole_options: types,
            current_class: null,
            static_signatures: ['U210'],
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

describe('WormholeTypeInput', () => {
    it('opens and renders grouped virtual rows without render errors', async () => {
        const wrapper = mountInput();
        await openPopup(wrapper);

        const text = document.body.textContent ?? '';
        expect(text).toContain('Statics');
        expect(text).toContain('K162');
        expect(text).toContain('Wormholes');
        expect(text).toContain('U210');
        expect(text).toContain('H296');
        expect(document.body.querySelectorAll('[role="option"]').length).toBeGreaterThan(0);
    });

    it('filters rows through the popup search input', async () => {
        const wrapper = mountInput();
        await openPopup(wrapper);

        const search = document.body.querySelector('input[placeholder="Search types"]') as HTMLInputElement;
        search.value = 'H296';
        search.dispatchEvent(new Event('input', { bubbles: true }));
        await flushVirtualizer();

        const text = document.body.textContent ?? '';
        expect(text).toContain('H296');
        expect(text).not.toContain('U210');
    });

    it('emits the selected type id and closes', async () => {
        const wrapper = mountInput();
        await openPopup(wrapper);

        const option = document.body.querySelector('[role="option"]') as HTMLElement;
        option.dispatchEvent(new MouseEvent('pointerup', { bubbles: true }));
        option.dispatchEvent(new MouseEvent('click', { bubbles: true }));
        await flushVirtualizer();

        const emitted = wrapper.emitted('update:modelValue');
        expect(emitted).toBeTruthy();
        expect(types.map((type) => type.id)).toContain(emitted![0][0]);
    });

    it('shows the selected type on the trigger', async () => {
        const wrapper = mountInput({ modelValue: 3 });

        expect(wrapper.text()).toContain('H296');
    });
});

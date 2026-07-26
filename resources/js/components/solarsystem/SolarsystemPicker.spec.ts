// @vitest-environment happy-dom
import SolarsystemPicker from '@/components/solarsystem/SolarsystemPicker.vue';
import { flushVirtualizer, stubElementMeasurements } from '@/components/ui/combobox/comboboxTestUtils';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import type { TStaticSolarsystem } from '@/types/static-data';
import { mount, type VueWrapper } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { defineComponent, reactive } from 'vue';

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

const solarsystems = [solarsystem(30000142, 'Jita'), solarsystem(30000144, 'Perimeter'), solarsystem(30002187, 'Amarr')];

const { staticDataRef } = await vi.hoisted(async () => {
    const { shallowRef } = await import('vue');
    return { staticDataRef: shallowRef<{ solarsystems: TStaticSolarsystem[] } | null>(null) };
});

vi.mock('@/composables/useStaticData', () => ({
    useStaticData: () => ({
        staticData: staticDataRef,
        loadStaticData: async () => {},
    }),
}));

let wrapper: VueWrapper;

function mountPicker(modelValue = 0): VueWrapper {
    wrapper = mount(SolarsystemPicker, { props: { modelValue }, attachTo: document.body });
    return wrapper;
}

/**
 * Mirrors the alert dialog: the picker sits in a tab panel that unmounts while
 * another tab is open, so the selection has to live in the surrounding form.
 */
function mountInTabs(): VueWrapper {
    const harness = defineComponent({
        components: { SolarsystemPicker, Tabs, TabsContent, TabsList, TabsTrigger },
        setup() {
            return { form: reactive({ target_solarsystem_id: 0 }) };
        },
        template: `
            <Tabs default-value="conditions">
                <TabsList>
                    <TabsTrigger value="conditions">Conditions</TabsTrigger>
                    <TabsTrigger value="delivery">Delivery</TabsTrigger>
                </TabsList>
                <TabsContent value="conditions">
                    <SolarsystemPicker v-model="form.target_solarsystem_id" />
                </TabsContent>
                <TabsContent value="delivery">Delivery settings</TabsContent>
            </Tabs>
        `,
    });

    wrapper = mount(harness, { attachTo: document.body });
    return wrapper;
}

async function openTab(wrapper: VueWrapper, label: string): Promise<void> {
    const trigger = wrapper.findAll('[role="tab"]').find((tab) => tab.text() === label);
    expect(trigger).toBeTruthy();
    await trigger!.trigger('mousedown');
    await flushVirtualizer();
}

function selectedId(wrapper: VueWrapper): number | undefined {
    return (wrapper.emitted('update:modelValue')?.at(-1) as number[] | undefined)?.at(0);
}

function input(wrapper: VueWrapper): HTMLInputElement {
    return wrapper.find('input').element as HTMLInputElement;
}

async function type(wrapper: VueWrapper, value: string): Promise<void> {
    const field = wrapper.find('input');
    (field.element as HTMLInputElement).value = value;
    await field.trigger('input');
    await flushVirtualizer();
}

async function clickRow(name: string): Promise<void> {
    const row = [...document.body.querySelectorAll('[role="option"]')].find((option) => option.textContent?.includes(name));
    expect(row).toBeTruthy();
    row!.dispatchEvent(new MouseEvent('pointerup', { bubbles: true }));
    row!.dispatchEvent(new MouseEvent('click', { bubbles: true }));
    await flushVirtualizer();
}

/** Closing the list resets the search term asynchronously, one tick after the close. */
async function closeList(wrapper: VueWrapper): Promise<void> {
    await wrapper.find('input').trigger('keydown', { key: 'Escape' });
    await new Promise((resolve) => setTimeout(resolve, 20));
    await wrapper.vm.$nextTick();
}

beforeEach(() => {
    staticDataRef.value = { solarsystems };
    stubElementMeasurements();
    // The rows' sovereignty badge lazily fetches /api/sovereignties.
    vi.stubGlobal(
        'fetch',
        vi.fn(async () => new Response('{}', { status: 200, headers: { 'Content-Type': 'application/json' } })),
    );
});

afterEach(() => {
    wrapper?.unmount();
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
    document.body.innerHTML = '';
});

describe('SolarsystemPicker', () => {
    it('selects a searched system', async () => {
        const wrapper = mountPicker();
        await type(wrapper, 'jit');
        await clickRow('Jita');

        expect(selectedId(wrapper)).toBe(30000142);
        expect(input(wrapper).value).toBe('Jita');
    });

    it('keeps the selection and the query when the list closes', async () => {
        const wrapper = mountPicker();
        await type(wrapper, 'jit');
        await clickRow('Jita');
        await wrapper.setProps({ modelValue: 30000142 });

        await closeList(wrapper);

        expect(selectedId(wrapper)).toBe(30000142);
        expect(input(wrapper).value).toBe('Jita');
        expect(wrapper.text()).toContain('Jita');
    });

    it('drops the selection when the query is edited after picking', async () => {
        const wrapper = mountPicker();
        await type(wrapper, 'jit');
        await clickRow('Jita');
        await wrapper.setProps({ modelValue: 30000142 });

        await type(wrapper, 'Amar');

        expect(selectedId(wrapper)).toBe(0);
        expect(input(wrapper).value).toBe('Amar');
    });

    it('keeps a preselected system when it mounts', async () => {
        const wrapper = mountPicker(30000142);
        await flushVirtualizer();

        expect(wrapper.emitted('update:modelValue')).toBeUndefined();
        expect(input(wrapper).value).toBe('Jita');
        expect(wrapper.text()).toContain('Jita');
    });

    it('shows the query for a selection made from the outside', async () => {
        const wrapper = mountPicker();

        await wrapper.setProps({ modelValue: 30002187 });

        expect(input(wrapper).value).toBe('Amarr');
    });

    it('fills the query for a preselected system once the static data arrives', async () => {
        staticDataRef.value = null;
        const wrapper = mountPicker(30000144);
        expect(input(wrapper).value).toBe('');

        staticDataRef.value = { solarsystems };
        await wrapper.vm.$nextTick();

        expect(input(wrapper).value).toBe('Perimeter');
    });

    it('clears the query when the selection is reset from the outside', async () => {
        const wrapper = mountPicker(30002187);
        expect(input(wrapper).value).toBe('Amarr');

        await wrapper.setProps({ modelValue: 0 });

        expect(input(wrapper).value).toBe('');
    });

    it('survives the tab panel unmounting while another tab is open', async () => {
        const wrapper = mountInTabs();
        await type(wrapper, 'jit');
        await clickRow('Jita');
        expect((wrapper.vm as unknown as { form: { target_solarsystem_id: number } }).form.target_solarsystem_id).toBe(30000142);

        await openTab(wrapper, 'Delivery');
        expect(wrapper.find('input').exists()).toBe(false);

        await openTab(wrapper, 'Conditions');

        expect((wrapper.vm as unknown as { form: { target_solarsystem_id: number } }).form.target_solarsystem_id).toBe(30000142);
        expect(input(wrapper).value).toBe('Jita');
        expect(wrapper.text()).toContain('Jita');
    });
});

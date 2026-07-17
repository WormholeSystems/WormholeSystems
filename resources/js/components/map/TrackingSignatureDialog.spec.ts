// @vitest-environment happy-dom
import TrackingSignatureDialog from '@/components/map/TrackingSignatureDialog.vue';
import type { TMapConnection, TMapSolarsystem, TWormhole } from '@/pages/maps';
import type { TSignature } from '@/types/models';
import { mount, type VueWrapper } from '@vue/test-utils';
import { afterEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/composables/useShowMap', () => ({
    useShowMap: () => ({ props: { map: { slug: 'test-map' } } }),
}));

vi.mock('@/map/api', () => ({
    updateMapUserSettings: vi.fn(),
}));

const ORIGIN = { id: 1, alias: 'HOME', solarsystem: { name: 'J152820' } } as TMapSolarsystem;

const MAP_SOLARSYSTEMS = [ORIGIN, { id: 2, alias: 'HOME-A', solarsystem: { name: 'J145510' } } as TMapSolarsystem];

let nextId = 1;

function sig(input: {
    signatureId: string;
    category?: string;
    targetClass?: string;
    connectedToId?: number;
    lifetime?: TSignature['lifetime'];
    shipSize?: TSignature['ship_size'];
    jumpMass?: number;
}): TSignature {
    const id = nextId++;

    return {
        id,
        signature_id: input.signatureId,
        signature_type_id: input.targetClass ? id : null,
        raw_type_name: null,
        map_connection_id: input.connectedToId ? id : null,
        map_connection: input.connectedToId
            ? ({ id, from_map_solarsystem_id: ORIGIN.id, to_map_solarsystem_id: input.connectedToId } as TMapConnection)
            : null,
        signature_type: input.targetClass ? { id, name: `X702 - ${input.targetClass}`, signature: 'X702', target_class: input.targetClass } : null,
        signature_category: input.category ? { id, name: input.category, code: input.category } : null,
        wormhole: input.jumpMass ? ({ id, name: 'X702', maximum_jump_mass: input.jumpMass } as TWormhole) : null,
        lifetime: input.lifetime ?? 'healthy',
        mass_status: null,
        ship_size: input.shipSize ?? null,
    } as TSignature;
}

// Alphabetical likely order: AAA (typed C4, eol, frigate), BBB (unscanned).
// CCC is connected, DDD leads elsewhere, EEE is a gas site and must not render.
function makeSignatures(): TSignature[] {
    nextId = 1;
    return [
        sig({ signatureId: 'AAA-111', category: 'wormhole', targetClass: '4', lifetime: 'eol', shipSize: 'frigate' }),
        sig({ signatureId: 'BBB-222' }),
        sig({ signatureId: 'CCC-333', category: 'wormhole', targetClass: '4', connectedToId: 2 }),
        sig({ signatureId: 'DDD-444', category: 'wormhole', targetClass: 'n' }),
        sig({ signatureId: 'EEE-555', category: 'gas' }),
    ];
}

type DialogProps = Partial<InstanceType<typeof TrackingSignatureDialog>['$props']>;

async function mountOpenDialog(props: DialogProps = {}): Promise<VueWrapper> {
    const wrapper = mount(TrackingSignatureDialog, {
        props: {
            open: false,
            originMapSolarsystem: ORIGIN,
            targetSolarsystemName: 'J145510',
            targetSolarsystemClass: '4',
            signatures: makeSignatures(),
            mapSolarsystems: MAP_SOLARSYSTEMS,
            ...props,
        },
        global: {
            // The Disable button's tooltip needs a TooltipProvider; stub it away.
            stubs: {
                Tooltip: { template: '<div><slot /></div>' },
                TooltipTrigger: { template: '<div><slot /></div>' },
                TooltipContent: { template: '<div><slot /></div>' },
            },
        },
        attachTo: document.body,
    });

    // Open after mounting so the dialog's open watcher (reset + preselect) runs.
    await wrapper.setProps({ open: true });
    await wrapper.vm.$nextTick();

    return wrapper;
}

function submitForm(): void {
    document.body.querySelector('form')!.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }));
}

function lastSelection(wrapper: VueWrapper): Record<string, unknown> {
    const events = wrapper.emitted('selectSignature')!;
    return events[events.length - 1][0] as Record<string, unknown>;
}

function searchInput(): HTMLInputElement {
    return document.body.querySelector('#tracking-search') as HTMLInputElement;
}

afterEach(() => {
    vi.clearAllMocks();
    document.body.innerHTML = '';
});

describe('TrackingSignatureDialog', () => {
    it('renders wormhole and unscanned signatures but never site signatures', async () => {
        await mountOpenDialog();

        const text = document.body.textContent!;
        expect(text).toContain('AAA-111');
        expect(text).toContain('BBB-222');
        expect(text).toContain('CCC-333');
        expect(text).toContain('DDD-444');
        expect(text).not.toContain('EEE-555');
    });

    it('labels the connected and unlikely sections', async () => {
        await mountOpenDialog();

        const text = document.body.textContent!;
        expect(text).toContain('Already connected');
        expect(text).toContain('Unlikely · leads elsewhere');
    });

    it('shows where an already connected signature leads', async () => {
        await mountOpenDialog();

        expect(document.body.textContent).toContain('→ HOME-A (J145510)');
    });

    it('emits the unknown selection when confirmed without a choice', async () => {
        const wrapper = await mountOpenDialog();

        submitForm();

        expect(lastSelection(wrapper)).toMatchObject({ signatureId: null, lifetime: 'healthy', massStatus: 'fresh', shipSize: null });
    });

    it('preselects the first likely signature and adopts its values when enabled', async () => {
        const wrapper = await mountOpenDialog({ preselectFirstSignature: true });

        submitForm();

        expect(lastSelection(wrapper)).toMatchObject({ signatureId: 1, lifetime: 'eol', shipSize: 'frigate' });
    });

    it('does not preselect anything when the setting is off', async () => {
        const wrapper = await mountOpenDialog({ preselectFirstSignature: false });

        submitForm();

        expect(lastSelection(wrapper)).toMatchObject({ signatureId: null });
    });

    it('walks the options with arrow keys from the search field', async () => {
        const wrapper = await mountOpenDialog();

        searchInput().dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }));
        await wrapper.vm.$nextTick();
        submitForm();

        expect(lastSelection(wrapper)).toMatchObject({ signatureId: 1 });
    });

    it('wraps backwards from unknown to the last option', async () => {
        const wrapper = await mountOpenDialog();

        searchInput().dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true }));
        await wrapper.vm.$nextTick();
        submitForm();

        expect(lastSelection(wrapper)).toMatchObject({ signatureId: 4 });
    });

    it('filters the list through the search field and shows an empty state', async () => {
        const wrapper = await mountOpenDialog();

        const input = searchInput();
        input.value = 'bbb';
        input.dispatchEvent(new Event('input', { bubbles: true }));
        await wrapper.vm.$nextTick();

        expect(document.body.textContent).toContain('BBB-222');
        expect(document.body.textContent).not.toContain('AAA-111');

        input.value = 'zzz';
        input.dispatchEvent(new Event('input', { bubbles: true }));
        await wrapper.vm.$nextTick();

        expect(document.body.textContent).toContain('No signatures match');
    });

    it('locks the ship size to the identified wormhole type', async () => {
        const wrapper = await mountOpenDialog({
            signatures: [sig({ signatureId: 'AAA-111', category: 'wormhole', targetClass: '4', jumpMass: 2_000_000_000, shipSize: 'frigate' })],
            preselectFirstSignature: true,
        });

        submitForm();

        expect(lastSelection(wrapper)).toMatchObject({ shipSize: 'xlarge' });

        const triggers = [...document.body.querySelectorAll('button')];
        expect(triggers.some((button) => button.textContent?.includes('Extra Large') && button.hasAttribute('disabled'))).toBe(true);
    });

    it('keeps the ship size editable for signatures without an identified wormhole', async () => {
        await mountOpenDialog({
            signatures: [sig({ signatureId: 'AAA-111', category: 'wormhole', targetClass: '4' })],
            preselectFirstSignature: true,
        });

        const triggers = [...document.body.querySelectorAll('button')];
        expect(triggers.some((button) => button.textContent?.includes('Auto') && !button.hasAttribute('disabled'))).toBe(true);
    });
});

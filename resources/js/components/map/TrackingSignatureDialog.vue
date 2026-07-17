<script setup lang="ts">
import WormholeOption from '@/components/signatures/WormholeOption.vue';
import { Button } from '@/components/ui/button';
import { Dialog, DialogDescription, DialogFooter, DialogHeader, DialogScrollContent, DialogTitle } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { Select, SelectContent, SelectItem, SelectTrigger } from '@/components/ui/select';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useShowMap } from '@/composables/useShowMap';
import { Data } from '@/lib/data';
import { SHIP_SIZE_OPTIONS, shipSizeFromJumpMass } from '@/lib/shipSize';
import { groupSignatureOptions } from '@/lib/signatureCompatibility';
import { aliasedSolarsystemLabel } from '@/lib/solarsystem';
import { updateMapUserSettings } from '@/map/api';
import { TMapSolarsystem } from '@/pages/maps';
import { TLifetimeStatus, TMassStatus, TShipSize, TSignature, TStringedSolarsystemClass } from '@/types/models';
import { SearchIcon } from 'lucide-vue-next';
import { AcceptableValue } from 'reka-ui';
import { computed, nextTick, ref, useTemplateRef, watch } from 'vue';

const props = defineProps<{
    originMapSolarsystem: TMapSolarsystem | null;
    targetSolarsystemName: string | null;
    targetSolarsystemClass?: TStringedSolarsystemClass | null;
    signatures: TSignature[] | null | undefined;
    suggestedAlias?: string | null;
    /** The map's systems, used to name where already-connected signatures lead. */
    mapSolarsystems?: TMapSolarsystem[];
    /** Pre-select the first likely signature on open so Enter confirms it immediately. */
    preselectFirstSignature?: boolean;
}>();

const page = useShowMap();
const search = ref('');

const originSystemLabel = computed(() => {
    const origin = props.originMapSolarsystem;
    return origin ? aliasedSolarsystemLabel(origin.alias, origin.solarsystem.name) : '';
});

const filtered = computed(() => {
    if (!props.signatures) return [];
    if (!search.value) return props.signatures;

    return props.signatures.filter((s) => {
        const sig = (s.signature_id || '').toLocaleLowerCase();
        const type = (s.signature_type?.name || '').toLocaleLowerCase();
        const rawType = (s.raw_type_name || '').toLocaleLowerCase();
        return sig.includes(search.value) || type.includes(search.value) || rawType.includes(search.value);
    });
});

// Three sections: unmapped signatures that can be the jumped hole, ones already
// tied to a mapped connection, and ones typed with a destination class that
// cannot match. Site signatures are dropped entirely. Everything stays
// selectable in case a signature was mistyped or a connection is stale.
const groups = computed(() => groupSignatureOptions(filtered.value, props.targetSolarsystemClass));

const sections = computed(() => [
    { key: 'likely', label: null, options: groups.value.likely },
    { key: 'connected', label: 'Already connected', options: groups.value.connected },
    { key: 'unlikely', label: 'Unlikely · leads elsewhere', options: groups.value.unlikely },
]);

const listElement = useTemplateRef<InstanceType<typeof RadioGroup>>('signatureList');

/** All selectable option ids in visual order, starting with the Unknown row. */
const orderedIds = computed<(number | null)[]>(() => [null, ...sections.value.flatMap((section) => section.options.map((option) => option.id))]);

/**
 * Arrow keys in the search field walk the visible options without leaving the
 * field, wrapping at both ends. The checked row is kept scrolled into view.
 */
function moveSelection(delta: number): void {
    const ids = orderedIds.value;
    if (!ids.length) return;

    const index = ids.indexOf(selectedSignatureId.value);
    selectedSignatureId.value = ids[(index + delta + ids.length) % ids.length];

    void nextTick(() => {
        listElement.value?.$el?.querySelector('[data-state="checked"]')?.closest('label')?.scrollIntoView({ block: 'nearest' });
    });
}

/** Focus the search field when the dialog opens instead of the first input. */
function focusSearch(): void {
    document.getElementById('tracking-search')?.focus();
}

/** Where each already-connected signature's connection leads, keyed by signature id. */
const destinationLabels = computed<Map<number, string>>(() => {
    const labels = new Map<number, string>();
    const origin = props.originMapSolarsystem;
    if (!origin) return labels;

    for (const signature of props.signatures ?? []) {
        const connection = signature.map_connection;
        if (!connection) continue;

        const destinationId = connection.to_map_solarsystem_id === origin.id ? connection.from_map_solarsystem_id : connection.to_map_solarsystem_id;
        const destination = props.mapSolarsystems?.find((solarsystem) => solarsystem.id === destinationId);
        if (destination) {
            labels.set(signature.id, aliasedSolarsystemLabel(destination.alias, destination.solarsystem.name));
        }
    }

    return labels;
});

const open = defineModel<boolean>('open', { required: true });

const emit = defineEmits<{
    selectSignature: [
        selection: {
            signatureId: number | null;
            alias: string | null;
            lifetime: TLifetimeStatus;
            massStatus: TMassStatus;
            shipSize: TShipSize | null;
        },
    ];
}>();

const selectedSignatureId = ref<number | null>(null);
const alias = ref('');
const lifetime = ref<TLifetimeStatus>('healthy');
const massStatus = ref<TMassStatus>('fresh');
const shipSize = ref<TShipSize | 'auto'>('auto');

const selectedSignature = computed(() => props.signatures?.find((s) => s.id === selectedSignatureId.value) ?? null);

/**
 * An identified wormhole type dictates the hole's ship size, so the select is
 * locked to it while such a signature is chosen.
 */
const lockedShipSize = computed(() => shipSizeFromJumpMass(selectedSignature.value?.wormhole?.maximum_jump_mass));

const effectiveShipSize = computed<TShipSize | 'auto'>(() => lockedShipSize.value ?? shipSize.value);

// Reset inputs when the dialog opens. The alias is prefilled with the suggested
// chain alias (or the target's existing alias when it is already on the map),
// and with the preselect setting the first likely signature starts checked so
// jumping holes in alphabetical order only takes Enter.
watch(open, (isOpen) => {
    if (isOpen) {
        search.value = '';
        selectedSignatureId.value = props.preselectFirstSignature ? (groups.value.likely[0]?.id ?? null) : null;
        alias.value = props.suggestedAlias ?? '';
        lifetime.value = 'healthy';
        massStatus.value = 'fresh';
        shipSize.value = 'auto';
    }
});

// Adopt the signature's lifetime / mass / stored ship size when it carries a
// meaningful value, otherwise keep whatever the user manually selected. A
// wormhole type's size is not adopted here — it is derived as lockedShipSize.
watch(selectedSignature, (signature) => {
    if (!signature) return;
    if (signature.lifetime && signature.lifetime !== 'healthy') {
        lifetime.value = signature.lifetime;
    }
    if (signature.mass_status) {
        massStatus.value = signature.mass_status;
    }
    if (!lockedShipSize.value && signature.ship_size) {
        shipSize.value = signature.ship_size;
    }
});

function buildSelection(signatureId: number | null) {
    return {
        signatureId,
        alias: alias.value.trim() || null,
        lifetime: lifetime.value,
        massStatus: massStatus.value,
        shipSize: lockedShipSize.value ?? (shipSize.value === 'auto' ? null : shipSize.value),
    };
}

function handleConfirm() {
    emit('selectSignature', buildSelection(selectedSignatureId.value));
}

function handleOpenChange(isOpen: boolean) {
    if (!isOpen) {
        emit('selectSignature', buildSelection(null));
    }
}

function handleDontAskAgain() {
    updateMapUserSettings(page.props.map.slug, {
        prompt_for_signature_enabled: false,
    });
    emit('selectSignature', buildSelection(null));
}

function handleLifetimeChange(value: AcceptableValue) {
    if (value === 'healthy' || value === 'eol' || value === 'critical') {
        lifetime.value = value;
    }
}

function handleMassStatusChange(value: AcceptableValue) {
    if (value === 'fresh' || value === 'reduced' || value === 'critical') {
        massStatus.value = value;
    }
}

function handleShipSizeChange(value: AcceptableValue) {
    if (value === 'auto' || value === 'frigate' || value === 'medium' || value === 'large' || value === 'xlarge') {
        shipSize.value = value;
    }
}

/* One source for each select's options: dot / badge, label, and threshold hint. */
const lifetimeMeta: Record<TLifetimeStatus, { label: string; dot: string; hint?: string }> = {
    healthy: { label: 'Healthy', dot: 'bg-neutral-500' },
    eol: { label: 'End of Life', dot: 'bg-purple-500', hint: '< 4h' },
    critical: { label: 'Critical', dot: 'bg-red-500', hint: '< 1h' },
};

const massMeta: Record<TMassStatus, { label: string; dot: string; hint?: string }> = {
    fresh: { label: 'Fresh', dot: 'bg-neutral-500', hint: '≥ 50%' },
    reduced: { label: 'Reduced', dot: 'bg-amber-500', hint: '< 50%' },
    critical: { label: 'Critical', dot: 'bg-red-500', hint: '≤ 15%' },
};

const shipSizeOptions: { value: TShipSize | 'auto'; label: string; letter: string }[] = [
    { value: 'auto', label: 'Auto', letter: '·' },
    ...SHIP_SIZE_OPTIONS,
];

const selectedShipSizeOption = computed(() => shipSizeOptions.find((option) => option.value === effectiveShipSize.value) ?? shipSizeOptions[0]);
</script>

<template>
    <Dialog v-model:open="open" @update:open="handleOpenChange">
        <DialogScrollContent
            v-if="originMapSolarsystem && targetSolarsystemName"
            class="max-w-lg translate-y-0 gap-0 overflow-hidden p-0"
            @open-auto-focus.prevent="focusSearch"
        >
            <!-- Header -->
            <DialogHeader class="gap-1.5 border-b border-border/50 bg-muted/30 px-6 py-4 text-left">
                <DialogTitle>Which signature did you jump?</DialogTitle>
                <DialogDescription>
                    You jumped from <strong>{{ originSystemLabel }}</strong> to <strong>{{ targetSolarsystemName }}</strong
                    >. Select the wormhole connection you used.
                </DialogDescription>
            </DialogHeader>
            <form @submit.prevent="handleConfirm" class="contents">
                <!-- Connection details -->
                <div class="grid gap-3 px-6 py-5">
                    <div class="grid grid-cols-3 gap-3">
                        <div class="col-span-2 grid gap-1.5">
                            <Label for="tracking-alias" class="text-xs">Alias</Label>
                            <Input id="tracking-alias" v-model:model-value="alias" placeholder="Optional system alias" />
                        </div>
                        <div class="grid content-start gap-1.5">
                            <Label class="text-xs">Ship size</Label>
                            <Select :model-value="effectiveShipSize" :disabled="lockedShipSize !== null" @update:model-value="handleShipSizeChange">
                                <SelectTrigger class="w-full">
                                    <span class="flex items-center gap-2">
                                        <span class="inline-flex w-6 justify-center font-mono text-[10px] leading-4 text-muted-foreground">{{
                                            selectedShipSizeOption.letter
                                        }}</span>
                                        {{ selectedShipSizeOption.label }}
                                    </span>
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem v-for="option in shipSizeOptions" :key="option.value" :value="option.value">
                                        <span class="flex items-center gap-2">
                                            <span class="inline-flex w-6 justify-center font-mono text-[10px] leading-4 text-muted-foreground">{{
                                                option.letter
                                            }}</span>
                                            {{ option.label }}
                                        </span>
                                    </SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                    </div>
                    <div class="grid grid-cols-2 gap-3">
                        <div class="grid gap-1.5">
                            <Label class="text-xs">Lifetime</Label>
                            <Select :model-value="lifetime" @update:model-value="handleLifetimeChange">
                                <SelectTrigger class="w-full">
                                    <span class="flex items-center gap-2">
                                        <span class="inline-block size-2 rounded-full" :class="lifetimeMeta[lifetime].dot" />
                                        {{ lifetimeMeta[lifetime].label }}
                                    </span>
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem v-for="(meta, value) in lifetimeMeta" :key="value" :value="value">
                                        <span class="flex items-center gap-2">
                                            <span class="inline-block size-2 rounded-full" :class="meta.dot" />
                                            {{ meta.label }}
                                            <span v-if="meta.hint" class="text-muted-foreground">{{ meta.hint }}</span>
                                        </span>
                                    </SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                        <div class="grid gap-1.5">
                            <Label class="text-xs">Mass</Label>
                            <Select :model-value="massStatus" @update:model-value="handleMassStatusChange">
                                <SelectTrigger class="w-full">
                                    <span class="flex items-center gap-2">
                                        <span class="inline-block size-2 rounded-full" :class="massMeta[massStatus].dot" />
                                        {{ massMeta[massStatus].label }}
                                    </span>
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem v-for="(meta, value) in massMeta" :key="value" :value="value">
                                        <span class="flex items-center gap-2">
                                            <span class="inline-block size-2 rounded-full" :class="meta.dot" />
                                            {{ meta.label }}
                                            <span v-if="meta.hint" class="text-muted-foreground">{{ meta.hint }}</span>
                                        </span>
                                    </SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                    </div>
                </div>
                <!-- Signature list -->
                <div class="flex flex-col gap-3 px-6 pb-5">
                    <div class="relative">
                        <SearchIcon class="absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
                        <Input
                            id="tracking-search"
                            v-model:model-value="search"
                            placeholder="Search signatures"
                            class="pl-9"
                            @keydown.down.prevent="moveSelection(1)"
                            @keydown.up.prevent="moveSelection(-1)"
                        />
                    </div>
                    <RadioGroup
                        ref="signatureList"
                        class="grid h-64 grid-cols-[auto_auto_auto_1fr] content-start gap-0 gap-x-4 overflow-y-auto"
                        v-model:model-value="selectedSignatureId"
                        @keydown.enter="handleConfirm"
                    >
                        <label
                            class="col-span-4 grid grid-cols-subgrid items-center-safe rounded-sm p-2 text-left text-xs transition-colors hover:bg-muted/40 has-data-[state=checked]:bg-muted/60"
                        >
                            <RadioGroupItem :value="null" />
                            <div class="font-medium">Unknown</div>
                            <div class="text-muted-foreground">—</div>
                            <div />
                        </label>
                        <template v-for="section in sections" :key="section.key">
                            <div
                                v-if="section.label && section.options.length"
                                class="col-span-4 mt-2 border-t border-border/50 px-2 pt-2.5 pb-1 font-mono text-[10px] tracking-wider text-muted-foreground uppercase"
                            >
                                {{ section.label }}
                            </div>
                            <label
                                v-for="option in section.options"
                                :key="option.id"
                                class="col-span-4 grid grid-cols-subgrid items-center-safe rounded-sm p-2 text-left text-xs transition-colors hover:bg-muted/40 has-data-[state=checked]:bg-muted/60 has-data-[state=checked]:opacity-100 data-demoted:opacity-60"
                                :data-demoted="Data(section.key !== 'likely')"
                            >
                                <RadioGroupItem :value="option.id" />
                                <div class="font-medium">{{ option.signature_id }}</div>
                                <WormholeOption :wormhole="option.signature_type" v-if="option.signature_type" />
                                <div class="text-muted-foreground" v-else-if="option.raw_type_name">{{ option.raw_type_name }}</div>
                                <div class="text-muted-foreground" v-else>Unknown</div>
                                <div class="truncate text-right text-xs text-muted-foreground">
                                    <template v-if="destinationLabels.has(option.id)">→ {{ destinationLabels.get(option.id) }}</template>
                                </div>
                            </label>
                        </template>
                        <div v-if="search && !filtered.length" class="col-span-4 px-2 py-3 text-xs text-muted-foreground">
                            No signatures match "{{ search }}"
                        </div>
                    </RadioGroup>
                </div>
                <!-- Footer -->
                <DialogFooter class="border-t border-border/50 bg-muted/30 px-6 py-4 sm:justify-between">
                    <Tooltip>
                        <TooltipTrigger as-child>
                            <Button @click="handleDontAskAgain" variant="outline" role="button" type="button">Disable</Button>
                        </TooltipTrigger>
                        <TooltipContent>
                            <p class="text-xs">You can re-enable this in map preferences</p>
                        </TooltipContent>
                    </Tooltip>
                    <Button type="submit">Confirm</Button>
                </DialogFooter>
            </form>
        </DialogScrollContent>
    </Dialog>
</template>

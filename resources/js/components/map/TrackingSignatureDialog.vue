<script setup lang="ts">
import WormholeOption from '@/components/signatures/WormholeOption.vue';
import { Button } from '@/components/ui/button';
import { Dialog, DialogDescription, DialogFooter, DialogHeader, DialogScrollContent, DialogTitle } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useShowMap } from '@/composables/useShowMap';
import { Data } from '@/lib/data';
import { updateMapUserSettings } from '@/map/api';
import { TMapSolarsystem } from '@/pages/maps';
import { TLifetimeStatus, TMassStatus, TSignature } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import { formatDistanceToNowStrict } from 'date-fns';
import { AcceptableValue } from 'reka-ui';
import { computed, ref, watch } from 'vue';

const props = defineProps<{
    originMapSolarsystem: TMapSolarsystem | null;
    targetSolarsystemName: string | null;
    signatures: TSignature[] | null | undefined;
    suggestedAlias?: string | null;
}>();

const page = useShowMap();
const search = ref('');

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

const open = defineModel<boolean>('open', { required: true });

const emit = defineEmits<{
    selectSignature: [selection: { signatureId: number | null; alias: string | null; lifetime: TLifetimeStatus; massStatus: TMassStatus }];
}>();

const selectedSignatureId = ref<number | null>(null);
const alias = ref('');
const lifetime = ref<TLifetimeStatus>('healthy');
const massStatus = ref<TMassStatus>('fresh');

// Reset inputs when the dialog opens. The alias is prefilled with the suggested
// chain alias (or the target's existing alias when it is already on the map).
watch(open, (isOpen) => {
    if (isOpen) {
        selectedSignatureId.value = null;
        alias.value = props.suggestedAlias ?? '';
        lifetime.value = 'healthy';
        massStatus.value = 'fresh';
    }
});

// Adopt the signature's lifetime / mass when it carries a meaningful value,
// otherwise keep whatever the user manually selected.
watch(selectedSignatureId, (id) => {
    const signature = props.signatures?.find((s) => s.id === id);
    if (!signature) return;
    if (signature.lifetime && signature.lifetime !== 'healthy') {
        lifetime.value = signature.lifetime;
    }
    if (signature.mass_status) {
        massStatus.value = signature.mass_status;
    }
});

function buildSelection(signatureId: number | null) {
    return {
        signatureId,
        alias: alias.value.trim() || null,
        lifetime: lifetime.value,
        massStatus: massStatus.value,
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

function formatDate(date: string) {
    return formatDistanceToNowStrict(new UTCDate(date)) + ' ago';
}
</script>

<template>
    <Dialog v-model:open="open" @update:open="handleOpenChange">
        <DialogScrollContent v-if="originMapSolarsystem && targetSolarsystemName" class="max-w-md translate-y-0">
            <DialogHeader>
                <DialogTitle>Which signature did you jump?</DialogTitle>
                <DialogDescription>
                    You jumped from <strong>{{ originMapSolarsystem.solarsystem.name }}</strong> to <strong>{{ targetSolarsystemName }}</strong
                    >. Please select which wormhole connection you used.
                </DialogDescription>
            </DialogHeader>
            <form @submit.prevent="handleConfirm" class="contents">
                <Input v-model:model-value="search" placeholder="Search" />
                <RadioGroup
                    class="grid max-h-full grid-cols-[auto_auto_auto_1fr] gap-0 gap-x-4 divide-y overflow-y-auto rounded-lg border"
                    v-model:model-value="selectedSignatureId"
                    @keydown.enter="handleConfirm"
                    autofocus
                >
                    <label class="col-span-4 grid grid-cols-subgrid items-center-safe p-1.5 text-left text-xs">
                        <RadioGroupItem :value="null" />
                        <div class="font-medium">Unknown</div>
                        <div class="text-muted-foreground">—</div>
                        <div class="text-right text-xs text-muted-foreground">—</div>
                    </label>
                    <label
                        v-for="option in filtered"
                        :key="option.id"
                        class="col-span-4 grid grid-cols-subgrid items-center-safe p-1.5 text-left text-xs data-connected:opacity-50"
                        :data-connected="Data(Boolean(option.map_connection_id))"
                    >
                        <RadioGroupItem :value="option.id" />
                        <div class="font-medium">{{ option.signature_id }}</div>
                        <WormholeOption :wormhole="option.signature_type" v-if="option.signature_type" />
                        <div class="text-muted-foreground" v-else-if="option.raw_type_name">{{ option.raw_type_name }}</div>
                        <div class="text-muted-foreground" v-else-if="option.map_connection_id">Already connected</div>
                        <div class="text-muted-foreground" v-else>Unknown</div>
                        <div class="text-right text-xs text-muted-foreground">
                            {{ option.created_at ? formatDate(option.created_at) : 'Unknown' }}
                        </div>
                    </label>
                </RadioGroup>
                <div class="grid gap-3">
                    <div class="grid gap-1.5">
                        <Label for="tracking-alias" class="text-xs">Alias</Label>
                        <Input id="tracking-alias" v-model:model-value="alias" placeholder="Optional system alias" />
                    </div>
                    <div class="grid grid-cols-2 gap-3">
                        <div class="grid gap-1.5">
                            <Label class="text-xs">Lifetime</Label>
                            <Select :model-value="lifetime" @update:model-value="handleLifetimeChange">
                                <SelectTrigger class="w-full">
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="healthy">Healthy</SelectItem>
                                    <SelectItem value="eol">End of Life</SelectItem>
                                    <SelectItem value="critical">Critical</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                        <div class="grid gap-1.5">
                            <Label class="text-xs">Mass</Label>
                            <Select :model-value="massStatus" @update:model-value="handleMassStatusChange">
                                <SelectTrigger class="w-full">
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="fresh">Fresh</SelectItem>
                                    <SelectItem value="reduced">Reduced</SelectItem>
                                    <SelectItem value="critical">Critical</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                    </div>
                </div>
                <DialogFooter class="sm:justify-between">
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

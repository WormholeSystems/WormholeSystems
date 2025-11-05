<script setup lang="ts">
import WormholeOption from '@/components/signatures/WormholeOption.vue';
import { Button } from '@/components/ui/button';
import { Dialog, DialogDescription, DialogFooter, DialogHeader, DialogScrollContent, DialogTitle } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { updateMapUserSettings } from '@/composables/map';
import { Data } from '@/composables/map/utils/data';
import { useMapUserSettings } from '@/composables/useMapUserSettings';
import { TMapSolarsystem } from '@/pages/maps';
import { TSignature } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import { formatDistanceToNowStrict } from 'date-fns';
import { computed, ref, watch } from 'vue';

const props = defineProps<{
    originMapSolarsystem: TMapSolarsystem | null;
    targetSolarsystemName: string | null;
    signatures: TSignature[] | null | undefined;
}>();

const map_user_settings = useMapUserSettings();
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
    selectSignature: [signatureId: number | null];
}>();

const selectedSignatureId = ref<number | null>(null);

// Reset selection when dialog opens
watch(open, (isOpen) => {
    if (isOpen) {
        selectedSignatureId.value = null;
    }
});

function handleConfirm() {
    emit('selectSignature', selectedSignatureId.value);
}

function handleOpenChange(isOpen: boolean) {
    if (!isOpen) {
        emit('selectSignature', null);
    }
}

function handleDontAskAgain() {
    updateMapUserSettings(map_user_settings.value, {
        prompt_for_signature_enabled: false,
    });
    emit('selectSignature', null);
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

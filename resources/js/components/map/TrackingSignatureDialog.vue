<script setup lang="ts">
import WormholeOption from '@/components/signatures/WormholeOption.vue';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { Data } from '@/composables/map/utils/data';
import { TMapSolarSystem, TSignature } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import { formatDistanceToNowStrict } from 'date-fns';
import { computed, ref, watch } from 'vue';

const props = defineProps<{
    originMapSolarsystem: TMapSolarSystem | null;
    targetSolarsystemName: string | null;
    signatures: TSignature[] | null | undefined;
}>();

const search = ref('');

const filtered = computed(() => {
    if (!props.signatures) return [];
    if (!search.value) return props.signatures;

    return props.signatures.filter((s) => {
        const sig = (s.signature_id || '').toLocaleLowerCase();
        const type = (s.signature_type?.name || '').toLocaleLowerCase();
        return sig.includes(search.value) || type.includes(search.value);
    });
});

const open = defineModel<boolean>('open', { required: true });

const emit = defineEmits<{
    selectSignature: [signatureId: number | null];
    skip: [];
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
    open.value = false;
}

function handleCancel() {
    emit('skip');
    open.value = false;
}

function formatDate(date: string) {
    return formatDistanceToNowStrict(new UTCDate(date)) + ' ago';
}
</script>

<template>
    <Dialog v-model:open="open">
        <DialogContent v-if="originMapSolarsystem && targetSolarsystemName" class="max-w-md">
            <DialogHeader>
                <DialogTitle>Which signature did you jump?</DialogTitle>
                <DialogDescription>
                    You jumped from <strong>{{ originMapSolarsystem.name }}</strong> to <strong>{{ targetSolarsystemName }}</strong
                    >. Please select which wormhole connection you used.
                </DialogDescription>
            </DialogHeader>
            <Input v-model:model-value="search" type="search" placeholder="Search" />
            <div class="h-60">
                <RadioGroup
                    class="grid max-h-full grid-cols-[auto_auto_auto_1fr] gap-0 gap-x-4 divide-y overflow-y-auto rounded-lg border"
                    v-model:model-value="selectedSignatureId"
                >
                    <label
                        v-for="option in filtered"
                        :key="option.id"
                        class="col-span-4 grid grid-cols-subgrid items-center-safe p-2 text-left text-sm data-connected:opacity-50"
                        :data-connected="Data(Boolean(option.map_connection_id))"
                    >
                        <RadioGroupItem :value="option.id" />
                        <div class="font-medium">{{ option.signature_id }}</div>
                        <WormholeOption :wormhole="option.signature_type" v-if="option.signature_type" />
                        <div class="text-muted-foreground" v-else>Unknown</div>
                        <div class="text-right text-xs text-muted-foreground">
                            {{ option.created_at ? formatDate(option.created_at) : 'Unknown' }}
                        </div>
                    </label>
                </RadioGroup>
            </div>
            <DialogFooter>
                <Button @click="handleCancel" variant="outline">Skip</Button>
                <Button @click="handleConfirm">Confirm</Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>
</template>

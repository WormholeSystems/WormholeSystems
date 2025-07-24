<script setup lang="ts">
import PasteIcon from '@/components/icons/PasteIcon.vue';
import PlusIcon from '@/components/icons/PlusIcon.vue';
import Signature from '@/components/signatures/Signature.vue';
import { Button } from '@/components/ui/button';
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useHasWritePermission } from '@/composables/useHasPermission';
import { signatureParser } from '@/lib/SignatureParser';
import { TMapSolarSystem, TSignature } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { useActiveElement, useMagicKeys, whenever } from '@vueuse/core';
import { logicAnd, logicOr } from '@vueuse/math';
import { computed, ref, watch } from 'vue';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TMapSolarSystem;
}>();

const can_write = useHasWritePermission();

const pasted_signatures = ref<Partial<TSignature>[] | null>();
const new_signatures = ref<Partial<TSignature>[]>([]);
const updated_signatures = ref<Partial<TSignature>[]>([]);
const deleted_signatures = ref<Partial<TSignature>[]>([]);

const signatures = computed(() => {
    return map_solarsystem.signatures
        ?.map((sig) => ({
            ...sig,
            deleted: isSignatureDeleted(sig),
            new: isSignatureNew(sig),
            updated: isSignatureUpdated(sig),
        }))
        .sort((a, b) => {
            if (a.signature_id && b.signature_id) {
                return a.signature_id.localeCompare(b.signature_id);
            }
            if (a.signature_id) {
                return -1;
            }

            if (b.signature_id) {
                return 1;
            }

            return 0;
        });
});

const activeElement = useActiveElement();
const { ctrl_v, cmd_v } = useMagicKeys();
const not_using_input = computed(() => {
    return !activeElement.value || !['INPUT', 'TEXTAREA'].includes(activeElement.value.tagName);
});

whenever(logicAnd(logicOr(ctrl_v, cmd_v), not_using_input), () => {
    handlePaste();
});

watch(
    () => map_solarsystem.solarsystem_id,
    () => {
        pasted_signatures.value = null;
        new_signatures.value = [];
        updated_signatures.value = [];
        deleted_signatures.value = [];
    },
);

async function handlePaste() {
    const signatures = await getSignaturesFromClipboard();
    new_signatures.value = getNewSignatures(signatures);
    updated_signatures.value = getUpdatedSignatures(signatures);
    deleted_signatures.value = getDeletedSignatures(signatures);
    pasted_signatures.value = signatures;
    router.post(
        route('paste-signatures.store'),
        {
            map_solarsystem_id: map_solarsystem.id,
            signatures: signatures.map((signature) => ({
                signature_id: signature.signature_id,
                type: signature.type,
                category: signature.category,
            })),
        },
        {
            preserveScroll: true,
            preserveState: true,
            only: ['selected_map_solarsystem'],
        },
    );
}

async function getSignaturesFromClipboard() {
    const clipboardText = await navigator.clipboard.readText();
    return signatureParser.parseSignatures(clipboardText);
}

function getNewSignatures(parsed_signatures: Partial<TSignature>[]) {
    return parsed_signatures.filter((signature) => {
        return !map_solarsystem.signatures!.some((existing_signature) => {
            return existing_signature.signature_id === signature.signature_id;
        });
    });
}

function isSignatureDeleted(signature: Partial<TSignature>) {
    return Boolean(
        pasted_signatures.value &&
            !pasted_signatures.value.some((pasted_signature) => pasted_signature.signature_id === signature.signature_id && !pasted_signature.id),
    );
}

function isSignatureNew(signature: Partial<TSignature>) {
    return Boolean(new_signatures.value.some((new_signature) => new_signature.signature_id === signature.signature_id));
}

function isSignatureUpdated(signature: Partial<TSignature>) {
    return Boolean(updated_signatures.value.some((updated_signature) => updated_signature.signature_id === signature.signature_id));
}

function getUpdatedSignatures(parsed_signatures: Partial<TSignature>[]) {
    return parsed_signatures.filter((signature) => {
        return map_solarsystem.signatures!.some((existing_signature) => {
            return existing_signature.signature_id === signature.signature_id;
        });
    });
}

function getDeletedSignatures(parsed_signatures: Partial<TSignature>[]) {
    return map_solarsystem.signatures!.filter((signature) => {
        return !parsed_signatures.some((parsed_signature) => parsed_signature.signature_id === signature.signature_id);
    });
}

function createNewSignature() {
    router.post(
        route('signatures.store'),
        {
            map_solarsystem_id: map_solarsystem.id,
            signature_id: '',
            type: null,
            category: null,
        } satisfies Partial<TSignature>,
        {
            preserveScroll: true,
            preserveState: true,
            only: ['selected_map_solarsystem'],
        },
    );
}
</script>

<template>
    <Card class="overflow-hidden pb-0">
        <CardHeader>
            <CardTitle> Signatures</CardTitle>
            <CardDescription> All the signatures in this solarsystem. You can paste, copy and clear signatures here. </CardDescription>

            <CardAction class="flex gap-2" v-if="can_write">
                <Tooltip>
                    <TooltipTrigger as-child>
                        <Button @click="handlePaste" variant="outline" size="icon">
                            <PasteIcon />
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent> Paste signatures from clipboard</TooltipContent>
                </Tooltip>
                <Tooltip>
                    <TooltipTrigger as-child>
                        <Button @click="createNewSignature" variant="outline" size="icon">
                            <PlusIcon />
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent> Create new signature</TooltipContent>
                </Tooltip>
            </CardAction>
        </CardHeader>
        <CardContent class="px-1 pb-1">
            <div class="grid grid-cols-[auto_auto_1fr_auto_auto_auto] gap-x-2 divide-y">
                <Signature
                    v-for="signature in signatures"
                    :signature="signature"
                    :key="signature.id"
                    :deleted="signature.deleted"
                    :new="signature.new"
                    :updated="signature.updated"
                />
            </div>
        </CardContent>
    </Card>
</template>

<style scoped>
tr:has([data-status='new']) {
    background-color: var(--color-red-100);
}
</style>

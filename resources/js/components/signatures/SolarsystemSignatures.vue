<script setup lang="ts">
import DataTable from '@/components/DataTable.vue';
import CopyIcon from '@/components/icons/CopyIcon.vue';
import PasteIcon from '@/components/icons/PasteIcon.vue';
import TrashIcon from '@/components/icons/TrashIcon.vue';
import columns, { TRawSignature } from '@/components/signatures/column';
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { getMapChannelName } from '@/const/channels';
import { SignaturesUpdatedEvent } from '@/const/events';
import { TMapSolarSystem } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { useEchoPublic } from '@laravel/echo-vue';
import { useMagicKeys, whenever } from '@vueuse/core';
import { computed, ref, useTemplateRef, watch, watchEffect } from 'vue';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TMapSolarSystem;
}>();

const pasted_signatures = ref<TRawSignature[]>([]);

const signature_difference = computed<TRawSignature[]>(showDifference);

const { Ctrl_v, Cmd_V } = useMagicKeys();

const confirm_button = useTemplateRef('confirm_button');

whenever(Ctrl_v, handleClipboardPaste);
whenever(Cmd_V, handleClipboardPaste);

watch(
    () => map_solarsystem.id,
    () => {
        pasted_signatures.value = [];
    },
);

watchEffect(() => {
    if (confirm_button.value) confirm_button.value.focus();
});

async function handleClipboardPaste() {
    // check permission to access clipboard
    const clipboardData = navigator.clipboard?.readText();

    if (!clipboardData) {
        console.warn('Clipboard access is not supported or permission denied.');
        return;
    }
    const text = (await clipboardData).trim();

    pasted_signatures.value = parseSignatures(text);
}

function handleClipbordCopy() {
    if (!map_solarsystem.signatures?.length) return;
    const signatures = map_solarsystem.signatures.map((sig) => [sig.signature_id, sig.type, sig.category || '', sig.name || '']);

    const text = signatures.map((parts) => parts.join('\t')).join('\n');

    navigator.clipboard.writeText(text);
}

function clearSignatures() {
    if (!confirm('Are you sure you want to clear all signatures?')) {
        return;
    }

    updateSignatures([]);
}

function parseSignatures(text: string): TRawSignature[] {
    if (!text) {
        return [] satisfies TRawSignature[];
    }

    return text
        .split('\n')
        .map((sig) => sig.split('\t'))
        .map((parts) => ({
            signature_id: parts[0],
            type: parts[1],
            category: parts[2] || null,
            name: parts[3] || null,
            status: null,
        }));
}

function updateSignatures(signatures: TRawSignature[]) {
    router.post(
        route('map-solarsystems.signatures.store', map_solarsystem.id),
        {
            signatures,
        },
        {
            preserveScroll: true,
            preserveState: true,
            only: ['map', 'selected_map_solarsystem'],
            onSuccess: () => {
                pasted_signatures.value = [];
            },
        },
    );
}

function showDifference() {
    if (!pasted_signatures.value.length)
        return map_solarsystem
            .signatures!.map(
                (sig): TRawSignature => ({
                    signature_id: sig.signature_id,
                    type: sig.type,
                    category: sig.category,
                    name: sig.name,
                    status: null,
                }),
            )
            .sort(sortSignatures);
    const signatures = new Set(map_solarsystem.signatures!.map((sig) => sig.signature_id));
    pasted_signatures.value.forEach((sig) => {
        signatures.add(sig.signature_id);
    });

    return Array.from(signatures)
        .map((signature_id): TRawSignature => {
            const existingSignature = map_solarsystem.signatures!.find((sig) => sig.signature_id === signature_id);
            const pastedSignature = pasted_signatures.value.find((sig) => sig.signature_id === signature_id);

            const is_missing = existingSignature && !pastedSignature;
            const is_new = pastedSignature && !existingSignature;
            const is_modified = pastedSignature && existingSignature;

            return {
                signature_id,
                type: pastedSignature?.type || existingSignature?.type || '',
                category: pastedSignature?.category || existingSignature?.category || null,
                name: pastedSignature?.name || existingSignature?.name || null,
                status: is_missing ? 'missing' : is_new ? 'new' : is_modified ? 'modified' : null, // null means no change
            };
        })
        .sort(sortSignatures);
}

function sortSignatures(a: TRawSignature, b: TRawSignature) {
    if (a.status === 'new' && b.status !== 'new') return -1;
    if (b.status === 'new' && a.status !== 'new') return 1;
    if (a.status === 'missing' && b.status !== 'missing') return -1;
    if (b.status === 'missing' && a.status !== 'missing') return 1;
    return a.signature_id.localeCompare(b.signature_id);
}

useEchoPublic(getMapChannelName(map_solarsystem.map_id), SignaturesUpdatedEvent, () => {
    router.reload({
        only: ['selected_map_solarsystem', 'map'],
    });
});
</script>

<template>
    <div class="mb-4 flex justify-between gap-2">
        <h3 class="mr-auto">Signatures</h3>
        <div v-if="!pasted_signatures.length" class="flex gap-2">
            <Tooltip>
                <TooltipTrigger>
                    <Button @click="handleClipboardPaste" variant="outline" size="icon" title="Paste signatures from clipboard">
                        <PasteIcon />
                    </Button>
                </TooltipTrigger>
                <TooltipContent> Paste signatures from clipboard (Ctrl + V)</TooltipContent>
            </Tooltip>
            <Tooltip v-if="map_solarsystem.signatures?.length">
                <TooltipTrigger>
                    <Button @click="handleClipbordCopy" variant="outline" size="icon" title="Copy signatures to clipboard">
                        <CopyIcon />
                    </Button>
                </TooltipTrigger>
                <TooltipContent> Copy signatures to clipboard</TooltipContent>
            </Tooltip>
            <Tooltip>
                <TooltipTrigger>
                    <Button @click="clearSignatures" variant="destructive" size="icon" title="Clear all signatures">
                        <TrashIcon />
                    </Button>
                </TooltipTrigger>
                <TooltipContent> Clear all signatures</TooltipContent>
            </Tooltip>
        </div>
        <div v-else class="flex gap-2">
            <Button @click="pasted_signatures = []" variant="outline" size="icon" title="Clear pasted signatures">
                <TrashIcon />
            </Button>
            <Button as-child>
                <button @click="updateSignatures(pasted_signatures)" ref="confirm_button">Save changes</button>
            </Button>
        </div>
    </div>
    <DataTable :columns="columns" :data="signature_difference" class="text-sm" />
</template>

<style scoped>
tr:has([data-status='new']) {
    background-color: var(--color-red-100);
}
</style>

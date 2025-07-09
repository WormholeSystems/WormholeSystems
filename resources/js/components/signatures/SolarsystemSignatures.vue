<script setup lang="ts">
import CopyIcon from '@/components/icons/CopyIcon.vue';
import PasteIcon from '@/components/icons/PasteIcon.vue';
import TrashIcon from '@/components/icons/TrashIcon.vue';
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { TMapSolarSystem } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { useMagicKeys, whenever } from '@vueuse/core';
import { computed, nextTick, ref, useTemplateRef } from 'vue';

type TRawSignature = {
    signature_id: string;
    type: string;
    category: string | null;
    name: string | null;
};

const { map_solarsystem } = defineProps<{
    map_solarsystem: TMapSolarSystem;
}>();

const pasted_signatures = ref<TRawSignature[]>([]);

const signature_difference = computed(showDifference);

const { Ctrl_v, Cmd_V } = useMagicKeys();

const confirmButton = useTemplateRef('confirm_button');

whenever(Ctrl_v, handleClipboardPaste);
whenever(Cmd_V, handleClipboardPaste);

async function handleClipboardPaste() {
    // check permission to access clipboard
    const clipboardData = navigator.clipboard?.readText();

    if (!clipboardData) {
        console.warn('Clipboard access is not supported or permission denied.');
        return;
    }
    const text = (await clipboardData).trim();

    pasted_signatures.value = parseSignatures(text);

    await nextTick();

    if (pasted_signatures.value.length) {
        confirmButton.value?.focus();
    }
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
        return map_solarsystem.signatures!.map((sig) => ({
            signature_id: sig.signature_id,
            type: sig.type,
            category: sig.category,
            name: sig.name,
            status: 'unchanged',
        }));
    const signatures = new Set(map_solarsystem.signatures!.map((sig) => sig.signature_id));
    pasted_signatures.value.forEach((sig) => {
        signatures.add(sig.signature_id);
    });

    return Array.from(signatures)
        .map((signature_id) => {
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
                status: is_missing ? 'missing' : is_new ? 'new' : is_modified ? 'modified' : 'unchanged',
            };
        })
        .sort((a, b) => {
            if (a.status === 'new' && b.status !== 'new') return -1;
            if (b.status === 'new' && a.status !== 'new') return 1;
            if (a.status === 'missing' && b.status !== 'missing') return -1;
            if (b.status === 'missing' && a.status !== 'missing') return 1;
            return a.signature_id.localeCompare(b.signature_id);
        });
}
</script>

<template>
    <div class="mb-4 flex justify-between gap-2">
        <h3 class="mr-auto">Signatures</h3>
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
    <div class="overflow-x-auto">
        <table class="w-full text-xs">
            <thead>
                <tr class="border-b text-left">
                    <th class="p-1">ID</th>
                    <th class="p-1">Type</th>
                    <th class="p-1">Category</th>
                    <th class="p-1">Name</th>
                    <th v-if="pasted_signatures.length" class="p-1">Status</th>
                </tr>
            </thead>
            <tbody class="text-muted-foreground">
                <tr
                    v-for="signature in signature_difference"
                    :key="signature.signature_id"
                    :data-status="signature.status"
                    class="border-b last:border-b-0 data-[status=missing]:bg-red-900 data-[status=modified]:bg-yellow-900 data-[status=new]:bg-green-900"
                >
                    <td class="p-1">{{ signature.signature_id }}</td>
                    <td class="p-1">{{ signature.type }}</td>
                    <td class="p-1">{{ signature.category }}</td>
                    <td class="p-1">{{ signature.name }}</td>
                    <td class="p-1" v-if="pasted_signatures.length">
                        <span v-if="signature.status === 'missing'" class="text-red-500">Missing</span>
                        <span v-if="signature.status === 'modified'" class="text-yellow-500">Modified</span>
                        <span v-if="signature.status === 'new'" class="text-green-500">New</span>
                        <span v-if="signature.status === 'unchanged'" class="text-muted-foreground">Unchanged</span>
                    </td>
                </tr>
                <tr v-if="map_solarsystem.signatures?.length === 0" class="border-b last:border-b-0">
                    <td colspan="4" class="py-4 text-left text-muted-foreground">No signatures found</td>
                </tr>
            </tbody>
        </table>
    </div>
    <div v-if="pasted_signatures.length" class="mt-4 flex justify-between">
        <Button @click="pasted_signatures = []" class="" variant="secondary"> Cancel</Button>
        <Button @click="updateSignatures(pasted_signatures)" :disabled="!pasted_signatures.length" as-child>
            <button ref="confirm_button">Update Signatures</button>
        </Button>
    </div>
</template>

<style scoped></style>

<script setup lang="ts">
import CopyIcon from '@/components/icons/CopyIcon.vue';
import PasteIcon from '@/components/icons/PasteIcon.vue';
import TrashIcon from '@/components/icons/TrashIcon.vue';
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { TMapSolarSystem } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { useMagicKeys, whenever } from '@vueuse/core';

type TRawSignature = {
    signature_id: string;
    type: string;
    category: string | null;
    name: string | null;
};

const { map_solarsystem } = defineProps<{
    map_solarsystem: TMapSolarSystem;
}>();

const { Ctrl_v } = useMagicKeys();

whenever(Ctrl_v, handleClipboardPaste);

async function handleClipboardPaste() {
    // check permission to access clipboard
    const clipboardData = navigator.clipboard?.readText();

    if (!clipboardData) {
        console.warn('Clipboard access is not supported or permission denied.');
        return;
    }
    const text = (await clipboardData).trim();

    let signatures = parseSignatures(text);

    updateSignatures(signatures);
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
        },
    );
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
                </tr>
            </thead>
            <tbody class="text-muted-foreground">
                <tr v-for="signature in map_solarsystem.signatures" :key="signature.id" class="border-b last:border-b-0">
                    <td class="p-1">{{ signature.signature_id }}</td>
                    <td class="p-1">{{ signature.type }}</td>
                    <td class="p-1">{{ signature.category }}</td>
                    <td class="p-1">{{ signature.name }}</td>
                </tr>
                <tr v-if="map_solarsystem.signatures?.length === 0" class="border-b last:border-b-0">
                    <td colspan="4" class="py-4 text-left text-muted-foreground">No signatures found</td>
                </tr>
            </tbody>
        </table>
    </div>
</template>

<style scoped></style>

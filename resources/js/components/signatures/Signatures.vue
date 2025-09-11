<script setup lang="ts">
import PasteIcon from '@/components/icons/PasteIcon.vue';
import PlusIcon from '@/components/icons/PlusIcon.vue';
import TrashIcon from '@/components/icons/TrashIcon.vue';
import Signature from '@/components/signatures/Signature.vue';
import { Button } from '@/components/ui/button';
import { CardAction, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
import SortHeader from '@/components/ui/SortHeader.vue';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { createSignature, deleteSignatures, pasteSignatures, useSignatures } from '@/composables/map';
import useHasWritePermission from '@/composables/useHasWritePermission';
import { useSortPreference } from '@/composables/useSortPreference';
import { signatureParser, TRawSignature } from '@/lib/SignatureParser';
import { TMapSolarSystem, TSignature } from '@/types/models';
import { useActiveElement, useEventListener } from '@vueuse/core';
import { computed, ref, watch } from 'vue';
import { toast } from 'vue-sonner';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TMapSolarSystem;
}>();

const { relevant_signatures, connections } = useSignatures();

const can_write = useHasWritePermission();

const { sortPreferences, updateSortPreferences } = useSortPreference('signatures');

const pasted_signatures = ref<Partial<TSignature>[] | null>();
const new_signatures = computed(() => {
    if (!pasted_signatures.value) {
        return [];
    }
    return getNewSignatures(pasted_signatures.value);
});
const updated_signatures = computed(() => {
    if (!pasted_signatures.value) {
        return [];
    }
    return getUpdatedSignatures(pasted_signatures.value);
});

const deleted_signatures = computed(() => {
    if (!pasted_signatures.value) {
        return [];
    }
    return getDeletedSignatures(pasted_signatures.value);
});

function compareNullableStrings(a: string | null, b: string | null): number {
    if (!a && !b) return 0;

    if (!a) return 1;

    if (!b) return -1;

    return a.localeCompare(b);
}

function getSortComparison(
    a: TSignature & { deleted: boolean; new: boolean; updated: boolean },
    b: TSignature & { deleted: boolean; new: boolean; updated: boolean },
): number {
    let primaryComparison = 0;

    switch (sortPreferences.value.column) {
        case 'id':
            primaryComparison = compareNullableStrings(a.signature_id, b.signature_id);
            break;
        case 'category':
            primaryComparison = compareNullableStrings(a.category, b.category);
            break;
        case 'type':
            primaryComparison = compareNullableStrings(a.type, b.type);
            break;
        default:
            primaryComparison = 0;
    }

    const directedPrimaryComparison = sortPreferences.value.direction === 'desc' ? -primaryComparison : primaryComparison;

    if (primaryComparison === 0) {
        return compareNullableStrings(a.signature_id, b.signature_id);
    }

    return directedPrimaryComparison;
}

const signatures = computed(() => {
    const enriched_signatures = map_solarsystem.signatures!.map((sig) => ({
        ...sig,
        deleted: isSignatureDeleted(sig),
        new: isSignatureNew(sig),
        updated: isSignatureUpdated(sig),
    }));

    return enriched_signatures.sort((a, b) => {
        return getSortComparison(a, b);
    });
});

const connected_connections = computed(() => {
    return connections.value.filter((connection) => {
        return signatures.value.some((signature) => {
            return signature.map_connection_id === connection.id;
        });
    });
});

const unconnected_connections = computed(() => {
    return connections.value.filter((connection) => {
        return !signatures.value.some((signature) => {
            return signature.map_connection_id === connection.id;
        });
    });
});

const activeElement = useActiveElement();
const using_input = computed(() => {
    return activeElement.value && ['INPUT', 'TEXTAREA'].includes(activeElement.value.tagName);
});

watch(
    () => map_solarsystem.solarsystem_id,
    () => {
        pasted_signatures.value = null;
    },
);

function handleSort(column: string) {
    const sortColumn = column as 'id' | 'category' | 'type';
    let newDirection: 'asc' | 'desc';

    if (sortPreferences.value.column === sortColumn) {
        newDirection = sortPreferences.value.direction === 'asc' ? 'desc' : 'asc';
    } else {
        newDirection = 'asc';
    }

    updateSortPreferences(sortColumn, newDirection);
}

async function handlePaste() {
    const signatures = await getSignaturesFromClipboard();
    if (!signatures) {
        return;
    }

    processSignatures(signatures);
}

function processSignatures(signatures: TRawSignature[]) {
    pasted_signatures.value = signatures;
    pasteSignatures(map_solarsystem.id, signatures);
}

async function getSignaturesFromClipboard() {
    // ask for permission to read clipboard
    if (!navigator.clipboard || !navigator.clipboard.readText) {
        toast.error('Clipboard access is not supported in this browser or permission is denied.');
        return false;
    }
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
    createSignature(map_solarsystem.id);
}

function deleteMissingSignatures(with_solarsystems = false) {
    deleteSignatures(
        map_solarsystem.id,
        deleted_signatures.value.map((signature) => signature.id),
        with_solarsystems,
    );
}

useEventListener('paste', (event) => {
    if (using_input.value) {
        return;
    }
    event.preventDefault();
    const clipboardData = event.clipboardData?.getData('text/plain');

    if (!clipboardData) {
        toast.error('No text found in clipboard.');
        return;
    }
    const signatures = signatureParser.parseSignatures(clipboardData);
    if (!signatures) {
        toast.error('Failed to parse signatures from clipboard.');
        return;
    }
    processSignatures(signatures);
});
</script>

<template>
    <MapPanel class="overflow-hidden">
        <CardHeader>
            <CardTitle
                >Signatures
                <span class="text-muted-foreground" v-if="signatures.length">({{ signatures.length }})</span>
            </CardTitle>
            <CardDescription> All the signatures in this solarsystem. You can paste, copy and clear signatures here. </CardDescription>

            <CardAction class="flex gap-2" v-if="can_write">
                <Tooltip v-if="pasted_signatures">
                    <TooltipTrigger as-child>
                        <Button v-if="pasted_signatures" @click="pasted_signatures = null" variant="secondary"> Unselect </Button>
                    </TooltipTrigger>
                    <TooltipContent> Unselect signatures</TooltipContent>
                </Tooltip>
                <Tooltip v-if="deleted_signatures.length > 0">
                    <TooltipTrigger as-child>
                        <Button @click="deleteMissingSignatures(true)" variant="destructive" size="icon"> <TrashIcon /> </Button>
                    </TooltipTrigger>
                    <TooltipContent> Delete missing signatures and their connections </TooltipContent>
                </Tooltip>
                <Tooltip>
                    <TooltipTrigger as-child>
                        <Button @click="handlePaste" variant="secondary" size="icon">
                            <PasteIcon />
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent> Paste signatures from clipboard (Ctrl/Cmd + V)</TooltipContent>
                </Tooltip>
                <Tooltip>
                    <TooltipTrigger as-child>
                        <Button @click="createNewSignature" variant="secondary" size="icon">
                            <PlusIcon />
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent> Create new signature</TooltipContent>
                </Tooltip>
            </CardAction>
        </CardHeader>
        <MapPanelContent>
            <div class="overflow-hidden rounded-lg border">
                <div class="grid grid-cols-[auto_1fr_1fr_1fr_auto_auto] gap-x-2 divide-y">
                    <div class="col-span-full grid grid-cols-subgrid border-b bg-muted/50 px-2 py-1.5 text-xs font-medium text-muted-foreground">
                        <SortHeader
                            label="ID"
                            column="id"
                            :is-current-column="sortPreferences.column === 'id'"
                            :current-direction="sortPreferences.direction"
                            @sort="handleSort"
                        />
                        <SortHeader
                            label="Category"
                            column="category"
                            :is-current-column="sortPreferences.column === 'category'"
                            :current-direction="sortPreferences.direction"
                            @sort="handleSort"
                        />
                        <SortHeader
                            label="Type"
                            column="type"
                            :is-current-column="sortPreferences.column === 'type'"
                            :current-direction="sortPreferences.direction"
                            @sort="handleSort"
                        />
                        <div class="text-left">Connection</div>
                        <div class="text-left">Age</div>
                        <div></div>
                    </div>
                    <Signature
                        v-for="signature in signatures"
                        :signature="signature"
                        :key="signature.id"
                        :deleted="signature.deleted"
                        :new="signature.new"
                        :updated="signature.updated"
                        :unconnected_connections="unconnected_connections"
                        :connected_connections="connected_connections"
                        :selected_map_solarsystem="map_solarsystem"
                        :possible_signatures="relevant_signatures"
                    />
                </div>
                <div v-if="!signatures.length" class="p-4 text-center text-sm text-muted-foreground">No signatures found</div>
            </div>
        </MapPanelContent>
    </MapPanel>
</template>

<style scoped>
tr:has([data-status='new']) {
    background-color: var(--color-red-100);
}
</style>

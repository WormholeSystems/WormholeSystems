<script setup lang="ts">
import SignatureController from '@/actions/App/Http/Controllers/SignatureController';
import ChevronDownIcon from '@/components/icons/ChevronDownIcon.vue';
import PasteIcon from '@/components/icons/PasteIcon.vue';
import PlusIcon from '@/components/icons/PlusIcon.vue';
import Signature from '@/components/signatures/Signature.vue';
import { Button } from '@/components/ui/button';
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useHasWritePermission } from '@/composables/useHasPermission';
import { useSignatures } from '@/composables/useSignatures';
import { signatureParser, TRawSignature } from '@/lib/SignatureParser';
import Signatures from '@/routes/map-solarsystems/signatures';
import PasteSignatures from '@/routes/paste-signatures';
import { TMapSolarSystem, TSignature } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { useActiveElement, useEventListener } from '@vueuse/core';
import { computed, ref, watch } from 'vue';
import { toast } from 'vue-sonner';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TMapSolarSystem;
}>();

const { relevant_signatures, connections } = useSignatures();

const can_write = useHasWritePermission();

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

const signatures = computed(() => {
    return map_solarsystem
        .signatures!.map((sig) => ({
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

async function handlePaste() {
    const signatures = await getSignaturesFromClipboard();
    if (!signatures) {
        return;
    }

    processSignatures(signatures);
}

function processSignatures(signatures: TRawSignature[]) {
    pasted_signatures.value = signatures;
    router.post(
        PasteSignatures.store().url,
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
    router.post(
        SignatureController.store().url,
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

function deleteMissingSignatures(with_solarsystems = false) {
    router.delete(Signatures.destroy(map_solarsystem.id).url, {
        preserveScroll: true,
        preserveState: true,
        only: ['selected_map_solarsystem', 'map'],
        data: {
            signature_ids: deleted_signatures.value.map((signature) => signature.id),
            remove_map_solarsystems: with_solarsystems,
        },
    });
}

useEventListener('paste', (event) => {
    event.preventDefault();
    if (using_input.value) {
        return;
    }
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
    <Card class="overflow-hidden pb-0">
        <CardHeader>
            <CardTitle> Signatures</CardTitle>
            <CardDescription> All the signatures in this solarsystem. You can paste, copy and clear signatures here. </CardDescription>

            <CardAction class="flex gap-2" v-if="can_write">
                <Tooltip v-if="pasted_signatures">
                    <TooltipTrigger as-child>
                        <Button v-if="pasted_signatures" @click="pasted_signatures = null" variant="secondary"> Unselect </Button>
                    </TooltipTrigger>
                    <TooltipContent> Unselect signatures</TooltipContent>
                </Tooltip>
                <div class="flex divide-x" v-if="deleted_signatures.length > 0">
                    <Button @click="deleteMissingSignatures" variant="destructive" class="rounded-r-none"> Delete Missing </Button>
                    <DropdownMenu>
                        <DropdownMenuTrigger as-child>
                            <Button variant="destructive" class="rounded-l-none" size="icon">
                                <ChevronDownIcon />
                            </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent>
                            <DropdownMenuItem @click="deleteMissingSignatures(true)" as-child>
                                <Button variant="destructive" class="w-full"> Delete Missing with Solarsystems</Button>
                            </DropdownMenuItem>
                        </DropdownMenuContent>
                    </DropdownMenu>
                </div>
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
        <CardContent class="px-1 pb-1">
            <div class="grid grid-cols-[auto_1fr_1fr_1fr_auto_auto] gap-x-2 divide-y">
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
        </CardContent>
    </Card>
</template>

<style scoped>
tr:has([data-status='new']) {
    background-color: var(--color-red-100);
}
</style>

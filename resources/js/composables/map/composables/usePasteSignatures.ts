import { deleteSignatures, pasteSignatures } from '@/composables/map';
import { useIsUsingInput } from '@/composables/map/composables/useIsUsingInput';
import { signatureParser, TRawSignature } from '@/lib/SignatureParser';
import { TMapSolarSystem, TSignature } from '@/types/models';
import { useEventListener } from '@vueuse/core';
import { computed, ref, Ref, watch } from 'vue';
import { toast } from 'vue-sonner';

export function usePasteSignatures(map_solarsystem: Ref<TMapSolarSystem>) {
    const is_using_input = useIsUsingInput();

    const signatures_with_status = computed(() =>
        map_solarsystem.value.signatures!.map((sig) => ({
            ...sig,
            deleted: isSignatureDeleted(sig),
            new: isSignatureNew(sig),
            updated: isSignatureUpdated(sig),
        })),
    );

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

    watch(
        () => map_solarsystem.value.id,
        () => {
            pasted_signatures.value = null;
        },
    );

    useEventListener('paste', (event) => {
        if (is_using_input.value) {
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

    function getNewSignatures(parsed_signatures: Partial<TSignature>[]) {
        return parsed_signatures.filter((signature) => {
            return !map_solarsystem.value.signatures!.some((existing_signature) => {
                return existing_signature.signature_id === signature.signature_id;
            });
        });
    }

    function getUpdatedSignatures(parsed_signatures: Partial<TSignature>[]) {
        return parsed_signatures.filter((signature) => {
            return map_solarsystem.value.signatures!.some((existing_signature) => {
                return existing_signature.signature_id === signature.signature_id;
            });
        });
    }

    function getDeletedSignatures(parsed_signatures: Partial<TSignature>[]) {
        return map_solarsystem.value.signatures!.filter((signature) => {
            return !parsed_signatures.some((parsed_signature) => parsed_signature.signature_id === signature.signature_id);
        });
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

    function deleteMissingSignatures(with_solarsystems = false) {
        deleteSignatures(
            map_solarsystem.value.id,
            deleted_signatures.value.map((signature) => signature.id),
            with_solarsystems,
        );
    }

    function processSignatures(signatures: TRawSignature[]) {
        pasted_signatures.value = signatures;
        pasteSignatures(map_solarsystem.value.id, signatures);
    }

    async function handlePasteSignatures() {
        const signatures = await getSignaturesFromClipboard();
        if (!signatures) {
            return;
        }

        processSignatures(signatures);
    }

    return {
        signatures: signatures_with_status,
        pasted_signatures,
        new_signatures,
        updated_signatures,
        deleted_signatures,
        pasteSignatures: handlePasteSignatures,
        deleteMissingSignatures,
    };
}

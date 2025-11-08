<script setup lang="ts">
import { ContextMenuItem, ContextMenuSub, ContextMenuSubContent, ContextMenuSubTrigger } from '@/components/ui/context-menu';
import { TProcessedConnection } from '@/composables/map';
import { TTailoredSignature } from '@/pages/maps';
import { computed } from 'vue';
import { toast } from 'vue-sonner';

const { map_connection } = defineProps<{
    map_connection: TProcessedConnection;
}>();

function getClassString(solarsystem: typeof map_connection.source.solarsystem) {
    if (solarsystem.class) return `C${solarsystem.class}`;
    if (solarsystem.security >= 0.5) return 'HS';
    if (solarsystem.security > 0.0) return 'LS';
    return 'NS';
}

const source_signature = computed(() => {
    if (!map_connection.signatures?.length) return null;
    return map_connection.signatures.find((sig) => sig.map_solarsystem_id === map_connection.source.id) || null;
});

const target_signature = computed(() => {
    if (!map_connection.signatures?.length) return null;
    return map_connection.signatures.find((sig) => sig.map_solarsystem_id === map_connection.target.id) || null;
});

function getSignatureIdShort(signature: TTailoredSignature | null): string {
    return signature ? signature.signature_id.substring(0, 3) : '';
}

function formatSystemName(system: typeof map_connection.source, signature: TTailoredSignature | null): string {
    const class_string = getClassString(system.solarsystem);
    const sig_string = getSignatureIdShort(signature);

    // For wormhole systems: "alias sig_id class"
    if (system.solarsystem.class) {
        const parts = [system.alias || system.solarsystem.name];
        if (sig_string) parts.push(sig_string);
        parts.push(class_string);
        return parts.join(' ');
    }

    // For k-space systems: "class_string sig_id solarsystem_name region_name"
    const parts = [class_string];
    if (sig_string) parts.push(sig_string);
    parts.push(system.solarsystem.name);
    parts.push(system.solarsystem.region?.name);
    return parts.join(' ');
}

const source_name = computed(() => formatSystemName(map_connection.source, target_signature.value));

const target_name = computed(() => formatSystemName(map_connection.target, source_signature.value));

function copyNameToClipboard(value: string) {
    navigator.clipboard.writeText(value);
    toast.success('Copied to clipboard', { description: 'You successfully copied the name to your clipboard.' });
}
</script>

<template>
    <ContextMenuSub>
        <ContextMenuSubTrigger>Copy name</ContextMenuSubTrigger>
        <ContextMenuSubContent>
            <ContextMenuItem @select="copyNameToClipboard(source_name)">
                {{ source_name }}
            </ContextMenuItem>
            <ContextMenuItem @select="copyNameToClipboard(target_name)">
                {{ target_name }}
            </ContextMenuItem>
        </ContextMenuSubContent>
    </ContextMenuSub>
</template>

<style scoped></style>
